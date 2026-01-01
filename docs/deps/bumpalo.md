# `bumpalo` (audit)
A fast bump allocation arena for Rust.
## Upstream
- crates.io: `https://crates.io/crates/bumpalo`
- latest observed: `3.19.1`
- repository: `https://github.com/fitzgen/bumpalo`
- documentation: `https://docs.rs/bumpalo`

## Why we care (engine mapping)
- Intended role: Arena allocator for search nodes
- Planned gate: `alloc-bumpalo`
- Adoption status: `now` (integrated behind feature; used for solver scratch allocations)

## Where it is used today
- `kenken-solver/src/solver.rs` (propagation scratch arenas under `alloc-bumpalo`)

## Notable features (from upstream docs, heuristic)
- (no Features section detected in README)

## Cargo features (from crates.io metadata)
- `allocator_api`
- `bench_allocator_api`
- `boxed`
- `collections`
- `default`
- `serde`
- `std`
