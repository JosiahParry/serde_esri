use arrow_schema::ArrowError;
use esripbf::convert::FromPbfError;
use parquet::errors::ParquetError;
use serde_esri::enginex::shape::ShapeError;

#[derive(Debug)]
pub enum Error {
    MissingMetadata,
    BothMetadataForms,
    MultipleIndexes,
    MissingColumn(String),
    ColumnType {
        column: String,
        expected: &'static str,
    },
    NoLevels,
    Resolution(f64),
    Json(serde_json::Error),
    Pbf(prost::DecodeError),
    FromPbf(FromPbfError),
    Shape(ShapeError),
    Parquet(ParquetError),
    Arrow(ArrowError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingMetadata => write!(f, "the file has no geodisplay metadata"),
            Error::BothMetadataForms => write!(
                f,
                "the file defines geodisplay both at the root and in Spark field metadata"
            ),
            Error::MultipleIndexes => write!(
                f,
                "the Spark schema puts a display index on more than one field"
            ),
            Error::MissingColumn(column) => write!(f, "column `{column}` not found"),
            Error::ColumnType { column, expected } => {
                write!(
                    f,
                    "the decoder expects {expected} values in column `{column}`"
                )
            }
            Error::NoLevels => write!(f, "the XZ index has no multiscale levels"),
            Error::Resolution(resolution) => write!(
                f,
                "resolution must be a positive finite number, found {resolution}"
            ),
            Error::Json(e) => write!(f, "invalid geodisplay metadata: {e}"),
            Error::Pbf(e) => write!(f, "invalid esriPBF geometry: {e}"),
            Error::FromPbf(e) => write!(f, "{e}"),
            Error::Shape(e) => write!(f, "{e}"),
            Error::Parquet(e) => write!(f, "{e}"),
            Error::Arrow(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

impl From<prost::DecodeError> for Error {
    fn from(e: prost::DecodeError) -> Self {
        Error::Pbf(e)
    }
}

impl From<FromPbfError> for Error {
    fn from(e: FromPbfError) -> Self {
        Error::FromPbf(e)
    }
}

impl From<ShapeError> for Error {
    fn from(e: ShapeError) -> Self {
        Error::Shape(e)
    }
}

impl From<ParquetError> for Error {
    fn from(e: ParquetError) -> Self {
        Error::Parquet(e)
    }
}

impl From<ArrowError> for Error {
    fn from(e: ArrowError) -> Self {
        Error::Arrow(e)
    }
}

impl From<Error> for ArrowError {
    fn from(e: Error) -> Self {
        ArrowError::ExternalError(Box::new(e))
    }
}
