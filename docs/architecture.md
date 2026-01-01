# Architecture Overview

Note: This file is a supporting sketch. The synthesized, reconciled plan lives in `docs/plan.md`.

## Scope
Cleanroom KenKen (sgt-puzzles “Keen”-compatible ruleset) solver/generator in pure Rust nightly.

Primary deliverable is a **library-first** engine that can be embedded in:
- Android/iOS apps (via bindings/adapters)
- desktop apps
- servers / headless tooling

Targets: grids up to 16×16 in the core model (solver/generator performance goals focus on 4×4–9×9), cages with `+ − × ÷` and equality, deterministic reproducibility, and opt-in high-performance backends.

## Workspace Layout
The current workspace is intentionally staged:
- `kenken-core`: model + validation + formats
  - sgt-puzzles “desc” import/export for corpus/regression (`format::sgt_desc`)
  - optional bitvec-backed domains (`core-bitvec`)
- `kenken-solver`: deterministic solver + solution counting
  - optional `bumpalo` scratch arenas (`alloc-bumpalo`)
  - optional DLX Latin utilities (`solver-dlx`)
  - optional Varisat SAT Latin utilities (`sat-varisat`)
- `kenken-gen`: generation scaffolding
  - batch solve/uniqueness APIs (optionally parallel via `rayon`)
  - deterministic RNG plumbing (`seed` module)
- `kenken-io`: versioned snapshots (currently `rkyv` snapshot v1 behind `io-rkyv`)
- `kenken-uniffi`: UniFFI bindings crate (minimal solve/count surface via sgt “desc”)
- `kenken-cli`: reference CLI tooling (`solve`/`count` over sgt “desc`)

## Data Flow
Generator → Solver (uniqueness, difficulty) → IO snapshot → adapters (CLI / UniFFI / …)

## Threading Model
Default engine logic is deterministic; optional parallelism is used for **batch** workloads (generation pipelines, corpus evaluation).

## Determinism
Determinism is a first-class requirement:
- Explicit RNG algorithm (`ChaCha20Rng`) and seed mapping in `kenken-gen`
- No reliance on hashmap iteration order in solver hotpaths
- Solution counting uses early-exit limits (uniqueness = count up to 2)

## FFI Boundary
Near-term FFI boundary is UniFFI:
- Parse a puzzle from sgt “desc”
- Solve / count solutions

Future adapters can add:
- C ABI layer (cbindgen) for non-Swift/Kotlin targets
- platform-specific integration examples (Android/iOS)

## Targets
- Toolchain pinned in `rust-toolchain.toml`.
- CI is aligned to the pinned nightly in `.github/workflows/ci.yml`.
