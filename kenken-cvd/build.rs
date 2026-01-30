// Build script for libDaltonLens FFI bindings
// Compiles vendored libDaltonLens.c and exposes it to Rust via FFI

fn main() {
    // Compile libDaltonLens.c
    cc::Build::new()
        .file("vendor/libDaltonLens.c")
        .warnings(false) // Suppress C compiler warnings from vendored code
        .compile("daltonlens");

    // Tell cargo to link against the compiled library
    println!("cargo:rustc-link-lib=static=daltonlens");
    println!(
        "cargo:rustc-link-search=native={}",
        std::env::var("OUT_DIR").unwrap()
    );

    // Rerun build if libDaltonLens files change
    println!("cargo:rerun-if-changed=vendor/libDaltonLens.c");
    println!("cargo:rerun-if-changed=vendor/libDaltonLens.h");
    println!("cargo:rerun-if-changed=build.rs");
}
