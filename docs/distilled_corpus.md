# Distilled Architectural Corpus (v2026)

## Cleanroom Crate Stack
- Solver: dlx_rs (Algorithm X) for Latin; varisat (CDCL) to prove uniqueness and handle complex cages.
- Data layout: bitvec/fixedbitset for compressed candidates and masks; portable-simd/wide for vectorized checks.
- Memory: mimalloc (global allocator, non-iOS) + bumpalo (arena) for hotpath nodes.
- Concurrency: rayon + parking_lot + dashmap (NoHashHasher) for parallel generation and caches.
- Topology: petgraph (DSU/union-find) for cage shapes.
- I/O: rkyv zero-copy snapshots; rust-embed to ship seeds/topologies.
- Telemetry: tracing + tracing-subscriber, tracing-android/oslog, tracing-tracy for remote profiling.
- Architecture: crux for headless UI; UniFFI bridges to Kotlin/Swift; C ABI or WASM as needed.

## Hybrid Lazy Solver
- Fast propagation on small cages via bitvec masks.
- Latin-only DLX matrix; lazy callback verifies cage math when cages become full; SAT fallback for tuple explosion.

## Metal Data Layout
- Const-generics Grid<N> with 16-byte alignment; flat cells [u8; N*N]; row/col conflict masks as u16; SIMD fast-path val checks.

## Riced Build Pipeline
- Cargo profiles: fat LTO, codegen-units=1, panic=abort; .cargo/config per-target flags; mold/lld linkers.
- PGO (cargo-pgo) then BOLT (llvm-bolt) training; emit-relocs + frame pointers for Android; iOS falls back to system allocator.

## build.rs Orchestration
- Detect Android → emit-relocs, force frame pointers; iOS → disable BOLT/mimalloc; rerun on changes.

## Vendor-Neutral Execution
- Core in pure Rust, no_std-ready; adapters feature-gated (android/ios/wasm/c_abi); tracing init centralized; assets embedded; reproducible outputs.

## Actionable Blueprint
- Implement DLX+lazy SolverContext (done in docs/solve_dlx.rs), MRV ordering, node cover/uncover correctness.
- Wire tracing/tracy init; cargo-mobile2 projects; riced profiles and flags; global allocator gating.
