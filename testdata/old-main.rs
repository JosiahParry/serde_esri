pub mod de_array;
pub mod features;
pub mod geometry;
pub mod spatial_reference;
pub mod sqltype;

use geometry::{EsriGeometry, *};
use serde_json::Map;
use spatial_reference::*;



// fn main() {
//     let wgs84 = Some(SpatialReference {
//         wkid: Some(4326u32),
//         latest_wkid: None,
//         vcs_wkid: None,
//         latest_vcs_wkid: None,
//         wkt: None,
//     });
//     let pnt = EsriPoint {
//         x: 3.0,
//         y: 0.14,
//         z: None,
//         m: None,
//         spatialReference: wgs84.clone(),
//     };

//     let jsn = serde_json::to_string(&pnt).unwrap();

//     // println!("{jsn}");

//     let crd = EsriCoord([3.0, 0.14, 0.00159]);
//     let mpnt = EsriMultiPoint {
//         hasZ: Some(true),
//         hasM: Some(false),
//         points: vec![crd],
//         spatialReference: wgs84.clone(),
//     };

//     let mpnt_jsn = serde_json::to_string(&mpnt).unwrap();
//     println!("{}", &mpnt_jsn);

//     let m2: EsriMultiPoint<3> = serde_json::from_str(&mpnt_jsn).unwrap();
//     // println!("{:?}", m2);
//     let mut attr = Map::new();
//     attr.insert("my_key".into(), "my_value".into());

//     let f = Feature {
//         geometry: Some(EsriGeometry::MultiPoint(m2)),
//         attributes: Some(attr),
//     };

//     println!("{:#?}", &f);

//     let f_str = serde_json::to_string(&f).unwrap();

//     let f2: Feature<3> = serde_json::from_str(&f_str).unwrap();
//     println!("{}", f_str);

//     let fset_str = r#"
//         {
//           "rings" : 
//           [
//             [
//               [-86.8206698922349, 32.3473096163641],  
//               [-86.8165954009227, 32.323052478491], 
//               [-86.8206698922349, 32.3473096163641]
//             ]
//           ]
//         }
// "#;

//     let fset: EsriPolygon<2> = serde_json::from_str(&fset_str).unwrap();
//     println!("{:#?}", serde_json::to_string(&fset));
//     let poly = EsriGeometry::Polygon(fset);
//     println!("{}", poly.to_string());


//     let fset = r#"
//     {
//         "geometryType": "esriGeometryPoint",
//         "spatialReference": {
//             "wkid": 4267
//         },
//         "hasZ": false,
//         "hasM": false,
//         "features": [
//             {
//                 "attributes": {
//                     "AREA": 0.114,
//                     "PERIMETER": 1.442
//                 },
//                 "geometry": {
//                     "x": -81.4982290095261,
//                     "y": 36.43139560823758
//                 }
//             },
//             {
//                 "attributes": {
//                     "AREA": 0.061,
//                     "PERIMETER": 1.231
//                 },
//                 "geometry": {
//                     "x": -81.12512977849917,
//                     "y": 36.49110847237506
//                 }
//             },
//             {
//                 "attributes": {
//                     "AREA": 0.143,
//                     "PERIMETER": 1.63
//                 },
//                 "geometry": {
//                     "x": -80.68573408861785,
//                     "y": 36.41252140662614
//                 }
//             }
//         ]
//     }"#;

//     let ff: FeatureSet<2> = serde_json::from_str(fset).unwrap();
//     println!{"{:#?}", ff};

//     use std::fs::read_to_string;

    
//     let jsn = read_to_string("testdata/poly.json")
//         .expect("file to be found");

//     let fset: FeatureSet<2> = serde_json::from_str(&jsn).unwrap();
//     println!("{:#?}", fset);
// }
