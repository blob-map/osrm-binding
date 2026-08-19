use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum OsrmError {
    #[error("Failed to create OSRM instance")]
    Initialization,
    #[error("Sources or destinations are invalid")]
    InvalidTableArgument,
    #[error("A route requires at least two points")]
    InvalidRouteArgument,
    #[error("Invalid trip request: {0}")]
    InvalidTripArgument(String),
    #[error("Failed to parse OSRM response: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("Internal FFI error: {0}")]
    FfiError(String),
}
