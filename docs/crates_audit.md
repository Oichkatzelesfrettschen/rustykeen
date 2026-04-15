# Crates Audit and 2026 Optimization Stack (2026-04-09)

This is the **single source of truth** for third-party crate selection and integration status.
For per-crate upstream links, see `docs/deps/README.md`.

## Master Crate List (comma-separated)
uniffi, fixed, rkyv, smallvec, rand, rand_pcg, wide, dlx_rs, rayon, bumpalo, bitvec, varisat, dashmap, num-integer, petgraph, criterion, parking_lot, itertools, proptest, once_cell, mimalloc, anyhow, thiserror, nom, tracing, tracing-android, z3, bolero, kani, creusot

## Current Stack (audited)

This document separates three states so the architecture notes do not imply
more integration than the code actually has today:

- `adopted`: present in production or supported code paths now
- `partial`: present, but only in a narrow path, adapter, or research lane
- `planned`: selected as part of the intended stack, but not integrated yet

### Adopted now (feature-gated unless noted)
  - `bumpalo` (`kenken-solver/alloc-bumpalo`) — solver propagation scratch arena (`kenken-solver/src/solver.rs`).
  - `rayon` (`kenken-gen/parallel-rayon`) — batch count/uniqueness parallelism (`kenken-gen/src/lib.rs`).
  - `dlx_rs` (`kenken-solver/solver-dlx`) — Latin-square exact cover (`kenken-solver/src/dlx_latin.rs`).
  - `varisat` (`kenken-solver/sat-varisat`) — SAT uniqueness (Latin + staged cage allowlists) (`kenken-solver/src/sat_latin.rs`, `kenken-solver/src/sat_cages.rs`).
  - `rkyv` (`kenken-io/io-rkyv`) — Snapshot v1 encode/decode (`kenken-io/src/rkyv_snapshot.rs`).
  - `uniffi` (`kenken-uniffi/ffi-uniffi`) — bindings crate + scaffolding (`kenken-uniffi/*`).
  - `criterion` — active bench harness (`kenken-solver/benches/*`).
  - `smallvec` — core hotpath storage for cage cells.
  - `thiserror` — typed library errors.
  - `z3` — optional verification backend (`kenken-solver/src/z3_verify.rs`, `kenken-verify/*`).

### Partial integrations / decision debt
  - `bitvec` (`kenken-core/core-bitvec`) — `BitDomain` exists in `kenken-core`, but the main solver still uses fixed-width integer masks. This is decision debt, not an automatic integration backlog.
  - `mimalloc` (`kenken-cli/alloc-mimalloc`) — CLI-only allocator integration; core crates remain allocator-agnostic.
  - `likely_stable` (`kenken-solver/perf-likely`) — narrow hotpath hinting only.
  - `tracing` (`kenken-solver/tracing`) — instrumentation hooks are present, but subscriber installation is still adapter-owned.
  - `static_assertions` — useful and available, but only lightly exercised today.
  - `proptest` — present in tests, but not yet the backbone of a broader property-testing story.

### Planned / aspirational stack
These crates are still part of the intended ecosystem, but the docs should not
imply that they are already integrated:

  - `fixed`
  - `rand_pcg`
  - `wide`
  - `dashmap`
  - `parking_lot`
  - `num-integer`
  - `petgraph`
  - `anyhow`
  - `nom`
  - `bytemuck`
  - `soa_derive`
  - `tracing-android`
  - `bolero`
  - `kani`
  - `creusot`

## Synergy & Architecture
- Metal: bumpalo + smallvec are real current choices; `mimalloc`, `wide`, and `bitvec`
  are narrower or still under evaluation.
- Brain: `dlx_rs`, `varisat`, and `z3` are real optional solver/verification backends;
  `petgraph` remains aspirational.
- Scale: `rayon` and `rkyv` are current; `parking_lot` and `dashmap` are still planned.

## Integration Plan
- Make an explicit decision on `bitvec`: promote with benchmarks, demote to research
  status, or remove.
- Use `docs/canonical/bitvec_decision.md` as the canonical exit-criteria record.
- Replace the handwritten CLI parser with a standard crate before expanding the
  command surface further.
- Continue growing property tests and benchmark coverage around the already-adopted
  `criterion` and `proptest` lanes.
- Keep `nom`, `dashmap`, `parking_lot`, and similar crates clearly documented as
  planned until code integration actually lands.
