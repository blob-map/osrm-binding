// osrm/src/lib.rs

use std::borrow::Borrow;

use crate::errors::OsrmError;
use crate::point::Point;
use crate::route::{RouteRequest, RouteResponse, SimpleRouteResponse};
use crate::tables::{TableRequest, TableResponse};
use crate::trip::{TripRequest, TripResponse};
use crate::{algorithm, Osrm};

pub struct OsrmEngine {
    instance: Osrm,
}

impl OsrmEngine {
    pub fn new(base_path: &str, algorithm: algorithm::Algorithm) -> Result<Self, OsrmError> {
        let osrm =
            Osrm::new(base_path, algorithm.as_str()).map_err(|_| OsrmError::Initialization)?;
        Ok(OsrmEngine { instance: osrm })
    }

    pub fn table(
        &self,
        table_request: impl Borrow<TableRequest>,
    ) -> Result<TableResponse, OsrmError> {
        let table_request = table_request.borrow();
        let len_sources = table_request.sources.len();
        let len_destinations = table_request.destinations.len();
        if len_sources == 0 || len_destinations == 0 {
            return Err(OsrmError::InvalidTableArgument);
        }
        let mut coordinates = Vec::with_capacity((len_sources + len_destinations) * 2);
        for point in table_request
            .sources
            .iter()
            .chain(&table_request.destinations)
        {
            coordinates.extend_from_slice(&[point.longitude, point.latitude]);
        }
        let result = self
            .instance
            .table(&coordinates, len_sources, len_destinations)
            .map_err(OsrmError::FfiError)?;
        serde_json::from_str::<TableResponse>(&result).map_err(OsrmError::JsonParse)
    }

    pub fn route(
        &self,
        route_request: impl Borrow<RouteRequest>,
    ) -> Result<RouteResponse, OsrmError> {
        let route_request = route_request.borrow();
        let len = route_request.points.len();
        if len == 0 {
            return Err(OsrmError::InvalidTableArgument);
        }
        let mut coordinates = Vec::with_capacity(len * 2);
        for point in &route_request.points {
            coordinates.extend_from_slice(&[point.longitude, point.latitude]);
        }
        let result = self
            .instance
            .route(&coordinates)
            .map_err(OsrmError::FfiError)?;
        serde_json::from_str::<RouteResponse>(&result).map_err(OsrmError::JsonParse)
    }

    pub fn trip(&self, trip_request: impl Borrow<TripRequest>) -> Result<TripResponse, OsrmError> {
        let trip_request = trip_request.borrow();
        let len = trip_request.points.len();
        if len < 2 {
            return Err(OsrmError::InvalidTripArgument);
        }
        let mut coordinates = Vec::with_capacity(len * 2);
        for point in &trip_request.points {
            coordinates.extend_from_slice(&[point.longitude, point.latitude]);
        }
        let result = self
            .instance
            .trip(&coordinates)
            .map_err(OsrmError::FfiError)?;
        serde_json::from_str::<TripResponse>(&result).map_err(OsrmError::JsonParse)
    }

