# `uniffi` (audit)
a multi-language bindings generator for rust
## Upstream
- crates.io: `https://crates.io/crates/uniffi`
- latest observed: `0.30.0`
- repository: `https://github.com/mozilla/uniffi-rs`
- documentation: `https://mozilla.github.io/uniffi-rs`

## Why we care (engine mapping)
- Intended role: Kotlin/Swift bindings generator
- Planned gate: `ffi-uniffi`
- Adoption status: `now` (bindings crate exists; minimal solve/count surface)

## Where it is used today
- `kenken-uniffi/src/keen.udl` (API surface definition)
- `kenken-uniffi/build.rs` (scaffolding generation)
- `kenken-uniffi/src/lib.rs` (bindings implementation)

## Notable features (from upstream docs, heuristic)
- (no Features section detected in README)

## Cargo features (from crates.io metadata)
- `bindgen`
- `bindgen-tests`
- `build`
- `cargo-metadata`
- `cli`
- `default`
- `ffi-trace`
- `scaffolding-ffi-buffer-fns`
- `tokio`
- `wasm-unstable-single-threaded`
