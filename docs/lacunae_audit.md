# Audit and Lacunae Resolution (as of 2025-12-31T21:59:35.795Z)

## Identified Gaps
- DLX cage tuple explosion for large additive/multiplicative cages.
- Exact CNF mapping for SAT side-constraints not fully specified.
- Difficulty grader heuristics only outlined; no formal rubric or benchmarks.
- Android build integration examples (Gradle/Cargo) not included.
- UniFFI data model lacks enums for ops (currently strings).
- Deterministic arithmetic (fixed-point) usage not wired.
- Verification (creusot) optional but no spec harness.

## Resolutions/Plans
- Use hybrid DLX+SAT: precompute bounded tuples; if |tuples|>threshold, switch to SAT pruning with varisat; document thresholds per N.
- Write CNF templates: Add/Mul via cardinality constraints (pairwise or ladder encoding), Sub/Div via pair selection clauses; include examples for 2–4 cell cages.
- Define difficulty rubric: tiers by lookahead depth, number of bifurcations, propagation counts; add criterion benches and gold puzzles with target tiers.
- Provide Gradle/Cargo skeleton: mozilla-rust-android-gradle config, cargo-ndk targets, UniFFI build steps.
- Replace `op: String` with Rust enum and UDL union/enum; add validation at parse.
- Integrate `fixed` for cage math determinism; benchmark vs integer-only approach.
- Add creusot specs for core invariants (Latin constraints, cage satisfaction) and run in CI.

## Next Actions
- Implement DLX matrix builder and SAT encoder; add unit/property tests.
- Generate UniFFI bindings; create Android sample app module.
- Write difficulty grader; calibrate with sgt-puzzles outputs.
