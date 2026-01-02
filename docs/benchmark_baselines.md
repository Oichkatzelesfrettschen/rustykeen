# Benchmark Baselines

Last updated: 2026-01-01

## Test Environment

- **CPU**: AMD Ryzen 5 5600X3D 6-Core Processor
- **Rust**: rustc 1.94.0-nightly (8d670b93d 2025-12-31)
- **OS**: Linux 6.12.63-2-cachyos-lts x86_64
- **Profile**: release (opt-level=3, lto=thin, codegen-units=1)

## Solver Performance

### solve_one (Normal tier)

| Puzzle | Grid | Time (median) | Notes |
|--------|------|---------------|-------|
| 2x2_singleton | 2x2 | 207 ns | All Eq cages |
| 2x2_add | 2x2 | 781 ns | Two Add-3 cages |
| 3x3_singleton | 3x3 | 444 ns | All Eq cages |
| 3x3_rows | 3x3 | 3.34 us | Three row Add-6 cages |
| 4x4_singleton | 4x4 | 665 ns | All Eq cages |
| 5x5_singleton | 5x5 | 1.03 us | All Eq cages |

### count_solutions (2x2, Normal tier)

| Limit | Time (median) | Notes |
|-------|---------------|-------|
| 1 | 775 ns | Early exit |
| 2 | 1.14 us | Full enumeration (2 solutions) |
| 10 | 1.18 us | Same as limit=2 |

### Deduction Tier Comparison (2x2 puzzle)

| Tier | Time (median) | Overhead vs None |
|------|---------------|------------------|
| None | 585 ns | baseline |
| Easy | 1.42 us | +143% |
| Normal | 1.15 us | +97% |
| Hard | 1.75 us | +199% |

## Scaling Observations

1. **Singleton puzzles scale sub-linearly**: 5x5 (25 cells) is only ~5x slower than 2x2 (4 cells)
2. **Deduction overhead**: Higher tiers add ~1-1.2us per solve on small puzzles
3. **Normal tier sweet spot**: Faster than Easy (less propagation) and Hard (simpler rules)

## Regression Thresholds

For CI regression detection, use these thresholds (2x baseline):

| Benchmark | Threshold |
|-----------|-----------|
| solve_one/2x2_singleton | < 500 ns |
| solve_one/4x4_singleton | < 1.5 us |
| solve_one/5x5_singleton | < 2.5 us |
| count_solutions/2x2/limit_2 | < 2.5 us |

## Running Benchmarks

```bash
# Full benchmark suite
cargo bench --bench solver_smoke

# Specific benchmark
cargo bench --bench solver_smoke -- solve_one/4x4

# With comparison to baseline (requires saved results)
cargo bench --bench solver_smoke -- --baseline main
```

## References

- Benchmark harness: `kenken-solver/benches/solver_smoke.rs`
- Criterion docs: https://bheisler.github.io/criterion.rs/book/
