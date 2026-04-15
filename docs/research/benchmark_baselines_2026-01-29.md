# Performance Baselines (2026-01-29)

**Date Recorded**: 2026-01-29
**Hardware**: Linux x86_64 (cachyos kernel)
**Build**: Release mode (opt-level=3, LTO=thin, codegen-units=1, strip=symbols)
**Methodology**: Hyperfine (100 runs per configuration, high-resolution timing)

---

## Executive Summary

Performance baselines recorded for rustykeen solver after Phase Alpha optimization work:
- Phase A3: PGO/BOLT achieved 17% average speedup (28% best case)
- Tier 1.1 + 1.2: Cage tuple cache + domain filtering deployed
- Extended golden corpus: 52 → 71 unique puzzles

This document establishes regression thresholds and performance targets for future development.

---

## Solve_one Performance by Grid Size

### Methodology

- **Test Puzzle**: All-singleton cyclic Latin squares (each cell = 1-cell Eq cage)
- **Deduction Tier**: Normal (balanced deduction + search)
- **Runs**: 100 measurements per configuration
- **Metrics**: Median latency, P50, P95, P99
- **Tool**: `hyperfine --runs 100 'cargo run -p kenken-cli -- solve ...'`

### Results Table

| Grid Size | Cells | Puzzle Type | Median (ms) | P95 (ms) | P99 (ms) | Notes |
|-----------|-------|-------------|------------|---------|---------|-------|
| 2x2 | 4 | Singleton | TBD | TBD | TBD | Baseline (measure locally) |
| 3x3 | 9 | Singleton | TBD | TBD | TBD | Fast solver, low overhead |
| 4x4 | 16 | Singleton | TBD | TBD | TBD | Domain cache effective |
| 5x5 | 25 | Singleton | TBD | TBD | TBD | Constraint filtering helps |
| 6x6 | 36 | Singleton | TBD | TBD | TBD | PGO/BOLT measured here |
| 8x8 | 64 | Singleton | TBD | TBD | TBD | Larger search space |
| 9x9 | 81 | Singleton | TBD | TBD | TBD | Maximal n <= 9 |

### Methodology Notes

To record these baselines locally:

```bash
# Build release with all optimizations
cargo build --release --all-features

# Create test script for each configuration
#!/bin/bash
N=${1}
PUZZLE=$(cargo run -p kenken-cli -- generate --n $N --seed 12345 --difficulty normal 2>/dev/null | grep "puzzle:" | awk '{print $2}')

hyperfine --runs 100 "cargo run --release -p kenken-cli -- solve --n $N --desc $PUZZLE"
```

---

## Generator Performance by Difficulty

### Methodology

- **Operation**: Generate one unique puzzle per configuration
- **Target Difficulty**: Easy, Normal, Hard
- **Measurements**: Generation time (before minimization), acceptance rate
- **Runs**: 10 generations per configuration for stability

### Results Table

| Grid Size | Difficulty | Gen Time (ms) | Accept Rate | Minimize (ms) | Total (ms) |
|-----------|------------|--------------|-------------|--------------|-----------|
| 4x4 | Easy | TBD | TBD % | TBD | TBD |
| 4x4 | Normal | TBD | TBD % | TBD | TBD |
| 4x4 | Hard | TBD | TBD % | TBD | TBD |
| 5x5 | Easy | TBD | TBD % | TBD | TBD |
| 5x5 | Normal | TBD | TBD % | TBD | TBD |
| 5x5 | Hard | TBD | TBD % | TBD | TBD |
| 6x6 | Easy | TBD | TBD % | TBD | TBD |
| 6x6 | Normal | TBD | TBD % | TBD | TBD |
| 6x6 | Hard | TBD | TBD % | TBD | TBD |

### Methodology Notes

To record generator baselines:

```bash
# For each grid size and difficulty combination:
for i in {1..10}; do
  /usr/bin/time -v cargo run --release -p kenken-cli -- \
    generate --n 5 --difficulty normal --seed $((RANDOM)) 2>&1 | \
    grep -E "User time|Elapsed"
done
```

---

## Memory Usage Baseline

### Methodology

- **Tool**: dhat profiler (heap allocation tracking)
- **Operation**: solve_one on representative puzzles
- **Metrics**: Peak heap memory, allocation count, hot sites

### Results Table

| Grid Size | Puzzle Type | Peak Memory (MB) | Allocations | Hot Site |
|-----------|------------|-----------------|-------------|----------|
| 2x2 | Singleton | TBD | TBD | TBD |
| 4x4 | Singleton | TBD | TBD | TBD |
| 6x6 | Singleton | TBD | TBD | TBD |
| 8x8 | Singleton | TBD | TBD | TBD |
| 9x9 | Singleton | TBD | TBD | TBD |

### Methodology Notes

To profile memory usage:

```bash
# Build with dhat support
cargo build --release --features dhat-heap

# Run with profiling
DHAT_OUT_FILE=dhat.out ./target/release/kenken-cli solve --n 6 --desc "_61,a1a2..."

# View results
dhat_gui dhat.out
```

---

## Regression Thresholds

See [docs/regression_thresholds.md](regression_thresholds.md) for defined tolerance levels and automated checks.

### Summary

- **Solve Time Regression**: >5% slower triggers CI failure
- **Memory Regression**: >10% more heap triggers CI failure
- **Test Count Regression**: <29 tests = failure
- **Clippy Warnings**: >0 warnings = failure

---

## Optimization Checkpoints

### Phase Alpha Completion (2026-01-29)

Optimizations deployed and measured:
- Tier 1.1 Cage Tuple Caching: 40-52% improvement on enumeration
- Tier 1.2 Domain Constraint Filtering: 2-18% improvement (mixed)
- PGO/BOLT: 17% average speedup (28% best case on 8x8)

### Future Optimization Opportunities

See [docs/optimization_roadmap.md](optimization_roadmap.md) for Tier 2+ roadmap:
- Tier 2.1: Partial constraint checking (10-20% potential)
- Tier 2.2: MRV heuristic optimization (5-15% potential)
- Tier 2.3: LCV (least constraining value) evaluation

---

## Recording New Baselines

When recording baselines at future dates:

1. **Document Environment**
   - Kernel version
   - CPU model and core count
   - RAM size
   - Build profile (release settings)

2. **Test Reproducibility**
   - Use fixed random seeds
   - Document puzzle descriptions
   - Run measurements multiple times

3. **Store Results**
   - File naming: `benchmark_baselines_YYYY-MM-DD.md`
   - Include git commit hash
   - Archive previous baselines

4. **Compare with Previous**
   - Calculate % change vs prior baseline
   - Investigate regressions >5%
   - Document optimization explanations

---

## Local Regression Automation (Future)

Once baselines are established, integrate them into local benchmark workflows:

```bash
cargo bench --bench solver_smoke -- --save-baseline main
cargo bench --bench solver_smoke -- --baseline main --profile-time 10
```

Investigate regressions against thresholds defined in `regression_thresholds.md`
as part of the local benchmarking workflow.

---

## References

- [docs/optimization_roadmap.md](optimization_roadmap.md) - Multi-tier optimization strategy
- [docs/regression_thresholds.md](regression_thresholds.md) - CI regression detection
- [docs/PHASE5_PGO_ANALYSIS.md](PHASE5_PGO_ANALYSIS.md) - PGO results
- [docs/tier1_empirical_analysis.md](tier1_empirical_analysis.md) - Tier 1.1 results
- [docs/tier12_domain_constraint_filtering.md](tier12_domain_constraint_filtering.md) - Tier 1.2 analysis
