use extendr_api::prelude::*;
use serde_esri::{
    arrow_compat::featureset_to_arrow,
    features::FeatureSet
};
use arrow_extendr::to::IntoArrowRobj;
// use arrow::record_batch::RecordBatch;

#[extendr]
/// @export
fn parse_esri_json_str(str: &str, n_dim: i32) -> Robj {
    let n_dim = n_dim as usize;

    match n_dim {
        0 => fset_to_robj(serde_json::from_str::<FeatureSet<0>>(str).unwrap()),
        2 => fset_to_robj(serde_json::from_str::<FeatureSet<2>>(str).unwrap()),
        3 => fset_to_robj(serde_json::from_str::<FeatureSet<3>>(str).unwrap()),
        4 => fset_to_robj(serde_json::from_str::<FeatureSet<4>>(str).unwrap()),
        _ => unimplemented!()
    }
}


#[extendr]
/// @export
fn parse_esri_json_raw(raw: Raw, n_dim: i32) -> Robj {
    let n_dim = n_dim as usize;
    let bytes = raw.as_slice();

    match n_dim {
        0 => fset_to_robj(serde_json::from_slice::<FeatureSet<0>>(bytes).unwrap()),
        2 => fset_to_robj(serde_json::from_slice::<FeatureSet<2>>(bytes).unwrap()),
        3 => fset_to_robj(serde_json::from_slice::<FeatureSet<3>>(bytes).unwrap()),
        4 => fset_to_robj(serde_json::from_slice::<FeatureSet<4>>(bytes).unwrap()),
        _ => unimplemented!()
    }
}


fn fset_to_robj<const N: usize>(fset: FeatureSet<N>) -> Robj {
    featureset_to_arrow(fset)
        .unwrap()
        .into_arrow_robj()
        .unwrap()
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod serdesri;
    fn parse_esri_json_str;
    fn parse_esri_json_raw;
}

