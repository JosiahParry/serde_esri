use serde_esri::geometry::{EsriMultiPoint, EsriPolygon, EsriPolyline};

mod linestring;
mod multilinestring;
mod multipoint;
mod multipolygon;
mod point;
mod polygon;
// this is used for geometries that have a const usize parameter
// they will always need type annotions
// point is implemented independently since it does not have a
// const generic. It is implemented on the struct itself
pub trait AsEsriGeometry<const N: usize> {
    fn as_multipoint(&self) -> Option<EsriMultiPoint<N>> {
        None
    }
    fn as_polyline(&self) -> Option<EsriPolyline<N>> {
        None
    }
    fn as_polygon(&self) -> Option<EsriPolygon<N>> {
        None
    }
}
