# osrm-binding

Rust bindings for [OSRM (Open Source Routing Machine)](http://project-osrm.org/), providing an idiomatic and type-safe interface to access core OSRM functionalities (`route`, `table`, `trip`) from Rust.

## 🚀 Features

- 🗺️ Calculate routes, trips, and distance/duration tables using OSRM
- 🦀 Fully written in Rust
- 💡 Simple and safe API for high-performance routing
- 🧪 Includes integration tests for route and table APIs

## 📦 Installation

Add the crate to your `Cargo.toml`:

```shell
cargo add osrm-binding
```

### Building Dependencies

This library requires OSRM to be built and linked. Below are instructions for setting up the dependencies.

#### Nix development shell

With [Nix](https://nixos.org/) and flakes enabled, enter a shell containing Rust and all native OSRM build dependencies:

```shell
nix develop
cargo build
```

The first Cargo build downloads and compiles the OSRM source, so it can take a few minutes. To use a different OSRM version, set `OSRM_BACKEND_REF` as described below before running Cargo.

#### Local Installation (Ubuntu 24.04)

Install the required system dependencies:

```shell
sudo apt update
sudo apt install build-essential git cmake pkg-config \
                libbz2-dev libxml2-dev libzip-dev libboost-all-dev \
                lua5.2 liblua5.2-dev libtbb-dev libfmt-dev
```

#### Dockerfile

Use the following Dockerfile to build your application in a containerized environment:

```dockerfile
FROM rust:1.88.0-bookworm AS builder

WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
COPY ./src ./src

RUN apt-get update && \
    apt-get -y --no-install-recommends --no-install-suggests install \
        ca-certificates \
        cmake \
        g++ \
        gcc \
        git \
        libboost1.81-all-dev \
        libbz2-dev \
        liblua5.4-dev \
        libtbb-dev \
        libxml2-dev \
        libzip-dev \
        lua5.4 \
        make \
        pkg-config \
        libfmt-dev

RUN ls -la /usr/lib/x86_64-linux-gnu/libboost_thread*

RUN cargo build --release -vv

FROM debian:bookworm-slim

WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/target/release/my-bin ./

RUN apt-get update && \
    apt-get install -y --no-install-recommends --no-install-suggests \
        expat \
        libboost-date-time1.81.0 \
        libboost-iostreams1.81.0 \
        libboost-program-options1.81.0 \
        libboost-thread1.81.0 \
        liblua5.4-0 \
        libtbb12 && \
        rm -rf /var/lib/apt/lists/* && \
        ldconfig /usr/local/lib

CMD ["./my-bin"]
```

> **Note**: Replace `my-bin` with your actual binary name. This Dockerfile installs OSRM build dependencies and runtime libraries.

### OSRM version

By default this crate downloads and links **osrm-backend `v6.0.0`**. OSRM stamps a version fingerprint into the preprocessed `.osrm.*` files and refuses to load data prepared by a different version (`File is incompatible with this version of OSRM ...`). **The version that prepared your data must match the version this crate links against.**

If your `.osrm` files were generated with a different OSRM version, either regenerate them with `v6.0.0`, or build this crate against the matching version using the `OSRM_BACKEND_REF` environment variable (any git tag, branch, or commit hash from `Project-OSRM/osrm-backend`):

```shell
OSRM_BACKEND_REF=v5.27.1 cargo build
```

## 🛠️ Usage

### Initialization

Initialize the OSRM engine with a preprocessed OSRM data file. The `Algorithm` you pass **must match the preprocessing pipeline** used to build the `.osrm` files:

- **CH (Contraction Hierarchies):** `osrm-extract` → `osrm-contract`, then use `Algorithm::CH`
- **MLD (Multi-Level Dijkstra):** `osrm-extract` → `osrm-partition` → `osrm-customize`, then use `Algorithm::MLD`

```rust
use osrm_binding::{OsrmEngine, Algorithm};

// Data prepared with osrm-contract:
let engine = OsrmEngine::new("/path/to/france-latest.osrm", Algorithm::CH)
    .expect("Failed to initialize OSRM engine");

// Or, data prepared with osrm-partition + osrm-customize:
let engine = OsrmEngine::new("/path/to/france-latest.osrm", Algorithm::MLD)
    .expect("Failed to initialize OSRM engine");
```

### Route Calculation

Build and execute a route request:

```rust
use osrm_binding::{RouteRequestBuilder, Point};

let request = RouteRequestBuilder::default()
    .points(vec![
        Point { longitude: 2.3522, latitude: 48.8566 }, // Paris
        Point { longitude: 5.3698, latitude: 43.2965 }, // Marseille
    ])
    .build()
    .unwrap();

let result = engine.route(&request).unwrap();
println!("{:?}", result.routes.first().unwrap());
```

### Table (Distance/Duration Matrix)

Compute a distance/duration table:

```rust
use osrm_binding::{TableRequest, Point};

let request = TableRequest {
    sources: vec![Point { longitude: 2.3522, latitude: 48.8566 }],
    destinations: vec![
        Point { longitude: 5.3698, latitude: 43.2965 },
        Point { longitude: 4.8357, latitude: 45.7640 },
    ],
};

let response = engine.table(&request).unwrap();
println!("{:?}", response.durations);
```

### Simple Route

For quick single-origin to single-destination routing:

```rust
use osrm_binding::Point;

let result = engine.simple_route(
    Point { longitude: 2.3522, latitude: 48.8566 },
    Point { longitude: 5.3698, latitude: 43.2965 },
).unwrap();

println!("Duration: {}s, Distance: {}m", result.duration, result.distance);
```

### Trip API

The transport profile is part of the preprocessed dataset rather than the request. Load the bicycle dataset in a separate engine, then optimize a round trip with multiple waypoints:

```rust
use osrm_binding::algorithm::Algorithm;
use osrm_binding::osrm_engine::OsrmEngine;
use osrm_binding::point::Point;
use osrm_binding::trip::TripRequest;

let bicycle_engine = OsrmEngine::new(
    "/path/to/france-bicycle-latest",
    Algorithm::MLD,
).unwrap();

let request = TripRequest {
    points: vec![
        Point { longitude: 2.2945, latitude: 48.8584 }, // Eiffel Tower
        Point { longitude: 2.3364, latitude: 48.8606 }, // Louvre Museum
        Point { longitude: 2.3690, latitude: 48.8530 }, // Bastille
    ],
};

let response = bicycle_engine.trip(&request).unwrap();
let trip = &response.trips[0];

println!("Distance: {}m, duration: {}s", trip.distance, trip.duration);
for waypoint in response.waypoints {
    println!("Optimized position: {}", waypoint.waypoint_index);
}
```

## 🔬 Tests

The test suite uses routes between Paris, Lyon, and Marseille. From the Nix development shell, download the current France extract, prepare OSRM v6.0.0 car and bicycle datasets for both MLD and CH, and populate `.env` automatically:

```shell
setup-test-data
cargo test
```

The France PBF is downloaded only once and shared by both profiles. Car and bicycle still need separate processed routing graphs, so expect substantial disk, memory, and processing-time requirements. Downloads resume if interrupted, and completed data is reused on subsequent runs. The generated files live in `.test-data/` and are ignored by Git.

The generated `.env` contains `OSRM_TEST_DATA_PATH_MLD` and `OSRM_TEST_DATA_PATH_CH` for car routing, plus `OSRM_TEST_DATA_PATH_BICYCLE_MLD` and `OSRM_TEST_DATA_PATH_BICYCLE_CH` for bicycle routing. To use data you prepared yourself instead, set those paths manually. The dataset version must match the linked OSRM version, and each path must use the preprocessing pipeline matching its algorithm and transport profile.

### 🚀 Performance

Native performance using `cargo bench`

```shell
calculate_multiple_routes_around_paris_10km_mld
                        time:   [5.4872 ms 5.6545 ms 5.8246 ms]

calculate_multiple_routes_around_paris_100km_mld
                        time:   [13.063 ms 13.877 ms 14.652 ms]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild

calculate_multiple_routes_around_paris_10km_ch
                        time:   [3.8034 ms 3.8599 ms 3.9175 ms]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

calculate_multiple_routes_around_paris_100km_ch
                        time:   [5.9891 ms 6.2444 ms 6.4946 ms]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
```

## 📖 License

This project is licensed under the MIT License.

## ✨ Contributions

Contributions are welcome! Feel free to open issues or pull requests to improve performance, add more OSRM API bindings, or enhance usability.

---

Made with ❤️ in Rust.
