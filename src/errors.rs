use thiserror::Error;

#[derive(Error, Debug)]
pub enum OsrmError {
    #[error("Failed to create OSRM instance")]
    Initialization,
    #[error("Invalid path parameter: {0}")]
    InvalidPath(String),
    #[error("OSRM API error: {0}")]
    ApiError(String),
    #[error("Sources or destinations are invalid")]
    InvalidTableArgument,
    #[error("A trip requires at least two points")]
    InvalidTripArgument,
    #[error("Failed to parse OSRM response: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("Internal FFI error: {0}")]
    FfiError(String),
}
