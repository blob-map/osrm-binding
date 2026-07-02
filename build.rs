use std::io::Cursor;
use std::path::{Path, PathBuf};


fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // The OSRM version this crate links against must match the version of the
    // osrm-extract/partition/customize tools used to prepare the `.osrm` files,
    // otherwise OSRM refuses to load them ("File is incompatible with this
    // version of OSRM"). Override via OSRM_BACKEND_REF with any git ref
    // (tag, branch, or commit hash) from Project-OSRM/osrm-backend.
    println!("cargo:rerun-if-env-changed=OSRM_BACKEND_REF");
    let osrm_ref = std::env::var("OSRM_BACKEND_REF").unwrap_or_else(|_| "v6.0.0".to_string());
    let osrm_url = format!(
        "https://github.com/Project-OSRM/osrm-backend/archive/{}.tar.gz",
        osrm_ref
    );

    eprintln!("Downloading OSRM source ({}) from {}...", osrm_ref, osrm_url);

    let mut response = reqwest::blocking::get(osrm_url).unwrap();
    let mut buffer = Vec::new();
    response.copy_to(&mut buffer).unwrap();

    eprintln!("Decompressing OSRM source...");
    let cursor = Cursor::new(buffer);
    let tar_gz = flate2::read::GzDecoder::new(cursor);
    let mut archive = tar::Archive::new(tar_gz);
    archive.unpack(&out_dir).unwrap();

    let osrm_source_path = find_osrm_source(&out_dir);
    eprintln!("OSRM source path: {}", osrm_source_path.display());

    let cxx_flags = "-Wno-array-bounds -Wno-uninitialized -Wno-stringop-overflow -std=c++17 -Wno-error";

    let dst = cmake::Config::new(&osrm_source_path)
        .env("CXXFLAGS", cxx_flags)
        .define("CMAKE_CXX_STANDARD", "17")
        .define("CMAKE_CXX_STANDARD_REQUIRED", "ON")
        .define("CMAKE_CXX_FLAGS_RELEASE", "-DNDEBUG")
        .define("ENABLE_ASSERTIONS", "Off")
        .define("ENABLE_LTO", "Off")
        .build();

    cc::Build::new()
        .cpp(true)
        .file("src/wrapper.cpp")
        .flag("-std=c++17")
        .include(dst.join("include"))
        .include(osrm_source_path.join("include"))
        .include(osrm_source_path.join("third_party/fmt/include"))
        .define("FMT_HEADER_ONLY", None)
        .compile("osrm_wrapper");

    let lib_path = dst.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_path.display());

    println!("cargo:rustc-link-lib=static=osrm_wrapper");
    println!("cargo:rustc-link-lib=static=osrm");
    println!("cargo:rustc-link-lib=static=osrm_store");
    println!("cargo:rustc-link-lib=static=osrm_extract");
    println!("cargo:rustc-link-lib=static=osrm_partition");
    println!("cargo:rustc-link-lib=static=osrm_update");
    println!("cargo:rustc-link-lib=static=osrm_guidance");
    println!("cargo:rustc-link-lib=static=osrm_customize");
    println!("cargo:rustc-link-lib=static=osrm_contract");

    println!("cargo:rustc-link-lib=dylib=boost_thread");
    println!("cargo:rustc-link-lib=dylib=boost_filesystem");
    println!("cargo:rustc-link-lib=dylib=boost_iostreams");
    println!("cargo:rustc-link-lib=dylib=tbb");
    println!("cargo:rustc-link-lib=dylib=fmt");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    println!("cargo:rustc-link-lib=dylib=z");
    println!("cargo:rustc-link-lib=dylib=bz2");
    println!("cargo:rustc-link-lib=dylib=expat");

}

fn find_osrm_source(path: &Path) -> PathBuf {
    for entry in path.read_dir().expect("Failed to read directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_dir() && path.file_name().unwrap().to_str().unwrap().starts_with("osrm-backend-") {
            return path;
        }
    }
    panic!("Could not find OSRM source directory");
}