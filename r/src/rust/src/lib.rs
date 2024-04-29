use extendr_api::prelude::*;
// mod sf_compat;
mod sfc;
mod sfg;
mod to;
use crate::sfg::{Dim, SfgDim};
use serde_esri::{
    features::Feature,
    geometry::{EsriMultiPoint, EsriPolygon, EsriPolyline},
    spatial_reference::SpatialReference,
};
use sfg::{
    SfgLineString, SfgMultiLineString, SfgMultiPoint, SfgMultiPolygon, SfgPoint, SfgPolygon,
};
use to::AsEsriGeometry;

pub fn deserialize_sr(sr: &Robj) -> Option<SpatialReference> {
    extendr_api::deserializer::from_robj::<SpatialReference>(&sr).map_or(None, |sr| Some(sr))
}

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
            let poly: Option<EsriPolygon<3>> = sfg.as_polygon();
            serde_json::to_string(&poly).unwrap()
        }
        SfgDim::XYM => {
            let poly: Option<EsriPolygon<3>> = sfg.as_polygon();
            serde_json::to_string(&poly).unwrap()
        }
        SfgDim::XYZM => {
            let poly: Option<EsriPolygon<4>> = sfg.as_polygon();
            serde_json::to_string(&poly).unwrap()
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
            let poly: Option<EsriPolygon<3>> = sfg.as_polygon();
            serde_json::to_string(&poly).unwrap()
        }
        SfgDim::XYM => {
            let poly: Option<EsriPolygon<3>> = sfg.as_polygon();
            serde_json::to_string(&poly).unwrap()
        }
        SfgDim::XYZM => {
            let poly: Option<EsriPolygon<4>> = sfg.as_polygon();
            serde_json::to_string(&poly).unwrap()
        }
    }
}

use serde_json::{Map, Number, Value};

// TODO if a column is a factor, make it a character factor
// TODO handle geometry list as well to create Vec<Features>
#[extendr]
pub fn as_attribute(x: List, n: i32) {
    let n = n as usize;
    let ncol = x.len();

    // extract columns from the data.frame
    let col_names = x
        .names()
        .unwrap()
        .map(|si| si.to_string())
        .collect::<Vec<_>>();

    let mut feats = Vec::with_capacity(n);

    for i in 0..(n) {
        let mut map = Map::with_capacity(ncol);
        for j in 0..ncol {
            let name = col_names[j].clone();
            let col = &x[j];
            match col.rtype() {
                Rtype::Doubles => {
                    let col_typed = Doubles::try_from(col).unwrap();
                    let v = &col_typed[i];

                    match !v.is_na() {
                        true => {
                            let num = Number::from_f64(v.inner())
                                .expect("double can't be converted to serde_json::Number");
                            map.insert(name, Value::Number(num))
                        }
                        false => map.insert(name, Value::Null),
                    };
                }
                Rtype::Integers => {
                    let col_typed = Integers::try_from(col).unwrap();
                    let v = &col_typed[i];

                    match !v.is_na() {
                        true => {
                            let num = Number::from_f64(v.inner() as f64)
                                .expect("integer can't be converted to serde_json::Number");
                            map.insert(name, Value::Number(num))
                        }
                        false => map.insert(name, Value::Null),
                    };
                }
                Rtype::Strings => {
                    let col_typed = Strings::try_from(col).unwrap();
                    let rstr = &col_typed[i];
                    match !rstr.is_na() {
                        true => map.insert(name, Value::String(rstr.to_string())),
                        false => map.insert(name, Value::Null),
                    };
                }
                Rtype::Logicals => {
                    let col_typed = Logicals::try_from(col).unwrap();
                    let v = &col_typed[i];

                    match !v.is_na() {
                        true => map.insert(name, Value::Bool(v.to_bool())),
                        false => map.insert(name, Value::Null),
                    };
                }
                _ => unimplemented!(),
            }
        }

        let feat = Feature::<2> {
            attributes: Some(map),
            geometry: None,
        };
        feats.push(feat);
    }

    rprintln!("{:#?}", feats);
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod serdesri;
    use to;
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
