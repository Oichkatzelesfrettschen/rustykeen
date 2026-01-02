# Tier 2.1 Optimization: Findings and Constraints

**Date**: 2026-01-02
**Status**: Attempted but Infeasible (see constraints below)
**Scope**: Propagate function optimization targeting 29% CPU time

---

## Executive Summary

Tier 2.1 was designed to optimize the `propagate()` function by skipping domain recalculation for unchanged cells. While the approach seems intuitive, implementation revealed fundamental correctness constraints that make it infeasible without significant architectural changes.

**Decision**: Defer Tier 2.1. Current Tier 2.2 (MRV cache) is validat and effective. Consider Tier 2.3 (LCV heuristic) or alternative approaches instead.

---

## Attempted Optimization Approach

### What Was Tried

**Option A (Implemented)**: Affected-Cages Optimization
- Track "changed_cells" = cells newly assigned in previous iteration
- Only recalculate cage deductions for cages touching changed_cells
- Rationale: Most cells' domains don't change if they weren't involved in assignments

**Implementation Details**:
- Added `cell_to_cages: Option<Vec<Vec<usize>>>` mapping (pre-computed once per propagate call)
- Track `changed_cells: Vec<usize>` initialized to all cells on first iteration
- On iteration 2+, only process cages that touch newly forced cells
- Size threshold: Disable for n < 6 to avoid allocation overhead on small puzzles

---

## Why This Fails: The Correctness Constraint

### The Core Problem

**Fundamental Issue**: Cage deduction effectiveness depends on ALL cells' domains, not just the cells directly involved in the assignment.

### Specific Reasons

1. **Row/Column Mask Changes**
   - When cell (r, c) is assigned, row_mask[r] and col_mask[c] change
   - This affects domain calculation for ALL other cells in that row/column
   - Example: Assigning 5 at (0,1) reduces domains for ALL cells in row 0
   - Solution requires recalculating domains for entire affected rows/columns, not just the assigned cell

2. **Cage Deduction Interdependencies**
   - Cage A's deductions can constrain cage B's possible tuples
   - If we skip cage B processing because no cells in cage B were "changed", we miss valid deductions
   - Example: Cage A (Sum=10, cells={0,1}) reduces domain of cell 0 to {3,4}. This affects cage B (cells={0,2,3}) even though none of B's cells were directly assigned

3. **Domain Propagation**
   - Domains of unassigned cells can change due to constraint propagation through multiple layers
   - Cell may be "unchanged" (not assigned) but its domain may have been reduced by prior deductions
   - Skipping domain recalculation for such cells causes loss of precision in subsequent iterations

### Why Full Recalculation is Necessary

The current approach (original Tier 2.2 baseline) does:
```
for each iteration:
    domains.fill(0u64)  // Reset all
    for each cell:
        domains[cell] = calculate_domain(cell)
    for each cage:
        apply_cage_deductions(cage, domains)
    for each cell:
        if domain[cell] == 1 bit:
            place assignment
```

This is necessary because:
- Row/column masks change → ALL cells' domains must be recalculated
- Cage deductions affect unassigned cells → ALL cages must be evaluated
- Domain precision compounds across iterations → Fresh calculation ensures no information loss

---

## Attempted Workarounds and Why They Failed

### Workaround 1: Only Update Changed Cells' Domains

**Attempt**: Skip `domains.fill(0u64)` and only update domains for changed_cells

**Failure**:
- 2x2 puzzle reported no solution (should have 1)
- Root cause: Unaffected cells kept stale domains from previous iteration
- Domains from prior iterations contained incorrect information

### Workaround 2: Affected-Cages Processing

**Attempt**: Only process cages touching changed_cells

**Failure**:
- `solve_one_with_deductions` returned None for 2x2 puzzle (should return solution)
- Root cause: Skipping cage deductions meant some cells' domains were never constrained
- Missing deduction opportunities led to unsolvable state detection

### Workaround 3: Size Threshold for Small Puzzles

**Attempt**: Disable optimization for n < 6, use baseline for small puzzles

