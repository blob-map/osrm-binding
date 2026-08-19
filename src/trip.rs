use crate::point::Point;
use crate::route::Route;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// Controls where an optimized trip starts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TripSource {
    /// Let OSRM choose the starting waypoint.
    #[default]
    Any,
    /// Keep the first request point as the starting waypoint.
    First,
}

/// Controls where an optimized trip ends.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TripDestination {
    /// Let OSRM choose the ending waypoint.
    #[default]
    Any,
    /// Keep the last request point as the ending waypoint.
    Last,
}

/// A request to optimize the visiting order of multiple points.
#[derive(Debug, Builder, Clone, PartialEq)]
#[builder(pattern = "owned")]
pub struct TripRequest {
    points: Vec<Point>,
    #[builder(default = "true")]
    roundtrip: bool,
    #[builder(default)]
    source: TripSource,
    #[builder(default)]
    destination: TripDestination,
    #[builder(default)]
    steps: bool,
}

impl TripRequest {
    /// Creates a closed round trip with OSRM's default start and no detailed steps.
    pub fn new(points: Vec<Point>) -> Self {
        Self {
            points,
            roundtrip: true,
            source: TripSource::Any,
            destination: TripDestination::Any,
            steps: false,
        }
    }

    /// Creates a builder for round-trip, endpoint, and detailed-step options.
    pub fn builder() -> TripRequestBuilder {
        TripRequestBuilder::default()
    }

    pub fn points(&self) -> &[Point] {
        &self.points
    }

    pub const fn roundtrip(&self) -> bool {
        self.roundtrip
    }

    pub const fn source(&self) -> TripSource {
        self.source
    }

    pub const fn destination(&self) -> TripDestination {
        self.destination
    }

    pub const fn steps(&self) -> bool {
        self.steps
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct TripResponse {
    pub code: String,
    pub trips: Vec<Route>,
    pub waypoints: Vec<TripWaypoint>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct TripWaypoint {
    pub hint: String,
    pub location: [f64; 2],
    pub name: String,
    pub distance: f64,
    pub trips_index: usize,
    pub waypoint_index: usize,
}
