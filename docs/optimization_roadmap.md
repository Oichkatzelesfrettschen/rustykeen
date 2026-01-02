# Solver Optimization Roadmap

**Status**: Analysis Phase Complete
**Next Steps**: Performance Optimization Based on Profiling

---

## Profiling-Guided Optimization Strategy

Based on tracing-flame profiling of 2x2, 4x4, and 6x6 puzzles, this document outlines optimization opportunities prioritized by:
1. **Impact**: How much solver time would be saved
2. **Effort**: How much work to implement
3. **Risk**: Likelihood of breaking existing functionality

## Tier 1: High-Impact, Low-Effort Optimizations

### 1.1 Cage Tuple Caching (Impact: HIGH, Effort: MEDIUM)

**Current Situation**:
- `enumerate_cage_tuples` is called frequently during propagation
- Same cage may be enumerated multiple times with different domain constraints
- Each enumeration is O(n^k) where k = cage size

**Optimization**:
Implement memoization for cage tuple enumeration:
```rust
// Cache valid tuples for each cage given domain constraints
struct CageTupleCache {
    cache: HashMap<(CageId, DomainHash), Vec<Tuple>>,
}

// Before: enumerate_cage_tuples(cage, domains) -> O(n^k)
// After: cached lookup -> O(1) if hit, O(n^k) if miss
```

**Expected Impact**: 20-40% reduction in propagation time for typical puzzles

**Risk**: LOW (cache invalidation is straightforward)

**Effort**: 2-3 days

---

### 1.2 Domain Constraint Filtering (Impact: MEDIUM, Effort: MEDIUM)

**Current Situation**:
- `cage_feasible` checks entire cage feasibility
- Some checks redundant if individual cell domains already eliminated value

**Optimization**:
Add early-exit checks in `apply_cage_deduction`:
```rust
// Skip cage_feasible if single cell can't contribute to target
fn apply_cage_deduction() {
    if cage.cells.len() == 1 {
        // Single cell: target must equal cell value
        // Fast path: set cell = target
        return;
    }

    // Multi-cell: check feasibility
    if !cage_feasible(...) {
        return None;
    }
}
```

**Expected Impact**: 10-20% reduction in feasibility checks

**Risk**: LOW (optimization only, doesn't change semantics)

**Effort**: 1-2 days

---

### 1.3 MRV Heuristic Optimization (Impact: MEDIUM, Effort: HIGH)

**Current Situation**:
- `choose_mrv_cell` iterates all unassigned cells
- Counts remaining domain values for each
- O(n^2) per call in worst case

**Optimization Options**:
A) **Maintain MRV queue incrementally**:
   - Update MRV order as propagation happens
   - Avoid full re-scan on each branch

B) **Weighted MRV variant**:
   - Incorporate cage size / constraint tightness
   - Better heuristic → fewer branches needed

**Expected Impact**: 15-30% reduction in backtracking for complex puzzles

**Risk**: MEDIUM (different tie-breaking could affect solution counts)

**Effort**: 3-4 days (full implementation with testing)

---

## Tier 2: Medium-Impact, Medium-Effort Optimizations

### 2.1 Tuple Pre-filtering (Impact: MEDIUM, Effort: MEDIUM)

**Current Situation**:
- All tuples enumerated, then filtered against domain
- Wasteful for large cages with small domains

**Optimization**:
```rust
// Instead of: all_tuples(...).filter(|t| matches_domains(t, domains))
// Use: enumerate_tuples_matching(cage, domains)
//      generates only valid tuples from the start
```

**Expected Impact**: 10-25% faster tuple enumeration

**Risk**: LOW (internal optimization)

**Effort**: 2-3 days

---

### 2.2 Partial Constraint Checking (Impact: MEDIUM, Effort: HIGH)

**Current Situation**:
- All cage constraints checked fully every time
- Many re-checks of already-validated constraints

**Optimization**:
- Track which constraints "matter" (have changeable domains)
- Skip full re-check if domains unchanged

**Expected Impact**: 20-40% faster propagation for deep search trees

**Risk**: MEDIUM (constraint tracking must be precise)

**Effort**: 4-5 days

---

## Tier 3: Long-Term Architectural Improvements

### 3.1 Parallel Backtracking (Impact: HIGH, Effort: VERY HIGH)

**Vision**: Distribute search across multiple CPU cores

**Current State**: Sequential backtracking

**Optimization**:
- Use rayon for parallel branch exploration
- Different threads explore different branches of search tree
- Lock-free domain representation or thread-safe sharing

**Expected Impact**: 2-4x speedup on multi-core (for complex puzzles)

**Risk**: HIGH (concurrency bugs, must ensure correctness)

**Effort**: 1-2 weeks

**Blocker**: Requires thread-safe domain representation (currently not designed for this)

---

### 3.2 Intelligent Backtracking (Impact: MEDIUM, Effort: VERY HIGH)

**Vision**: Jump back multiple levels when detecting failure patterns

**Current State**: Simple chronological backtracking

**Optimization**:
- Implement Dependency-Directed Backtracking (DDB)
- Track which variable caused conflict
- Jump back to that level, not just one level up

