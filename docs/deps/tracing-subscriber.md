# `tracing-subscriber` (audit)
Utilities for implementing and composing `tracing` subscribers.
## Upstream
- crates.io: `https://crates.io/crates/tracing-subscriber`
- latest observed: `0.3.22`
- repository: `https://github.com/tokio-rs/tracing`

## Why we care (engine mapping)
- Intended role: Tracing output routing
- Planned gate: `telemetry-subscriber`
- Adoption status: `now` (CLI installs a default subscriber so solver/SAT traces are visible)

## Where it is used today
- `kenken-cli/src/main.rs` (`init_tracing()` behind `kenken-cli/telemetry-subscriber`)

## Notable features (from upstream docs, heuristic)
- (no Features section detected in README)

## Cargo features (from crates.io metadata)
- `alloc`
- `ansi`
- `default`
- `env-filter`
- `fmt`
- `json`
- `local-time`
- `nu-ansi-term`
- `regex`
- `registry`
- `std`
- `valuable`
