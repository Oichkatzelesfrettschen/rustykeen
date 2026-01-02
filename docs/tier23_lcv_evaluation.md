# Tier 2.3: LCV Value Ordering - Evaluation and Planning

**Date**: 2026-01-02
**Status**: Pre-Implementation Research
**Target**: Search tree width optimization via Least Constraining Value heuristic

---

## Executive Summary

Tier 2.3 targets search tree width rather than CPU efficiency. While Tier 2.2 optimized the constraint propagation machinery, Tier 2.3 optimizes VALUE SELECTION during backtracking - choosing values that constrain the remaining puzzle least.

**Key Difference from Tier 2.1/2.2**:
- Tier 1-2.2: Optimize constraint evaluation and caching
- Tier 2.3: Optimize search branch selection
- Impact: Fewer failed branches, faster backtracking

**Viability**: HIGHER than Tier 2.1 (no architectural conflicts)

---

## Problem Context

### Current Value Selection Strategy

In `backtrack()` function, when trying values for a cell:
```rust
for value in 1..=n {
    // Try assigning cell to value
    // If works: continue search
    // If fails: backtrack
}
```

Values are tried in order **1 through n** - first value first, regardless of constraint impact.

### The Limitation

Some values are "better" for search efficiency:
- Value A: Constrains 20 cells, fails immediately → Bad (wasted branch)
- Value B: Constrains 2 cells, likely solvable → Good (promising branch)

Current solver tries A first, wastes time, then tries B.

### LCV Heuristic Concept

**Least Constraining Value**: For each candidate value, estimate how many cells it constrains.
- Score each value by "constrainingness"
- Try least constraining values first
- Rationale: Save constraints for later choices (more flexibility)

---

## Technical Approach

### Phase 1: Research & Measurement

**Questions to answer**:
1. How many cells does each value typically constrain?
2. What's the correlation between "constrainingness" and search success?
3. What's the measurement overhead per value?

**Measurement plan**:
```rust
// For each value being considered:
// 1. Simulate assignment (place cell)
// 2. Run propagate() to get new domain constraints
// 3. Count how many cells were affected (domain reduced)
// 4. Score = 1 / (cells_affected + 1)  // Lower score = less constraining
// 5. Restore state (undo)
```

### Phase 2: Implementation

**Location**: In `backtrack()` before the value selection loop

**Changes needed**:
1. Add `value_scores: Vec<u32>` to track constrainingness
2. For each candidate value:
   - Measure propagation impact
   - Compute score
   - Store in value_scores
3. Sort values by score (ascending)
4. Try values in score order

**Code structure**:
```rust
fn backtrack(...) {
    // ... existing code ...

    // Get candidate values for this cell
    let domain = domain_for_cell(...)?;
    let mut candidate_values: Vec<u8> = extract_values_from_domain(domain);

    // TIER 2.3: Score values by constrainingness
    #[cfg(feature = "lcv-heuristic")]
    {
        let mut scores = Vec::with_capacity(candidate_values.len());
        for &value in &candidate_values {
            let score = measure_value_constrainingness(puzzle, state, idx, value);
            scores.push((value, score));
        }
        // Sort by score (lower = less constraining = try first)
        scores.sort_by_key(|(_, s)| *s);
        candidate_values = scores.into_iter().map(|(v, _)| v).collect();
    }

    // Try values in order (LCV order if enabled)
    for value in candidate_values {
        // ... existing backtrack logic ...
    }
}
```

### Phase 3: Measurement & Validation

**Metrics to track**:
- Backtrack count (before/after LCV)
- Search depth (before/after)
- LCV measurement overhead (per value scored)
- Overall wall-clock time

**Benchmarks**:
- Small puzzles (2x2, 3x3): Overhead may dominate benefit
- Medium puzzles (6x6, 8x8): Sweet spot for LCV benefit
- Large puzzles (12x12+): Maximum benefit expected

---

## Expected Benefits

### Search Tree Impact

LCV typically reduces search tree by 20-50% on puzzles requiring backtracking.

### Puzzle Categories

| Category | Benefit | Notes |
|----------|---------|-------|
| Uniqueness checking (already solved) | LOW | Propagation finds solution, minimal backtrack |
| Easy/Normal (pure deduction) | NEGLIGIBLE | Solution found via propagation, no backtrack |
| Hard tier (some backtrack) | MEDIUM | Few branches tried before solution |
| Extreme (heavy backtrack) | HIGH | Many branches - LCV ordering saves time |

