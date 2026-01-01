# Synthesis Plan: Pure-Rust “Keen” (KenKen-style) Engine

This document reconciles existing `docs/*` into a single, engineering-first plan: architecture, APIs, build/verification, and optimization/instrumentation practices for a **pure Rust** puzzle logic base usable by many frontends.

## 0) Problem statement and goals

### Primary deliverable
A Rust library (and optionally a workspace of crates) that provides:
- Puzzle model + validation
- Solver (find solution(s), enumerate, count for uniqueness)
- Generator (create puzzles with guaranteed uniqueness and difficulty targets)
- Deterministic reproducibility (seeded RNG, stable ordering, snapshot support)

### Non-goals (for v0)
- UI framework (Android/desktop/web UI belongs to adapters)
- Rendering/graphics engine
- Network/multiplayer
- “Perfect” difficulty grading (start with calibrated heuristics + corpus)

## 1) Cleanroom constraints (operational)
The repo should treat “reverse engineering” as a process, not just an intent.

Recommended rules:
- Keep “specifications from behavior” (black-box observations) separate from any source-derived notes.
- Tests/corpora should be derived from puzzle definitions and observed behavior, not copied text/code.
- Record provenance for every non-trivial decision (where did this rule/format come from?).
- Avoid importing upstream code or copying constants/heuristics without re-deriving them.

See also: `docs/cleanroom_plan.md`.
Upstream study notes and compatibility-relevant details: `docs/upstream_sgt_puzzles_keen.md`.
Categorized gaps to resolve: `docs/lacunae_deep_dive.md`.
Dependency/feature audit: `docs/dependencies.md`.

## 2) Proposed crate/workspace architecture

The existing docs propose a workspace layout; this plan keeps that separation but allows staging:

### Phase A (minimal viable engine)
- `kenken-core`: model + parsing/format + validation
- `kenken-solver`: deterministic solver + solution counting (uniqueness)

### Phase B (generation and IO hardening)
- `kenken-gen`: generator + minimizer + difficulty scoring
- `kenken-io`: versioned schema + round-trip compatibility tests (optional if `core` owns IO)

### Phase C (adapters)
- `kenken-cli`: reference CLI runner + corpus tooling
- `kenken-wasm`: wasm-bindgen (optional)
- `kenken-uniffi`: UniFFI bindings (optional)
- `kenken-cabi`: stable C ABI surface (optional)

Cross-cutting principle (from `docs/vendor_neutral.md`):
- **Core** contains no platform logic; adapters are feature-gated.

## 2.5) Decisions to lock (to stop doc drift)

These are the minimum decisions that should be recorded (ADR-style is fine) so that implementation, docs, and adapters converge.

- RNG: use a single, explicit algorithm type for reproducibility across platforms/builds (recommended: `rand_chacha::ChaCha20Rng`); avoid “whatever `thread_rng()` does”.
- Uniqueness definition: “unique” means exactly one solution under the same rule set; implement by counting solutions up to 2 (early exit).
- Adapter strategy: UniFFI for Kotlin/Swift; optional C ABI for other native toolkits; WASM bindings separate. Core stays platform-agnostic.
- `no_std` posture: `kenken-core` is written “no_std-ready” but ships `std`-enabled by default until a real `no_std+alloc` story is validated in CI.
- Feature gates: keep `simd`, `rayon`, `verification`, and adapter features off by default; treat them as opt-in capabilities.

## 2.6) Execution order (current focus)

To keep the optimization stack incremental and auditable, implementation work proceeds in this order:
1) `bumpalo` (solver arena / scratch buffers)
2) `rayon` (generation and batch parallelism)
3) `dlx-rs` (Latin-square exact cover core)
4) `varisat` (SAT-based uniqueness proof hooks)
5) `rkyv` (snapshots and migration away from serde-based persistence)
6) `uniffi` (Kotlin/Swift bindings)

Current status snapshot lives in `docs/dependency_matrix.md`.

## 3) Data model and invariants

