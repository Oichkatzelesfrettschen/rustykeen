# Tier 1.2: Domain Constraint Filtering - Benchmark Analysis

**Date**: 2026-01-01
**Status**: IMPLEMENTED - Mixed Results (Benefits and Regressions)
**Focus**: Analyzing performance impact of skipping enumeration for fully-assigned cages

---

## Executive Summary

Tier 1.2 Domain Constraint Filtering implementation is complete, but shows **mixed benchmark results**:
- Strong improvements (8-19%) on multi-cell enumeration workloads (Add/Mul cages)
- Significant regressions (8-24%) on some solution counting workloads
- No clear net benefit across all puzzle types

**Key Finding**: The cost of checking all_cells_fully_assigned() sometimes exceeds the benefit of skipping enumeration, particularly for puzzles with few cages or simple constraint structures.

---

## Tier 1.2 Implementation Details

### What Was Implemented

**Sub/Div Operations (2-cell constraints)**:
- Lines 752-773 in solver.rs
- Fast path: When both cells are fully assigned, directly verify constraint without enumeration
- Extracts values using `trailing_zeros() + 1` conversion
- Early return if constraint violated (sets domains to 0)

**Add/Mul Operations (multi-cell constraints)**:
- Lines 863-867 in solver.rs
- Fast path: When all cage cells are fully assigned (domain.popcount() == 1), skip enumeration
- Computes union of values from all cells
- Returns (per_pos, any_mask, 0, 0, true) without enumeration cost

### Design Rationale

When all cells in a cage have exactly one possible value remaining:
1. Enumeration would produce a single valid tuple (the assigned values)
2. No constraint propagation occurs (values already fixed)
3. Skipping enumeration saves the cost of generating and filtering tuples

---

## Benchmark Results: Full Data

### Performance Changes by Category

#### Improvements (Tier 1.2 helps)

| Benchmark | Change | p-value | Workload Type |
|-----------|--------|---------|---------------|
| solve_one/2x2_add/Normal | -8.4% | p<0.001 | Multi-cell Add cage |
| solve_one/3x3_rows/Normal | -4.2% | p<0.001 | Multi-cell Add cage |
| count_solutions/2x2/limit_10 | -11.3% | p<0.001 | Solution counting (limit=10) |
| deduction_tiers/count_2x2/Easy | -18.5% | p<0.001 | Easy tier deduction |
| deduction_tiers/count_2x2/Normal | -11.7% | p<0.001 | Normal tier deduction |

**Pattern**: Improvements occur on workloads with many constraint propagation iterations or solution counting limits.

#### Regressions (Tier 1.2 hurts)

| Benchmark | Change | p-value | Workload Type |
|-----------|--------|---------|---------------|
| solve_one/4x4_singleton/Normal | +8.3% | p<0.001 | Pure singleton puzzle |
| solve_one/5x5_singleton/Normal | +7.4% | p<0.001 | Pure singleton puzzle |
| count_solutions/2x2/limit_1 | +23.6% | p<0.001 | Single solution check |
| count_solutions/2x2/limit_2 | +7.9% | p<0.001 | Two solution check |

**Pattern**: Regressions occur on simple puzzles or early termination checks where fully-assigned validation is expensive relative to enumeration.

#### No Change (within noise)

| Benchmark | Change | p-value |
|-----------|--------|---------|
| solve_one/2x2_singleton/Normal | +1.8% | p=0.13 (noise) |
| solve_one/3x3_singleton/Normal | +2.4% | p=0.17 (noise) |
| deduction_tiers/count_2x2/None | +1.7% | p=0.20 (noise) |
| deduction_tiers/count_2x2/Hard | -0.9% | p=0.56 (noise) |

---

## Root Cause Analysis

### Why Tier 1.2 Helps (Multi-cell enumeration)

1. **Fully-assigned detection cost**: O(cells) - check popcount for each cell
2. **Enumeration cost**: O(n^cells) - generate and filter all tuples
3. **Crossover point**: When n^cells >> cells, fully-assigned check saves time
4. **Typical case**: 3-4 cell Add/Mul cages on 3x3-6x6 grids benefit

**Evidence**:
- 2x2_add: Skips expensive enumeration on multi-cell Add cage (-8.4%)
- 3x3_rows: Multiple row cages benefit from fast path (-4.2%)
- count_solutions/limit_10: Many solution attempts benefit (-11.3%)

### Why Tier 1.2 Hurts (Simple puzzles)

1. **Fully-assigned overhead**: Still O(cells) even when enumeration is cheap
2. **Simple puzzles**: Few cages, low propagation iterations
3. **Early termination**: count_solutions limit=1 exits immediately, so overhead is pure cost
4. **Singleton puzzles**: No multi-cell cages to skip, but fully-assigned check still called

**Evidence**:
- 4x4_singleton/5x5_singleton: Overhead of checks without benefit (+8.3%, +7.4%)
- count_solutions/limit_1: Single solution found quickly, fully-assigned check wasted (+23.6%)
- count_solutions/limit_2: Similar issue (+7.9%)

