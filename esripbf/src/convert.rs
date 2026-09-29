//! Converts the protocol buffer types into their `serde_esri` counterparts.

#[cfg(feature = "geoarrow")]
pub mod arrow;
mod feature_set;
mod geometry;
mod scalars;


#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FromPbfError {
    Dimensions { expected: usize, found: usize },
    NotFeatureResult,
    UnsupportedGeometry(&'static str),
    MissingTransform,
    TooLarge,
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
            FromPbfError::TooLarge => write!(f, "geometry exceeds 32-bit path offsets"),
            FromPbfError::Coordinates { expected, found } => write!(
                f,
                "part lengths need {expected} coordinate values, found {found}"
            ),
        }
    }
}

impl std::error::Error for FromPbfError {}
