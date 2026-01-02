# Tier 2.1: Propagation Optimization - Skip Unchanged Cells

**Date**: 2026-01-02
**Status**: Research and Planning Phase
**Target**: ~29% of CPU time (propagate function from flamegraph)
**Estimated Complexity**: Medium (state tracking, API adjustments)

---

## Executive Summary

Tier 2.1 optimizes the constraint propagation loop by skipping unnecessary domain recalculations for cells whose constraints haven't changed. This builds on Tier 2.2 (MRV cache) to target the next major bottleneck: the propagate() function that recalculates domains from scratch every iteration.

**Hypothesis**: By tracking which cells were modified in the previous propagation iteration, we can avoid recalculating domains for unaffected cells in subsequent iterations, reducing redundant computation.

**Expected Benefit**: 8-15% overall improvement on large puzzles (where propagate dominates).

---

## Part 1: Current propagate() Implementation Analysis

### Current Algorithm (lines 750-822 in solver.rs)

```rust
fn propagate(...) {
    let mut domains = vec![0u64; a];  // Working domain buffer

    loop {
        // FULL RESCAN: Rebuild domains from scratch EVERY iteration
        domains.fill(0u64);
        for (idx, dom_slot) in domains.iter_mut().enumerate() {
            if state.grid[idx] != 0 {
                *dom_slot = 1u64 << (state.grid[idx] as u32);
                continue;
            }
            let r = idx / n;
            let c = idx % n;
            // Compute domain from row/col masks
            *dom_slot = full_domain(n) & !state.row_mask[r] & !state.col_mask[c];
        }

        // Apply cage deductions to ALL cages
        for cage in &puzzle.cages {
            apply_cage_deduction(...)?;
        }

        // Check for contradictions and forced assignments
        let mut any_forced = false;
        for (idx, &dom) in domains.iter().enumerate() {
            if state.grid[idx] != 0 { continue; }
            if popcount_u64(dom) == 1 {
                place(...);
                forced.push(...);
                any_forced = true;
            }
        }

        if !any_forced {
            return Ok(true);  // Fixed point reached
        }
    }
}
```

### Key Inefficiency

