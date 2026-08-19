use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use dotenvy::dotenv;
use osrm_binding::{Algorithm, OsrmEngine, Point, RouteRequest, TableRequest};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::env;
use std::hint::black_box;

const ROUTABLE_PAIR_POOL_SIZE: usize = 1_024;

fn routable_pairs_around_paris(engine: &OsrmEngine, maximum_offset: f64) -> Vec<(Point, Point)> {
    let mut rng = StdRng::seed_from_u64(0x05_52_4d);
    let mut pairs = Vec::with_capacity(ROUTABLE_PAIR_POOL_SIZE);

    for _ in 0..ROUTABLE_PAIR_POOL_SIZE * 100 {
        let start = Point {
            latitude: 48.8566 + rng.random_range(-maximum_offset..maximum_offset),
            longitude: 2.3522 + rng.random_range(-maximum_offset..maximum_offset),
        };
        let end = Point {
            latitude: 48.8566 + rng.random_range(-maximum_offset..maximum_offset),
            longitude: 2.3522 + rng.random_range(-maximum_offset..maximum_offset),
        };

        if engine.simple_route(start, end).is_ok() {
            pairs.push((start, end));
            if pairs.len() == ROUTABLE_PAIR_POOL_SIZE {
                return pairs;
            }
        }
    }

    panic!(
        "Only found {} routable pairs out of {} requested",
        pairs.len(),
        ROUTABLE_PAIR_POOL_SIZE
    );
}

