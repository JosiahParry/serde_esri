//! Reads each `.shp` given into engine geometries, builds a GeoArrow geometry array, and checks
//! every array value against the engine geometry it came from.
//!
//! ```sh
//! cargo run --example shp_to_geoarrow --features geoarrow -- a.shp b.shp
//! ```

use geo_traits::{
    to_geo::{ToGeoGeometry, ToGeoMultiLineString, ToGeoMultiPolygon},
    GeometryTrait, GeometryType, PointTrait,
};
use geoarrow_array::{
    array::{GeometryArray, MultiLineStringArray, MultiPolygonArray},
    GeoArrowArray, GeoArrowArrayAccessor,
};
use serde_esri::{
    enginex::{
        geometry::{Geometry, Point, Polygon, Polyline},
        to_geoarrow::GeometryColumn,
    },
    shape::file::ShapeFile,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    for path in std::env::args().skip(1) {
        let bytes = std::fs::read(&path)?;
        let name = path.rsplit('/').next().unwrap_or_default().to_string();
        let geometries = match ShapeFile::try_from(bytes.as_slice())
            .map_err(|e| e.to_string())
            .and_then(|file| file.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string()))
        {
            Ok(records) => records
                .into_iter()
                .map(|record| Geometry::try_from(record.shape).ok())
                .collect::<Vec<_>>(),
            Err(e) => {
                println!("{name:<45} skipped: {e}");
                continue;
            }
        };

        let array = GeometryArray::try_from(GeometryColumn(&geometries))?;
        let mut mismatches = 0;
        for (i, geometry) in geometries.iter().enumerate() {
            let matches = match geometry {
                // geo-types has no empty point, so check the array holds one directly.
                Some(Geometry::Point(Point(None))) => matches!(
                    array.value(i)?.as_type(),
                    GeometryType::Point(point) if point.coord().is_none()
                ),
                Some(geometry) => {
                    // Debug output compares NaN coordinates as equal.
                    let value = array.value(i)?.to_geometry();
                    format!("{value:?}") == format!("{:?}", geometry.to_geometry())
                }
                None => array.is_null(i),
            };
            mismatches += usize::from(!matches);
        }
        // Polygons and polylines also go through the native typed builders.
        let polygons = geometries
            .iter()
            .map(|g| match g {
                Some(Geometry::Polygon(p)) => Some(p.clone()),
                _ => None,
            })
            .collect::<Vec<Option<Polygon>>>();
        let native = MultiPolygonArray::try_from(GeometryColumn(&polygons))?;
        for (i, polygon) in polygons.iter().enumerate() {
            if let Some(polygon) = polygon {
                let value = native.value(i)?.to_multi_polygon();
                mismatches += usize::from(format!("{value:?}") != format!("{:?}", polygon.to_multi_polygon()));
            }
        }
        let polylines = geometries
            .iter()
            .map(|g| match g {
                Some(Geometry::Polyline(p)) => Some(p.clone()),
                _ => None,
            })
            .collect::<Vec<Option<Polyline>>>();
        let native = MultiLineStringArray::try_from(GeometryColumn(&polylines))?;
        for (i, polyline) in polylines.iter().enumerate() {
            if let Some(polyline) = polyline {
                let value = native.value(i)?.to_multi_line_string();
                mismatches += usize::from(
                    format!("{value:?}") != format!("{:?}", polyline.to_multi_line_string()),
                );
            }
        }

        let nulls = geometries.iter().filter(|g| g.is_none()).count();
        println!(
            "{name:<45} {} values, {nulls} null, {mismatches} mismatched",
            array.len()
        );
    }
    Ok(())
}
