# Dependency audit (workspace)

This repo is **library-first** and **cleanroom-driven**. Dependency policy:
- Keep the default build lean and portable.
- Introduce “heavy” capabilities behind Cargo features.
- Record a clear mapping: crate → purpose → feature gate → where used.

For the aspirational 2026 stack list, see `docs/crate_audit_list.md` and `docs/dependency_matrix.md`.

## Workspace crates (current)
- `kenken-core`
- `kenken-solver`
- `kenken-gen`
- `kenken-io`
- `kenken-uniffi`
- `kenken-cli`

## Core dependencies (always-on)

### `smallvec`
- Role: hotpath storage of small cage cell lists without heap allocation.
- Usage: `kenken_core::Cage.cells`.

### `thiserror`
- Role: typed errors for libraries.
- Usage: `kenken_core::CoreError`, `kenken_solver::SolveError`, `kenken_io::IoError`, etc.

## Feature-gated dependencies (in use)

### `bumpalo` (`kenken-solver/alloc-bumpalo`)
- Role: arena allocation for propagation scratch buffers.
- Usage: `kenken-solver/src/solver.rs` propagation loop.

### `rayon` (`kenken-gen/parallel-rayon`)
- Role: parallel batch solving/uniqueness checks; foundation for parallel generation.
- Usage: `kenken-gen/src/lib.rs` `par_iter()` path.

### `dlx-rs` (`kenken-solver/solver-dlx`)
- Role: Latin-square exact-cover solver utilities (DLX / Algorithm X).
- Usage: `kenken-solver/src/dlx_latin.rs`.

### `varisat` (`kenken-solver/sat-varisat`)
- Role: SAT uniqueness proof hooks.
- Current scope: Latin constraints + staged cage arithmetic encoding (tuple allowlists with overflow guardrails).
- Usage: `kenken-solver/src/sat_latin.rs`, `kenken-solver/src/sat_cages.rs`.

### `rkyv` (`kenken-io/io-rkyv`)
- Role: versioned, engine-owned snapshots with fast (zero-copy-friendly) decode.
- Usage: `kenken-io/src/rkyv_snapshot.rs`.

### `uniffi` (`kenken-uniffi/ffi-uniffi`)
- Role: Kotlin/Swift bindings generation.
- Usage: `kenken-uniffi/build.rs`, `kenken-uniffi/src/keen.udl`, `kenken-uniffi/src/lib.rs`.

### `mimalloc` (`kenken-cli/alloc-mimalloc`)
- Role: opt-in high-performance global allocator for CLI workloads.
- Usage: `kenken-cli/src/main.rs` global allocator hook.

### `tracing-subscriber` (`kenken-cli/telemetry-subscriber`)
- Role: installs a default tracing subscriber so solver/SAT tracepoints are visible without custom wiring.
- Usage: `kenken-cli/src/main.rs` `init_tracing()`.

## Optional but currently unused in code

### `bitvec` (`kenken-core/core-bitvec`)
- Role: bit-level candidate domains.
- Current status: `kenken_core::BitDomain` exists; solver still uses `u32` masks.

### `static_assertions` (`kenken-core/perf-assertions`)
- Role: compile-time layout checks for cache/ABI expectations.

### `tracing` (`kenken-solver/tracing`)
- Role: structured instrumentation hooks; library does not install subscribers.

### `likely_stable` (`kenken-solver/perf-likely`)
- Role: branch prediction hints in solver hot branches.

## Formatting / drift guardrails
- `docs/features.md`: feature-gate rules and where they live.
- `docs/dependency_matrix.md`: planned vs now adoption status.
- `docs/work_done.md`: what is implemented today.
