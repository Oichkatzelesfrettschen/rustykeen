# rustykeen

Cleanroom/reverse-engineering effort to port the classic “Keen” (KenKen-style) puzzle into a **pure Rust** solver/generator **game-logic crate**, with a build + instrumentation workflow suitable for iterative optimization.

This repo currently contains **planning documentation** and a couple of verification stubs; the implementation workspace described in the docs is not yet bootstrapped.

## Goals
- Pure Rust core puzzle model + solver + generator (library-first).
- Deterministic, reproducible generation/solving (seeded RNG, stable ordering).
- Performance-oriented architecture (cache-local data layout, optional SIMD, optional parallelism).
- Portable bindings layer so other projects can build Android/iOS/desktop/web apps on top.

## Docs (start here)
- `docs/plan.md` (primary synthesized plan)
- `docs/cleanroom_plan.md` (cleanroom constraints and porting approach)
- `docs/architecture.md`, `docs/design.md`, `docs/engineering.md` (supporting, partially overlapping)
- `docs/upstream_sgt_puzzles_keen.md` (upstream behavior/format/algorithm distillation)
- `docs/lacunae_deep_dive.md` (categorized gap analysis)

## Toolchain
- Rust nightly via rustup (see `docs/plan.md` for recommended repository-level setup).
