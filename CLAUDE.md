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

**Status**: COMPLETE AND VALIDATED - Production Ready (2026-01-01)

Implemented **Tier 1.1: Cage Tuple Caching** - HashMap-based memoization eliminating redundant tuple enumeration. Empirically validated through comprehensive benchmarking to provide 40-52% improvement on enumeration-heavy workloads.

### Implementation Details

- **File**: `kenken-solver/src/solver.rs` (lines 236-297 cache infrastructure, 809-875 integration)
- **Changes**: +70 LOC net (91 added, 21 removed)
- **Cache Key**: (op_hash, tier_byte, target, cells_count, cells_hash, domain_hash) - includes deduction tier for correctness
- **Integration**: Lookup/update in `apply_cage_deduction` with n >= 6 threshold to avoid small-puzzle overhead
- **Breaking Changes**: None; internal struct changes only

### Critical Correctness Fix

**Deduction Tier Bug (discovered via benchmarking)**: Initial cache key lacked tier discrimination, causing +85-95% regressions on Easy/Normal deduction tiers. Root cause: cache entries from different tiers (None, Easy, Normal, Hard) collided and reused incorrect results.

**Solution**: Extended cache key to include tier_byte (None=0, Easy=1, Normal=2, Hard=3). Completely resolved the issue, transforming regressions into -46-48% improvements.

### Empirical Performance Validation

Benchmarks show **data-driven evidence** of optimization effectiveness:
- **Multi-cell enumeration**: -42-52% improvement (Add/Mul/Div cages with multiple cells)
- **Deduction tiers**: -46-48% improvement (Easy/Normal tiers now correct)
- **Small puzzles (n<=5)**: 0-2% change (cache correctly disabled via n >= 6 threshold)
- **Singleton cages**: Minimal benefit (-3-10%, expected as they bypass enumeration)

### Verification

```bash
# Validate implementation
cargo test --all-features           # All 26 tests passing
cargo clippy --all-targets          # Zero warnings
cargo build --release               # Clean build

# Measure performance
cargo bench --bench solver_smoke    # See multi-cell improvements
cargo bench --bench deduction_tiers # See tier-specific improvements
```

### Tier 1.2 Implementation: Domain Constraint Filtering (2026-01-01)

**Status**: IMPLEMENTED - Mixed Results (Benefits and Regressions)

Implemented **Tier 1.2: Domain Constraint Filtering** - Skip enumeration when all cage cells are fully assigned (exactly 1 bit set in domain bitmask).

**Benchmark Results** (100 sample criterion benchmarks):

*Improvements*:
- solve_one/2x2_add: -8.4% improvement
- solve_one/3x3_rows: -4.2% improvement
- deduction_tiers/Easy: -18.5% improvement
- deduction_tiers/Normal: -11.7% improvement

*Regressions*:
- solve_one/4x4_singleton: +8.3% regression
- solve_one/5x5_singleton: +7.4% regression
- count_solutions/limit_1: +23.6% regression

**Analysis**: Tier 1.2 helps multi-cell enumeration workloads but introduces overhead on simple puzzles. The fully-assigned check cost sometimes exceeds the benefit of skipping enumeration.

**Implementation**:
- **File**: `kenken-solver/src/solver.rs`
  - Lines 262-285: Helper functions (all_cells_fully_assigned, compute_any_mask_from_assigned)
  - Lines 752-773: Sub/Div fast path (2-cell constraints)
  - Lines 863-867: Add/Mul fast path (multi-cell constraints)
- **Changes**: +62 LOC net
- **Tests**: All 26 passing, zero correctness issues
- **Verification**: Use `cargo bench --bench solver_smoke` to see mixed results

**Future Refinement**: Consider conditional application - only apply Tier 1.2 to cages with 3+ cells to eliminate regressions on simple puzzles.

See `docs/tier12_domain_constraint_filtering.md` for detailed analysis.

### Tier 1.3 Re-evaluation: Tuple Pre-filtering (POST TIER 1.2)

**Status**: DEFERRED - Diminishing Returns Confirmed

Re-evaluated Tier 1.3 after Tier 1.2 implementation as per user directive: "once finished, then re-evaluate tier 1.3 once tier 1.2 fully exists"

**Re-evaluation Findings**:
- **Estimated Benefit**: 2-5% additional (with Tier 1.1+1.2), down from original 3-8%
- **Implementation Complexity**: HIGH (250-300 LOC, tight bounds logic)
- **Code Risk**: MEDIUM (off-by-one errors in bounds computation)
- **ROI**: LOW - Diminishing returns not worth added complexity

**Recommendation**: DEFER Tier 1.3 indefinitely. Better alternatives available:
- **Tier 2.1 (Partial Constraint Checking)**: 10-20% potential benefit (higher ROI)
- **Tier 2.2 (MRV Heuristic Optimization)**: 5-15% potential benefit (quick win)

**Conditions to Reconsider**: Only implement Tier 1.3 if real-world profiling shows enumerate_cage_tuples still 30%+ of total time post Tier 1.1+1.2, or performance targets unmet.

See `docs/tier13_reevaluation_with_tier12.md` for comprehensive analysis and decision framework.

### Optimization Summary: Tier 1 Complete

**Tier 1.1 Cage Tuple Cache**: 40-52% improvement on enumeration - DEPLOYED
**Tier 1.2 Domain Constraint Filtering**: Mixed 2-18% improvement - DEPLOYED
**Tier 1.3 Tuple Pre-filtering**: 2-5% diminishing return - DEFERRED

**Next Steps**:
1. Deploy Tier 1.1 + 1.2 to production
2. Monitor real-world performance with diverse puzzle corpus
3. Profile with CPU flamegraph (perf/cargo-flamegraph) to identify remaining bottlenecks
4. Evaluate Tier 2 opportunities (Partial Constraint Checking, MRV Heuristic)
5. Conditionally reconsider Tier 1.3 if enumerate_cage_tuples remains 30%+ of time

See `docs/tier1_empirical_analysis.md` for Tier 1.1 analysis, `docs/optimization_session_tier1.md` for implementation guide, `docs/optimization_roadmap.md` for full multi-tier strategy.

## Important Constraints

1. **Cleanroom**: Avoid copying upstream sgt-puzzles code/constants directly; re-derive from behavior
2. **No fast-math**: Keep floating-point semantics deterministic across platforms
3. **Edition 2024**: Uses Rust edition 2024 with resolver = "3"
4. **Nightly required**: Pinned to `nightly-2026-01-01` via `rust-toolchain.toml`
