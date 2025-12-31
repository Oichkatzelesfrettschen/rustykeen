# Telemetry/Build/Assets Refinement (2025-12-31T22:11:50Z)

## Telemetry
- tracing: core instrumentation.
- tracing-subscriber: routing, env filters.
- tracing-android / tracing-oslog: platform sinks.
- tracing-tracy: remote profiling on-device.

## Architecture
- crux: headless UI; ViewModel in Rust; adapters render.

## Assets
- rust-embed: ship seeds/topologies in binary; rkyv for states.

## Build/Distro
- cargo-mobile2: generate iOS/Android projects; unify workflows.
- Linkers: mold/sold; PGO via cargo-pgo; cargo-bloat audit; panic = "abort".

## Perf configs
- Enable mimalloc except iOS; feature-gated.
- NoHashHasher/fxhash for integer-key maps.