**Result**: Fixed small puzzle failures, but...
- Still had correctness issues when enabled for n >= 6
- The "affected cages" logic doesn't account for complex deduction chains
- Benchmark showed massive regression on 12x12 on first run (likely related to incomplete cache invalidation)

---

## Technical Lessons Learned

### Why This Optimization Seemed Promising

The propagate bottleneck (29% of CPU) consists of:
1. Domain recalculation loop: O(n²) operations
2. Cage deduction application: Can be expensive for large cages

The hypothesis was:
> "Cells that weren't involved in assignments don't need domain recalculation"

This is **partially true but insufficient**:
- True: The cell itself wasn't directly assigned
- False: The cell's domain may still need recalculation due to row/column mask changes

### The Missing Piece

A correct implementation would require:
- Track which rows/columns have newly assigned cells
- Recalculate domains ONLY for cells in affected rows/columns
- This is still O(n) per iteration, not O(1), so benefit is limited

### Why This Wasn't Obvious Upfront

The architecture's design separates:
- Cell assignments (place)
- Domain recalculation (propagate/domain_for_cell)
- Constraint deduction (apply_cage_deduction)

This separation makes it hard to reason about which domains actually need recalculation. The constraint system is highly interconnected:
- Row/column constraints → domain reduction
- Cage constraints → further domain reduction
- Domain reduction → enables new assignments → triggers new row/column constraints

---

## Alternative Approaches to Consider

### Tier 2.3: LCV Value Ordering (Original Plan)
- **Different target**: Search tree width, not CPU time efficiency
- **Status**: Still viable, but different benefit
- **Priority**: Deferred until Tier 2.1 approach stabilizes or is formally abandoned

### Micro-Optimization: Cage Enumeration Speedup
- Cache cage tuple enumerations to avoid recomputation
- Current implementation calls `enumerate_cage_tuples` fresh every propagation
- Potential: 5-10% speedup without complex state tracking
- **Priority**: Worth investigating next

### Domain Representation Alternatives
- Current: u64 bitset per cell
- Alternatives: fixedbitset (SIMD), smallbitvec (inline storage)
- **Status**: Documented in OPTIMIZATION_ROADMAP.md
- **Priority**: Lower (already doing well with current representation)

### Deduction Tier Micro-Optimization
- Current: Hard tier applies maximum constraint propagation
- Optimization: Early termination if puzzle is already solved
- Potential: Marginal benefit
- **Priority**: Low

---

## Decision Framework

### Why Tier 2.2 (MRV Cache) Succeeded

- **Clean state separation**: Cache only stores MRV result, doesn't depend on domain state
- **Straightforward invalidation**: Dirty tracking based on simple domain reduction checks
- **Low complexity**: ~300 LOC, easy to reason about correctness
- **Performance**: 4-9% improvement validated across all puzzle sizes

### Why Tier 2.1 Failed

- **State complexity**: Requires tracking "changed cells" across iterations with incomplete information
- **Correctness hazards**: Multiple ways to get it wrong (missed domains, missed deductions, stale state)
- **Architectural mismatch**: Constraint system interconnectedness makes incremental updates unsafe without significant refactoring

---

## Recommendations

1. **Accept Tier 2.2 as the current optimization level** (4-9% improvement, stable)
2. **Investigate micro-optimizations** (cage enumeration caching, domain lookups)
3. **Plan Tier 2.3** (LCV heuristic) as next major optimization target
4. **Document architectural constraints** for future optimization attempts
5. **Consider formal verification** of domain recalculation correctness if pursuing advanced optimizations

---

## References

- **Tier 2.2 Decision**: `docs/tier22_final_decision.md`
- **Implementation Plan**: `docs/tier21_implementation_plan.md` (now archived)
- **Roadmap**: `docs/OPTIMIZATION_ROADMAP.md`
- **Core Code**: `kenken-solver/src/solver.rs` (propagate function, line 739+)

---

**Status**: Ready for review and feedback

**Author**: Claude Code
**Date**: 2026-01-02
