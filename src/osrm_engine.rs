use std::borrow::Borrow;

use crate::errors::OsrmError;
use crate::point::Point;
use crate::route::{RouteRequest, RouteResponse, SimpleRouteResponse};
use crate::tables::{TableRequest, TableResponse};
use crate::trip::{TripDestination, TripRequest, TripResponse, TripSource};
use crate::{Algorithm, Osrm};

/// An in-process OSRM engine loaded from one preprocessed dataset.
pub struct OsrmEngine {
    instance: Osrm,
}

impl OsrmEngine {
    /// Loads a dataset by base path, without the `.osrm` suffix.
    ///
    /// `algorithm` must match the pipeline used to prepare the dataset.
    pub fn new(base_path: &str, algorithm: Algorithm) -> Result<Self, OsrmError> {
        let osrm = Osrm::new(base_path, algorithm).map_err(|_| OsrmError::Initialization)?;
        Ok(OsrmEngine { instance: osrm })
    }

    /// Calculates a source-by-destination duration matrix.
    pub fn table(
        &self,
        table_request: impl Borrow<TableRequest>,
    ) -> Result<TableResponse, OsrmError> {
        let table_request = table_request.borrow();
        let len_sources = table_request.sources().len();
        let len_destinations = table_request.destinations().len();
        if len_sources == 0 || len_destinations == 0 {
            return Err(OsrmError::InvalidTableArgument);
        }
        let mut coordinates = Vec::with_capacity((len_sources + len_destinations) * 2);
        for point in table_request
            .sources()
            .iter()
            .chain(table_request.destinations())
        {
            coordinates.extend_from_slice(&[point.longitude, point.latitude]);
        }
        let result = self
            .instance
            .table(&coordinates, len_sources, len_destinations)
            .map_err(OsrmError::FfiError)?;
        serde_json::from_str::<TableResponse>(&result).map_err(OsrmError::JsonParse)
    }

    /// Calculates a route through the request points in their given order.
    pub fn route(
        &self,
        route_request: impl Borrow<RouteRequest>,
    ) -> Result<RouteResponse, OsrmError> {
        let route_request = route_request.borrow();
        let len = route_request.points().len();
        if len < 2 {
            return Err(OsrmError::InvalidRouteArgument);
        }
        let mut coordinates = Vec::with_capacity(len * 2);
        for point in route_request.points() {
            coordinates.extend_from_slice(&[point.longitude, point.latitude]);
        }
        let result = self
            .instance
            .route(&coordinates, route_request.steps())
            .map_err(OsrmError::FfiError)?;
        serde_json::from_str::<RouteResponse>(&result).map_err(OsrmError::JsonParse)
    }

    /// Optimizes the order in which the request points are visited.
    pub fn trip(&self, trip_request: impl Borrow<TripRequest>) -> Result<TripResponse, OsrmError> {
        let trip_request = trip_request.borrow();
        let len = trip_request.points().len();
        if len < 2 {
            return Err(OsrmError::InvalidTripArgument(
                "at least two points are required".to_owned(),
            ));
        }
        if !trip_request.roundtrip()
            && trip_request.source() == TripSource::Any
            && trip_request.destination() == TripDestination::Any
        {
            return Err(OsrmError::InvalidTripArgument(
                "a non-roundtrip requires a fixed first source or last destination".to_owned(),
            ));
        }
        let mut coordinates = Vec::with_capacity(len * 2);
        for point in trip_request.points() {
            coordinates.extend_from_slice(&[point.longitude, point.latitude]);
        }
        let result = self
            .instance
            .trip(
                &coordinates,
                trip_request.roundtrip(),
                trip_request.source() == TripSource::First,
                trip_request.destination() == TripDestination::Last,
                trip_request.steps(),
            )
            .map_err(OsrmError::FfiError)?;
        serde_json::from_str::<TripResponse>(&result).map_err(OsrmError::JsonParse)
    }

    /// Calculates only total distance and duration for two points.
    ///
    /// This avoids allocating and parsing the full route geometry and waypoint response.
    pub fn simple_route(&self, from: Point, to: Point) -> Result<SimpleRouteResponse, OsrmError> {
        let (distance, duration) = self
            .instance
            .simple_route(from.longitude, from.latitude, to.longitude, to.latitude)
            .map_err(OsrmError::FfiError)?;
        Ok(SimpleRouteResponse { distance, duration })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RouteRequest, TableRequest, TripDestination, TripRequest, TripSource};

    fn point(longitude: f64, latitude: f64) -> Point {
        Point {
            longitude,
            latitude,
        }
    }

    fn test_data_path(variable: &str) -> String {
        dotenvy::dotenv().expect(".env file could not be read");
        std::env::var(variable).unwrap_or_else(|_| {
            panic!("Environment variable {variable} must point to a prepared France dataset")
        })
    }

    fn assert_send_and_sync<T: Send + Sync>() {}

    #[test]
    fn engine_is_send_and_sync() {
        assert_send_and_sync::<OsrmEngine>();
    }