    pub fn simple_route(&self, from: Point, to: Point) -> Result<SimpleRouteResponse, OsrmError> {
        let (distance, duration) = self
            .instance
            .simple_route(from.longitude, from.latitude, to.longitude, to.latitude)
            .map_err(OsrmError::FfiError)?;
        Ok(SimpleRouteResponse {
            code: "Ok".to_owned(),
            distance,
            durations: duration,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import OsrmEngine, TableRequest, etc.
    use crate::algorithm::Algorithm;
    use crate::route::RouteRequestBuilder;
    use crate::tables::Point;
    use crate::trip::TripRequest;
    #[test]
    fn it_calculates_a_table_successfully() {
        dotenvy::dotenv().expect(".env file could not be read");
        let path = std::env::var("OSRM_TEST_DATA_PATH_MLD").expect(
            "Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map",
        );
        let engine =
            OsrmEngine::new(&path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

        let request = TableRequest {
            sources: vec![
                Point {
                    longitude: 2.3522,
                    latitude: 48.8566,
                }, // Paris
            ],
            destinations: vec![
                Point {
                    longitude: 5.3698,
                    latitude: 43.2965,
                }, // Marseille
                Point {
                    longitude: 4.8357,
                    latitude: 45.7640,
                }, // Lyon
            ],
        };
        let response = engine.table(request).expect("Table request failed");

        println!("{:?}", response.durations);

        assert_eq!(response.code, "Ok");
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
        dotenvy::dotenv().expect(".env file could not be read");
        let path = std::env::var("OSRM_TEST_DATA_PATH_MLD").expect(
            "Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map",
        );
        let engine =
            OsrmEngine::new(&path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

        let request = RouteRequestBuilder::default()
            .points(vec![
                Point {
                    longitude: 2.3522,
                    latitude: 48.8566,
                },
                Point {
                    longitude: 5.3698,
                    latitude: 43.2965,
                },
            ])
            .build()
            .expect("Failed to build RouteRequest");
        let response = engine.route(request).expect("route request failed");

        let duration = response
            .routes
            .first()
            .unwrap()
            .legs
            .first()
            .unwrap()
            .duration;
        let distance = response
            .routes
            .first()
            .unwrap()
            .legs
            .first()
            .unwrap()
            .distance
            / 1000.0; // kilometer
        assert_eq!(response.code, "Ok");
        assert_eq!(response.routes.len(), 1, "Should have 1 row for 1 route");
        assert!(700.0 < distance && distance < 800.0); // between 700 and 800 km (google map used)
        assert!(27000.0 < duration && duration < 30600.0); // between 7h30 and 8h30 (google map used)
    }

    #[test]
    fn it_calculates_a_simple_route_successfully() {
        dotenvy::dotenv().expect(".env file could not be read");
        let path = std::env::var("OSRM_TEST_DATA_PATH_MLD").expect(
            "Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map",
        );
        let engine =
            OsrmEngine::new(&path, Algorithm::MLD).expect("Failed to initialize OSRM engine");
        let response = engine
            .simple_route(
                Point {
                    longitude: 2.3522,
                    latitude: 48.8566,
                },
                Point {
                    longitude: 5.3698,
                    latitude: 43.2965,
                },
            )
            .expect("route request failed");
        assert_eq!(response.code, "Ok");
        println!("{:?}", response);
        assert!(700.0 < (response.distance / 1000.0) && (response.distance / 1000.0) < 800.0); // between 700 and 800 km (google map used)
        assert!(27000.0 < response.durations && response.durations < 30600.0); // between 7h30 and 8h30 (google map used)
    }

    #[test]
    fn it_calculates_a_bicycle_trip_successfully() {
        dotenvy::dotenv().expect(".env file could not be read");
        let request = TripRequest {
            points: vec![
                Point {
                    longitude: 2.2945,
                    latitude: 48.8584,
                }, // Eiffel Tower
                Point {
                    longitude: 2.3364,
                    latitude: 48.8606,
                }, // Louvre Museum
                Point {
                    longitude: 2.3690,
                    latitude: 48.8530,
                }, // Place de la Bastille
            ],
        };

        for (path_variable, algorithm) in [
            ("OSRM_TEST_DATA_PATH_BICYCLE_MLD", Algorithm::MLD),
            ("OSRM_TEST_DATA_PATH_BICYCLE_CH", Algorithm::CH),
        ] {
            let path = std::env::var(path_variable).unwrap_or_else(|_| {
                panic!(
                    "Environment variable {path_variable} must be defined with a French bicycle map"
                )
            });
            let engine =
                OsrmEngine::new(&path, algorithm).expect("Failed to initialize bicycle engine");
            let response = engine.trip(&request).expect("Bicycle trip request failed");

            assert_eq!(response.code, "Ok");
            assert_eq!(response.trips.len(), 1);
            assert_eq!(response.waypoints.len(), request.points.len());

            let trip = &response.trips[0];
            assert_eq!(trip.legs.len(), request.points.len());
            assert!(trip.distance > 0.0);
            assert!(trip.duration > 0.0);

            let mut optimized_positions = response
                .waypoints
                .iter()
                .map(|waypoint| waypoint.waypoint_index)
                .collect::<Vec<_>>();
            optimized_positions.sort_unstable();
            assert_eq!(optimized_positions, vec![0, 1, 2]);
            assert!(response
                .waypoints
                .iter()
                .all(|waypoint| waypoint.trips_index == 0));
        }
    }
}
