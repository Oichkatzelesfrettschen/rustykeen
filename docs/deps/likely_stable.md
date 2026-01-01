# `likely_stable` (audit)
likely and unlikely compiler hints in stable rust
## Upstream
- crates.io: `https://crates.io/crates/likely_stable`
- latest observed: `0.1.3`
- repository: `https://gitlab.com/okannen/likely`

## Why we care (engine mapping)
- Intended role: Branch prediction hints
- Planned gate: `perf-likely`
- Adoption status: `now` (hooked behind feature in solver hot paths)

## Where it is used today
- `kenken-solver/src/solver.rs` (`likely(...)` hints under `perf-likely`)

## Notable features (from upstream docs, heuristic)
- (no Features section detected in README)

## Cargo features (from crates.io metadata)
- `check_assembly`
- `default`