### Puzzle model (core)
- `N` is the grid size (typically 4–9).
- Each cell value is in `1..=N`.
- Row/column “Latin” constraints: all-different per row and per column.
- Cages:
  - `cells: SmallVec<CellId, K>` (K tuned; typically 4–6)
  - `op: Op = Add | Sub | Mul | Div | Eq`
  - `target: i32` (exact type depends on bounds; prefer signed for convenience)
  - Cage semantics defined in `docs/design.md` + `docs/cnf_templates.md`.
  - If upstream compatibility is desired for corpora/tests, support parsing/printing the sgt-puzzles “desc” format (`docs/upstream_sgt_puzzles_keen.md`).

### Solver state (solver)
- Domain representation: bitset (e.g., `u16` for N≤16), stored SoA for cache locality.
- Deterministic ordering of candidate iteration (avoid hash-map iteration in core logic).
- Explicit recursion limits / step budgets to prevent pathological puzzles from hanging.

## 4) Solver architecture (deterministic, instrumentable)

The docs converge on a hybrid approach:
- **Fast path**: propagation + DFS with MRV/LCV and cage-specific pruning
- **Exact-cover path**: DLX for Latin constraints (and optionally some cage constraints)
- **Fallback/verification**: SAT/Z3 only where needed (tuple explosion, certification)

### Recommended staged approach
1) Implement a baseline deterministic backtracking solver with:
   - MRV (minimum remaining values)
   - Forward checking + simple cage pruning
   - Solution enumeration and “count up to 2” for uniqueness
2) Add DLX for Latin core (see `docs/solve_dlx.rs`, `docs/exact_cover_matrix.md`)
3) Add cage arithmetic acceleration:
   - Precomputed tuple tables for small cages
   - Threshold-based fallback to SAT encoding when tuple explosion occurs
4) (Optional) Z3 certification step for “uniqueness proof” on final outputs

Key lacuna to resolve early:
- Define the exact thresholds, encodings, and when SAT/Z3 is invoked (cost model + tests).

## 5) Generator architecture (uniqueness + difficulty)

Generator pipeline (from `docs/design.md`, `docs/cleanroom_plan.md`):
1) Produce a full solution grid (Latin) deterministically from seed.
2) Partition into cages (topology/shapes) with constraints (avoid degenerate cages).
3) Assign operations/targets consistent with the solution.
4) Validate uniqueness by counting solutions (early exit at >1).
5) Minimize clues/cages while preserving uniqueness.
6) Score difficulty using solver-instrumented metrics; accept/reject until target tier.

Lacuna to resolve:
- A concrete, versioned **difficulty rubric** and a calibration corpus.
Current placeholder rubric notes: `docs/difficulty.md`.

## 6) Public API strategy (library-first)

Design goals:
- Small, stable, well-tested API surface in `kenken-core`
- “Heavy” options behind features (parallelism, SIMD, verification, tracing sinks)
- Versioned IO schema for forward/backward compatibility

Suggested minimal API (Phase A):
- `Puzzle::validate() -> Result<(), Error>`
- `Solver::solve_one(&Puzzle) -> Result<Option<Solution>, Error>`
- `Solver::count_solutions_up_to(&Puzzle, limit: u32) -> Result<u32, Error>` (uniqueness = `<=1`)
- `Generator::generate(config) -> Result<Puzzle, Error>` (Phase B)

## 7) Performance and instrumentation (what “best practice” means here)

What the docs already capture well:
- Cache-local SoA layouts, avoid heap in hot loops (`docs/engineering.md`, `docs/distilled_corpus.md`)
- Feature gating for platform adapters (`docs/feature_gating.md`, `docs/vendor_neutral.md`)
- Build/profile tuning (LTO, codegen-units, panic=abort) (`docs/riced_build.md`)
- Tracing-based instrumentation strategy (`docs/telemetry_build_assets.md`)
- Optional formal methods + fuzzing (`docs/formal_verification.md`)

Gaps to resolve:
- Define a **baseline benchmark suite** and “perf budgets” (time/allocs) per grid size.
- Define an instrumentation contract:
  - What spans/events exist (solve, propagate, branch, backtrack, tuple-gen)?
  - What metrics are exported (counts, timings, max depth)?
  - How are they named/versioned so tools can compare runs over time?

### 7.1) Instrumentation contract (v0 proposal)

