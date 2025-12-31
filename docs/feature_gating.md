# Feature Gating Examples (2025-12-31T22:07:20Z)

## Cargo.toml
[features]
default = ["std"]
std = []
android = ["uniffi"]
wasm = []
c_abi = []
simd = []
mimalloc = ["dep:mimalloc"]

[dependencies]
mimalloc = { version = "*", optional = true }

## lib.rs
#![cfg_attr(feature = "simd", feature(portable_simd))]

#[cfg(feature = "mimalloc")]
mod alloc;

#[cfg(all(feature = "android", not(target_os = "android")))]
compile_error!("android feature requires Android target");

#[cfg(feature = "c_abi")]
pub mod c_api;

#[cfg(feature = "wasm")]
pub mod wasm_api;
