//! A geometry column built straight from Esri JSON geometries into GeoArrow buffers.
//!
//! Esri coordinates are `[x, y, z?, m?]`, GeoArrow's interleaved order, so ordinates are copied
//! as read. Rings stay as stored, already closed, and a polygon's rings are grouped into
//! polygons by orientation as the engine groups them. Missing ordinates and `null`, `"NaN"`
//! values are `NaN`; ordinates beyond the column's dimension are skipped.

use crate::{
    arrow_compat::ToArrowError,
    enginex::{
        geometry::{Envelope, Envelope2D, Interval},
        to_geoarrow::GeometryColumn,
    },
};
use arrow_array::ArrayRef;
use arrow_buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow_schema::Field;
use geoarrow_array::{
    array::{
        CoordBuffer, InterleavedCoordBuffer, MultiLineStringArray, MultiPointArray,
        MultiPolygonArray, PointArray, RectArray,
    },
    GeoArrowArray,
};
use geoarrow_schema::{Dimension, Metadata};
use serde::de::{self, DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use std::{fmt, sync::Arc};

/// The geometry types a FeatureSet column can hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GeometryType {
    Point,
    MultiPoint,
    Polyline,
    Polygon,
    Envelope,
}

impl TryFrom<&str> for GeometryType {
    type Error = ToArrowError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "esriGeometryPoint" => Ok(GeometryType::Point),
            "esriGeometryMultipoint" => Ok(GeometryType::MultiPoint),
            "esriGeometryPolyline" => Ok(GeometryType::Polyline),
            "esriGeometryPolygon" => Ok(GeometryType::Polygon),
            "esriGeometryEnvelope" => Ok(GeometryType::Envelope),
            other => Err(ToArrowError::UnsupportedGeometryType(other.to_string())),
        }
    }
}

/// Interleaved coordinates and offsets for one geometry column. Offsets are cumulative:
/// `geom_offsets` counts points, lines, or polygons per geometry, `part_offsets` coordinates per
/// line or rings per polygon, and `ring_offsets` coordinates per ring.
pub(super) struct GeometryBuilder {
    geometry_type: GeometryType,
    dim: Dimension,
    width: usize,
    coords: Vec<f64>,
    geom_offsets: Vec<i32>,
    part_offsets: Vec<i32>,
    ring_offsets: Vec<i32>,
    validity: Vec<bool>,
    envelopes: Vec<Option<Envelope>>,
}

impl GeometryBuilder {
    pub(super) fn new(geometry_type: GeometryType, dim: Dimension, capacity: usize) -> Self {
        let width = match dim {
            Dimension::XY => 2,
            Dimension::XYZ | Dimension::XYM => 3,
            Dimension::XYZM => 4,
        };
        GeometryBuilder {
            geometry_type,
            dim,
            width,
            coords: Vec::with_capacity(capacity * width),
            geom_offsets: vec![0],
            part_offsets: vec![0],
            ring_offsets: vec![0],
            validity: Vec::with_capacity(capacity),
            envelopes: Vec::new(),
        }
    }

    pub(super) fn len(&self) -> usize {
        self.validity.len()
    }

    pub(super) fn append_null(&mut self) {
        self.validity.push(false);
        match self.geometry_type {
            GeometryType::Point => self.coords.extend(std::iter::repeat_n(f64::NAN, self.width)),
            GeometryType::Envelope => self.envelopes.push(None),
            _ => {
                let last = self.geom_offsets.last().copied().unwrap_or(0);
                self.geom_offsets.push(last);
            }
        }
    }

    fn coord_count(&self) -> usize {
        self.coords.len() / self.width
    }

    /// Groups the rings from `first_ring` on into polygons: a ring with the orientation of the
    /// first non-zero ring starts a polygon (`_updateOGCFlagsHelper`).
    fn group_rings<E: de::Error>(&mut self, first_ring: usize) -> Result<(), E> {
        let ring_count = self.ring_offsets.len() - 1;
        let (coords, width) = (&self.coords, self.width);
        let xy = |i: usize| {
            let at = |j: usize| coords.get(i * width + j).copied().unwrap_or(f64::NAN);
            (at(0), at(1))
        };
        let bounds = |ring: usize| {
            let at = |i: usize| self.ring_offsets.get(i).map_or(0, |&o| o as usize);
            (at(ring), at(ring + 1))
        };
        let mut first_sign = 0.0;
        for ring in first_ring..ring_count {
            let (start, end) = bounds(ring);
            let (x0, y0) = xy(start);
            let area = (start..end)
                .map(|i| {
                    let (x1, y1) = xy(i);
                    let (x2, y2) = xy(if i + 1 < end { i + 1 } else { start });
                    ((x2 - x0) - (x1 - x0)) * ((y2 - y0) + (y1 - y0)) * 0.5
                })
                .sum::<f64>();
            if first_sign == 0.0 && area != 0.0 {
                first_sign = area.signum();
            }
            let starts_polygon = first_sign == 0.0 || area * first_sign > 0.0;
            if ring > first_ring && starts_polygon {
                self.part_offsets.push(offset(ring)?);
            }
        }
        if ring_count > first_ring {
            self.part_offsets.push(offset(ring_count)?);
        }
        Ok(())
    }

