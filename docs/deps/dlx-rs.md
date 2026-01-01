# `dlx-rs` (audit)
Implementation of dancing links in Rust
## Upstream
- crates.io: `https://crates.io/crates/dlx-rs`
- latest observed: `1.3.0`
- repository: `https://github.com/tveness/dlx-rs`

## Why we care (engine mapping)
- Intended role: Latin exact-cover solver (DLX / Algorithm X)
- Planned gate: `solver-dlx`
- Adoption status: `now` (DLX Latin utilities exist behind feature)

## Where it is used today
- `kenken-solver/src/dlx_latin.rs` (Latin-only exact cover)

## Notable features (from upstream docs, heuristic)
- (no Features section detected in README)

## Cargo features (from crates.io metadata)
- `aztec`
- `default`
- `queens`
- `sudoku`
