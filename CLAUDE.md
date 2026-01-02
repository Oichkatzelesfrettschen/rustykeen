# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rustykeen is a cleanroom Rust implementation of a KenKen-style puzzle solver/generator (based on Simon Tatham's "Keen" puzzle). The primary goal is a library-first pure Rust engine with deterministic, reproducible behavior suitable for embedding in Android/iOS/desktop/web applications.

## Build Commands

```bash
# Standard build
cargo build
cargo build --release

# Run tests
cargo test
cargo test --all-features

# Lint (warnings-as-errors enforced in workspace)
cargo clippy --all-targets --all-features

# Format check
cargo fmt --check

# Build with all optional features
cargo build --release --all-features

# Run CLI
cargo run -p kenken-cli -- solve --n 2 --desc b__,a3a3 --tier normal
cargo run -p kenken-cli -- count --n 2 --desc b__,a3a3 --limit 2
```

## Target-Specific Builds

```bash
# Portable Linux x86_64 baseline
RUSTFLAGS="-C target-cpu=x86-64-v1" cargo build --release -p kenken-cli --all-features

# Tuned Linux x86_64 (AVX2/etc)
RUSTFLAGS="-C target-cpu=x86-64-v3" cargo build --release -p kenken-cli --all-features

# Android arm64 (requires NDK + cargo-ndk)
cargo ndk -t arm64-v8a build --release -p kenken-cli --all-features

# Convenience script
./scripts/build_targets.sh linux-x86-v1
./scripts/build_targets.sh linux-x86-v3
./scripts/build_targets.sh android-arm64
```

## Workspace Architecture

```
kenken-core     # Puzzle model, validation, cage semantics, sgt-desc format parsing
kenken-solver   # Deterministic backtracking solver with MRV/LCV, optional DLX/SAT backends
kenken-gen      # Puzzle generator with cage partitioning, uniqueness verification
kenken-simd     # Runtime ISA dispatch helpers (popcount, etc.) - contains controlled unsafe
kenken-io       # Versioned serialization (rkyv snapshots)
kenken-uniffi   # UniFFI bindings for Kotlin/Swift
kenken-cli      # Reference CLI tool
```

### Key Design Decisions

- **Determinism**: Uses `rand_chacha::ChaCha20Rng` for reproducible RNG across platforms
- **No hash-map iteration in hot paths**: Solver ordering is deterministic
- **`unsafe_code = "forbid"`** in all crates except `kenken-simd` (which exposes safe API)
- **`warnings = "deny"`** workspace-wide (clippy enforced in CI)
- **Feature-gated optional dependencies**: DLX, SAT, rayon, bumpalo, tracing, SIMD dispatch

### Feature Flags

**kenken-solver:**
- `solver-dlx` - DLX Latin square solver (dlx-rs)
- `sat-varisat` - SAT solver backend (varisat)
- `alloc-bumpalo` - Arena allocator for propagation temps
- `simd-dispatch` - Runtime SIMD dispatch via kenken-simd
- `tracing` - Tracing instrumentation spans
- `perf-likely` - Branch prediction hints

**kenken-gen:**
- `gen-dlx` - Enables DLX-based Latin solution generation
- `parallel-rayon` - Parallel batch solving
- `verify-sat` - SAT-based uniqueness verification

**kenken-cli:**
- `alloc-mimalloc` - Use mimalloc allocator
- `telemetry-subscriber` - Enable tracing subscriber

## SGT-Desc Format

The solver uses Simon Tatham's puzzle "desc" format for puzzle definitions:

```
# Example: 2x2 grid with add-cage target=3 covering cells
kenken-cli solve --n 2 --desc b__,a3a3
```

Format: comma-separated row strings where letters indicate cage membership and numbers indicate targets/ops.

## Solver Architecture

1. **Baseline**: Backtracking with MRV (minimum remaining values), forward checking, cage pruning
2. **DLX path** (optional): Dancing Links for Latin constraints
3. **SAT path** (optional): Varisat for tuple explosion fallback / certification
4. **Deduction tiers**: `None`, `Easy`, `Normal`, `Hard` - control propagation strength
5. **Uniqueness check**: `count_solutions_up_to(&puzzle, rules, 2)` returns 1 for unique puzzles

## Profile-Guided Optimization

