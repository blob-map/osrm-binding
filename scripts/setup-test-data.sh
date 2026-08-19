#!/usr/bin/env bash

set -euo pipefail

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<'EOF'
Download and prepare the French OSRM car and bicycle datasets used by this
project's tests.

Usage: setup-test-data

Optional environment variables:
  OSRM_TEST_DATA_DIR  Destination directory (default: .test-data)
  OSRM_TEST_DATA_URL  Source PBF URL (default: Geofabrik's France extract)
  CARGO_TARGET_DIR    Cargo build directory (default: target)

The France PBF is downloaded once and reused for both profiles. The generated
car and bicycle routing graphs require substantial disk space, memory, and
preprocessing time. An interrupted download is resumed the next time the
command runs.
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
bicycle_pbf_path="$data_dir/france-bicycle-latest.osm.pbf"
# OSRM takes a base path and appends .osrm plus the file type itself.
car_dataset_path="$data_dir/france-latest"
bicycle_dataset_path="$data_dir/france-bicycle-latest"
completion_marker="$data_dir/.prepared-car-and-bicycle-v6.0.0"

write_env() {
  local env_path="$project_root/.env"
  local temporary_env
  temporary_env="$(mktemp "$project_root/.env.tmp.XXXXXXXX")"

  if [[ -f "$env_path" ]]; then
    grep -Ev '^OSRM_TEST_DATA_PATH_(BICYCLE_)?(MLD|CH)=' "$env_path" > "$temporary_env" || true
  fi

  {
    printf 'OSRM_TEST_DATA_PATH_MLD="%s"\n' "$car_dataset_path"
    printf 'OSRM_TEST_DATA_PATH_CH="%s"\n' "$car_dataset_path"
    printf 'OSRM_TEST_DATA_PATH_BICYCLE_MLD="%s"\n' "$bicycle_dataset_path"
    printf 'OSRM_TEST_DATA_PATH_BICYCLE_CH="%s"\n' "$bicycle_dataset_path"
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

if [[ -e "$bicycle_pbf_path" || -L "$bicycle_pbf_path" ]]; then
  if [[ ! "$bicycle_pbf_path" -ef "$pbf_path" ]]; then
    echo "Bicycle PBF path does not refer to the France download: $bicycle_pbf_path" >&2
    exit 1
  fi
else
  ln --symbolic -- "$(basename "$pbf_path")" "$bicycle_pbf_path"
fi

echo "Building the OSRM v6.0.0 command-line tools used by this crate..."
OSRM_BACKEND_REF=v6.0.0 cargo build --release

target_dir="${CARGO_TARGET_DIR:-$project_root/target}"
if [[ "$target_dir" != /* ]]; then
  target_dir="$project_root/$target_dir"
fi

osrm_extract=""
car_profile=""
bicycle_profile=""
while IFS= read -r candidate; do
  candidate_bin_dir="$(dirname "$candidate")"
  candidate_out_dir="$(dirname "$candidate_bin_dir")"
  candidate_profiles_dir="$candidate_out_dir/osrm-backend-6.0.0/profiles"
  candidate_car_profile="$candidate_profiles_dir/car.lua"
  candidate_bicycle_profile="$candidate_profiles_dir/bicycle.lua"
  if [[ -f "$candidate_car_profile" && -f "$candidate_bicycle_profile" ]]; then
    osrm_extract="$candidate"
    car_profile="$candidate_car_profile"
    bicycle_profile="$candidate_bicycle_profile"
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

prepare_dataset() {
  local profile_name="$1"
  local profile_path="$2"
  local input_path="$3"
  local dataset_path="$4"

  if [[ -f "$dataset_path.osrm.ebg" && -f "$dataset_path.osrm.cnbg" ]]; then
    echo "[$profile_name] Using existing extracted routing graph."
  else
    echo "[$profile_name] Extracting the routing graph..."
    "$osrm_bin_dir/osrm-extract" --profile "$profile_path" "$input_path"
  fi

  if [[ -f "$dataset_path.osrm.partition" && -f "$dataset_path.osrm.cells" ]]; then
    echo "[$profile_name] Using existing MLD partitions."
  else
    echo "[$profile_name] Preparing MLD partitions..."
    "$osrm_bin_dir/osrm-partition" "$dataset_path"
  fi

  if [[ -f "$dataset_path.osrm.hsgr" ]]; then
    echo "[$profile_name] Using existing CH graph."
  else
    echo "[$profile_name] Preparing the CH graph..."
    "$osrm_bin_dir/osrm-contract" "$dataset_path"
  fi

  if [[ -f "$dataset_path.osrm.cell_metrics" && -f "$dataset_path.osrm.mldgr" ]]; then
    echo "[$profile_name] Using existing MLD customization."
  else
    echo "[$profile_name] Customizing the MLD graph..."
    "$osrm_bin_dir/osrm-customize" "$dataset_path"
  fi
}

prepare_dataset "car" "$car_profile" "$pbf_path" "$car_dataset_path"
prepare_dataset "bicycle" "$bicycle_profile" "$bicycle_pbf_path" "$bicycle_dataset_path"

touch "$completion_marker"
write_env

echo "France car and bicycle test data is ready. Run: cargo test"
