// docs/global_allocator.rs
// Configure mimalloc as the global allocator for CLI/Android crates.
#![allow(unused)]

#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// Note: enable with `--features mimalloc` and add `mimalloc = { version = "*", optional = true }` in Cargo.toml.
// For Android, ensure allocator is compatible with NDK; test with HWASan/MTE disabled when benchmarking raw alloc throughput.
