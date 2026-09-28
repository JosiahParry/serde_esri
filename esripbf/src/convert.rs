//! Converts the protocol buffer types into their `serde_esri` counterparts.

#[cfg(feature = "geoarrow")]
mod arrow;
mod feature_set;
mod geometry;
mod scalars;

#[cfg(feature = "geoarrow")]
pub use arrow::PbfToArrowError;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FromPbfError {
    /// `hasZ` and `hasM` call for a different number of ordinates than the `FeatureSet<N>`.
    Dimensions { expected: usize, found: usize },
    /// The query result holds counts, object ids, or an extent rather than features.
    NotFeatureResult,
    /// Multipatch, envelope, and curve geometries have no `EsriGeometry` equivalent here.
    UnsupportedGeometry(&'static str),
    /// A feature carries a geometry but the result has no quantization transform.
    MissingTransform,
    /// Part lengths and coordinates disagree on the number of vertices.
    Coordinates { expected: usize, found: usize },
}

impl std::fmt::Display for FromPbfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FromPbfError::Dimensions { expected, found } => write!(
                f,
                "hasZ and hasM need {expected} ordinates per coordinate, the FeatureSet has {found}"
            ),
            FromPbfError::NotFeatureResult => write!(f, "query result does not hold features"),
            FromPbfError::UnsupportedGeometry(name) => {
                write!(f, "{name} geometries are not supported")
            }
            FromPbfError::MissingTransform => write!(f, "geometry has no quantization transform"),
            FromPbfError::Coordinates { expected, found } => write!(
                f,
                "part lengths need {expected} coordinate values, found {found}"
            ),
        }
    }
}

impl std::error::Error for FromPbfError {}
