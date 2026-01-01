# Crate feature plan (cleanroom + performance, 2026 target)

This document answers two recurring questions:
1) **What do these crates do for Keen?** (role and integration point)
2) **Which features do we intend to use?** (both our workspace feature gates and upstream crate features)

It is intentionally opinionated and performance-aware, but **library-first**:
- default builds stay lean and portable
- heavy dependencies are **opt-in** behind Cargo features
- platform-specific choices (allocators, profilers, mobile logging) live in adapters

For upstream links and crate-level summaries, see `docs/crate_audit_list.md` and `docs/deps/README.md`.

## Status legend
- `now`: integrated in code today (typically behind a feature)
- `partial`: integrated, but only used in a narrow path
- `planned`: audited, but not yet wired

## I. Blue Smoke Performance Core (solver & memory)

### `dlx-rs` (`dlx_rs`)
- Our gate: `kenken-solver/solver-dlx`
- Upstream features: avoid puzzle-specific examples (`sudoku`, `queens`, …) unless we need them for tooling.
- Status: `now`
- Granular usage plan:
  - Use DLX for “Latin hotpath” generation and for counting Latin completions quickly.
  - Keep cage arithmetic out of DLX until we have a proven exact-cover encoding.

### `bitvec`
- Our gate: `kenken-core/core-bitvec`
- Upstream features: prefer `alloc`/`std` only; avoid `atomic` unless we prove it helps.
- Status: `partial` (`BitDomain` exists; solver still uses `u32` masks)
- Granular usage plan:
  - Replace solver `u32` masks with `BitDomain` only if it beats a fixed-width integer mask on target sizes.
  - Use `iter_ones()`/`count_ones()` for candidate enumeration and MRV selection.

### `mimalloc`
- Our gate: `kenken-cli/alloc-mimalloc` (and later: generator benchmarks/tooling binaries)
- Upstream features: default, optionally `secure`/`override` for instrumentation builds.
- Status: `partial` (CLI-only)
- Granular usage plan:
  - Never enable on iOS targets (platform restrictions); gate by `cfg`.
  - Prefer to keep the core crates allocator-agnostic.

### `bumpalo`
- Our gate: `kenken-solver/alloc-bumpalo`
- Upstream features: `collections` only (already enabled); avoid `serde` unless needed for debug dumps.
- Status: `partial`
- Granular usage plan:
  - Use `Bump` for propagation temporaries and per-node scratch space.
  - Avoid long-lived arena allocations that outlive a solve call.

### `smallvec`
- Our gate: always-on (core)
- Upstream features: default.
- Status: `now`
- Granular usage plan:
  - Store cage cells in `SmallVec` (2–6 typical).
  - Use `SmallVec<[u8; 6]>` for tuple allowlists (SAT encoding).

### `wide`
- Our gate: `simd-wide` (planned)
- Status: `planned`
- Granular usage plan:
  - Vectorize row/col checks and candidate elimination once baseline perf is measured.
  - Only adopt if it improves *real* solve/generate throughput (bench-driven).

### `soa_derive`
- Our gate: `layout-soa` (planned)
- Status: `planned`
- Granular usage plan:
  - Batch generation: turn “many puzzles in flight” into SoA for cache and SIMD friendliness.

### `likely_stable`
- Our gate: `kenken-solver/perf-likely`
- Status: `now`
- Granular usage plan:
  - Use `likely(...)` only on proven hot branches (profile-guided).

### `static_assertions`
- Our gate: `kenken-core/perf-assertions`
- Status: `partial` (available; only valuable once we lock layouts)
- Granular usage plan:
  - Add compile-time checks when we freeze public/FFI-visible structs.

## II. Hyper-Scale Logic (concurrency & math)

### `rayon`
- Our gate: `kenken-gen/parallel-rayon`
- Status: `now`
- Granular usage plan:
  - Parallelize “generate until accepted” loops with deterministic per-task seeds.
  - Keep solver itself single-threaded by default (predictable latency).

### `parking_lot`
- Our gate: `sync-parking_lot` (planned)
- Status: `planned`
- Granular usage plan:
  - Use for shared caches only (difficulty, uniqueness memoization), not core state.

### `ringbuf`
- Our gate: `telemetry-ringbuf` (planned)
- Status: `planned`
- Granular usage plan:
  - Stream solver/gen telemetry without locks (SPSC) into an observer thread.

