# Crates Audit and 2026 Optimization Stack (2025-12-31T22:04:43Z)

## Master Crate List (comma-separated)
uniffi, fixed, rkyv, smallvec, rand, rand_pcg, wide, dlx_rs, rayon, bumpalo, bitvec, varisat, dashmap, num-integer, petgraph, criterion, parking_lot, itertools, proptest, once_cell, mimalloc, anyhow, thiserror, nom, tracing, tracing-android, z3, bolero, kani, creusot

## Current Stack (audited)
- uniffi: Kotlin/Swift bindings generation, avoids manual JNI.
- fixed: Deterministic fixed-point arithmetic for cage math.
- rkyv: Zero-copy snapshot/save-state; mmap-friendly.
- smallvec: Stack-backed small arrays for cages and hotpaths.
- rand / rand_pcg: Deterministic seeded RNG; PCG recommended.
- wide / portable-simd: SIMD for row/col validation and fast masks.
- dlx_rs: Algorithm X (DLX) exact-cover solver for Latin constraints.
- rayon: Parallel batch generation and heuristic racing.
- bumpalo: Arena allocator for solver nodes; reduces alloc overhead.
- bitvec / fixedbitset: Dense candidate domains and constraint masks.
- varisat: CDCL SAT solver for complex cage arithmetic constraints.
- dashmap: Concurrent cache for difficulty scores/partials.
- num-integer: Integer ops (gcd/lcm) for division/multiplication cages.
- petgraph: Cage partition (DSU/Union-Find) and graph utilities.
- criterion: Microbenchmarks; regression detection; statistically robust.
- parking_lot: Fast Mutex/RwLock for coordination alongside rayon/dashmap.
- itertools: Iterator utilities (cartesian_product, permutations, unique).
- proptest: Property-based fuzzing for solver/generator correctness.
- once_cell (or OnceLock): Safe global tables for precomputed sums/factors.
- mimalloc: High-performance global allocator.
- anyhow/thiserror: Error context and typed errors at edges.
- nom: Fast parsing for legacy test suites and formats.
- tracing / tracing-android: Instrumentation and Perfetto bridging on Android.

## Synergy & Architecture
- Metal: mimalloc + bumpalo; wide/portable-simd + bitvec for register-friendly masks.
- Brain: dlx_rs for Latin; varisat for arithmetic cages; petgraph + smallvec for cage partition.
- Scale: rayon + parking_lot + dashmap for concurrency; rkyv for zero-copy IO; tracing for perf visibility.

## Integration Plan
- Enable mimalloc via `#[global_allocator]` (see docs/global_allocator.rs) and feature gates.
- Add criterion benches; wire proptest suites; precompute tables with once_cell.
- Use nom to ingest legacy puzzle suites and convert to rkyv.
- Add tracing spans in hotpaths; export to Perfetto via tracing-android on Android.
