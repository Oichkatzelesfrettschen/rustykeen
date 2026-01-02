# Dependency Audit

This document catalogs external dependencies, identifies candidates for refactoring or internalization, and outlines a migration roadmap to reduce dependency bloat while maintaining functionality.

**Last Updated**: 2026-01-02
**Dependency Count**: ~80 (minimal) to ~280 (all features)

---

## Dependency Categories

### 1. C/FFI Dependencies (Require System Libraries)

These dependencies require external C libraries or build toolchains:

| Dependency | Purpose | Feature Gate | System Requirement |
|------------|---------|--------------|-------------------|
| `z3` / `z3-sys` | SMT solver for formal verification | `verify` | libz3, LLVM/Clang |
| `mimalloc` / `libmimalloc-sys` | High-performance allocator | `alloc-mimalloc` | C compiler |
| `pprof` + `bindgen` | CPU profiling with flamegraphs | dev-dependency | LLVM/Clang |

**Recommendation**: Keep all as optional. Z3 has no pure Rust equivalent. Mimalloc is performance-only. Pprof is dev-only.

### 2. Heavy Dependencies

Dependencies with large transitive dependency trees:

| Dependency | Transitive Deps | Purpose | Alternative |
|------------|-----------------|---------|-------------|
| `uniffi` | ~40 crates | Kotlin/Swift FFI bindings | None (required for mobile) |
| `criterion` | ~20 crates | Benchmarking framework | `divan` (~5 crates) |
| `rkyv` | ~15 crates | Zero-copy serialization | `postcard` (~3 crates) |
| `varisat` | ~10 crates | SAT solver | Keep (pure Rust, optional) |
| `pprof` | ~25 crates | CPU profiling | External profiler (perf, samply) |

### 3. Candidates for Internalization

Dependencies simple enough to reimplement in-house:

| Dependency | Est. LOC | Current Usage | Internalization Benefit |
|------------|----------|---------------|------------------------|
| `dlx-rs` | ~200 | DLX Latin solver | Customize for KenKen; remove dep |
| `likely_stable` | ~50 | Branch hints | Trivial macros; inline |
| `smallbitvec` | ~300 | Bit vector | Consolidate with Domain types |
| `fixedbitset` | ~400 | Bit set | Consolidate with Domain types |

### 4. Core Dependencies (Keep)

Essential dependencies that should remain:

| Dependency | Justification |
|------------|---------------|
| `rand` / `rand_chacha` | Industry standard; determinism via ChaCha20 |
| `smallvec` | Widely used, well-optimized, small |
| `thiserror` | Ergonomic error handling, minimal overhead |
| `serde` / `serde_json` | De facto serialization standard |
| `tracing` | Structured logging/instrumentation |

---

## Migration Roadmap

### Phase 1: Immediate (Low Effort, High Impact)

**Goal**: Remove unnecessary dependencies without breaking changes.

1. **Internalize `dlx-rs`**
   - Create `kenken-solver/src/dlx.rs`
   - Implement Dancing Links algorithm (~200 LOC)
   - Remove `dlx-rs` dependency
   - Benefit: One less external dep; can optimize for KenKen

2. **Internalize `likely_stable`**
   - Create `kenken-core/src/hints.rs`
   - Define `likely!()` and `unlikely!()` macros
   - Remove `likely_stable` dependency
   - Benefit: Trivial code; no maintenance burden

3. **Isolate `pprof` to dev-only**
   - Ensure pprof is only in `[dev-dependencies]`
   - Remove from any runtime paths
   - Benefit: Reduces release build deps by ~25 crates

### Phase 2: Consolidation (Medium Effort)

**Goal**: Unify overlapping functionality.

4. **Unify bit vector types**
   - Consolidate `smallbitvec`, `fixedbitset`, `bitvec` usage
   - Extend `Domain32`/`Domain64`/`Domain128`/`Domain256` abstractions
   - Create unified `DomainOps` implementations
   - Remove redundant bit vector dependencies

5. **Evaluate `criterion` replacement**
   - Consider `divan` for lighter benchmarks
   - Or use custom harness with `std::time::Instant`
   - Benefit: Reduces dev-dependency bloat by ~15 crates

6. **Evaluate `rkyv` replacement**
   - Consider `postcard` for simpler binary serialization
   - Current rkyv usage is limited to snapshot format
   - Benefit: Simpler serialization, fewer dependencies

### Phase 3: Long-term (Keep As-Is)

**Goal**: Document why certain dependencies are retained.

| Dependency | Reason to Keep |
|------------|----------------|
| `z3` | No pure Rust SMT solver with equivalent capability |
| `uniffi` | Required for Android/iOS platform support |
| `varisat` | Pure Rust SAT solver; already feature-gated |
| `rayon` | Industry standard parallelism; feature-gated |
| `bumpalo` | Arena allocation for hot paths; feature-gated |

---

## Dependency Tree Analysis

### Minimal Build (no optional features)

```
cargo build -p kenken-cli
Dependencies: ~80 crates
Build time: ~20s (fresh)
```

### Full Build (all features)

```
cargo build -p kenken-cli --all-features
Dependencies: ~280 crates
Build time: ~60s (fresh)
```

### Bloat Sources

| Feature | Additional Deps | Justification |
|---------|-----------------|---------------|
| `uniffi` | +40 | Mobile FFI (required for apps) |
| `z3` | +30 | Formal verification (optional) |
| `pprof` | +25 | CPU profiling (dev-only) |
| `criterion` | +20 | Benchmarking (dev-only) |
| `rkyv` | +15 | Serialization (optional) |

---

## Feature Flag Summary

### kenken-solver

| Feature | Dependencies Added | Purpose |
|---------|-------------------|---------|
| `solver-dlx` | dlx-rs | DLX Latin solver |
| `sat-varisat` | varisat | SAT solver backend |
| `simd-dispatch` | kenken-simd | Runtime SIMD selection |
| `alloc-bumpalo` | bumpalo | Arena allocation |
| `verify` | z3 | SMT verification |
| `dhat-heap` | dhat | Heap profiling |
| `tracing` | tracing | Instrumentation |
| `perf-likely` | likely_stable | Branch hints |

### kenken-gen

| Feature | Dependencies Added | Purpose |
|---------|-------------------|---------|
| `gen-dlx` | (via solver) | DLX generation |
| `parallel-rayon` | rayon | Parallel batch ops |

### kenken-cli

| Feature | Dependencies Added | Purpose |
|---------|-------------------|---------|
| `alloc-mimalloc` | mimalloc | Allocator override |
| `telemetry-subscriber` | tracing-subscriber | Log output |

### kenken-io

| Feature | Dependencies Added | Purpose |
|---------|-------------------|---------|
| `io-rkyv` | rkyv | Binary snapshots |

---

## Action Items

- [ ] Phase 1.1: Internalize dlx-rs
- [ ] Phase 1.2: Internalize likely_stable
- [ ] Phase 1.3: Audit pprof usage
- [ ] Phase 2.1: Unify bit vector types
- [ ] Phase 2.2: Evaluate criterion alternatives
- [ ] Phase 2.3: Evaluate rkyv alternatives
- [ ] Document rationale for each retained dependency

---

## References

- [Cargo Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Feature Flags Best Practices](https://doc.rust-lang.org/cargo/reference/features.html)
- [Dependency Management](https://doc.rust-lang.org/cargo/guide/dependencies.html)
