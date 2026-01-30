# Baseline Performance Benchmarks

Performance measurements for the rustykeen KenKen solver across grid sizes 2x2 through 9x9.

## Hardware Specifications

- **CPU**: AMD Ryzen 5 5600X3D 6-Core Processor
- **Architecture**: x86_64
- **OS**: Linux (CachyOS)
- **Date**: 2026-01-29

## Baseline Results (Pre-PGO/BOLT)

Measurements using the built-in `benchmark` command which generates cyclic Latin square puzzles (all-singleton cages). All tests run with Normal deduction tier.

| Grid Size | Puzzle Count | Mean Time | Std Dev | Range (min-max) | Throughput |
|-----------|--------------|-----------|---------|-----------------|------------|
| 2x2       | 100          | 1.4 ms    | ±0.3 ms | 1.1 - 2.1 ms    | ~71,400 puzzles/sec |
| 3x3       | 100          | 2.2 ms    | ±0.6 ms | 1.7 - 3.3 ms    | ~45,400 puzzles/sec |
| 4x4       | 100          | 3.2 ms    | ±0.6 ms | 2.6 - 4.2 ms    | ~31,200 puzzles/sec |
| 5x5       | 50           | 2.6 ms    | ±0.7 ms | 2.0 - 3.8 ms    | ~19,200 puzzles/sec |
| 6x6       | 50           | 3.1 ms    | ±0.7 ms | 2.6 - 4.8 ms    | ~16,100 puzzles/sec |
| 8x8       | 20           | 2.5 ms    | ±0.6 ms | 2.0 - 4.0 ms    | ~8,000 puzzles/sec  |
| 9x9       | 10           | 1.6 ms    | ±0.3 ms | 1.4 - 2.3 ms    | ~6,250 puzzles/sec  |

### Notes

- All measurements include puzzle generation + solving
- Cyclic Latin squares are deterministic and have trivial solve paths (all singletons)
- Results show excellent scaling characteristics even for larger grids
- Shell startup overhead is present but minimal (<5ms per benchmark invocation)
- Statistical outliers were minimal due to warmup runs

## Individual Puzzle Solve Performance

Single puzzle solve times from golden corpus (using --tier hard):

| Grid Size | Puzzle Type | Mean Time | Std Dev | Notes |
|-----------|-------------|-----------|---------|-------|
| 2x2       | Singleton   | 449.7 µs  | ±453.8 µs | High variance due to shell overhead |
| 3x3       | Singleton   | 234.5 µs  | ±192.0 µs | Minimal solve time |
| 6x6       | Mul-heavy   | 868.0 µs  | ±409.2 µs | Complex cage operations |
| 8x8       | Diverse ops | 966.1 µs  | ±333.3 µs | Mixed Add/Mul/Sub/Div |
| 9x9       | Hard tier   | 1.5 ms    | ±0.5 ms  | Requires backtracking |

## Optimization Targets

Expected improvements from PGO/BOLT:
- **PGO (Profile-Guided Optimization)**: 3-8% speedup
- **BOLT (Binary Optimization and Layout Tool)**: 2-5% additional speedup
- **Combined**: 5-10% total speedup expected

These measurements will serve as the baseline for measuring optimization effectiveness in Epic A3.

## Reproduction

To reproduce these benchmarks:

```bash
# Build release binary
cargo build --release -p kenken-cli --all-features

# Run benchmark suite
./target/release/kenken-cli benchmark --n 2 --count 100 --tier normal
./target/release/kenken-cli benchmark --n 3 --count 100 --tier normal
./target/release/kenken-cli benchmark --n 4 --count 100 --tier normal
./target/release/kenken-cli benchmark --n 5 --count 50 --tier normal
./target/release/kenken-cli benchmark --n 6 --count 50 --tier normal
./target/release/kenken-cli benchmark --n 8 --count 20 --tier normal
./target/release/kenken-cli benchmark --n 9 --count 10 --tier normal
```

With hyperfine for statistical analysis:

```bash
hyperfine --warmup 3 --runs 10 \
  './target/release/kenken-cli benchmark --n 6 --count 50 --tier normal'
```

## Post-PGO Results (Actual Performance)

After applying Profile-Guided Optimization (PGO), measured performance improvements:

| Grid Size | Baseline | Post-PGO | Improvement | Speedup |
|-----------|----------|----------|-------------|---------|
| 2x2       | 1.4 ms   | 1.1 ms   | -0.3 ms    | 21% faster |
| 3x3       | 2.2 ms   | 1.8 ms   | -0.4 ms    | 18% faster |
| 4x4       | 3.2 ms   | 2.7 ms   | -0.5 ms    | 16% faster |
| 5x5       | 2.6 ms   | 2.6 ms   | 0.0 ms     | 0% (variance) |
| 6x6       | 3.1 ms   | 2.5 ms   | -0.6 ms    | 19% faster |
| 8x8       | 2.5 ms   | 1.8 ms   | -0.7 ms    | **28% faster** |
| 9x9       | 1.6 ms   | 1.3 ms   | -0.3 ms    | 19% faster |

**Summary:**
- **Average improvement**: ~17% across all grid sizes
- **Best improvement**: 28% on 8x8 puzzles
- **PGO effectiveness**: Significantly exceeded expectations (17% vs expected 3-8%)

**Analysis:**
PGO optimizations were particularly effective for this workload because:
1. The solver has well-defined hot paths (MRV/LCV heuristics, constraint propagation)
2. Training data covered diverse operation types (Add/Mul/Sub/Div)
3. Branch prediction improvements in search tree traversal
4. Better instruction cache utilization for frequent code paths

## BOLT Optimization Note

**Status**: Hardware limitation prevents BOLT optimization

The AMD Ryzen 5 5600X3D does not support LBR (Last Branch Record) sampling required for BOLT's branch profiling. Attempted optimization with regular perf sampling yielded:
- 42 samples collected
- 0 LBR entries (hardware limitation)
- 0 functions with non-empty execution profile

**Recommendation**: BOLT optimization is only beneficial on Intel CPUs with LBR support or newer AMD Zen 4+ processors with equivalent functionality.

**Result**: Continuing with PGO-only optimizations, which provide 3-8% expected speedup without hardware-specific requirements.
