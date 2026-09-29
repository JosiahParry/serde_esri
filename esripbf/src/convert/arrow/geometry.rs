//! The GeoArrow geometry column, filled through geoarrow-array's builders.

use crate::{
    convert::{arrow::PbfToArrowError, geometry::Quantization, FromPbfError},
    esri_p_buffer::feature_collection_p_buffer::{self as pbf, feature::CompressedGeometry, GeometryType},
};
use arrow_array::ArrayRef;
use arrow_schema::Field;
use geoarrow_array::{
    builder::{MultiLineStringBuilder, MultiPointBuilder, MultiPolygonBuilder, PointBuilder},
    capacity::{MultiLineStringCapacity, MultiPointCapacity, MultiPolygonCapacity},
    GeoArrowArray,
};
use geoarrow_schema::{
    CoordType, Crs, Dimension, Metadata, MultiLineStringType, MultiPointType, MultiPolygonType,
    PointType,
};
use serde_esri::{
    enginex::geometry::{Geometry, MultiPoint, Polygon, Polyline},
    spatial_reference::SpatialReference,
};
use std::sync::Arc;

enum Builder {
    Point(PointBuilder),
    MultiPoint(MultiPointBuilder),
    Polyline(MultiLineStringBuilder),
    Polygon(MultiPolygonBuilder),
}

pub(super) struct GeometryColumnBuilder {
    geometry_type: GeometryType,
    quantization: Result<Quantization, FromPbfError>,
    builder: Builder,
}

impl GeometryColumnBuilder {
    /// The column for a layer's results, or `None` for a table, which has no spatial reference.
    pub(super) fn new(result: &pbf::FeatureResult) -> Result<Option<Self>, FromPbfError> {
        let Some(sr) = result.spatial_reference.clone() else {
            return Ok(None);
        };
        let dim = match (result.has_z, result.has_m) {
            (false, false) => Dimension::XY,
            (true, false) => Dimension::XYZ,
            (false, true) => Dimension::XYM,
            (true, true) => Dimension::XYZM,
        };
        let metadata = Arc::new(Metadata::new(Crs::from(&SpatialReference::from(sr)), None));
        let width = 2 + usize::from(result.has_z) + usize::from(result.has_m);
        let (mut coords, mut parts) = (0, 0);
        for feature in &result.features {
            if let Some(CompressedGeometry::Geometry(g)) = &feature.compressed_geometry {
                coords += g.coords.len() / width;
                parts += g.lengths.len();
            }
        }
        let rows = result.features.len();
        let interleaved = CoordType::Interleaved;
        let geometry_type = result.geometry_type();
        let builder = match geometry_type {
            GeometryType::EsriGeometryTypePoint => {
                let typ = PointType::new(dim, metadata).with_coord_type(interleaved);
                Builder::Point(PointBuilder::with_capacity(typ, rows))
            }
            GeometryType::EsriGeometryTypeMultipoint => {
                let typ = MultiPointType::new(dim, metadata).with_coord_type(interleaved);
                let capacity = MultiPointCapacity::new(coords, rows);
                Builder::MultiPoint(MultiPointBuilder::with_capacity(typ, capacity))
            }
            GeometryType::EsriGeometryTypePolyline => {
                let typ = MultiLineStringType::new(dim, metadata).with_coord_type(interleaved);
                let capacity = MultiLineStringCapacity::new(coords, parts, rows);
                Builder::Polyline(MultiLineStringBuilder::with_capacity(typ, capacity))
            }
            GeometryType::EsriGeometryTypePolygon => {
                let typ = MultiPolygonType::new(dim, metadata).with_coord_type(interleaved);
                // Closing vertices are added back, one per ring.
                let capacity = MultiPolygonCapacity::new(coords + parts, parts, parts, rows);
                Builder::Polygon(MultiPolygonBuilder::with_capacity(typ, capacity))
            }
            GeometryType::EsriGeometryTypeMultipatch => {
                return Err(FromPbfError::UnsupportedGeometry("multipatch"))
            }
            GeometryType::EsriGeometryTypeEnvelope => {
                return Err(FromPbfError::UnsupportedGeometry("envelope"))
            }
            GeometryType::EsriGeometryTypeNone => return Ok(None),
        };
        Ok(Some(GeometryColumnBuilder {
            geometry_type,
            quantization: Quantization::try_from(result),
            builder,
        }))
    }

    pub(super) fn push(
        &mut self,
        geometry: Option<CompressedGeometry>,
    ) -> Result<(), PbfToArrowError> {
        let geometry = match geometry {
            None => None,
            Some(CompressedGeometry::Geometry(g)) => {
                let quantization = self.quantization.as_ref().map_err(Clone::clone)?;
                Some(quantization.engine(self.geometry_type, &g)?)
            }
            Some(CompressedGeometry::ShapeBuffer(_)) => {
                return Err(FromPbfError::UnsupportedGeometry("shape buffer").into())
            }
            Some(CompressedGeometry::CurveGeometry(_)) => {
                return Err(FromPbfError::UnsupportedGeometry("curve").into())
            }
        };
        match (&mut self.builder, geometry) {
            (Builder::Point(b), Some(Geometry::Point(g))) => b.try_push_point(Some(&g))?,
            (Builder::Point(b), _) => b.push_null(),
            (Builder::MultiPoint(b), Some(Geometry::MultiPoint(g))) => {
                b.push_multi_point(Some(&g))?
            }
            (Builder::MultiPoint(b), _) => b.push_multi_point(None::<&MultiPoint>)?,
            (Builder::Polyline(b), Some(Geometry::Polyline(g))) => {
                b.push_multi_line_string(Some(&g))?
            }
            (Builder::Polyline(b), _) => b.push_multi_line_string(None::<&Polyline>)?,
            (Builder::Polygon(b), Some(Geometry::Polygon(g))) => {
                b.push_multi_polygon(Some(&g))?
            }
            (Builder::Polygon(b), _) => b.push_multi_polygon(None::<&Polygon>)?,
        }
        Ok(())
    }

    pub(super) fn finish(self) -> (Field, ArrayRef) {
        fn column(array: impl GeoArrowArray) -> (Field, ArrayRef) {
            (
                array.data_type().to_field("geometry", true),
                array.to_array_ref(),
            )
        }
        match self.builder {
            Builder::Point(b) => column(b.finish()),
            Builder::MultiPoint(b) => column(b.finish()),
            Builder::Polyline(b) => column(b.finish()),
            Builder::Polygon(b) => column(b.finish()),
        }
    }
}