    #[test]
    fn it_calculates_a_table_successfully() {
        let path = test_data_path("OSRM_TEST_DATA_PATH_MLD");
        let engine =
            OsrmEngine::new(&path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

        let request = TableRequest::new(
            vec![point(2.3522, 48.8566)], // Paris
            vec![
                point(5.3698, 43.2965), // Marseille
                point(4.8357, 45.7640), // Lyon
            ],
        );
        let response = engine.table(request).expect("Table request failed");

        assert_eq!(response.code, "Ok");
        assert_eq!(response.sources.len(), 1);
        assert_eq!(response.destinations.len(), 2);
        assert_eq!(
            response.durations.len(),
            1,
            "Should have 1 row for 1 source"
        );
        assert_eq!(
            response.durations[0].len(),
            2,
            "Should have 2 columns for 2 destinations"
        );
        assert!(
            response.durations[0][0].is_some(),
            "Paris-Marseille duration should exist"
        );
        assert!(
            response.durations[0][1].is_some(),
            "Paris-Lyon duration should exist"
        );
    }

    #[test]
    fn it_calculates_a_route_successfully() {
        let path = test_data_path("OSRM_TEST_DATA_PATH_MLD");
        let engine =
            OsrmEngine::new(&path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

        let request = RouteRequest::new(vec![point(2.3522, 48.8566), point(5.3698, 43.2965)]);
        let response = engine.route(request).expect("route request failed");

        let leg = &response.routes[0].legs[0];
        let duration = leg.duration;
        let distance = leg.distance / 1000.0;
        assert_eq!(response.code, "Ok");
        assert_eq!(response.routes.len(), 1);
        assert!(leg.steps.is_empty(), "steps should be opt-in");
        assert!(700.0 < distance && distance < 800.0);
        assert!(27000.0 < duration && duration < 30600.0);

        let request_with_steps = RouteRequest::builder()
            .points(vec![
                point(2.2945, 48.8584), // Eiffel Tower
                point(2.3364, 48.8606), // Louvre Museum
            ])
            .steps(true)
            .build()
            .expect("Failed to build route request");
        let response = engine
            .route(request_with_steps)
            .expect("Route request with steps failed");
        let first_step = &response.routes[0].legs[0].steps[0];
        assert!(!first_step.mode.is_empty());
        assert!(!first_step.maneuver.kind.is_empty());
    }

    #[test]
    fn it_calculates_a_simple_route_successfully() {
        let path = test_data_path("OSRM_TEST_DATA_PATH_MLD");
        let engine =
            OsrmEngine::new(&path, Algorithm::MLD).expect("Failed to initialize OSRM engine");
        let response = engine
            .simple_route(point(2.3522, 48.8566), point(5.3698, 43.2965))
            .expect("route request failed");
        assert!(700.0 < (response.distance / 1000.0) && (response.distance / 1000.0) < 800.0);
        assert!(27000.0 < response.duration && response.duration < 30600.0);
    }

    #[test]
    fn it_calculates_a_bicycle_trip_successfully() {
        dotenvy::dotenv().expect(".env file could not be read");
        let points = vec![
            point(2.2945, 48.8584), // Eiffel Tower
            point(2.3364, 48.8606), // Louvre Museum
            point(2.3690, 48.8530), // Place de la Bastille
        ];
        let roundtrip_request = TripRequest::builder()
            .points(points.clone())
            .steps(true)
            .build()
            .expect("Failed to build roundtrip request");
        let open_trip_request = TripRequest::builder()
            .points(points)
            .roundtrip(false)
            .source(TripSource::First)
            .destination(TripDestination::Last)
            .build()
            .expect("Failed to build open trip request");

        for (path_variable, algorithm) in [
            ("OSRM_TEST_DATA_PATH_BICYCLE_MLD", Algorithm::MLD),
            ("OSRM_TEST_DATA_PATH_BICYCLE_CH", Algorithm::CH),
        ] {
            let path = test_data_path(path_variable);
            let engine =
                OsrmEngine::new(&path, algorithm).expect("Failed to initialize bicycle engine");
            let response = engine
                .trip(&roundtrip_request)
                .expect("Bicycle roundtrip request failed");

            assert_eq!(response.code, "Ok");
            assert_eq!(response.trips.len(), 1);
            assert_eq!(response.waypoints.len(), roundtrip_request.points().len());

            let trip = &response.trips[0];
            assert_eq!(trip.legs.len(), roundtrip_request.points().len());
            assert!(trip.distance > 0.0);
            assert!(trip.duration > 0.0);
            assert!(trip.legs.iter().all(|leg| !leg.steps.is_empty()));

            let mut optimized_positions = response
                .waypoints
                .iter()
                .map(|waypoint| waypoint.waypoint_index)
                .collect::<Vec<_>>();
            optimized_positions.sort_unstable();
            assert_eq!(optimized_positions, vec![0, 1, 2]);
            assert!(
                response
                    .waypoints
                    .iter()
                    .all(|waypoint| waypoint.trips_index == 0)
            );

            let open_response = engine
                .trip(&open_trip_request)
                .expect("Open bicycle trip request failed");
            assert_eq!(open_response.trips[0].legs.len(), 2);
            assert_eq!(open_response.waypoints[0].waypoint_index, 0);
            assert_eq!(open_response.waypoints[2].waypoint_index, 2);

            let unsupported_request = TripRequest::builder()
                .points(roundtrip_request.points().to_vec())
                .roundtrip(false)
                .build()
                .expect("Failed to build unsupported trip request");
            assert!(matches!(
                engine.trip(unsupported_request),
                Err(OsrmError::InvalidTripArgument(_))
            ));
        }
    }
}
