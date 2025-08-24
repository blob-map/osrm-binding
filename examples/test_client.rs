use osrm_binding::algorithm::Algorithm;
use osrm_binding::osrm_engine::OsrmEngine;
use osrm_binding::point::Point;
use osrm_binding::tables::TableRequest;
use std::time::Instant;

fn main() {
    dotenvy::dotenv().expect(".env file could not be read");
    let path = std::env::var("OSRM_TEST_DATA_PATH_MLD")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map");
    let engine = OsrmEngine::new(&*path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

    let start_time = Instant::now();
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
    let table_response = engine.table(request).expect("Table request failed");
    println!("{:#?}", table_response);
    let duration = start_time.elapsed();
    println!("Time taken for table: {:?}", duration);

    let start_time = Instant::now();
    let route_response_marseille = engine
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
    println!("{:#?}", route_response_marseille);
    let duration = start_time.elapsed();
    println!("Time taken for route to marseille: {:?}", duration);

    let start_time = Instant::now();
    let route_response_paris = engine
        .simple_route(
            Point {
                longitude: 2.3522,
                latitude: 48.8566,
            },
            Point {
                longitude: 4.8357,
                latitude: 45.7640,
            },
        )
        .expect("route request failed");
    println!("{:#?}", route_response_paris);
    let duration = start_time.elapsed();
    println!("Time taken for route to Paris: {:?}", duration);
}
