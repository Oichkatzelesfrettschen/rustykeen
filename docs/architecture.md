# Architecture Overview

## Scope
Cleanroom KenKen solver/generator in pure Rust nightly with Android AGDK integration via GameActivity and a minimal C++/JNI shim; targets 4x4–9x9 grids, cages with +, −, ×, ÷, and equality; reproducible puzzles, deterministic cores, optional heuristics.

## Workspace Layout
- kenken-core: no_std-ready core types (Grid<N>, Cell, Domain, Cage, Op), bitsets, portable_simd helpers, constraint interfaces; const-generic sizes; feature flags `std`, `simd`.
- kenken-solver: propagation engine (arc consistency, forward checking), search (DFS, MRV, LCV, AC-3/PC), conflict tracking, solution enumerator; pluggable heuristics.
- kenken-gen: puzzle constructor, cage partitioner, clue assignment, uniqueness checker, difficulty estimator, minimizer.
- kenken-io: serde models, TOML/JSON formats, parser for textual cage specs, exporter/importer; schema versioning.
- kenken-cli: TUI/CLI (clap) to solve/generate/benchmark; seed control, output formats.
- kenken-wasm: WASM bindings via wasm-bindgen for web demos.
- android-app: GameActivity-based app; C++ shim (AGDK) calls Rust FFI; optional Vulkan renderer for native UI.

## Data Flow
Generator → Solver (uniqueness, difficulty) → IO serialize → CLI/Android consume → Render results.

## Threading Model
Core/solver worker pool; deterministic single-threaded mode; Android main thread for events, render thread for Vulkan, workers for compute.

## Determinism
Seeded ChaCha20 RNG; fixed ordering; snapshot/rehydration of solver state.

## FFI Boundary
C ABI exports: init, load_puzzle(json), solve_step, solve_all, generate(config), shutdown; headers via cbindgen; hidden visibility.

## Targets
Rust nightly; Android NDK r28+ (API 26+); CLI desktop; optional WASM; kenken-core supports no_std with alloc.
