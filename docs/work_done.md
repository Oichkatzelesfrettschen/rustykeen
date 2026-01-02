# Work done (audit snapshot)

This document is a "what exists today" counterbalance to `docs/plan.md` (what we're building toward).

Last updated: 2026-01-01

## Toolchain / CI
- Toolchain pinned: `rust-toolchain.toml` (`nightly-2026-01-01`)
- CI aligned to the pinned nightly: `.github/workflows/ci.yml`
- CI gates: `cargo fmt --check`, `cargo clippy --all-targets --all-features -D warnings`, `cargo test --all-targets`
- Fuzz harness: `fuzz/` with `fuzz_sgt_desc_parser` and `fuzz_solver` targets (cargo-fuzz)

## Workspace crates (implemented)

### `kenken-core`
- Model: `Puzzle`, `Cage`, `CellId`, `Coord` (`kenken-core/src/puzzle.rs`)
- Rules: `rules::{Ruleset, Op}` (`kenken-core/src/rules.rs`)
- Validation: coverage, duplicates, cage shape rules, connectivity (`Puzzle::validate`)
- Upstream format import/export: sgt-puzzles “desc” (`kenken-core/src/format/sgt_desc.rs`)
- Optional:
  - `core-bitvec`: `BitDomain` (not yet used by solver)
  - `perf-assertions`: compile-time layout checks (`static_assertions`)
  - `serde` (default off): derives for `Op` and `Ruleset` only
- Tuple enumeration helper:
  - `Cage::valid_permutations(...)` for SAT tuple allowlists (`kenken-core/src/puzzle.rs`)

### `kenken-solver`
- Deterministic backtracking solver + solution counting up to a limit (`kenken-solver/src/solver.rs`)
- Deduction tiers (`DeductionTier`) for propagation strength vs search
- Difficulty classification:
  - `TierRequiredResult` and `classify_tier_required()`: determine minimum deduction tier needed
  - `classify_difficulty_from_tier()`: primary difficulty classification matching upstream behavior
  - `SolveStats.backtracked`: tracks whether guessing was required
  - Calibration corpus: `kenken-solver/tests/corpus_difficulty.rs`
- Optional performance/certification modules:
  - `alloc-bumpalo`: bump allocation scratch buffers for propagation
  - `solver-dlx`: Latin exact-cover utilities via `dlx-rs` (`kenken-solver/src/dlx_latin.rs`)
  - `sat-varisat`: SAT uniqueness hooks via `varisat`:
    - Latin-only helper (`kenken-solver/src/sat_latin.rs`)
    - staged cage allowlist encoding with a tuple threshold (`kenken-solver/src/sat_cages.rs`)

### `kenken-gen`
- Batch solving/uniqueness plumbing:
  - `count_solutions_batch(...)`, `is_unique_batch(...)` (`kenken-gen/src/lib.rs`)
  - optional `parallel-rayon`: parallel execution via `rayon`
- Deterministic RNG mapping: `seed::rng_from_u64` (`kenken-gen/src/seed.rs`)
- Generator MVP (feature-gated):
  - `gen-dlx`: DLX-backed Latin solution generation + random cage partition + op/target assignment + reject-until-unique loop (`kenken-gen/src/generator.rs`)

### `kenken-io`
- `io-rkyv`: snapshot v1 encode/decode:
  - magic header + versioned structs
  - conversion to/from `kenken_core::Puzzle`
  - roundtrip test (`kenken-io/src/rkyv_snapshot.rs`)
  - Snapshot v2 added: persists `Ruleset` and provides `decode_snapshot(...)` compatibility entrypoint (`docs/rkyv_snapshot_v2.md`)

### `kenken-uniffi`
- UniFFI scaffolding (`kenken-uniffi/build.rs`, `kenken-uniffi/src/keen.udl`)
- Minimal exported API:
  - solve and count from sgt “desc” (`kenken-uniffi/src/lib.rs`)
  - optional `gen` feature: generate sgt “desc” + return solution grid (`kenken-uniffi/src/lib.rs`)

### `kenken-cli`
- Reference CLI for solve/count:
  - `kenken-cli solve --n N --desc DESC --tier ...`
  - `kenken-cli count --n N --desc DESC --limit ...`
  (`kenken-cli/src/main.rs`)
 - Installs a default tracing subscriber (`kenken-cli/telemetry-subscriber`) so solver/SAT traces are visible without extra wiring.

## Documentation tooling
- Crate-level rustdoc uses `#![doc = include_str!("../README.md")]` per crate.
- mdBook skeleton exists at `docs/book/` for narrative docs.

## Cleanroom posture (docs)
- Operational cleanroom policy: `docs/cleanroom_policy.md`
- Upstream distillation notes (no code copied): `docs/upstream_sgt_puzzles_keen.md`
- Dependency audit tooling + distilled docs: `scripts/` and `docs/deps/*.md`

## Testing infrastructure
- Criterion benchmarks: `kenken-solver/benches/solver_smoke.rs` (solve_one, count_solutions, deduction tiers)
- Proptest property tests: `kenken-core/tests/prop_cage_semantics.rs` (cage arithmetic invariants)
- Golden corpus tests: `kenken-solver/tests/corpus_sgt_desc.rs` (2x2, 3x3, 4x4 puzzles with known solution counts)
- Difficulty calibration tests: `kenken-solver/tests/corpus_difficulty.rs` (tier-required classification validation)
- Fuzz targets: `fuzz/fuzz_targets/` (parser and solver coverage)

## Major lacunae (next engineering milestones)
- SAT encoding for full cage arithmetic constraints (not just Latin) and uniqueness proofs that incorporate cages.
- Generator pipeline hardening (minimization + difficulty scoring + expanded calibration corpus).
- Expand difficulty calibration corpus with more diverse puzzles (Normal/Hard tier requirements).
- Stable public API policy (semver, feature gates, versioned snapshot evolution) and compatibility tests at scale.