**Every propagation iteration**:
1. Recalculates domains for ALL n² cells from scratch
2. Applies deductions to ALL cages (even if their cells haven't changed)
3. Scans ALL cells looking for forced assignments

This is expensive because:
- Early iterations may force many assignments (cells change)
- But later iterations have few forced assignments yet still do full rescan
- On large puzzles (12x12+), the repeated scanning dominates

### Flamegraph Evidence

From previous profiling:
- propagate() = ~29% of CPU time
- choose_mrv_cell = ~39% of CPU time (now optimized by Tier 2.2)
- apply_cage_deduction = ~15% of CPU time

After Tier 2.2 reduces choose_mrv_cell overhead, propagate() becomes relatively larger target.

---

## Part 2: Tier 2.1 Optimization Strategy

### Core Idea: Track Changed Cells

Instead of recalculating all domains every iteration, track which cells **must** be recomputed:

1. **Forced Assignment** → Cell placed, grid[idx] changed → Affects all cages containing idx
2. **Domain Reduction** → Cell domain shrunk → Affects all cages containing idx
3. **Other Cells** → No constraint changed → Can skip domain recalculation

### Implementation Approach

#### Option A: Minimal Change (Recommended)
Track which cells were modified in the last iteration and only recalculate:
- Domains for modified cells
- Deductions for cages touching modified cells

**Pros**:
- Surgical change to propagate() loop
- Minimal state management
- Works with existing cage structure

**Cons**:
- Requires identifying "affected cages" (all cages touching modified cells)
- Need to handle cage deduction interdependencies

#### Option B: Explicit Dirty Tracking (Like Tier 2.2)
Maintain a "dirty set" of cells that need recomputation:

**Pros**:
- Explicit, easy to reason about
- Similar pattern to Tier 2.2 MrvCache.mark_dirty()

**Cons**:
- Additional state in State struct
- Memory overhead (small but non-zero)

#### Option C: Incremental Domain Update (Advanced)
Instead of recalculating from scratch, maintain domains incrementally:

**Pros**:
- Maximum efficiency
- Domains only updated when necessary

**Cons**:
- Complex: must track all constraint changes
- Risk of correctness issues
- Requires careful reasoning about domain evolution

**Recommendation**: Start with **Option A** (minimal change, affected cages only).

---

## Part 3: Proposed Implementation (Option A)

### Phase 1: Identify Affected Cages

Add helper to find all cages touching a specific cell:

```rust
fn cages_for_cell(puzzle: &Puzzle, idx: usize) -> Vec<usize> {
    puzzle.cages.iter()
        .enumerate()
        .filter(|(_, cage)| cage.cells.iter().any(|&c| c.0 as usize == idx))
        .map(|(i, _)| i)
        .collect()
}
```

### Phase 2: Modified Propagate Loop

```rust
fn propagate(...) {
    let mut domains = vec![0u64; a];
    let mut changed_cells = (0..a).collect::<HashSet<_>>();  // Initially all cells changed

    loop {
        // OPTIMIZATION: Only recalculate domains for changed cells
        for idx in &changed_cells {
            if state.grid[*idx] != 0 {
                domains[*idx] = 1u64 << (state.grid[*idx] as u32);
                continue;
            }
            let r = idx / n;
            let c = idx % n;
            domains[*idx] = full_domain(n) & !state.row_mask[r] & !state.col_mask[c];
        }

        // OPTIMIZATION: Only apply deductions to cages touching changed cells
        let affected_cages = changed_cells
            .iter()
            .flat_map(|&idx| cages_for_cell(puzzle, idx))
            .collect::<HashSet<_>>();

        for cage_idx in affected_cages {
            let cage = &puzzle.cages[cage_idx];
            apply_cage_deduction(puzzle, rules, state, cage, tier, &mut domains)?;
        }

        // Find forced assignments (unchanged)
        let mut any_forced = false;
        let mut newly_forced = Vec::new();
        for (idx, &dom) in domains.iter().enumerate() {
            if state.grid[idx] != 0 { continue; }
            if popcount_u64(dom) == 1 {
                let val = dom.trailing_zeros() as u8;
                let r = idx / n;
                let c = idx % n;
                place(state, r, c, val);
                forced.push((idx, val));
                newly_forced.push(idx);
                any_forced = true;
            }
        }

        if !any_forced {
            return Ok(true);
        }

        // OPTIMIZATION: Only cells that were forced are changed for next iteration
        changed_cells = newly_forced.into_iter().collect();
    }
}
```

### Phase 3: Handle Interdependencies

**Problem**: Cage deductions can create cascading changes:
- Cage 1 deduction narrows domain of Cell A
- Cell A's narrowed domain affects Cage 2
- Cage 2 deduction narrows domain of Cell B
- etc.

**Current Behavior**: Full rescan handles this automatically (all domains recalculated).

**Tier 2.1 Challenge**: Need to detect second-order effects.

**Solution**:
- Option: Run propagate multiple times until fixed point (already done by outer loop)
- Or: Track domain changes more carefully (Option B/C)

**Recommendation**: Start with simple approach (Option A) and validate against full-rescan baseline.

---

## Part 4: Expected Benefits

### Analysis

For a typical puzzle solve:
- **Iteration 1**: 5-10 forced assignments, ~50% of cells change
- **Iteration 2**: 3-5 forced assignments, ~20% of cells change
- **Iteration 3**: 1-2 forced assignments, ~5% of cells change
- **Iteration 4+**: 0-1 forced assignments, <<1% of cells change

**Current Cost**: O(n²) rescan every iteration
**Optimized Cost**: O(|changed_cells|) × domain recalc

**Estimated Savings**:
- Early iterations: 20-50% time savings (overlaps with Tier 2.2 improvements)
- Later iterations: 50-80% time savings (most cells already stable)
- Average: 25-40% on propagate() = **7-12% overall** (since propagate ~29% of total)

### Expected Speedups

By puzzle size (after Tier 2.2):
- 2x2: -1-2% (propagate less dominant, not much saved)
- 4x4: -2-3% (minimal propagation iterations needed)
- 6x6: -5-8% (propagate still small fraction)
- 8x8: -8-12% (propagate becomes significant)
- 12x12: -10-15% (propagate dominant)
- 16x16+: -12-18% (propagate highly repetitive)

**Conservative Estimate**: 5-10% overall improvement across portfolio.

---

## Part 5: Implementation Risks

### Risk 1: Correctness (HIGH)

**Problem**: Interdependent cage deductions create cascading changes. Missing a cage could cause incorrect domain pruning.

**Mitigation**:
- Comprehensive test suite (29/29 tests must pass)
- Compare results with full-rescan baseline
- Incremental implementation: validate each phase
- Add assertions to verify domain correctness

**Validation**: Run property-based tests ensuring same final solutions as baseline.

### Risk 2: Complexity (MEDIUM)

**Problem**: Tracking changed cells adds state management complexity.

**Mitigation**:
- Keep change tracking simple (just a set of indices)
- Use helper function to find affected cages
- Document interdependency handling clearly

**Validation**: Code review focusing on domain recalculation logic.

### Risk 3: Performance Regression on Small Puzzles (LOW)

**Problem**: Additional bookkeeping (affected cage set) might add overhead on small puzzles where propagate is fast.

**Mitigation**:
- Profile before/after on 2x2-4x4
- If regression observed, add size threshold (skip optimization for n < 6)
- Optimize affected_cages computation (maybe pre-compute or cache)

**Validation**: Benchmark all sizes post-implementation.

### Risk 4: Cascading Changes Not Fully Captured (MEDIUM-HIGH)

**Problem**: If cage A's deduction narrows domain, and domain reduction enables cage B's deduction, we might not re-process cage B in the same iteration.

**Impact**: May need multiple outer loop iterations instead of converging in one. This is correctness-neutral but efficiency impact.

**Mitigation**:
- Iterative approach (Option A) naturally handles this via outer loop
- Worst case: N extra iterations where N = propagation depth (small N)
- Compare iteration count with baseline (should be same)

**Validation**: Profile propagate iteration counts before/after.

---

## Part 6: Testing Strategy

### Pre-Implementation Baseline

1. Run full test suite on current code
2. Measure propagate() CPU time breakdown:
   - Domain recalculation time
   - Cage deduction time
   - Forced assignment detection time
3. Record iteration counts and forced assignments per iteration
4. Benchmark across all sizes (2x2-12x12)

### Post-Implementation Validation

1. Run same test suite (all 29/29 tests must pass)
2. Compare results with baseline:
   - Same solutions found
   - Same iteration counts
   - Same forced assignments per iteration
3. Profile new implementation:
   - Measure domain recalculation time (should decrease)
   - Measure affected cage computation time (overhead)
   - Net effect on propagate() overall time
4. Verify no regressions on small puzzles
5. Confirm expected improvements on 8x8-12x12

### Flamegraph Analysis

- Expect propagate() width to decrease
- Expect domain recalculation to become narrower
- Expect cage deduction section to show only affected cages
- Should see skip patterns in profile

---

## Part 7: Implementation Plan

### Phase 1: Research and Profiling (Current)
- [x] Analyze current propagate() implementation
- [x] Identify optimization opportunity
- [x] Estimate expected benefits
- [ ] Baseline profiling (iteration counts, timing breakdown)

### Phase 2: Implementation (Next)
- [ ] Add helper function: cages_for_cell()
- [ ] Modify propagate() to track changed_cells set
- [ ] Test on minimal case (2x2)
- [ ] Verify correctness with full test suite
- [ ] Profile and compare to baseline

### Phase 3: Validation (After Phase 2)
- [ ] Run benchmarks across all sizes
- [ ] Check for regressions on small puzzles
- [ ] Validate iteration count matches baseline
- [ ] Generate new flamegraph
- [ ] Document results

### Phase 4: Optimization (If Needed)
- [ ] Profile affected_cages computation cost
- [ ] Consider caching or pre-computation if overhead significant
- [ ] Add size threshold if small-puzzle regressions observed
- [ ] Measure SIMD popcount effectiveness in new code

---

## Part 8: Success Criteria

**Tier 2.1 Success Metrics**:

1. **Correctness**: All 29/29 tests pass, same solutions as baseline
2. **Performance**: 5-10% overall improvement on portfolio
   - 8x8: ≥ 5% improvement
   - 12x12: ≥ 8% improvement
   - No regressions > 2% on any size
3. **No Cascading Failures**: Iteration count same as baseline (± 1 iteration)
4. **Code Quality**: <400 LOC, well-documented, minimal state management
5. **Profiling**: propagate() CPU time decreases, domain recalc visible as narrower span

---

## Part 9: Alternative Strategies

### Tier 2.1b: Incremental Domain Maintenance

Instead of recalculating domains, maintain them incrementally:
- When place(row, col, val): update row_mask, col_mask, grid
- When domain reduction in cage: update only affected domains
- Skip full rescan

**Pros**: Maximum efficiency, elegant design
**Cons**: High complexity, risk of correctness bugs
**Verdict**: Consider after Option A validated

### Tier 2.1c: Domain Caching Like MRV

Cache computed domains per cell per iteration state, invalidate only when constraints change.

**Pros**: Similar to Tier 2.2 pattern, proven approach
**Cons**: Memory overhead (caching domains for all cells), complexity
**Verdict**: Consider if Option A not effective enough

### Tier 2.2b: LCV Value Ordering (Tier 2.3)

Instead of optimizing propagate() structure, optimize which values are tried first (Least Constraining Value heuristic).

**Pros**: Orthogonal optimization, reduces search space
**Cons**: Different target (search optimization not propagation)
**Verdict**: Pursue after Tier 2.1 if needed

---

## Part 10: Decision Framework

**Proceed with Tier 2.1?** YES

**Why**:
- Clear optimization opportunity (propagate = 29% of CPU)
- Low implementation complexity (Option A is minimal change)
- Proven pattern (similar to Tier 2.2 dirty tracking)
- Minimal risk with comprehensive testing

**Start When**: After Tier 2.2 fully validated and documented
**Timeline**: 1-2 days for implementation + profiling
**Complexity**: Medium (tracking state, iterative validation)

---

## References

- **Tier 2.2 Decision Document**: docs/tier22_final_decision.md
- **Flamegraph Analysis**: From CPU profiling showing propagate = 29% time
- **Current Code**: kenken-solver/src/solver.rs lines 750-822
- **Previous Implementation**: Tier 2.2 smarter dirty tracking pattern
