use crate::point::Point;
use crate::route::Route;
use derive_builder::Builder;
use serde::Deserialize;

#[derive(Debug, Builder, Clone)]
pub struct TripRequest {
    pub points: Vec<Point>,
}

#[derive(Debug, Deserialize)]
pub struct TripResponse {
    pub code: String,
    pub trips: Vec<Route>,
    pub waypoints: Vec<TripWaypoint>,
}

#[derive(Debug, Deserialize)]
pub struct TripWaypoint {
    pub hint: String,
    pub location: [f64; 2],
    pub name: String,
    pub distance: f64,
    pub trips_index: usize,
    pub waypoint_index: usize,
}