### `dashmap`
- Our gate: `cache-dashmap` (planned)
- Status: `planned`
- Granular usage plan:
  - Cache “seen puzzle seeds → uniqueness result” for generator speedups.

### `nohash-hasher` / `fxhash`
- Our gate: `hash-fast` (planned)
- Status: `planned`
- Granular usage plan:
  - For integer-keyed maps where DoS resistance is not needed (internal-only).

### `rand_pcg`
- Our gate: `rng-pcg` (planned)
- Status: `planned` (today we use `rand_chacha` for determinism)
- Granular usage plan:
  - Decide one canonical RNG for cross-platform determinism; document and freeze it.

### `fixed`
- Our gate: `math-fixed` (planned)
- Status: `planned`
- Granular usage plan:
  - Difficulty scoring: deterministic fixed-point metrics to avoid x86/ARM float drift.

### `num-integer`
- Our gate: `math-num-integer` (planned)
- Status: `planned`
- Granular usage plan:
  - GCD/LCM utilities for div/mul cage reasoning and pruning.

## III. Zero-Overhead Architecture (I/O & bindings)

### `rkyv`
- Our gate: `kenken-io/io-rkyv`
- Status: `now`
- Granular usage plan:
  - Maintain versioned snapshot schemas and migration tests.
  - Avoid “serde-derived persistence” for engine-owned data.

### `crux_core`
- Our gate: `ui-crux` (planned)
- Status: `planned`
- Granular usage plan:
  - Keep all state transitions in Rust; UI becomes a renderer + event source.

### `uniffi`
- Our gate: `kenken-uniffi/ffi-uniffi` (crate-local default)
- Status: `now`
- Granular usage plan:
  - Expose generator as async-friendly APIs (non-blocking for mobile UI).
  - Keep core types stable and versioned; avoid exposing internal solver structs.

### `rust-embed`
- Our gate: `assets-embed` (planned)
- Status: `planned`
- Granular usage plan:
  - Embed seed/corpus/topology assets for mobile “no filesystem” environments.

### `bytemuck`
- Our gate: `bytes-bytemuck` (planned)
- Status: `planned`
- Granular usage plan:
  - Safe casts for hashing/IO boundaries once we freeze layouts and add assertions.

### `anyhow` / `thiserror`
- Our gates: `errors-anyhow` (planned), `errors-thiserror` (now)
- Status: `thiserror` is `now`, `anyhow` is `planned`
- Granular usage plan:
  - Core crates: typed errors (`thiserror`).
  - Binaries/adapters: context-rich errors (`anyhow`) at the edge.

## IV. Deep Vision Tooling (telemetry & verification)

### `tracing` (+ `tracing-subscriber`, `tracing-tracy`)
- Our gates: `telemetry-tracing` (now in solver), `telemetry-subscriber`/`telemetry-tracy` (planned)
- Status: `partial`
- Granular usage plan:
  - Define a stable span/event contract (`docs/plan.md` “Instrumentation contract”).
  - Install subscribers in binaries/adapters only.

### `criterion`
- Our gate: `bench-criterion` (planned)
- Status: `planned`
- Granular usage plan:
  - Add microbenchmarks for tuple enumeration, SAT encoding, and solver propagation.

### `ratatui`
- Our gate: `dev-tui` (planned)
- Status: `planned`
- Granular usage plan:
  - Developer dashboard for fuzz/generation stats on headless machines.

### `varisat` / `z3`
- Our gates: `sat-varisat` (now), `smt-z3` (planned)
- Status: `varisat` is `now`, `z3` is `planned`
- Granular usage plan:
  - `varisat`: uniqueness verification and regression tests against enumeration.
  - `z3`: optional “certification” and cross-check harnesses.

### `kani`
- Our gate: `verify-kani` (planned)
- Status: `planned`
- Granular usage plan:
  - Model-check panic-freedom for bitset/index arithmetic and SAT var mapping.

### `proptest` / `bolero`
- Our gate: `fuzz` (planned)
- Status: `planned`
- Granular usage plan:
  - Fuzz cage tuple enumeration and parser/serializer roundtrips.

### `nom`
- Our gate: `io-nom` (planned)
- Status: `planned`
- Granular usage plan:
  - Ingest legacy corpora and convert into internal canonical formats.

