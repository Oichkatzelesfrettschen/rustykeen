# Implementation Guide

## Crates
Workspace members (current):
- `kenken-core`
- `kenken-solver`
- `kenken-gen`
- `kenken-io`
- `kenken-cli`
- `kenken-uniffi`

Feature gates are the primary mechanism for keeping the default build lean while enabling a “2026 riced” stack in opt-in layers. See `docs/features.md`.

## Core
Types: `Puzzle`, `Cage`, `CellId`, `Coord`, `rules::{Ruleset, Op}`.

Current invariants are enforced by:
- `Puzzle::validate(rules)` (coverage, duplicate cells, cage shape + connectivity, op/size rules)
- `Cage::validate_shape(...)`

Formats:
- sgt-puzzles “desc” import/export: `kenken_core::format::sgt_desc::{parse_keen_desc, encode_keen_desc}`.

## Solver
Search:
- deterministic backtracking with MRV cell selection
- solution counting with early exit (`count_solutions_up_to`)
- optional deduction tiers (`DeductionTier`) to trade propagation cost vs branching

Performance features:
- `alloc-bumpalo`: arena-backed scratch buffers for propagation (reduces heap churn).
- `solver-dlx`: Latin-square exact-cover utilities (`dlx-rs`).
- `sat-varisat`: Latin-square SAT uniqueness utility (`varisat`).

## Generator
Create full solution → cages → targets → minimize with uniqueness checks.

## IO
Engine-owned, versioned snapshots:
- `io-rkyv`: snapshot v1 using `rkyv` (`kenken_io::rkyv_snapshot`)
  - Snapshot v2 adds `Ruleset` persistence + versioned decode entrypoint (`docs/rkyv_snapshot_v2.md`)

## CLI
Reference CLI:
- `solve` and `count` over sgt “desc”
 - installs a default `tracing-subscriber` (override with `RUST_LOG`) so solver/SAT traces show up in local runs

## WASM
wasm-bindgen exports for web demo.

## Android
C++ GameActivity shim; minimal JNI; link Rust staticlib via CMake; call exported C functions.
Flags: `-fvisibility=hidden`, ThinLTO, `-fsanitize=hwaddress` (debug); manifest MTE.
ATrace around FFI; Simpleperf usage documented.

## Testing/Benchmarks
cargo test; proptest; criterion benches; determinism tests.