Use `tracing` spans with stable names and a small set of numeric fields that allow regression tracking:
- Spans:
  - `kenken.solve_one` (fields: `n`, `seed?`, `strategy`)
  - `kenken.solve.count_solutions` (fields: `limit`)
  - `kenken.propagate` (fields: `queue_len`)
  - `kenken.search.branch` (fields: `cell`, `domain_size`, `depth`)
  - `kenken.search.backtrack` (fields: `depth`)
  - `kenken.gen.generate` (fields: `n`, `seed`, `target_tier`)
  - `kenken.gen.minimize` (fields: `passes`)
- Counters (record as fields on span exit or as events):
  - `propagations`, `assignments`, `failures`, `max_depth`, `branches`
  - `tuple_table_hits`, `tuple_table_misses`, `sat_invocations`

Rule: metrics names are treated as API (changing them requires a doc note and test updates).

### 7.2) Benchmark suite (v0 proposal)

Benchmarks should answer three questions:
1) “Do we solve fast enough?” (solver latency)
2) “Do we generate fast enough?” (throughput of unique puzzles)
3) “Do we regress?” (time/allocs/memory)

Recommended layout:
- `benches/solver_smoke.rs`: a small fixed corpus (4×4, 6×6, 9×9) with stable seeds.
- `benches/generator_smoke.rs`: generate N puzzles with fixed config; report acceptance rate and time.
- A single “perf budget” table tracked in docs (e.g., median solve time per tier for 6×6 on a reference machine).

## 8) Testing, verification, and corpora

Minimum bar for correctness:
- Unit tests for cage semantics and edge cases
- Property tests:
  - Generated puzzles validate
  - Solving returns a valid grid
  - Uniqueness checker matches enumeration for small N
- Golden corpus tests:
  - A fixed set of puzzles with known solutions/uniqueness/difficulty tiers

Optional verification layers:
- Kani harnesses for index/bitset arithmetic (stubs live in `tools/verify/*` today; will migrate into a dedicated verify crate/CI job when adopted)
- Z3 uniqueness/certification (stub exists; needs cage constraints and integration)

## 9) Build system and repo hygiene

Immediate repo-level lacunae to address (recommended):
- Add a real `Cargo.toml` workspace (or a single crate) so the repo builds.
- Add `rust-toolchain.toml` to pin nightly and required components.
- Add CI: `cargo fmt`, `cargo clippy`, `cargo test` on stable + nightly as appropriate.
- Fix licensing file to contain the intended license text (current `LICENSE` is an HTML redirect).

### 9.1) Quality gates (recommended defaults)
- `cargo fmt --check` (always)
- `cargo clippy --all-targets --all-features` (tiered: allow feature-matrix expansion later)
- `cargo test --all-targets` (with deterministic seeds)
- `cargo test -F verification` (optional, can be allowed-to-fail initially)
- `cargo bench` (nightly only if benches require unstable features)

## 10) Milestones (actionable)

### M0: Bootstrap (days)
- Workspace + crate skeletons compile
- Core types + parser + validator
- Basic solver with determinism + solution counting (N up to 6)

### M1: Correctness and corpus (weeks)
- Golden corpus + property tests + fuzz hooks
- Cage semantics complete for all ops and multi-cell cages
- Uniqueness robust + step budgets

### M2: Performance architecture (weeks)
- DLX Latin solver integrated
- Tuple tables + SAT fallback thresholds implemented
- Bench suite + regression tracking + tracing spans

### M3: Generator (weeks)
- End-to-end generator producing unique puzzles with target tiers
- Minimizer + difficulty scoring calibrated

### M4: Adapters (as needed)
- CLI + optional UniFFI + optional WASM

## 11) Open questions (need explicit answers)

- Licensing: current repo ships GPLv2 text in `LICENSE`, but decide whether a more permissive license is desired for broad embedding before publishing crates.
- Public crate naming: `kenken-*` workspace vs a single `keen`/`rustykeen` crate that re-exports modules.
- Grid size scope: hard cap at 9×9 (common) vs generalized N (impacts bitset types and perf assumptions).
- SAT/Z3 policy: which solver(s) are allowed as optional deps, and when they are invoked (generation only vs solve path too).
- Upstream parity: target solver/generator parity with sgt-puzzles difficulty tiers first, or diverge early toward DLX/SAT?
