use super::AsEsriGeometry;
use crate::sfg::{Dim, SfgDim, SfgMultiPolygon};
use extendr_api::prelude::*;
use serde_esri::geometry::*;

impl AsEsriGeometry<2> for SfgMultiPolygon {
    fn as_polygon(&self) -> Option<EsriPolygon<2>> {
        let dim = if let Some(dim) = self.sfg_dim() {
            dim
        } else {
            return None;
        };

        let n_elements = self.0.len();

        if n_elements == 0 {
            let poly: EsriPolygon<2> = EsriPolygon::default();
            return Some(poly);
        }

        match dim {
            SfgDim::XY => {
                let linestrings = self
                    .0
                    .iter()
                    .flat_map(|(_, list)| {
                        List::try_from(list).unwrap().into_iter().map(|(_, line)| {
                            let mat = RMatrix::<f64>::try_from(line);

                            let mat = match mat {
                                Ok(mat) => mat,
                                Err(_) => return EsriLineString::default(),
                            };

                            let slice = mat.as_real_slice().unwrap();

                            let mut points = Vec::with_capacity(mat.nrows());
                            let nrow = mat.nrows();

                            for i in 0..nrow {
                                let x = slice[i];
                                let y = slice[i + nrow];
                                let crd = EsriCoord([x, y]);
                                points.push(crd);
                            }

                            EsriLineString(points)
                        })
                    })
                    .collect::<Vec<_>>();

                let res = EsriPolygon {
                    hasZ: Some(false),
                    hasM: Some(false),
                    rings: linestrings,
                    spatialReference: None,
                };

                Some(res)
            }
            _ => None,
        }
    }
}
