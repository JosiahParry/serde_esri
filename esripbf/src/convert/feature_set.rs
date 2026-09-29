//! Converts query results into `FeatureSet`s.

use crate::{
    convert::{geometry::Quantization, FromPbfError},
    esri_p_buffer::feature_collection_p_buffer::{
        self as pbf, feature::CompressedGeometry, query_result::Results, GeometryType,
    },
    esri_p_buffer::FeatureCollectionPBuffer,
};
use serde_esri::features::{value::EsriValue, Feature, FeatureSet};

impl<const N: usize> TryFrom<pbf::FeatureResult> for FeatureSet<N> {
    type Error = FromPbfError;

    fn try_from(result: pbf::FeatureResult) -> Result<Self, Self::Error> {
        let expected = 2 + usize::from(result.has_z) + usize::from(result.has_m);
        if expected != N {
            return Err(FromPbfError::Dimensions { expected, found: N });
        }
        let quantization = Quantization::try_from(&result);
        let geometry_type = result.geometry_type();
        let names = result.fields.iter().map(|f| f.name.clone()).collect::<Vec<String>>();

        let features = result
            .features
            .into_iter()
            .map(|feature| {
                let geometry = match feature.compressed_geometry {
                    None => None,
                    Some(CompressedGeometry::Geometry(geometry)) => Some(
                        quantization
                            .as_ref()
                            .map_err(Clone::clone)?
                            .geometry(geometry_type, &geometry)?,
                    ),
                    Some(CompressedGeometry::ShapeBuffer(_)) => {
                        return Err(FromPbfError::UnsupportedGeometry("shape buffer"))
                    }
                    Some(CompressedGeometry::CurveGeometry(_)) => {
                        return Err(FromPbfError::UnsupportedGeometry("curve"))
                    }
                };
                let values = feature.attributes.into_iter().map(EsriValue::from);
                Ok(Feature {
                    geometry,
                    attributes: Some(names.iter().cloned().zip(values).collect()),
                })
            })
            .collect::<Result<Vec<_>, FromPbfError>>()?;

        // Tables have no spatial reference, and their geometry type reads as the default, point.
        let geometry_type = match (&result.spatial_reference, geometry_type) {
            (None, _) | (_, GeometryType::EsriGeometryTypeNone) => None,
            (_, GeometryType::EsriGeometryTypePoint) => Some("esriGeometryPoint"),
            (_, GeometryType::EsriGeometryTypeMultipoint) => Some("esriGeometryMultipoint"),
            (_, GeometryType::EsriGeometryTypePolyline) => Some("esriGeometryPolyline"),
            (_, GeometryType::EsriGeometryTypePolygon) => Some("esriGeometryPolygon"),
            (_, GeometryType::EsriGeometryTypeMultipatch) => Some("esriGeometryMultiPatch"),
            (_, GeometryType::EsriGeometryTypeEnvelope) => Some("esriGeometryEnvelope"),
        };
        let text = |s: String| (!s.is_empty()).then_some(s);

        Ok(FeatureSet {
            object_id_field_name: text(result.object_id_field_name),
            global_id_field_name: text(result.global_id_field_name),
            display_field_name: None,
            geometry_type: geometry_type.map(String::from),
            spatial_reference: result.spatial_reference.map(Into::into),
            has_z: result.has_z.then_some(true),
            has_m: result.has_m.then_some(true),
            fields: Some(result.fields.into_iter().map(Into::into).collect()),
            features,
        })
    }
}

impl<const N: usize> TryFrom<FeatureCollectionPBuffer> for FeatureSet<N> {
    type Error = FromPbfError;

    fn try_from(collection: FeatureCollectionPBuffer) -> Result<Self, Self::Error> {
        match collection.query_result.and_then(|q| q.results) {
            Some(Results::FeatureResult(result)) => FeatureSet::try_from(result),
            _ => Err(FromPbfError::NotFeatureResult),
        }
    }
}
