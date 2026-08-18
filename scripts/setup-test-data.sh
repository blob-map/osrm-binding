#!/usr/bin/env bash

set -euo pipefail

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<'EOF'
Download and prepare the French OSRM dataset used by this project's tests.

Usage: setup-test-data

Optional environment variables:
  OSRM_TEST_DATA_DIR  Destination directory (default: .test-data)
  OSRM_TEST_DATA_URL  Source PBF URL (default: Geofabrik's France extract)
  CARGO_TARGET_DIR    Cargo build directory (default: target)

The France extract and its generated routing data require substantial disk
space, memory, and preprocessing time. An interrupted download is resumed the
next time the command runs.
EOF
  exit 0
fi

if [[ $# -ne 0 ]]; then
  echo "Unexpected argument: $1" >&2
  echo "Run setup-test-data --help for usage." >&2
  exit 2
fi

project_root="${OSRM_BINDING_ROOT:-}"
if [[ -z "$project_root" ]]; then
  project_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
fi
project_root="$(cd "$project_root" && pwd)"
cd "$project_root"

default_data_dir="$project_root/.test-data"
data_dir="${OSRM_TEST_DATA_DIR:-$default_data_dir}"
if [[ "$data_dir" != /* ]]; then
  data_dir="$project_root/$data_dir"
fi
mkdir -p "$data_dir"
data_dir="$(cd "$data_dir" && pwd)"

if [[ "$data_dir" == *".osrm"* ]]; then
  echo "OSRM v6 cannot use a data directory containing '.osrm': $data_dir" >&2
  echo "Set OSRM_TEST_DATA_DIR to a path without '.osrm' and run the command again." >&2
  exit 2
fi

data_url="${OSRM_TEST_DATA_URL:-https://download.geofabrik.de/europe/france-latest.osm.pbf}"
pbf_path="$data_dir/france-latest.osm.pbf"
partial_pbf_path="$pbf_path.part"
# OSRM takes a base path and appends .osrm plus the file type itself.
dataset_path="$data_dir/france-latest"
completion_marker="$data_dir/.prepared-v6.0.0"

write_env() {
  local env_path="$project_root/.env"
  local temporary_env
  temporary_env="$(mktemp "$project_root/.env.tmp.XXXXXXXX")"

  if [[ -f "$env_path" ]]; then
    grep -Ev '^OSRM_TEST_DATA_PATH_(MLD|CH)=' "$env_path" > "$temporary_env" || true
  fi

  {
    printf 'OSRM_TEST_DATA_PATH_MLD="%s"\n' "$dataset_path"
    printf 'OSRM_TEST_DATA_PATH_CH="%s"\n' "$dataset_path"
  } >> "$temporary_env"

  mv "$temporary_env" "$env_path"
  echo "Wrote $env_path"
}

if [[ -f "$completion_marker" ]]; then
  echo "OSRM v6.0.0 France test data is already prepared in $data_dir"
  write_env
  exit 0
fi

echo "Test data directory: $data_dir"
df -h "$data_dir" | tail -n 1

if [[ ! -f "$pbf_path" ]]; then
  echo "Downloading the current Geofabrik France extract..."
  curl \
    --continue-at - \
    --fail \
    --location \
    --output "$partial_pbf_path" \
    --retry 3 \
    "$data_url"
  mv "$partial_pbf_path" "$pbf_path"
else
  echo "Using existing download: $pbf_path"
fi

echo "Building the OSRM v6.0.0 command-line tools used by this crate..."
OSRM_BACKEND_REF=v6.0.0 cargo build --release

target_dir="${CARGO_TARGET_DIR:-$project_root/target}"
if [[ "$target_dir" != /* ]]; then
  target_dir="$project_root/$target_dir"
fi

osrm_extract=""
car_profile=""
while IFS= read -r candidate; do
  candidate_bin_dir="$(dirname "$candidate")"
  candidate_out_dir="$(dirname "$candidate_bin_dir")"
  candidate_profile="$candidate_out_dir/osrm-backend-6.0.0/profiles/car.lua"
  if [[ -f "$candidate_profile" ]]; then
    osrm_extract="$candidate"
    car_profile="$candidate_profile"
  fi
done < <(find "$target_dir/release/build" -type f -path '*/out/bin/osrm-extract' -executable)

if [[ -z "$osrm_extract" ]]; then
  echo "Could not locate the osrm-extract binary produced by cargo build." >&2
  exit 1
fi

osrm_bin_dir="$(dirname "$osrm_extract")"

for command_name in osrm-extract osrm-partition osrm-contract osrm-customize; do
  if [[ ! -x "$osrm_bin_dir/$command_name" ]]; then
    echo "Missing expected OSRM command: $osrm_bin_dir/$command_name" >&2
    exit 1
  fi
done

if [[ -f "$dataset_path.osrm.ebg" && -f "$dataset_path.osrm.cnbg" ]]; then
  echo "Using existing extracted road graph."
else
  echo "Extracting the road graph..."
  "$osrm_bin_dir/osrm-extract" --profile "$car_profile" "$pbf_path"
fi

if [[ -f "$dataset_path.osrm.partition" && -f "$dataset_path.osrm.cells" ]]; then
  echo "Using existing MLD partitions."
else
  echo "Preparing MLD partitions..."
  "$osrm_bin_dir/osrm-partition" "$dataset_path"
fi

if [[ -f "$dataset_path.osrm.hsgr" ]]; then
  echo "Using existing CH graph."
else
  echo "Preparing the CH graph..."
  "$osrm_bin_dir/osrm-contract" "$dataset_path"
fi

if [[ -f "$dataset_path.osrm.cell_metrics" && -f "$dataset_path.osrm.mldgr" ]]; then
  echo "Using existing MLD customization."
else
  echo "Customizing the MLD graph..."
  "$osrm_bin_dir/osrm-customize" "$dataset_path"
fi

touch "$completion_marker"
write_env

echo "France test data is ready. Run: cargo test"