### Cost-Benefit Analysis

**Fully-assigned check cost** (O(cells)):
- For 1-2 cell cages: ~1-5 cycles (fast)
- For 3-4 cell cages: ~5-15 cycles (reasonable)
- For 5+ cell cages: ~15-50 cycles (expensive)

**Enumeration cost avoided** (O(n^cells)):
- n=2, cells=2: 4 tuples (cheap)
- n=3, cells=2: 9 tuples (cheap)
- n=3, cells=3: 27 tuples (expensive)
- n=6, cells=3: 216 tuples (very expensive)

**Conclusion**: Tier 1.2 is beneficial only when:
1. Enumeration cost >> fully-assigned check cost, OR
2. Cage is actually enumerated multiple times (cache misses on same cage)

---

## Data-Driven Decision: Keep or Refine?

### Current Status: IMPLEMENTED

Tier 1.2 is working correctly, but producing mixed results suggests:

1. **The optimization is not universally beneficial**
2. **Simple puzzles pay overhead without gain**
3. **Complex puzzles benefit substantially**

### Proposed Refinement: Conditional Application

Instead of always checking all_cells_fully_assigned, apply Tier 1.2 only when:
- Cage has 3+ cells (small cages don't benefit), OR
- Cage operation is Add/Mul (not Sub/Div which are already fast)

This would:
- Eliminate regressions on 1-2 cell cages
- Keep improvements on 3+ cell cages
- Reduce fully-assigned check overhead

### Risk Assessment

**Current Implementation**: SAFE (no correctness issues, only performance variance)
- All 26 tests passing
- No infinite loops or constraint violations
- Behaves correctly in all deduction tiers

**Proposed Refinement**: LOW RISK
- Only changes when optimization is applied, not if/how
- Can validate with same test suite
- Can measure impact incrementally

---

## Comparison to Tier 1.1

| Aspect | Tier 1.1 (Cache) | Tier 1.2 (Fast Path) |
|--------|-----------------|----------------------|
| Improvement | Consistent 5-50% | Mixed -20% to +24% |
| Risk | Low | Medium (overhead variance) |
| Complexity | Moderate (HashMap) | Low (one check) |
| Benefit consistency | High (amortizes over solve) | Variable (puzzle dependent) |
| Drawback | Memory overhead | Unconditional overhead on simple puzzles |

---

## Recommendation: Keep Tier 1.2, Plan Refinement

### Immediate Action
- Keep current Tier 1.2 implementation (no correctness issues)
- Document mixed results and crossover analysis
- Plan refinement: apply only to 3+ cell cages

### Short-term (1-2 weeks)
- Implement conditional Tier 1.2 (skip check for cells <= 2)
- Re-benchmark to verify regressions eliminated
- Measure improved benefit consistency

### Alternative Path
- If refinement doesn't improve cost-benefit significantly, consider:
  - Removing Tier 1.2 and focusing on Tier 1.3 (pre-filtering tuples)
  - Or deferring to Tier 2 (Partial Constraint Checking)

---

## Next Step: Tier 1.3 Re-evaluation

With Tier 1.2 complete, we now re-evaluate Tier 1.3 (Tuple Pre-filtering) per user's directive: "once finished, then re-evaluate tier 1.3 once tier 1.2 fully exists"

### Tier 1.3 Questions

1. **Is tuple pre-filtering still worthwhile with Tier 1.2 in place?**
   - Tier 1.2 skips enumeration for fully-assigned cages
   - Pre-filtering targets enumeration cost reduction
   - Complementary, not redundant

2. **Can Tier 1.3 eliminate the Tier 1.2 regressions?**
   - Pre-filtering reduces enumeration cost to near-zero
   - Tier 1.2 fully-assigned check becomes less needed
   - Could simplify optimization strategy

3. **What's the combined impact of Tier 1.1 + Tier 1.2 + Tier 1.3?**
   - Tier 1.1: Cache (5-50% improvement, consistent)
   - Tier 1.2: Fast path (mixed -20% to +24%)
   - Tier 1.3: Pre-filtering (estimated 3-8% additional)
   - Net: 5-50% with optimized composition

---

## Files Changed

### Implementation
- `kenken-solver/src/solver.rs`
  - Lines 262-285: Helper functions (all_cells_fully_assigned, compute_any_mask_from_assigned)
  - Lines 752-773: Sub/Div fast path (Tier 1.2)
  - Lines 863-867: Add/Mul fast path (Tier 1.2)

### Benchmarking
- `kenken-solver/benches/solver_smoke.rs` (existing benchmark suite)
- Results: `/tmp/bench_tier12_results.txt`

---

## Conclusion

**Tier 1.2 Domain Constraint Filtering is implemented and correct, but shows mixed performance results.** The optimization helps multi-cell enumeration workloads (8-19% improvement) while introducing overhead on simple puzzles (7-24% regression).

**Recommendation**: Keep implementation, plan refinement to apply only to cages with 3+ cells. Then proceed to Tier 1.3 re-evaluation as per user's directive.