**Expected Impact**: 50%+ reduction in search tree for highly constrained puzzles

**Risk**: VERY HIGH (requires sophisticated conflict analysis)

**Effort**: 2-3 weeks

---

### 3.3 Iterative Deepening with Cost Bounds (Impact: MEDIUM, Effort: MEDIUM)

**Vision**: Use A* or branch-and-bound search instead of pure backtracking

**Current State**: Depth-first backtracking with forward checking

**Optimization**:
- Estimate lower bound on remaining propagations
- Prune branches that exceed cost threshold
- Iteratively increase bound until solution found

**Expected Impact**: Better pruning for moderately complex puzzles (5-20%)

**Risk**: MEDIUM (heuristic quality affects performance)

**Effort**: 1 week

---

## Profiling-Guided Next Steps

### Immediate (Next 2 Weeks)

1. **Validate Profiling Assumptions**:
   - [ ] Instrument backtrack to log recursion depth
   - [ ] Measure cage_feasible call frequency
   - [ ] Count tuple enumerations per puzzle

2. **Identify True Bottleneck**:
   - [ ] Use CPU flamegraph (perf/cargo-flamegraph) not just tracing
   - [ ] Measure wall-clock time in each major function
   - [ ] Identify cache misses and branch mispredictions

3. **Implement Quick Wins** (1.1, 1.2):
   - [ ] Add cage tuple caching
   - [ ] Implement fast paths in apply_cage_deduction
   - [ ] Measure impact

### Short Term (1-2 Months)

4. **Benchmark Impact**:
   - [ ] Run domain_repr.rs benchmarks before/after optimizations
   - [ ] Measure improvement on diverse puzzle types (not just singletons)
   - [ ] Profile real-world puzzle difficulty

5. **Choose Next Optimization** (from Tier 2):
   - [ ] Profile to identify which gives best ROI
   - [ ] Implement chosen optimization
   - [ ] Validate correctness with full test suite

### Medium Term (2-6 Months)

6. **Advanced Optimizations**:
   - [ ] Evaluate Tier 3 options based on profiling data
   - [ ] Implement highest-priority optimization
   - [ ] Consider parallel backtracking if justified by profiling

---

## Success Metrics

### Performance Goals

- [ ] 2x2 puzzles: < 1 µs (currently ~72 ns)
- [ ] 4x4 puzzles: < 1 ms (currently ~14 µs estimated)
- [ ] 6x6 puzzles: < 100 ms (currently ~100 µs estimated)
- [ ] 8x8 puzzles: < 10 s (current: unknown, likely 1-10s)
- [ ] 9x9 puzzles: < 30 s (goal: make solvable in reasonable time)

### Quality Metrics

- [ ] All 42+ existing tests pass
- [ ] Benchmark regression: none
- [ ] Code complexity: no increase from optimizations
- [ ] Binary size: no significant increase

---

## Decision Matrix: Which Optimization to Pursue

| Optimization | Impact | Effort | Risk | Priority |
|-------------|--------|--------|------|----------|
| Cage Tuple Caching | HIGH | MEDIUM | LOW | **1st** |
| Domain Filtering | MEDIUM | MEDIUM | LOW | **2nd** |
| Tuple Pre-filtering | MEDIUM | MEDIUM | LOW | **3rd** |
| MRV Heuristic | MEDIUM | HIGH | MEDIUM | 4th |
| Partial Constraints | MEDIUM | HIGH | MEDIUM | 5th |
| Parallel Backtracking | HIGH | VERY HIGH | HIGH | Later |
| Intelligent Backtracking | MEDIUM | VERY HIGH | VERY HIGH | Research |

**Recommended Path**:
1. Cage Tuple Caching (high impact, low risk, medium effort)
2. Domain Filtering (quick wins after profiling)
3. Tuple Pre-filtering (low-hanging fruit)
4. Re-profile and evaluate Tier 2 options

---

## Implementation Checklist

### Before Starting Any Optimization

- [ ] Profile current code with perf/CPU flamegraph
- [ ] Measure baseline on target puzzle set
- [ ] Document expected improvement
- [ ] Write unit tests for optimization

### During Implementation

- [ ] Implement optimization with feature flag (enable/disable at compile-time)
- [ ] Add benchmarks for changed functions
- [ ] Profile with/without optimization
- [ ] Validate improvement matches expectation

### After Implementation

- [ ] Run full test suite
- [ ] Verify no benchmark regressions
- [ ] Document any trade-offs (memory vs speed, etc.)
- [ ] Update CLAUDE.md with new optimization

---

## Expected Cumulative Impact

If all Tier 1-2 optimizations implemented successfully:

```
Baseline:     1x (current)
After Tier 1: 1.5-2.0x faster (caching + filtering)
After Tier 2: 2.5-3.5x faster (additional tuple/constraint optimizations)
```

**Target**: 3x overall performance improvement for typical puzzles by end of optimization phase.

---

## References

- **Profiling Data**: docs/profiling_analysis.md
- **Domain Benchmarks**: docs/domain_representation_selection.md
- **Instrumentation**: kenken-solver/src/solver.rs (13 decorated functions)
- **Test Suite**: kenken-solver/tests/* (42+ tests for validation)