    /// The finished column with its GeoArrow extension field.
    pub(super) fn finish(self, metadata: Arc<Metadata>) -> Result<(Field, ArrayRef), ToArrowError> {
        fn column(array: impl GeoArrowArray) -> (Field, ArrayRef) {
            (array.data_type().to_field("geometry", true), array.to_array_ref())
        }
        let envelopes = self.envelopes;
        let offsets = |values: Vec<i32>| OffsetBuffer::new(ScalarBuffer::from(values));
        let has_nulls = self.validity.iter().any(|valid| !valid);
        let nulls = has_nulls.then(|| NullBuffer::from(self.validity));
        let coords = InterleavedCoordBuffer::try_new(ScalarBuffer::from(self.coords), self.dim)?;
        let coords = CoordBuffer::Interleaved(coords);
        let (geoms, parts, rings) = (self.geom_offsets, self.part_offsets, self.ring_offsets);
        Ok(match self.geometry_type {
            GeometryType::Point => column(PointArray::try_new(coords, nulls, metadata)?),
            GeometryType::MultiPoint => column(MultiPointArray::try_new(
                coords,
                offsets(geoms),
                nulls,
                metadata,
            )?),
            GeometryType::Polyline => column(MultiLineStringArray::try_new(
                coords,
                offsets(geoms),
                offsets(parts),
                nulls,
                metadata,
            )?),
            GeometryType::Polygon => column(MultiPolygonArray::try_new(
                coords,
                offsets(geoms),
                offsets(parts),
                offsets(rings),
                nulls,
                metadata,
            )?),
            GeometryType::Envelope => {
                let array = RectArray::try_from(GeometryColumn(&envelopes))?;
                column(array.with_metadata(metadata))
            }
        })
    }
}

fn offset<E: de::Error>(n: usize) -> Result<i32, E> {
    i32::try_from(n).map_err(|_| E::custom("offsets exceed GeoArrow's 32-bit limit"))
}

/// One ordinate: a number, or `NaN` for `null` and non-numeric text such as `"NaN"`.
struct Ordinate;

impl<'de> DeserializeSeed<'de> for Ordinate {
    type Value = f64;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<f64, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Ordinate {
    type Value = f64;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a coordinate value")
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<f64, E> {
        Ok(v)
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<f64, E> {
        Ok(v as f64)
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<f64, E> {
        Ok(v as f64)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<f64, E> {
        Ok(v.parse().unwrap_or(f64::NAN))
    }

    fn visit_unit<E: de::Error>(self) -> Result<f64, E> {
        Ok(f64::NAN)
    }
}

/// One coordinate array, appending the ordinates the column's dimension holds.
struct CoordSeed<'b> {
    coords: &'b mut Vec<f64>,
    width: usize,
}

impl<'de> DeserializeSeed<'de> for CoordSeed<'_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_seq(self)
    }
}

impl<'de> Visitor<'de> for CoordSeed<'_> {
    type Value = ();

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a coordinate array")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        let mut read = 0;
        while let Some(value) = seq.next_element_seed(Ordinate)? {
            if read < self.width {
                self.coords.push(value);
            }
            read += 1;
        }
        for _ in read..self.width {
            self.coords.push(f64::NAN);
        }
        Ok(())
    }
}

/// A path or point list: an array of coordinates.
struct PathSeed<'b> {
    coords: &'b mut Vec<f64>,
    width: usize,
}

impl<'de> DeserializeSeed<'de> for PathSeed<'_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_seq(self)
    }
}

impl<'de> Visitor<'de> for PathSeed<'_> {
    type Value = ();

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an array of coordinates")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        let width = self.width;
        while seq
            .next_element_seed(CoordSeed {
                coords: &mut *self.coords,
                width,
            })?
            .is_some()
        {}
        Ok(())
    }
}

/// `paths` or `rings`: arrays of coordinates, each ending an offset.
struct PathsSeed<'b> {
    coords: &'b mut Vec<f64>,
    width: usize,
    offsets: &'b mut Vec<i32>,
}

impl<'de> DeserializeSeed<'de> for PathsSeed<'_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_seq(self)
    }
}

impl<'de> Visitor<'de> for PathsSeed<'_> {
    type Value = ();

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an array of paths")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        let width = self.width;
        loop {
            let path = PathSeed {
                coords: &mut *self.coords,
                width,
            };
            if seq.next_element_seed(path)?.is_none() {
                return Ok(());
            }
            self.offsets.push(offset(self.coords.len() / width)?);
        }
    }
}

