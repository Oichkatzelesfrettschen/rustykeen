# Cleanroom Plan: Porting sgt-puzzles keen.c to Rust (2026)

## Source of Truth
- Analyze keen.c: focus on new_game_desc() (generator) and solve_game() (solver); confirm Latin Square + cage arithmetic.

## Rust Core Goals
- Memory safety; cache-local data (flattened arrays, SoA); deterministic RNG; reproducible puzzles; SIMD-assisted validation.

## Data Structures
- Board [u8; N*N]; Domains rows/cols u16 bitmasks; Cages with SmallVec cells; ops Add/Sub/Mul/Div/Eq; fixed-point math if needed (`fixed`).

## Algorithms
- Solver: Algorithm X (DLX) for Latin constraints + cage pruning; AC-3 propagation; uniqueness checker.
- Generator: backtracking with solver; parallel heuristic racing (rayon); minimization; difficulty via lookahead depth.

## Crates
- Core: bitvec, fixedbitset, smallvec, itertools, rand_pcg, rkyv, portable-simd, tracing, thiserror/anyhow.
- Optional: fixed (deterministic arithmetic), creusot (verification), ash/volk or wgpu (graphics), cxx (bridge for legacy).

## Android Integration
- UniFFI UDL defines API: generate_puzzle(seed,difficulty) -> PuzzleState; Kotlin/Compose renders; avoid manual JNI.

## Steps
1) Spec models/APIs; 2) Implement DLX + cage propagation; 3) Port generator semantics; 4) Uniqueness + minimizer; 5) Bench/prop-test; 6) UniFFI bindings; 7) Android Compose UI; 8) Perfetto/Simpleperf tuning.
