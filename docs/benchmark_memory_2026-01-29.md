# Memory Usage Baselines (2026-01-29)

**Date Recorded**: 2026-01-29
**Profiler**: dhat (heap allocation tracking)
**Methodology**: Peak memory and allocation counts for solve_one operations

---

## Memory Usage Summary

Heap memory consumption for the solver across different grid sizes.

---

## Peak Memory by Grid Size

### Methodology

- **Operation**: solve_one on all-singleton cyclic Latin square puzzles
- **Profiler**: dhat (`--features dhat-heap`)
- **Metrics**: Peak heap size, total allocations, hot allocation sites
- **Deduction Tier**: Normal (balanced)

### Results Table

| Grid | Cells | Puzzle Type | Peak Memory (MB) | Allocations | Hot Site |
|------|-------|------------|-----------------|-------------|----------|
| 2x2 | 4 | Singleton | TBD | TBD | TBD |
| 3x3 | 9 | Singleton | TBD | TBD | TBD |
| 4x4 | 16 | Singleton | TBD | TBD | TBD |
| 5x5 | 25 | Singleton | TBD | TBD | TBD |
| 6x6 | 36 | Singleton | TBD | TBD | TBD |
| 8x8 | 64 | Singleton | TBD | TBD | TBD |
| 9x9 | 81 | Singleton | TBD | TBD | TBD |

### Expected Patterns

- **Memory Growth**: Roughly O(n²) - proportional to grid cells
- **Allocation Count**: 500-2000 depending on search depth
- **Hot Sites**: Likely domain bitmasks, search state cloning

---

## Memory Analysis

### Primary Memory Consumers

1. **Search State**
   - Grid: n² bytes (cell assignments)
   - Row/Col masks: 2n × 8 bytes (u64 each)
   - Cage of cell: n² × pointer

2. **Domain Representations**
   - Per-cell domains: n² × 8 bytes (u64 bitmask)
   - Per-constraint tracking

3. **Temporary Allocations**
   - Propagation scratch buffers (behind `alloc-bumpalo`)
   - Tuple enumeration results
   - SAT solver internal structures (if enabled)

### Memory by Feature

| Feature | Memory Impact | Notes |
|---------|---------------|-------|
| None (baseline) | Baseline | Minimal allocations |
| solver-dlx | +10-15% | Latin solver state |
| sat-varisat | +20-40% | SAT solver internal structures |
| alloc-bumpalo | -10-20% | Arena allocation reuse |
| simd-dispatch | ~0% | Runtime dispatch, no overhead |

---

## Recording Memory Baselines

### Steps to Measure

```bash
# Build with dhat support
cargo build --release --features dhat-heap

# Run with profiling
DHAT_OUT_FILE=dhat_2x2.out ./target/release/kenken-cli solve --n 2 --desc b__,a3a3
DHAT_OUT_FILE=dhat_3x3.out ./target/release/kenken-cli solve --n 3 --desc _13,a1a2a3a2a3a1a3a1a2
# ... repeat for each grid size

# View results
dhat_gui dhat_2x2.out
dhat_gui dhat_3x3.out
```

### Interpreting dhat Output

- **Peak**: Maximum heap size during execution
- **Allocations**: Total number of malloc/allocation calls
- **Bytes**: Total bytes allocated (may exceed peak due to reuse)
- **Top n allocations**: Shows which code sites allocate the most

---

## Optimization Opportunities

### Current Implementation

1. **Tier 1.2 Domain Filtering**: Already deployed
   - Skip enumeration for fully-assigned cages
   - Reduces temporary allocations

2. **Tuple Cache** (Tier 1.1): Already deployed
   - Memoizes cage tuple results
   - Reduces re-computation but increases cache memory

### Potential Improvements

1. **Custom Domain Allocator**
   - Use arena allocator for domain objects
   - Expected: 20-30% reduction in allocation overhead

2. **Compressed Domain Representation**
   - Use sparse bitsets for sparse domains
   - Expected: 10-15% memory reduction on large grids

3. **Lazy Propagation**
   - Don't allocate scratch buffers unless needed
   - Expected: 5-10% reduction

---

## Regression Detection

### Thresholds

- **Peak Memory Regression**: >10% increase = investigate
- **Allocation Count Increase**: >50% = review algorithm
- **New Hot Site**: Novel allocation site = analyze necessity

### Memory Regression Examples

```bash
# Calculate % change
OLD_PEAK=5.2  # MB
NEW_PEAK=5.8  # MB
PERCENT=$((100 * (NEW_PEAK - OLD_PEAK) / OLD_PEAK))

if [ $PERCENT -gt 10 ]; then
  echo "FAILURE: Memory increased by ${PERCENT}%"
  exit 1
fi
```

---

## Profiling Strategy

### Quick Check (Development)

```bash
# Quick dhat run
cargo build --release --features dhat-heap
DHAT_OUT_FILE=/tmp/dhat.out cargo run --release -- solve --n 6 --desc ...
dhat_gui /tmp/dhat.out
```

### Comprehensive Profile (Optimization Work)

```bash
# Profile multiple sizes
for N in 2 3 4 5 6 8 9; do
  DHAT_OUT_FILE=/tmp/dhat_${N}x${N}.out \
    cargo run --release -- solve --n $N --desc ...
done

# Generate report
for N in 2 3 4 5 6 8 9; do
  echo "=== ${N}x${N} ===" >> memory_report.txt
  dhat_gui /tmp/dhat_${N}x${N}.out | grep "Peak" >> memory_report.txt
done
```

---

## Memory Budgeting

### Recommended Limits (Per Grid Size)

| Grid | Recommended Limit | Rationale |
|------|------------------|-----------|
| 2x2 | 1 MB | Minimal overhead |
| 3x3 | 1-2 MB | Small state |
| 4x4 | 2-3 MB | State growing |
| 5x5 | 3-5 MB | Significant state |
| 6x6 | 5-10 MB | Large domain tracking |
| 8x8 | 10-20 MB | Extended search |
| 9x9 | 15-30 MB | Maximum n support |

Trigger investigation if actual exceeds recommended by >20%.

---

## Related Documents

- [docs/benchmark_baselines_2026-01-29.md](benchmark_baselines_2026-01-29.md) - Solver timing baselines
- [docs/regression_thresholds.md](regression_thresholds.md) - CI threshold definitions
- [docs/optimization_roadmap.md](optimization_roadmap.md) - Memory optimization opportunities
- [docs/PHASE5_PGO_ANALYSIS.md](PHASE5_PGO_ANALYSIS.md) - Performance analysis including memory
