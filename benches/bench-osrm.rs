// benches/my_bench.rs
use criterion::{Criterion, criterion_group, criterion_main};
use dotenvy::dotenv;
use std::env;
use rand::Rng;
use osrm_binding::algorithm::Algorithm;
use osrm_binding::osrm_engine::OsrmEngine;
use osrm_binding::point::Point;
use osrm_binding::route::RouteRequestBuilder;
use osrm_binding::tables::TableRequest;

fn calculate_table_successfully(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");
    let path = env::var("OSRM_TEST_DATA_PATH_MLD")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map");
    let engine = OsrmEngine::new(&*path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

    let request = TableRequest {
        sources: vec![
            Point { longitude: 2.3522, latitude: 48.8566 } // Paris
        ],
        destinations: vec![
            Point { longitude: 5.3698, latitude: 43.2965 }, // Marseille
            Point { longitude: 4.8357, latitude: 45.7640 }  // Lyon
        ]
    };

    c.bench_function("calculate_table_successfully", |b| {
        b.iter(|| {
            let _response = engine.table(request.clone()).expect("Table request failed");
        });
    });
}

fn calculate_route_successfully(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");
    let path = env::var("OSRM_TEST_DATA_PATH_MLD")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map");
    let engine = OsrmEngine::new(&*path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

    let request = RouteRequestBuilder::default()
        .points(vec![Point { longitude: 2.3522, latitude: 48.8566 }, Point { longitude: 5.3698, latitude: 43.2965 }])
        .build()
        .expect("Failed to build RouteRequest");

    c.bench_function("calculate_route_successfully", |b| {
        b.iter(|| {
            let _response = engine.route(request.clone()).expect("Route request failed");
        });
    });
}

fn calculate_simple_route_successfully(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");
    let path = env::var("OSRM_TEST_DATA_PATH_MLD")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map");
    let engine = OsrmEngine::new(&*path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

    let start = Point { longitude: 2.3522, latitude: 48.8566 };
    let end = Point { longitude: 5.3698, latitude: 43.2965 };

    c.bench_function("calculate_simple_route_successfully", |b| {
        b.iter(|| {
            let _response = engine.simple_route(start.clone(), end.clone()).expect("Simple route request failed");
        });
    });
}

fn calculate_table_10_successfully_mld(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");
    let path = env::var("OSRM_TEST_DATA_PATH_MLD")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map");
    let engine = OsrmEngine::new(&*path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

    let base_lat = 48.8566;
    let base_lon = 2.3522;
    let mut rng = rand::rng();

    c.bench_function("calculate_table_10_successfully_mld", |b| {
        b.iter(|| {
            let request = TableRequest {
                sources: vec![
                    Point { longitude: base_lon, latitude: base_lat }
                ],
                destinations: (0..10).map( |_| Point { longitude: base_lon + rng.random_range(-0.1..0.1), latitude: base_lat + rng.random_range(-0.1..0.1) }).collect(),
            };

            let _response = engine.table(request.clone()).expect("Table request failed");
        });
    });
}

fn calculate_table_10_successfully_ch(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");
    let path = env::var("OSRM_TEST_DATA_PATH_CH")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map");
    let engine = OsrmEngine::new(&*path, Algorithm::CH).expect("Failed to initialize OSRM engine");

    let base_lat = 48.8566;
    let base_lon = 2.3522;
    let mut rng = rand::rng();

    c.bench_function("calculate_table_10_successfully_ch", |b| {
        b.iter(|| {
            let request = TableRequest {
                sources: vec![
                    Point { longitude: base_lon, latitude: base_lat }
                ],
                destinations: (0..10).map( |_| Point { longitude: base_lon + rng.random_range(-0.1..0.1), latitude: base_lat + rng.random_range(-0.1..0.1) }).collect(),
            };

            let _response = engine.table(request.clone()).expect("Table request failed");
        });
    });
}

fn calculate_table_100_successfully_mld(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");
    let path = env::var("OSRM_TEST_DATA_PATH_MLD")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map");
    let engine = OsrmEngine::new(&*path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

    let base_lat = 48.8566;
    let base_lon = 2.3522;
    let mut rng = rand::rng();

    c.bench_function("calculate_table_100_successfully_mld", |b| {
        b.iter(|| {
            let request = TableRequest {
                sources: vec![
                    Point { longitude: base_lon, latitude: base_lat }
                ],
                destinations: (0..100).map( |_| Point { longitude: base_lon + rng.random_range(-0.1..0.1), latitude: base_lat + rng.random_range(-0.1..0.1) }).collect(),
            };

            let _response = engine.table(request.clone()).expect("Table request failed");
        });
    });
}

fn calculate_table_100_successfully_ch(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");
    let path = env::var("OSRM_TEST_DATA_PATH_CH")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map");
    let engine = OsrmEngine::new(&*path, Algorithm::CH).expect("Failed to initialize OSRM engine");

    let base_lat = 48.8566;
    let base_lon = 2.3522;
    let mut rng = rand::rng();

    c.bench_function("calculate_table_100_successfully_ch", |b| {
        b.iter(|| {
            let request = TableRequest {
                sources: vec![
                    Point { longitude: base_lon, latitude: base_lat }
                ],
                destinations: (0..100).map( |_| Point { longitude: base_lon + rng.random_range(-0.1..0.1), latitude: base_lat + rng.random_range(-0.1..0.1) }).collect(),
            };

            let _response = engine.table(request.clone()).expect("Table request failed");
        });
    });
}
fn calculate_multiple_routes_around_paris_10km_mld(c: &mut Criterion) {
    use rand::Rng;
    dotenv().expect(".env file could not be read");

    let path = env::var("OSRM_TEST_DATA_PATH_MLD")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a French map");
    let engine = OsrmEngine::new(&*path, Algorithm::MLD)
        .expect("Failed to initialize OSRM engine");

    let base_lat = 48.8566;
    let base_lon = 2.3522;
    let mut rng = rand::rng();

    c.bench_function("calculate_multiple_routes_around_paris_10km_mld", |b| {
        b.iter(|| {
            let start = Point {
                latitude: base_lat + rng.random_range(-0.1..0.1),
                longitude: base_lon + rng.random_range(-0.1..0.1),
            };
            let end = Point {
                latitude: base_lat + rng.random_range(-0.1..0.1),
                longitude: base_lon + rng.random_range(-0.1..0.1),
            };

            let _response = engine.simple_route(start, end)
                .ok();
        });
    });
}

fn calculate_multiple_routes_around_paris_100km_mld(c: &mut Criterion) {
    use rand::Rng;
    dotenv().expect(".env file could not be read");

    let path = env::var("OSRM_TEST_DATA_PATH_MLD")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a French map");
    let engine = OsrmEngine::new(&*path, Algorithm::MLD)
        .expect("Failed to initialize OSRM engine");

    let base_lat = 48.8566;
    let base_lon = 2.3522;
    let mut rng = rand::rng();

    c.bench_function("calculate_multiple_routes_around_paris_100km_mld", |b| {
        b.iter(|| {
            let start = Point {
                latitude: base_lat + rng.random_range(-1..1) as f64,
                longitude: base_lon + rng.random_range(-1..1) as f64,
            };
            let end = Point {
                latitude: base_lat + rng.random_range(-1..1) as f64,
                longitude: base_lon + rng.random_range(-1..1) as f64,
            };

            let _response = engine.simple_route(start, end)
                .ok();
        });
    });
}

fn calculate_multiple_routes_around_paris_10km_ch(c: &mut Criterion) {
    use rand::Rng;
    dotenv().expect(".env file could not be read");

    let path = env::var("OSRM_TEST_DATA_PATH_CH")
        .expect("Environment variable OSRM_TEST_DATA_PATH_CH must be defined with a French map");
    let engine = OsrmEngine::new(&*path, Algorithm::CH)
        .expect("Failed to initialize OSRM engine");

    let base_lat = 48.8566;
    let base_lon = 2.3522;
    let mut rng = rand::rng();

    c.bench_function("calculate_multiple_routes_around_paris_10km_ch", |b| {
        b.iter(|| {
            let start = Point {
                latitude: base_lat + rng.random_range(-0.1..0.1),
                longitude: base_lon + rng.random_range(-0.1..0.1),
            };
            let end = Point {
                latitude: base_lat + rng.random_range(-0.1..0.1),
                longitude: base_lon + rng.random_range(-0.1..0.1),
            };

            let _response = engine.simple_route(start, end)
                .ok();
        });
    });
}

fn calculate_multiple_routes_around_paris_100km_ch(c: &mut Criterion) {
    use rand::Rng;
    dotenv().expect(".env file could not be read");

    let path = env::var("OSRM_TEST_DATA_PATH_CH")
        .expect("Environment variable OSRM_TEST_DATA_PATH_CH must be defined with a French map");
    let engine = OsrmEngine::new(&*path, Algorithm::CH)
        .expect("Failed to initialize OSRM engine");

    let base_lat = 48.8566;
    let base_lon = 2.3522;
    let mut rng = rand::rng();

    c.bench_function("calculate_multiple_routes_around_paris_100km_ch", |b| {
        b.iter(|| {
            let start = Point {
                latitude: base_lat + rng.random_range(-1..1) as f64,
                longitude: base_lon + rng.random_range(-1..1) as f64,
            };
            let end = Point {
                latitude: base_lat + rng.random_range(-1..1) as f64,
                longitude: base_lon + rng.random_range(-1..1) as f64,
            };

            let _response = engine.simple_route(start, end)
                .ok();
        });
    });
}

// Define the benchmark group
criterion_group!(benches,
    calculate_table_successfully,
    calculate_route_successfully,
    calculate_simple_route_successfully,
    calculate_table_10_successfully_mld,
    calculate_table_10_successfully_ch,
    calculate_table_100_successfully_mld,
    calculate_table_100_successfully_ch,
    calculate_multiple_routes_around_paris_10km_mld,
    calculate_multiple_routes_around_paris_100km_mld,
    calculate_multiple_routes_around_paris_10km_ch,
    calculate_multiple_routes_around_paris_100km_ch);

// Set the main function to run the benchmarks
criterion_main!(benches);
