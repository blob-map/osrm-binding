use crate::point::Point;
use crate::waypoints::Waypoint;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// A route request whose points are visited in their given order.
#[derive(Debug, Builder, Clone, PartialEq)]
#[builder(pattern = "owned")]
pub struct RouteRequest {
    points: Vec<Point>,
    #[builder(default)]
    steps: bool,
}

impl RouteRequest {
    /// Creates a request without detailed turn-by-turn steps.
    pub fn new(points: Vec<Point>) -> Self {
        Self {
            points,
            steps: false,
        }
    }

    /// Creates a builder for optional settings such as detailed steps.
    pub fn builder() -> RouteRequestBuilder {
        RouteRequestBuilder::default()
    }

    pub fn points(&self) -> &[Point] {
        &self.points
    }

    pub const fn steps(&self) -> bool {
        self.steps
    }
}

/// The lightweight result returned by [`crate::OsrmEngine::simple_route`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct SimpleRouteResponse {
    pub duration: f64,
    pub distance: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct RouteResponse {
    pub code: String,
    pub routes: Vec<Route>,
    pub waypoints: Vec<Waypoint>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct Route {
    pub legs: Vec<Leg>,
    pub weight_name: String,
    pub geometry: String,
    pub weight: f64,
    pub duration: f64,
    pub distance: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct Leg {
    pub steps: Vec<RouteStep>,
    pub weight: f64,
    pub summary: String,
    pub duration: f64,
    pub distance: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct RouteStep {
    pub distance: f64,
    pub duration: f64,
    pub geometry: String,
    pub weight: f64,
    pub name: String,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
    pub pronunciation: Option<String>,
    pub destinations: Option<String>,
    pub exits: Option<String>,
    pub mode: String,
    pub maneuver: StepManeuver,
    pub intersections: Vec<Intersection>,
    pub rotary_name: Option<String>,
    pub rotary_pronunciation: Option<String>,
    pub driving_side: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct StepManeuver {
    pub location: [f64; 2],
    pub bearing_before: u16,
    pub bearing_after: u16,
    #[serde(rename = "type")]
    pub kind: String,
    pub modifier: Option<String>,
    pub exit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct Intersection {
    pub location: [f64; 2],
    pub bearings: Vec<u16>,
    pub entry: Vec<bool>,
    #[serde(rename = "in")]
    pub incoming: Option<usize>,
    #[serde(rename = "out")]
    pub outgoing: Option<usize>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub lanes: Vec<Lane>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct Lane {
    pub indications: Vec<String>,
    pub valid: bool,
    pub active: Option<bool>,
}
