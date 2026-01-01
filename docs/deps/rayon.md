# `rayon` (audit)
Simple work-stealing parallelism for Rust
## Upstream
- crates.io: `https://crates.io/crates/rayon`
- latest observed: `1.11.0`
- repository: `https://github.com/rayon-rs/rayon`
- documentation: `https://docs.rs/rayon/`

## Why we care (engine mapping)
- Intended role: Parallel generation / batch solving
- Planned gate: `parallel-rayon`
- Adoption status: `now` (integrated behind feature; used for batch count/uniqueness)

## Where it is used today
- `kenken-gen/src/lib.rs` (`par_iter()` batch count / uniqueness under `parallel-rayon`)

## Notable features (from upstream docs, heuristic)
- (no Features section detected in README)

## Cargo features (from crates.io metadata)
- `web_spin_lock`