### Realistic Speedup Estimate

- **Worst case**: 0-2% (puzzles solved by propagation alone)
- **Typical case**: 3-8% (puzzles needing minor backtracking)
- **Best case**: 15-30% (puzzles needing extensive search)
- **Portfolio average**: 5-12% estimated

---

## Implementation Considerations

### Complexity vs. Benefit

**Pros**:
- No architectural constraints (unlike Tier 2.1)
- Works orthogonally with Tier 2.2 (MRV cache)
- Clear measurement methodology
- Moderate implementation complexity

**Cons**:
- Measurement overhead per value (propagate simulation)
- Only helps puzzles that backtrack (doesn't help pure deduction)
- Needs feature-gating to avoid overhead on all puzzles

### Feature Flag Strategy

```toml
[features]
lcv-heuristic = []  # Optional: Enable LCV value ordering
```

**Rationale**:
- LCV measurement has overhead (simulate propagate for each value)
- Only beneficial for backtracking puzzles
- Allow users to disable if measurement overhead outweighs benefit

### Measurement Overhead

For each value being scored:
1. Save state (cells assigned so far)
2. Run propagate() to completion
3. Count affected cells
4. Restore state

**Cost estimate**: 5-10x baseline propagate (for measurement only)
- If 10 candidates and measure takes 7x propagate: overhead = 70x for one cell
- But only on backtracking puzzles where this cost is tolerable

**Optimization**: Cache measurement results for repeated cells?

---

## When to Pursue Tier 2.3

### Recommended Sequencing

1. **After Tier 2.2 validation** ✅ Done - Tier 2.2 is stable (4-9% improvement)
2. **After Tier 2.1 evaluation** ✅ Done - Tier 2.1 deemed infeasible
3. **Benchmark on real puzzle sets** - Run against corpus to measure actual benefit
4. **Decision point**: If >3% average improvement → Implement full Tier 2.3

### Prerequisites

- [ ] Identify representative backtracking-heavy puzzle set
- [ ] Establish baseline backtrack counts and timings
- [ ] Measure LCV overhead in isolated setting
- [ ] Estimate portfolio impact

---

## Comparison: Tier 2.3 vs. Alternatives

### Micro-Optimization: Cage Enumeration Caching

**Target**: `enumerate_cage_tuples()` called repeatedly with same cage/domain state
**Approach**: Cache enumeration results, invalidate on domain change
**Complexity**: Low-Medium
**Estimated benefit**: 2-5%
**Viability**: High (simple, orthogonal to other optimizations)

### Tier 2.3: LCV Value Ordering

**Target**: Search tree width on backtracking puzzles
**Approach**: Score values by propagation impact, try least constraining first
**Complexity**: Medium
**Estimated benefit**: 5-12%
**Viability**: High (no architectural conflicts)

### Recommendation

**Short term** (next week):
- Implement micro-optimization: cage enumeration caching
- Low effort, immediate benefit

**Medium term** (2-3 weeks):
- Research Tier 2.3: measure overhead and benefit
- Decision: implement if >3% average improvement

**Long term** (future):
- Domain representation alternatives (fixedbitset, smallbitvec)
- Larger puzzle support (16x16+, 32x32)

---

## Success Criteria for Tier 2.3

### Measurement Phase
- [ ] Overhead per value scoring: <10x baseline propagate
- [ ] Portfolio backtrack count reduction: >20%
- [ ] Wall-clock improvement on puzzle corpus: >2%

### Implementation Phase
- [ ] All 42+ tests pass with LCV enabled
- [ ] Feature-gated (enabled via cargo feature)
- [ ] Benchmark shows consistent improvement across puzzle sizes
- [ ] Documentation complete

---

## References

- **LCV Heuristic**: Constraint Satisfaction Problem (CSP) literature
  - Dechter & Pearl (1989): "Tree Decomposition and Constraint Satisfaction"
  - Bessière & Régin (1996): "MAC and combined heuristics are tolerable"
- **Current Solver**: `kenken-solver/src/solver.rs` (backtrack function, line 233+)
- **Related Optimizations**: `docs/OPTIMIZATION_ROADMAP.md`

---

**Status**: Pre-Implementation - Ready for measurement phase

**Author**: Claude Code
**Date**: 2026-01-02
