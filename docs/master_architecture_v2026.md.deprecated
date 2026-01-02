# Master Architecture v2026 (2025-12-31T22:07:20Z)

## Metal Data Layout
- Const generics Grid<N>; aligned arrays; row/col conflict masks; SIMD fast-path is_safe_simd.

## Hybrid Solver Pipeline
- Propagate small cages; Lazy DLX with step callbacks; SAT fallback for complex cages; uniqueness with early-exit.
- Formal layer: Z3-backed uniqueness check; Kani proofs for hotpaths; TLA+ spec for state machine; Bolero fuzzing.

## Generator (Topology-First)
- Shapes via petgraph; Latin solution via DLX; assign ops/targets; validate via DLX/SAT; parallelize with rayon.

## Nightly Flags
- portable_simd, allocator_api, test, slice_group_by.

## Feature Gates
- android/wasm/c_abi/simd/mimalloc; see feature_gating.md.
