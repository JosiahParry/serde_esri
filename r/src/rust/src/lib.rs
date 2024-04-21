use extendr_api::prelude::*;
// mod sf_compat;
mod sfg;
mod to;
use crate::sfg::{Dim, SfgDim};
use serde_esri::{
    features::Feature,
    geometry::{EsriMultiPoint, EsriPolygon, EsriPolyline},
};
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

use extendr_api::deserializer::from_robj;
// use serde_esri::features::Feature;
use serde_json::{Map, Value};

#[extendr]
pub fn as_attribute(x: List, n: i32) {
    let ncol = x.len();

    let i = 1_usize;

    let col_names = x
        .names()
        .unwrap()
        .map(|si| si.to_string())
        .collect::<Vec<_>>();

    for i in 0..(n as usize) {
        for j in 0..ncol {
            let mut map = Map::with_capacity(ncol);
            let name = col_names[j].clone();
            let col = x[j];
            match col.rtype() {
                Rtype::Doubles => {
                    let col_typed = Doubles::try_from(col).unwrap();
                    let v = from_robj::<Value>(&col_typed[i].into());
                    if let Ok(v) = v {
                        map.insert(name, v);
                    }
                }
                Rtype::Integers => todo!(),
                Rtype::Strings => todo!(),
                Rtype::Logicals => todo!(),
                _ => unimplemented!(),
            }
        }
    }

    // for j in 0..ncol {
    //     let name = col_names[j].clone();
    //     let v = from_robj::<Value>(&x[j]);
    //     map.insert(name, v.unwrap());
    // }

    // let feats = Feature::<0> {
    //     geometry: None,
    //     attributes: Some(map),
    // };

    // rprintln!("{:?}", feats);
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
    fn as_attribute;

    // fn parse_esri_json_str;
    // fn parse_esri_json_str_simd;
    // fn parse_esri_json_raw_simd;
    // fn parse_esri_json_raw;
    // fn parse_esri_json_raw_geoarrow;
}
