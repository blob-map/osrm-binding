use crate::point::Point;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// A duration-matrix request.
#[derive(Debug, Builder, Clone, PartialEq)]
#[builder(pattern = "owned")]
pub struct TableRequest {
    sources: Vec<Point>,
    destinations: Vec<Point>,
}

impl TableRequest {
    /// Creates a matrix request with sources as rows and destinations as columns.
    pub fn new(sources: Vec<Point>, destinations: Vec<Point>) -> Self {
        Self {
            sources,
            destinations,
        }
    }

    pub fn builder() -> TableRequestBuilder {
        TableRequestBuilder::default()
    }

    pub fn sources(&self) -> &[Point] {
        &self.sources
    }

    pub fn destinations(&self) -> &[Point] {
        &self.destinations
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct TableResponse {
    pub code: String,
    pub destinations: Vec<TableLocation>,
    pub durations: Vec<Vec<Option<f64>>>,
    pub sources: Vec<TableLocation>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct TableLocation {
    pub hint: String,
    pub location: [f64; 2],
    pub name: String,
    pub distance: f64,
}
