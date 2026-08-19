use crate::point::Point;
use crate::waypoints::Waypoint;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
#[derive(Debug, Builder, Clone)]
pub struct RouteRequest {
    pub points: Vec<Point>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SimpleRouteResponse {
    pub code: String,
    pub durations: f64,
    pub distance: f64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RouteResponse {
    pub code: String,
    pub routes: Vec<Route>,
    pub waypoints: Vec<Waypoint>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OsrmResponse {
    pub code: String,
    pub routes: Vec<Route>,
    pub waypoints: Vec<Waypoint>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Route {
    pub legs: Vec<Leg>,
    pub weight_name: String,
    pub geometry: String,
    pub weight: f64,
    pub duration: f64,
    pub distance: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Leg {
    pub steps: Vec<Step>,
    pub weight: f64,
    pub summary: String,
    pub duration: f64,
    pub distance: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Step {}