/// The keys an Esri geometry object can hold.
enum GeometryKey {
    X,
    Y,
    Z,
    M,
    Points,
    Paths,
    Rings,
    Bound(usize),
    Other,
}

impl<'de> Deserialize<'de> for GeometryKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct KeyVisitor;

        impl Visitor<'_> for KeyVisitor {
            type Value = GeometryKey;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a geometry key")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<GeometryKey, E> {
                Ok(match v {
                    "x" => GeometryKey::X,
                    "y" => GeometryKey::Y,
                    "z" => GeometryKey::Z,
                    "m" => GeometryKey::M,
                    "points" => GeometryKey::Points,
                    "paths" => GeometryKey::Paths,
                    "rings" => GeometryKey::Rings,
                    "xmin" => GeometryKey::Bound(0),
                    "ymin" => GeometryKey::Bound(1),
                    "xmax" => GeometryKey::Bound(2),
                    "ymax" => GeometryKey::Bound(3),
                    "zmin" => GeometryKey::Bound(4),
                    "zmax" => GeometryKey::Bound(5),
                    "mmin" => GeometryKey::Bound(6),
                    "mmax" => GeometryKey::Bound(7),
                    _ => GeometryKey::Other,
                })
            }
        }

        deserializer.deserialize_str(KeyVisitor)
    }
}

/// Appends one feature's geometry, or a null for `null`.
pub(super) struct GeometrySeed<'b>(pub(super) &'b mut GeometryBuilder);

impl<'de> DeserializeSeed<'de> for GeometrySeed<'_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for GeometrySeed<'_> {
    type Value = ();

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an Esri geometry object")
    }

    fn visit_unit<E: de::Error>(self) -> Result<(), E> {
        self.0.append_null();
        Ok(())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        let b = self.0;
        let width = b.width;
        let first_ring = b.ring_offsets.len() - 1;
        let mut point = [f64::NAN; 4];
        let mut bounds = [f64::NAN; 8];
        while let Some(key) = map.next_key::<GeometryKey>()? {
            match (b.geometry_type, key) {
                (GeometryType::Envelope, GeometryKey::Bound(i)) => {
                    let value = map.next_value_seed(Ordinate)?;
                    if let Some(bound) = bounds.get_mut(i) {
                        *bound = value;
                    }
                }
                (GeometryType::Point, GeometryKey::X) => point[0] = map.next_value_seed(Ordinate)?,
                (GeometryType::Point, GeometryKey::Y) => point[1] = map.next_value_seed(Ordinate)?,
                (GeometryType::Point, GeometryKey::Z) => point[2] = map.next_value_seed(Ordinate)?,
                (GeometryType::Point, GeometryKey::M) => point[3] = map.next_value_seed(Ordinate)?,
                (GeometryType::MultiPoint, GeometryKey::Points) => {
                    map.next_value_seed(PathSeed {
                        coords: &mut b.coords,
                        width,
                    })?;
                }
                (GeometryType::Polyline, GeometryKey::Paths) => {
                    map.next_value_seed(PathsSeed {
                        coords: &mut b.coords,
                        width,
                        offsets: &mut b.part_offsets,
                    })?;
                }
                (GeometryType::Polygon, GeometryKey::Rings) => {
                    map.next_value_seed(PathsSeed {
                        coords: &mut b.coords,
                        width,
                        offsets: &mut b.ring_offsets,
                    })?;
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }

        match b.geometry_type {
            GeometryType::Point => {
                let [x, y, z, m] = point;
                match b.dim {
                    Dimension::XY => b.coords.extend([x, y]),
                    Dimension::XYZ => b.coords.extend([x, y, z]),
                    Dimension::XYM => b.coords.extend([x, y, m]),
                    Dimension::XYZM => b.coords.extend([x, y, z, m]),
                }
            }
            GeometryType::MultiPoint => {
                let count = offset(b.coord_count())?;
                b.geom_offsets.push(count);
            }
            GeometryType::Polyline => {
                let count = offset(b.part_offsets.len() - 1)?;
                b.geom_offsets.push(count);
            }
            GeometryType::Polygon => {
                b.group_rings(first_ring)?;
                let count = offset(b.part_offsets.len() - 1)?;
                b.geom_offsets.push(count);
            }
            GeometryType::Envelope => {
                let [xmin, ymin, xmax, ymax, zmin, zmax, mmin, mmax] = bounds;
                let interval = |min: f64, max: f64| {
                    (!min.is_nan() && !max.is_nan()).then_some(Interval { min, max })
                };
                b.envelopes.push(Some(Envelope {
                    xy: Some(Envelope2D {
                        xmin,
                        ymin,
                        xmax,
                        ymax,
                    }),
                    z: interval(zmin, zmax),
                    m: interval(mmin, mmax),
                    id: None,
                }));
            }
        }
        b.validity.push(true);
        Ok(())
    }
}
