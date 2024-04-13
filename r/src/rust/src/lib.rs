use extendr_api::prelude::*;
// mod sf_compat;
mod sfg;
mod to;
use crate::sfg::{Dim, SfgDim};
use serde_esri::geometry::{EsriMultiPoint, EsriPolygon, EsriPolyline};
use sfg::{
    SfgLineString, SfgMultiLineString, SfgMultiPoint, SfgMultiPolygon, SfgPoint, SfgPolygon,
};
use to::AsEsriGeometry;

#[extendr]
fn as_multipoint(x: RMatrix<f64>) -> String {
    let sfg = SfgMultiPoint(x);
    let dim = sfg.sfg_dim().unwrap();

    match dim {
        SfgDim::XY => {
            let mpnt: EsriMultiPoint<2> = sfg.as_multipoint().unwrap();
            serde_json::to_string(&mpnt).unwrap()
        }
        SfgDim::XYZ => {
            let mpnt: EsriMultiPoint<3> = sfg.as_multipoint().unwrap();
            serde_json::to_string(&mpnt).unwrap()
        }
        SfgDim::XYM => {
            let mpnt: EsriMultiPoint<3> = sfg.as_multipoint().unwrap();
            serde_json::to_string(&mpnt).unwrap()
        }
        SfgDim::XYZM => {
            let mpnt: EsriMultiPoint<4> = sfg.as_multipoint().unwrap();
            serde_json::to_string(&mpnt).unwrap()
        }
    }
}

#[extendr]
fn as_point(x: Doubles) -> String {
    let sfg = SfgPoint(x);
    serde_json::to_string(&sfg.as_point().unwrap()).unwrap()
}

#[extendr]
fn as_linestring_polyline(x: RMatrix<f64>) -> String {
    let sfg = SfgLineString(x);
    let dim = sfg.sfg_dim().unwrap();

    match dim {
        SfgDim::XY => {
            let pline: Option<EsriPolyline<2>> = sfg.as_polyline();
            serde_json::to_string(&pline).unwrap()
        }
        SfgDim::XYZ => {
            let pline: EsriPolyline<3> = sfg.as_polyline().unwrap();
            serde_json::to_string(&pline).unwrap()
        }
        SfgDim::XYM => {
            let pline: EsriPolyline<3> = sfg.as_polyline().unwrap();
            serde_json::to_string(&pline).unwrap()
        }
        SfgDim::XYZM => {
            let pline: EsriPolyline<4> = sfg.as_polyline().unwrap();
            serde_json::to_string(&pline).unwrap()
        }
    }
}

#[extendr]
fn as_polyline(x: List) -> String {
    let sfg = SfgMultiLineString(x);
    let dim = sfg.sfg_dim().unwrap();

    match dim {
        SfgDim::XY => {
            let pline: Option<EsriPolyline<2>> = sfg.as_polyline();
            serde_json::to_string(&pline).unwrap()
        }
        SfgDim::XYZ => {
            let pline: EsriPolyline<3> = sfg.as_polyline().unwrap();
            serde_json::to_string(&pline).unwrap()
        }
        SfgDim::XYM => {
            let pline: EsriPolyline<3> = sfg.as_polyline().unwrap();
            serde_json::to_string(&pline).unwrap()
        }
        SfgDim::XYZM => {
            let pline: EsriPolyline<4> = sfg.as_polyline().unwrap();
            serde_json::to_string(&pline).unwrap()
        }
    }
}

#[extendr]
fn as_poly_polygon(x: List) -> String {
    let sfg = SfgPolygon(x);
    let dim = sfg.sfg_dim().unwrap();

    match dim {
        SfgDim::XY => {
            let poly: Option<EsriPolygon<2>> = sfg.as_polygon();
            serde_json::to_string(&poly).unwrap()
        }
        SfgDim::XYZ => {
            todo!()
        }
        SfgDim::XYM => {
            todo!()
        }
        SfgDim::XYZM => {
            todo!()
        }
    }
}

#[extendr]
fn as_polygon(x: List) -> String {
    let sfg = SfgMultiPolygon(x);
    let dim = sfg.sfg_dim().unwrap();

    match dim {
        SfgDim::XY => {
            let poly: Option<EsriPolygon<2>> = sfg.as_polygon();
            serde_json::to_string(&poly).unwrap()
        }
        SfgDim::XYZ => {
            todo!()
        }
        SfgDim::XYM => {
            todo!()
        }
        SfgDim::XYZM => {
            todo!()
        }
    }
}
// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod serdesri;
    fn as_multipoint;
    fn as_point;
    fn as_linestring_polyline;
    fn as_polyline;
    fn as_poly_polygon;
    fn as_polygon;

    // fn parse_esri_json_str;
    // fn parse_esri_json_str_simd;
    // fn parse_esri_json_raw_simd;
    // fn parse_esri_json_raw;
    // fn parse_esri_json_raw_geoarrow;
}
