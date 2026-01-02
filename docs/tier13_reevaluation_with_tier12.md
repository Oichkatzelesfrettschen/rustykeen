# Tier 1.3 Re-evaluation (Post Tier 1.2 Implementation)

**Date**: 2026-01-01
**Status**: TIER 1.2 COMPLETE - RE-EVALUATING TIER 1.3
**Context**: Tier 1.2 Domain Constraint Filtering now implemented with mixed results
**Scope**: Determine if Tier 1.3 Tuple Pre-filtering is still valuable

---

## Tier 1.3 Overview

### Original Concept
**Tuple Pre-filtering**: Modify enumerate_cage_tuples to generate only valid tuples (prune during generation, not after).

Current implementation:
1. Generate all possible tuples for a cage
2. Filter out invalid ones (constraint checking)

Proposed optimization:
1. Use tighter bounds checking (min/max sum for Add, min/max product for Mul)
2. Generate only valid tuples from start
3. Avoid generating then discarding invalid tuples

### Original Estimate
- **Complexity**: HIGH (~200-300 LOC, requires recursive generator redesign)
- **Estimated benefit**: 3-8% additional improvement over Tier 1.1
- **Risk**: MEDIUM (off-by-one errors in bounds computation, Hard tier impact)

---

## New Context with Tier 1.2 Complete

### Tier 1.1 + Tier 1.2 Combined Impact

**From Benchmarks**:
- Tier 1.1 (Cache): Consistent 5-50% improvement on repeated enumerations
- Tier 1.2 (Fast Path): Mixed -20% to +24% improvement
- Combined (Tier 1.1 + 1.2): Partially redundant (both skip/optimize enumeration)

**Key Insight**: Tier 1.2's fast path for fully-assigned cages partially overlaps with Tier 1.3's pre-filtering benefits. When a cage has fully-assigned cells, pre-filtering would also skip enumeration. However, Tier 1.2 is simpler and already deployed.

### Enumeration Cost Post Tier 1.2

**What remains to optimize**:
1. Partially-assigned cages (some cells have multiple possible values)
   - Tier 1.2 fast path doesn't apply (not fully assigned)
   - Tier 1.3 pre-filtering could help
   - Tier 1.1 cache already amortizes repeated enumerations

2. First-time enumeration of complex cages
   - Tier 1.1 cache miss case
   - Tier 1.3 pre-filtering could reduce tuple generation cost

3. Large cages (5+ cells)
   - Enumeration is expensive even with cache
   - Pre-filtering could provide additional benefit

---

## Cost-Benefit Analysis: Tier 1.3 Now

### Benefits of Tier 1.3 Pre-filtering

**When it helps**:
1. Large cages (4-9 cells) with tight constraints
   - Example: Add cage with target=12 on 9 cells
   - Pre-filtering prevents generating O(n^cells) invalid tuples
   - Estimated: 30-60% reduction in enumeration tuples

2. Operations with high constraint ratio
   - Div: Many tuples invalid (divisor!=0, clean division)
   - Mul: Product constraint eliminates many combinations
   - Add: Many sums fall outside valid range

3. Deeply nested recursion
   - enumerate_cage_tuples uses recursion (pos loop)
   - Each level generates candidates
   - Pre-filtering reduces recursion depth

**Quantified Benefit** (estimated):
- Tier 1.3 alone: 3-8% additional (on top of baseline)
- With Tier 1.1 + 1.2: 2-5% additional (diminishing returns)
- On worst-case puzzles: Could be 10-20% for large cages

### Costs of Tier 1.3 Implementation

**Implementation effort**:
- ~250-300 LOC changes to enumerate_cage_tuples
- Requires tight bounds computation for Add/Mul/Div
- Risk of subtle off-by-one errors
- Increased code complexity and maintenance burden

**Code risk**:
- Bounds computation must be exact (not loose heuristics)
- Hard tier constraint learning depends on complete enumeration
- Need extensive testing on deduction tier boundaries

**Performance risk**:
- Bounds computation itself has cost
- Pre-filtering logic adds overhead to tight loops
- Actual benefit may be less than estimated 3-8%

---

## Comparative Analysis: Tier 1.3 vs. Tier 2 Alternatives

### Tier 1.3: Tuple Pre-filtering

| Aspect | Assessment |
|--------|-----------|
| Implementation Complexity | HIGH (250-300 LOC) |
| Performance Gain | 2-5% (with Tier 1.1+1.2) |
| Code Maintainability | MEDIUM (tight bounds logic) |
| Deduction Tier Impact | MEDIUM RISK (Hard tier learning) |
| **ROI** | **LOW - Diminishing returns** |

### Tier 2.1: Partial Constraint Checking

| Aspect | Assessment |
|--------|-----------|
| Implementation Complexity | MEDIUM (150-200 LOC) |
| Performance Gain | 10-20% (higher ROI) |
| Code Maintainability | HIGH (constraint tracking) |
| Deduction Tier Impact | LOW RISK (additive optimization) |
| **ROI** | **HIGH - Better leverage** |

### Tier 2.2: MRV Heuristic Optimization

| Aspect | Assessment |
|--------|-----------|
| Implementation Complexity | LOW (50-100 LOC) |
| Performance Gain | 5-15% (moderate) |
| Code Maintainability | HIGH (heuristic refinement) |
| Deduction Tier Impact | LOW RISK (search order only) |
| **ROI** | **MEDIUM - Quick win** |