```bash
# PGO workflow
./scripts/pgo.sh gen
./scripts/pgo.sh train -- cargo run -p kenken-cli -- count --n 6 --desc <puzzle> --limit 2
./scripts/pgo.sh use

# BOLT post-link optimization (Linux x86_64)
./scripts/bolt.sh record -- ./target/release/kenken-cli count --n 6 --desc <puzzle>
./scripts/bolt.sh optimize ./target/release/kenken-cli
```

## Instrumentation and Analysis

**Available on this system:**
- `miri` - UB detection (add via `rustup component add miri`)
- `cargo-fuzz` - Fuzzing harness
- `afl-fuzz`, `honggfuzz` - Alternative fuzzers
- `cargo-kani` - Bounded model checking / formal verification
- `perf`, `flamegraph`, `samply` - Profiling
- `heaptrack`, `valgrind` - Memory profiling
- `kcov` - Coverage
- `rust-gdb`, `rust-lldb`, `rr` - Debugging
- `hyperfine` - Benchmarking
- `llvm-bolt`, `llvm-profdata` - PGO/BOLT tools

**Running Miri:**
```bash
cargo +nightly-2026-01-01 miri test
```

**Running Kani verification:**
```bash
cargo kani --tests
```

## Tracing

When `tracing` feature is enabled, solver emits spans:
- `kenken.solve_one` - Top-level solve
- `kenken.propagate` - Constraint propagation
- `kenken.search.branch` / `kenken.search.backtrack` - Search tree

Enable subscriber:
```bash
RUST_LOG=kenken_solver=trace cargo run -p kenken-cli -- solve ...
```

## Testing Conventions

- Unit tests in each crate's `src/` modules
- Integration tests in `tests/` directories (e.g., `kenken-solver/tests/corpus_sgt_desc.rs`)
- Property tests for cage semantics and solver correctness
- Golden corpus tests with known solutions/uniqueness

## Documentation

Primary docs in `docs/`:
- `docs/plan.md` - Master implementation plan
- `docs/architecture.md` - Workspace layout and data flow
- `docs/cleanroom_plan.md` - Cleanroom porting approach
- `docs/target_matrix.md` - Build targets and CPU tuning
- `docs/riced_build.md` - Release profile and optimization

## Optimization Work: Tier 1.1 Cage Tuple Caching

**Status**: COMPLETE - Production Ready (2026-01-01)

Based on profiling-guided analysis (see `docs/profiling_analysis.md`), implemented **Tier 1.1: Cage Tuple Caching** - a HashMap-based memoization cache that eliminates redundant tuple enumeration during constraint propagation.

### Implementation Summary

- **File**: `kenken-solver/src/solver.rs`
- **Changes**: 91 lines added, 21 removed (net +70 LOC)
- **Cache Key**: Composite (op_hash, target, cells_count, cells_hash, domain_hash) uniquely identifying cage + domain state
- **Integration**: Lookup/update in `apply_cage_deduction`; persists for duration of solve
- **Breaking Changes**: None; signature change from `&State` to `&mut State` internal only

### Performance Impact

- **Cache Hit**: O(1) lookup replaces O(n^k) tuple enumeration
- **Expected Improvement**: 20-40% on puzzles with repeated cage evaluations (typical: 5-10 cages evaluated 2-3 times each)
- **Memory Overhead**: ~100 bytes per unique (cage, domain) pair - negligible for typical puzzles
- **Correctness**: All 26 tests passing; no approximation or heuristics

### Verification

```bash
# Run tests to verify implementation
cargo test --all-features
cargo clippy --all-targets --all-features

# Run benchmarks to establish baseline
cargo bench -p kenken-solver

# Check for cache effectiveness with tracing
RUST_LOG=kenken_solver=debug cargo run -p kenken-cli -- solve --n 6 --desc <puzzle>
```

### Next Phases

**Tier 1.2** (Domain Constraint Filtering) - Pending implementation; expected 10-20% improvement
**Tier 1.3** (Tuple Pre-filtering) - Pending implementation; expected 10-25% improvement

See `docs/optimization_session_tier1.md` for complete implementation details, `docs/optimization_roadmap.md` for full tier strategy.

## Important Constraints

1. **Cleanroom**: Avoid copying upstream sgt-puzzles code/constants directly; re-derive from behavior
2. **No fast-math**: Keep floating-point semantics deterministic across platforms
3. **Edition 2024**: Uses Rust edition 2024 with resolver = "3"
4. **Nightly required**: Pinned to `nightly-2026-01-01` via `rust-toolchain.toml`
