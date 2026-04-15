# Work done (audit snapshot)

This document is a "what exists today" counterbalance to `docs/plan.md` (what we're building toward).

Last updated: 2026-04-06

## Toolchain / local validation
- Toolchain pinned: `rust-toolchain.toml` (`nightly-2026-04-06`)
- CI/CD workflows intentionally removed to stop hosted-runner quota consumption
- Mandatory local gate: `just ci`
- Individual local gates: `just fmt`, `just lint`, `just test`, `just audit`
- Local guardrail script: `./scripts/local_quality_gates.sh` (implementation behind the canonical local gate bundle)
- Dependency/security audit (2026-04-06): `cargo audit` clean, `cargo outdated` clean (root + transitive)
- Benchmark dependency overlap consolidated: active graph now uses `criterion 0.8.2` only (no `criterion 0.5.x` line)
- Fuzz harness: `fuzz/` with `fuzz_sgt_desc_parser` and `fuzz_solver` targets (cargo-fuzz)
- Durable benchmark and crash artifacts now live under `artifacts/` instead of the repo root

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
  - `solver-dlx`: Latin exact-cover utilities via internal DLX implementation (`kenken-solver/src/dlx.rs` + `kenken-solver/src/dlx_latin.rs`)
  - `sat-varisat`: SAT uniqueness hooks via `varisat`:
    - Latin-only helper (`kenken-solver/src/sat_latin.rs`)
    - staged cage allowlist encoding with tuple threshold policy:
      - permissive mode with native fallback (`puzzle_uniqueness_via_sat`)
      - strict fail-closed certification mode (`puzzle_uniqueness_via_sat_strict`)
      - DIMACS export for external SAT tooling (`export_puzzle_dimacs_strict`)
  - `verify`: Z3 uniqueness backend now encodes full cage arithmetic constraints (Eq/Add/Mul/Sub/Div) with alternate-model checks (`kenken-solver/src/z3_verify.rs`)
    - SMT2 export for external proof tooling (`export_puzzle_smt2`)

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
- `format-sgt-desc`: non-binary text backend for portable/auditable snapshots:
  - encode/decode helpers in `kenken-io/src/sgt_desc_snapshot.rs`
  - roundtrip test coverage (`sgt_desc_roundtrip`)

### `kenken-uniffi`
- UniFFI scaffolding (`kenken-uniffi/build.rs`, `kenken-uniffi/src/keen.udl`)
- Minimal exported API:
  - solve and count from sgt “desc” (`kenken-uniffi/src/lib.rs`)
  - optional `gen` feature: generate sgt “desc” + return solution grid (`kenken-uniffi/src/lib.rs`)

### `kenken-verify`
- Verified-solver APIs exposed from crate surface (`kenken-verify/src/lib.rs`)
- SAT/Z3 agreement interfaces are implemented and tested:
  - `verify_with_sat(...)` (`kenken-verify/src/sat_interface.rs`)
  - `verify_with_z3(...)` (`kenken-verify/src/z3_interface.rs`)
- Strict SAT certification mode is enforced for verification APIs (`puzzle_uniqueness_via_sat_strict`)
- External proof artifact exports are implemented:
  - `generate_cnf(...)` for DIMACS CNF
  - `generate_z3_smt2(...)` for SMT-LIB2

### `kenken-cli`
- Reference CLI for solve/count/benchmark:
  - `kenken-cli solve --n N --desc DESC --tier ...`
  - `kenken-cli count --n N --desc DESC --limit ...`
  - `kenken-cli benchmark --n N --count C --tier ...`
  (`kenken-cli/src/main.rs`)
 - Installs a default tracing subscriber (`kenken-cli/telemetry-subscriber`) so solver/SAT traces are visible without extra wiring.

### `kenken-sdl2`
- Cross-platform SDL2 demonstration frontend (`kenken-sdl2/src/main.rs`):
  - keyboard + mouse cell editing
  - native display-aware initial window sizing
  - dynamic scaling controls (`+/-`, mouse wheel) and fullscreen toggle (`F11`)
  - engine-backed solve/uniqueness actions (`S` / `U`)

## Documentation tooling
- Crate-level rustdoc uses `#![doc = include_str!("../README.md")]` per crate.
- mdBook skeleton exists at `docs/book/` for narrative docs.
- Cross-target API parity contract is documented and drift-tested (`docs/cross_target_api_parity_contract.md`, `kenken-wasm/tests/cross_target_api_parity_contract.rs`).
- ISA benchmark artifact registry is published (`docs/isa_benchmark_artifact_registry.md`).

## Cleanroom posture (docs)
- Operational cleanroom policy: `docs/cleanroom_policy.md`
- Upstream distillation notes (no code copied): `docs/upstream_sgt_puzzles_keen.md`
- Dependency audit tooling + distilled docs: `scripts/` and `docs/deps/*.md`

## Testing infrastructure
- Criterion benchmarks: `kenken-solver/benches/solver_smoke.rs` (solve_one, count_solutions, deduction tiers)
- Proptest property tests: `kenken-core/tests/prop_cage_semantics.rs` (cage arithmetic invariants)
- Golden corpus tests: `kenken-solver/tests/corpus_sgt_desc.rs` (2x2, 3x3, 4x4 puzzles with known solution counts)
- Difficulty calibration tests: `kenken-solver/tests/corpus_difficulty.rs` (tier-required classification validation)
- Cross-target contract drift tests: `kenken-wasm/tests/cross_target_api_parity_contract.rs`
- Z3 verification regression tests: `kenken-solver/tests/z3_golden_verify.rs`
- Fuzz targets: `fuzz/fuzz_targets/` (parser and solver coverage)

## Major lacunae (next engineering milestones)
- Generator pipeline hardening (minimization + difficulty scoring + expanded calibration corpus).
- Expand difficulty calibration corpus with more diverse puzzles (Normal/Hard tier requirements).
- Stable public API policy (semver, feature gates, versioned snapshot evolution) and compatibility tests at scale.
