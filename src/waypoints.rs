use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct Waypoint {
    pub hint: String,
    pub location: [f64; 2],
    pub name: String,
    pub distance: f64,
}
