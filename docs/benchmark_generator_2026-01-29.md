# Generator Performance Baselines (2026-01-29)

**Date Recorded**: 2026-01-29
**Build**: Release mode (opt-level=3, LTO=thin)

---

## Generator Performance Summary

Performance baselines for puzzle generation across different grid sizes and target difficulties.

---

## Generation Time & Acceptance Rate

### Methodology

- **Operation**: Generate unique puzzles via `kenken-gen` generator
- **Target Difficulties**: Easy, Normal, Hard
- **Grid Sizes**: 4x4, 5x5, 6x6
- **Runs**: 10 generations per configuration
- **Metrics**:
  - Generation time (before minimization)
  - Acceptance rate (% of candidates passing uniqueness check)
  - Minimization time (cage reduction)
  - Total end-to-end time

### Results Table

| Grid | Difficulty | Gen Time (ms) | Accept Rate | Minimize (ms) | Total (ms) |
|------|------------|--------------|-------------|--------------|-----------|
| 4x4 | Easy | TBD | TBD % | TBD | TBD |
| 4x4 | Normal | TBD | TBD % | TBD | TBD |
| 4x4 | Hard | TBD | TBD % | TBD | TBD |
| 5x5 | Easy | TBD | TBD % | TBD | TBD |
| 5x5 | Normal | TBD | TBD % | TBD | TBD |
| 5x5 | Hard | TBD | TBD % | TBD | TBD |
| 6x6 | Easy | TBD | TBD % | TBD | TBD |
| 6x6 | Normal | TBD | TBD % | TBD | TBD |
| 6x6 | Hard | TBD | TBD % | TBD | TBD |

### Analysis Notes

Expected patterns:
- **Generation Time**: Increases with grid size and difficulty (harder puzzles require more iterations)
- **Acceptance Rate**: Typically 10-30% (most candidates have multiple solutions)
- **Minimization**: Takes 5-15% of total time (greedy cage removal)
- **Difficulty Trend**: Hard > Normal > Easy (inversely)

---

## Performance by Stage

### Generation Stage

Characteristics:
- Create random cage partition
- Assign operations and targets
- Test candidate solvability
- Time dominated by: Cage enumeration + solvability checks

### Uniqueness Verification Stage

Characteristics:
- Run uniqueness check (count solutions up to 2)
- Expensive for Hard difficulty (requires more deduction)
- Time dominated by: SAT encoding (if enabled) or backtracking solver

### Minimization Stage

Characteristics:
- Greedy cage removal (try removing each cage)
- Verify puzzle still has unique solution
- Time dominated by: Multiple uniqueness checks

---

## Grid Size Scaling

| Grid | Cells | Cages (typical) | Time Growth |
|------|-------|-----------------|-------------|
| 4x4 | 16 | 6-8 | Baseline |
| 5x5 | 25 | 8-12 | 2-3x slower |
| 6x6 | 36 | 10-15 | 4-6x slower |

### Rationale

- More cells → larger search space → harder to find solutions
- Minimization: O(n²) cage removal attempts
- Uniqueness checks: More expensive per attempt

---

## Recording Generator Baselines

### Steps to Measure

```bash
# Build release
cargo build --release --all-features

# Generate with timing
/usr/bin/time -v cargo run --release -p kenken-cli -- \
  generate --n 5 --difficulty normal --seed 12345

# For acceptance rate (over 10 trials):
for i in {1..10}; do
  cargo run --release -p kenken-cli -- \
    generate --n 5 --difficulty normal --seed $((12345 + i))
done | grep -c "Successfully generated"
```

### Expected Output

```
Generated unique puzzle:
...puzzle description...
Generation time: XXX ms
Minimization time: YYY ms
Total time: ZZZ ms
```

---

## Optimization Opportunities

### Current Implementation

- Sequential generation (no parallelization for single puzzle)
- Greedy minimization (suboptimal cage reduction)
- No difficulty pre-filtering (tries many candidates)

### Potential Improvements

1. **Parallel Candidate Generation** (rayon)
   - Generate multiple candidates in parallel
   - Take first one that passes uniqueness check
   - Expected: 2-4x faster

2. **Smart Minimization**
   - Use SAT solver to minimize directly
   - Avoid trying obviously-redundant cages
   - Expected: 20-30% faster

3. **Difficulty Calibration**
   - Pre-filter candidates by expected difficulty
   - Reduce rejection rate for Hard puzzles
   - Expected: 10-20% faster

---

## Regression Detection

### Thresholds

- **Generation Time Regression**: >5% slower = investigate
- **Acceptance Rate Drop**: >10% absolute = critical
- **Minimization Overhead**: >50% of total = review algorithm

### CI Integration (Future)

```bash
# Store baseline
cargo run --release -p kenken-cli -- \
  generate --n 5 --difficulty normal --seed 12345 > /tmp/baseline.txt

# Compare in CI
if generate_time > baseline_time * 1.05:
  echo "FAILURE: Generation 5% slower than baseline"
  exit 1
fi
```

---

## Related Documents

- [docs/research/benchmark_baselines_2026-01-29.md](research/benchmark_baselines_2026-01-29.md) - Solver baselines
- [docs/regression_thresholds.md](regression_thresholds.md) - CI threshold definitions
- [docs/work_done.md](work_done.md) - Generator status
- [docs/optimization_roadmap.md](optimization_roadmap.md) - Future optimization work
