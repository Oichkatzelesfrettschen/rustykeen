# Dependency / feature matrix (2026 target stack)

This document maps the audited dependency list to:
- intended subsystem (“where it plugs in”)
- Cargo feature gate (“how we keep it optional”)
- adoption status (now vs planned)

See also:
- `docs/crate_audit_list.md` (canonical list)
- `docs/deps/README.md` (per-crate summaries)
- `docs/features.md` (current workspace feature strategy)

Legend:
- Status: `now` = in-use today; `planned` = target stack but not integrated yet.

## I. Blue Smoke Performance Core
- `dlx-rs` → Latin core exact-cover solver → feature `solver-dlx` → status `now` (initial Latin-square DLX solver module exists)
- `bitvec` → candidate domains / bit-level constraints → feature `core-bitvec` → status `now` (initial `BitDomain` exists; solver still uses `u32` domains)
- `mimalloc` → global allocator (non-iOS) → feature `alloc-mimalloc` → status `now` (wired in `kenken-cli` behind feature)
- `bumpalo` → arena allocation for solver scratch space → feature `alloc-bumpalo` → status `now` (propagation uses bump-allocated temporaries; expanding coverage is planned)
- `smallvec` → small cage cell-lists hotpath → always-on (core) → status `now`
- `wide` → SIMD-friendly constraint checks → feature `simd-wide` → status `planned`
- `soa_derive` → SoA layout for batch generation → feature `layout-soa` → status `planned`
- `likely_stable` → branch prediction hints → feature `perf-likely` → status `now` (solver uses `likely(...)` when enabled)
- `static_assertions` → compile-time layout/size contracts → feature `perf-assertions` → status `now` (core asserts `CellId`/`Coord` layout)

## II. Hyper-Scale Logic
- `rayon` → parallel generation / batch solving → feature `parallel-rayon` → status `now` (kenken-gen has a parallel batch uniqueness/count API)
- `parking_lot` → fast locks for caches → feature `sync-parking_lot` → status `planned`
- `ringbuf` → lock-free telemetry queue → feature `telemetry-ringbuf` → status `planned`
- `dashmap` → concurrent caches (uniqueness/difficulty) → feature `cache-dashmap` → status `planned`
- `nohash-hasher` / `fxhash` → fast hashing for integer keys → feature `hash-fast` → status `planned`
- `rand_pcg` → deterministic RNG streams (candidate) → feature `rng-pcg` → status `planned`
- `fixed` → deterministic fixed-point difficulty math → feature `math-fixed` → status `planned`
- `num-integer` → gcd/lcm/div constraints utilities → feature `math-num-integer` → status `planned`

## III. Zero-Overhead Architecture
- `rkyv` → zero-copy snapshots / state persistence → feature `io-rkyv` → status `now` (kenken-io has Snapshot v1 encode/decode + roundtrip test)
- `crux_core` → headless UI architecture → feature `ui-crux` → status `planned`
- `uniffi` → Kotlin/Swift bindings → feature `ffi-uniffi` → status `now` (kenken-uniffi has UDL + scaffolding + minimal solve/count API)
- `rust-embed` → embed assets/seeds/topologies → feature `assets-embed` → status `planned`
- `bytemuck` → safe zero-copy casting in perf paths → feature `bytes-bytemuck` → status `planned`
- `anyhow` → ergonomic edge errors (CLI/adapters) → feature `errors-anyhow` → status `planned`
- `thiserror` → typed errors in libraries → feature `errors-thiserror` → status `now`

## IV. Deep Vision Tooling
- `tracing` → structured spans/events → feature `telemetry-tracing` → status `now` (in solver, optional)
- `tracing-subscriber` → route tracing output → feature `telemetry-subscriber` → status `now` (installed by `kenken-cli` for out-of-box visibility)
- `tracing-tracy` → tracy profiler integration → feature `telemetry-tracy` → status `planned`
- `criterion` → statistical benchmarks → feature `bench-criterion` → status `planned`
- `ratatui` → developer TUI dashboard → feature `dev-tui` → status `planned`
- `varisat` → SAT uniqueness proofs (optional) → feature `sat-varisat` → status `now` (Latin + staged cage allowlist encoding; tuple-thresholded)
- `z3` → SMT proofs (optional) → feature `smt-z3` → status `planned`
- `kani` → model checking harnesses → feature `verify-kani` → status `planned`
- `proptest` / `bolero` → fuzz/property testing → feature `fuzz` → status `planned`
- `nom` → legacy corpus parsing → feature `io-nom` → status `planned`