fn calculate_table_successfully(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");
    let path = env::var("OSRM_TEST_DATA_PATH_MLD")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map");
    let engine = OsrmEngine::new(&path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

    let request = TableRequest::new(
        vec![Point {
            longitude: 2.3522,
            latitude: 48.8566,
        }], // Paris
        vec![
            Point {
                longitude: 5.3698,
                latitude: 43.2965,
            }, // Marseille
            Point {
                longitude: 4.8357,
                latitude: 45.7640,
            }, // Lyon
        ],
    );

    c.bench_function("calculate_table_successfully", |b| {
        b.iter(|| {
            black_box(
                engine
                    .table(black_box(&request))
                    .expect("Table request failed"),
            );
        });
    });
}

fn calculate_route_successfully(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");
    let path = env::var("OSRM_TEST_DATA_PATH_MLD")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map");
    let engine = OsrmEngine::new(&path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

    let request = RouteRequest::new(vec![
        Point {
            longitude: 2.3522,
            latitude: 48.8566,
        },
        Point {
            longitude: 5.3698,
            latitude: 43.2965,
        },
    ]);

    c.bench_function("calculate_route_successfully", |b| {
        b.iter(|| {
            black_box(
                engine
                    .route(black_box(&request))
                    .expect("Route request failed"),
            );
        });
    });
}

fn calculate_simple_route_successfully(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");
    let path = env::var("OSRM_TEST_DATA_PATH_MLD")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map");
    let engine = OsrmEngine::new(&path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

    let start = Point {
        longitude: 2.3522,
        latitude: 48.8566,
    };
    let end = Point {
        longitude: 5.3698,
        latitude: 43.2965,
    };

    c.bench_function("calculate_simple_route_successfully", |b| {
        b.iter(|| {
            black_box(
                engine
                    .simple_route(black_box(start), black_box(end))
                    .expect("Simple route request failed"),
            );
        });
    });
}

fn calculate_table_10_successfully_mld(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");
    let path = env::var("OSRM_TEST_DATA_PATH_MLD")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map");
    let engine = OsrmEngine::new(&path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

    let base_lat = 48.8566;
    let base_lon = 2.3522;
    let mut rng = rand::rng();

    c.bench_function("calculate_table_10_successfully_mld", |b| {
        b.iter_batched(
            || {
                TableRequest::new(
                    vec![Point {
                        longitude: base_lon,
                        latitude: base_lat,
                    }],
                    (0..10)
                        .map(|_| Point {
                            longitude: base_lon + rng.random_range(-0.1..0.1),
                            latitude: base_lat + rng.random_range(-0.1..0.1),
                        })
                        .collect(),
                )
            },
            |request| {
                black_box(engine.table(&request).expect("Table request failed"));
            },
            BatchSize::SmallInput,
        );
    });
}

fn calculate_table_100_successfully_mld(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");
    let path = env::var("OSRM_TEST_DATA_PATH_MLD")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a french map");
    let engine = OsrmEngine::new(&path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

    let base_lat = 48.8566;
    let base_lon = 2.3522;
    let mut rng = rand::rng();

    c.bench_function("calculate_table_100_successfully_mld", |b| {
        b.iter_batched(
            || {
                TableRequest::new(
                    vec![Point {
                        longitude: base_lon,
                        latitude: base_lat,
                    }],
                    (0..100)
                        .map(|_| Point {
                            longitude: base_lon + rng.random_range(-0.1..0.1),
                            latitude: base_lat + rng.random_range(-0.1..0.1),
                        })
                        .collect(),
                )
            },
            |request| {
                black_box(engine.table(&request).expect("Table request failed"));
            },
            BatchSize::SmallInput,
        );
    });
}
fn calculate_multiple_routes_around_paris_10km_mld(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");

    let path = env::var("OSRM_TEST_DATA_PATH_MLD")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a French map");
    let engine = OsrmEngine::new(&path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

    let pairs = routable_pairs_around_paris(&engine, 0.1);
    let mut pair_index = 0;

    c.bench_function("calculate_multiple_routes_around_paris_10km_mld", |b| {
        b.iter_batched(
            || {
                let pair = pairs[pair_index];
                pair_index = (pair_index + 1) % pairs.len();
                pair
            },
            |(start, end)| {
                black_box(
                    engine
                        .simple_route(start, end)
                        .expect("Simple route request failed"),
                );
            },
            BatchSize::SmallInput,
        );
    });
}

fn calculate_multiple_routes_around_paris_100km_mld(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");

    let path = env::var("OSRM_TEST_DATA_PATH_MLD")
        .expect("Environment variable OSRM_TEST_DATA_PATH_MLD must be defined with a French map");
    let engine = OsrmEngine::new(&path, Algorithm::MLD).expect("Failed to initialize OSRM engine");

    let pairs = routable_pairs_around_paris(&engine, 1.0);
    let mut pair_index = 0;

    c.bench_function("calculate_multiple_routes_around_paris_100km_mld", |b| {
        b.iter_batched(
            || {
                let pair = pairs[pair_index];
                pair_index = (pair_index + 1) % pairs.len();
                pair
            },
            |(start, end)| {
                black_box(
                    engine
                        .simple_route(start, end)
                        .expect("Simple route request failed"),
                );
            },
            BatchSize::SmallInput,
        );
    });
}

fn calculate_multiple_routes_around_paris_10km_ch(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");

    let path = env::var("OSRM_TEST_DATA_PATH_CH")
        .expect("Environment variable OSRM_TEST_DATA_PATH_CH must be defined with a French map");
    let engine = OsrmEngine::new(&path, Algorithm::CH).expect("Failed to initialize OSRM engine");

    let pairs = routable_pairs_around_paris(&engine, 0.1);
    let mut pair_index = 0;

    c.bench_function("calculate_multiple_routes_around_paris_10km_ch", |b| {
        b.iter_batched(
            || {
                let pair = pairs[pair_index];
                pair_index = (pair_index + 1) % pairs.len();
                pair
            },
            |(start, end)| {
                black_box(
                    engine
                        .simple_route(start, end)
                        .expect("Simple route request failed"),
                );
            },
            BatchSize::SmallInput,
        );
    });
}

fn calculate_multiple_routes_around_paris_100km_ch(c: &mut Criterion) {
    dotenv().expect(".env file could not be read");

    let path = env::var("OSRM_TEST_DATA_PATH_CH")
        .expect("Environment variable OSRM_TEST_DATA_PATH_CH must be defined with a French map");
    let engine = OsrmEngine::new(&path, Algorithm::CH).expect("Failed to initialize OSRM engine");

    let pairs = routable_pairs_around_paris(&engine, 1.0);
    let mut pair_index = 0;

    c.bench_function("calculate_multiple_routes_around_paris_100km_ch", |b| {
        b.iter_batched(
            || {
                let pair = pairs[pair_index];
                pair_index = (pair_index + 1) % pairs.len();
                pair
            },
            |(start, end)| {
                black_box(
                    engine
                        .simple_route(start, end)
                        .expect("Simple route request failed"),
                );
            },
            BatchSize::SmallInput,
        );
    });
}

// Define the benchmark group
criterion_group!(
    benches,
    calculate_table_successfully,
    calculate_route_successfully,
    calculate_simple_route_successfully,
    calculate_table_10_successfully_mld,
    calculate_table_100_successfully_mld,
    calculate_multiple_routes_around_paris_10km_mld,
    calculate_multiple_routes_around_paris_100km_mld,
    calculate_multiple_routes_around_paris_10km_ch,
    calculate_multiple_routes_around_paris_100km_ch
);

// Set the main function to run the benchmarks
criterion_main!(benches);
