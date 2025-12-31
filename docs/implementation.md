# Implementation Guide

## Crates
Workspace members: kenken-core, kenken-solver, kenken-gen, kenken-io, kenken-cli, kenken-wasm.
Features: `std` (off in core), `simd`, `rayon`, `deterministic`.

## Core
Types: Grid<N>, CellId, BitDomain (u16 for 1..=9), Cage, Op; bit ops for speed; invariants documented; no unsafe.
Constraints: trait Constraint; propagate(&mut State) → Delta; unit + property tests.

## Solver
State: domains, constraint graph; AC-3 queue; search: MRV, LCV, fail-fast; backtracking stack; uniqueness validator; config toggles.

## Generator
Create full solution → cages → targets → minimize with uniqueness checks.

## IO
Serde structs; schema version; good errors; round-trip tests.

## CLI
clap commands: solve, generate, minimize, bench; seed option; JSON/TOML/ASCII output.

## WASM
wasm-bindgen exports for web demo.

## Android
C++ GameActivity shim; minimal JNI; link Rust staticlib via CMake; call exported C functions.
Flags: `-fvisibility=hidden`, ThinLTO, `-fsanitize=hwaddress` (debug); manifest MTE.
ATrace around FFI; Simpleperf usage documented.

## Testing/Benchmarks
cargo test; proptest; criterion benches; determinism tests.