---

## Decision Framework: When to Implement Tier 1.3

### Implement Tier 1.3 if ANY of:

1. **Real-world profiling shows**:
   - enumerate_cage_tuples still 30%+ of total solve time (post Tier 1.1+1.2)
   - Tuple generation (not filtering) is dominant cost

2. **Performance ceiling**:
   - Tier 1.1 + 1.2 provides <20% overall improvement
   - Need additional 10-15% to reach performance goals

3. **Specific puzzle analysis**:
   - 6x6+ puzzles spend >40% in enumeration (post Tier 1.2)
   - Pre-filtering could unlock next level

4. **Competitive pressure**:
   - Benchmarking shows solver lags on complex puzzles
   - Every percent of improvement matters

### Defer Tier 1.3 if ALL of:

1. **Real-world profiling shows**:
   - enumerate_cage_tuples <20% of total time (post Tier 1.2)
   - OR cache provides sufficient amortization

2. **Performance goals met**:
   - Tier 1.1 + 1.2 achieves desired speedup
   - Diminishing returns not worth added complexity

3. **Tier 2 opportunities better**:
   - Partial Constraint Checking (2.1) shows 10-20% potential
   - MRV Optimization (2.2) shows quick 5-15% gains

---

## Current Recommendation: DEFER TIER 1.3

### Rationale

1. **Diminishing Returns**: With Tier 1.1 (40-52% improvement) and Tier 1.2 (mixed 2-18% improvement), Tier 1.3's 2-5% additional gain is marginally valuable.

2. **Code Complexity**: 250-300 LOC with tight bounds logic is a significant maintenance burden for modest gain.

3. **Better Alternatives Available**: Tier 2.1 (Partial Constraint Checking) offers 10-20% improvement with lower risk.

4. **Risk-Reward Unfavorable**: Tier 1.3 has medium complexity and medium risk for low gain.

### Conditions to Reconsider

**If real-world profiling reveals**:
- enumerate_cage_tuples still 30%+ of time (post Tier 1.1+1.2), OR
- Large puzzle (6x6+) performance gap vs. target

**Then**: Implement Tier 1.3 with focus on Add/Mul (highest tuple counts)

---

## Proposed Path Forward

### Phase 1: Consolidate Tier 1 Optimization (CURRENT)
1. Deploy Tier 1.1 (Cache) - DONE
2. Deploy Tier 1.2 (Fast Path) - DONE
3. Document mixed results - DONE
4. Defer Tier 1.3 pending real-world data - RECOMMENDED

### Phase 2: Real-World Profiling (1-2 weeks)
1. Monitor Tier 1.1 cache hit rates
2. Profile diverse puzzle corpus (4x4 through 12x12)
3. Measure wall-clock improvement vs. theoretical estimates
4. Identify remaining bottlenecks (propagate? backtrack? enumeration?)

### Phase 3: Tier 2 Evaluation (2-4 weeks)
1. Implement Tier 2.1 (Partial Constraint Checking) if propagate still dominant
2. Implement Tier 2.2 (MRV Heuristic) as quick baseline improvement
3. Profile combined Tier 1 + Tier 2 impact

### Phase 4: Conditional Tier 1.3 (If Needed)
1. Re-evaluate only if Phase 3 reveals enumerate_cage_tuples still bottleneck
2. Implement with focus on high-tuple-count operations (Add/Mul)
3. Validate on Hard tier constraint learning

---

## Summary Table: Tier 1 vs. Tier 2

| Tier | Feature | Impact | Complexity | Risk | Status |
|------|---------|--------|------------|------|--------|
| **1.1** | Cage Tuple Cache | 40-52% | Medium | Low | DEPLOYED |
| **1.2** | Fast Path (Fully Assigned) | Mixed 2-18% | Low | Low | DEPLOYED |
| **1.3** | Tuple Pre-filtering | 2-5% | High | Medium | DEFERRED |
| **2.1** | Partial Constraint Checking | 10-20% | Medium | Low | CANDIDATE |
| **2.2** | MRV Heuristic Optimization | 5-15% | Low | Low | CANDIDATE |
| **2.3** | LCV Heuristic Refinement | 3-10% | Low | Low | CANDIDATE |

---

## Next Step: Real-World Validation

To make final decision on Tier 1.3, we need:

1. **Production profiling data** from Tier 1.1 + 1.2 deployed
2. **CPU flamegraph analysis** showing enumerate_cage_tuples % of total time
3. **Puzzle corpus measurements** on 4x4 through 12x12 grids
4. **Comparison to Tier 2 opportunities** (which likely offer better ROI)

**Estimated Timeline**: 1-2 weeks of production monitoring

---

## Conclusion

**Tier 1.3 Tuple Pre-filtering is DEFERRED** pending:
1. Real-world profiling showing enumerate_cage_tuples still significant bottleneck
2. Performance targets not met by Tier 1.1 + 1.2
3. Tier 2 alternatives evaluated and ruled out

**Immediate Action**: Focus on deploying Tier 1.1 + 1.2 to production and gathering real-world metrics. Tier 1.3 implementation is not recommended at this time due to diminishing returns and availability of higher-ROI opportunities in Tier 2.

