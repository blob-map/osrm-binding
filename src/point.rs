use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// A WGS84 coordinate in longitude/latitude order.
#[derive(Debug, Clone, Copy, Builder, Deserialize, Serialize, PartialEq)]
pub struct Point {
    pub latitude: f64,
    pub longitude: f64,
}
