# Tier 2.2: MRV Heuristic Optimization Plan

**Date**: 2026-01-01
**Status**: READY FOR IMPLEMENTATION
**Profiling Data**: CPU flamegraph shows choose_mrv_cell at 39% of solver time
**Expected Impact**: 20-40% overall improvement (estimated, vs. Tier 1.3's 2-5%)

---

## Problem Statement

### Current Bottleneck

**Function**: `choose_mrv_cell` (minimum remaining values heuristic)
- **Current Time**: 39% of total solver CPU (131,200 samples out of 336,100)
- **Invocation Pattern**: Called ~131 times per 3x3 puzzle solve
- **Current Algorithm**: Full linear scan of all cells, checking domain sizes

```rust
// Current implementation (lines 492-525 in solver.rs)
fn choose_mrv_cell<D: DomainOps>(puzzle: &Grid, state: &State) -> usize {
    let n = state.n;
    let mut min_cell = 0;
    let mut min_count = n as u32 + 1;

    for cell in 0..(n * n) as usize {
        if state.domains[cell] != D::empty(n) {
            let count = state.domains[cell].popcount();
            if count > 0 && count < min_count {
                min_cell = cell;
                min_count = count;
            }
        }
    }
    min_cell
}
```

**Performance Characteristics**:
- **Time complexity**: O(n^2) per call (scan all cells)
- **Calls per solve**: ~131 for 3x3 puzzle
- **Total**: O(n^4) for complete solve (exponential in backtracking depth)

### Why It's Inefficient

1. **Redundant Computation**
   - Most cells' domains unchanged since last call
   - Full rescan wastes work on stable cells

2. **No Caching or Amortization**
   - Result not cached
   - No incremental updates
   - Pure recomputation each time

3. **High Frequency**
   - Called at every backtracking decision point
   - Executed even when result hasn't changed

---

## Solution: Incremental MRV Tracking

### Core Idea

Maintain state about the minimum-remaining-value cell and invalidate it selectively:

```rust
/// State for incremental MRV computation
struct MrvCache {
    min_cell: usize,          // Cell with minimum remaining values
    min_count: u32,           // Its domain size
    valid: bool,              // Cache validity flag
    dirty_cells: Vec<bool>,   // Cells with changed domains since last query
}
```

### Implementation Strategy

**Step 1: Extend State struct (solver.rs)**

Add MrvCache to the State structure:

```rust
pub struct State {
    // ... existing fields ...
    mrv_cache: MrvCache,
}
```

Initialize in `State::new`:

```rust
impl State {
    fn new(puzzle: &Grid) -> Self {
        let n = puzzle.size();
        Self {
            // ... existing initialization ...
            mrv_cache: MrvCache {
                min_cell: 0,
                min_count: n as u32 + 1,
                valid: false,
                dirty_cells: vec![false; (n * n) as usize],
            },
        }
    }
}
```

**Step 2: Mark Domains as Dirty When Changed**

Whenever a domain is updated, mark the cell as dirty:

```rust
// In apply_cage_deduction and other domain updates
domains[cell_idx] = new_domain;
state.mrv_cache.dirty_cells[cell_idx] = true;  // Mark as dirty
state.mrv_cache.valid = false;                 // Invalidate cache
```

**Step 3: Implement Incremental choose_mrv_cell**

```rust
fn choose_mrv_cell_optimized<D: DomainOps>(
    puzzle: &Grid,
    state: &mut State,
) -> usize {
    let n = state.n;

    // If cache is valid and no dirty cells, return cached result
    if state.mrv_cache.valid && !state.mrv_cache.dirty_cells.iter().any(|&d| d) {
        return state.mrv_cache.min_cell;
    }

    // Re-scan only dirty cells
    let mut min_cell = state.mrv_cache.min_cell;
    let mut min_count = state.mrv_cache.min_count;

    for cell in 0..(n * n) as usize {
        // Skip if clean and cache valid
        if !state.mrv_cache.dirty_cells[cell] && state.mrv_cache.valid {
            continue;
        }

        let domain_count = state.domains[cell].popcount();
        if domain_count > 0 && domain_count < min_count {
            min_cell = cell;
            min_count = domain_count;
        }

        // Mark as clean after checking
        state.mrv_cache.dirty_cells[cell] = false;
    }

    // Update cache
    state.mrv_cache.min_cell = min_cell;
    state.mrv_cache.min_count = min_count;
    state.mrv_cache.valid = true;

    min_cell
}
```

**Step 4: Invalidate Cache on Backtrack**

When backtracking restores previous state, invalidate cache:

```rust
// In backtrack function, when restoring domains
state.mrv_cache.valid = false;
state.domains = saved_domains.clone();
```

---

## Implementation Details

### Code Changes Summary

| File | Location | Change | Lines | Impact |
|------|----------|--------|-------|--------|
| solver.rs | Line 50 (State struct) | Add MrvCache field | +15 | State storage |
| solver.rs | Line 75 (State::new) | Initialize MrvCache | +10 | Initialization |
| solver.rs | Line 492 (choose_mrv_cell) | Replace with incremental version | +40 | Main optimization |
| solver.rs | Lines 750, 810, 867 | Mark dirty on domain update | +3 (per location) | Dirty tracking |
| solver.rs | Line 405 (backtrack restore) | Invalidate cache | +2 | Cache invalidation |

**Total**: ~80-120 LOC changes

### Critical Integration Points

1. **Domain Update Locations** (where dirty flag must be set):
   - `apply_cage_deduction` (line 810-875)
   - `backtrack` when pruning cells (line 405-450)
   - `propagate` when assigning values (line 560-600)

2. **Backtrack Restoration** (where cache must be invalidated):
   - `backtrack` function (line 405-450)
   - After domain rollback: `state.mrv_cache.valid = false`

3. **Safety Guarantees**:
   - Every domain write must set dirty flag
   - Cache only valid when all dirty cells processed
   - Fallback: valid flag prevents returning stale data

---

## Performance Analysis

### Expected Improvement

**Current State**:
- Full scan: O(n^2) per call
- 131 calls per 3x3 solve
- Total: O(n^4) exponential

**Optimized State**:
- Cache hits: O(1) - no scan needed
- Cache misses: O(k) where k = cells modified
- Typical: k = 1-2 (only cell being assigned changes domain)
- Speedup: 5-9x for choose_mrv_cell function

**Overall Impact**:
- choose_mrv_cell: 39% of time
- Speedup: 5-9x
- Overall: 39% * 80% improvement = **31-36% overall**
- Conservative estimate: 20-40% (accounting for overhead and cache invalidation)

### Benchmark Targets

Test with existing benchmark suite:
```bash
cargo bench --bench solver_smoke --bench deduction_tiers
```

Expected changes:
- solve_one/2x2_add: -20% to -35%
- solve_one/3x3_rows: -25% to -40%
- deduction_tiers/Easy: -25% to -40%
- deduction_tiers/Normal: -20% to -35%
- deduction_tiers/Hard: -15% to -25% (fewer cells filled)

### Profiling Verification

After implementation:
1. Run benchmarks: `cargo bench --bench solver_smoke`
2. Measure actual improvement
3. Re-profile with flamegraph: `cargo flamegraph --release --bin profile_spans ...`
4. Verify choose_mrv_cell time reduced from 39% to < 10%

---

## Risk Assessment

### Correctness Risk: LOW

- Pure heuristic optimization (cell selection)
- No logic changes to constraint propagation
- All 26 existing tests must pass
- No functional behavior changes

### Performance Risk: LOW

- Dirty tracking overhead minimal (one bit array update per domain change)
- Cache invalidation simple (boolean flag)
- Fallback to full scan if cache invalid (safe)
- No correctness impact if optimization disabled

### Complexity Risk: LOW

- 80-120 LOC isolated to choose_mrv_cell and dirty tracking
- No cascading changes needed
- Clear separation of concerns

---

## Testing Strategy

### Unit Tests

Verify incremental computation equals full scan:

```rust
#[test]
fn mrv_optimization_equivalence() {
    // Test that:
    // 1. First call: mrv_cache computes correctly
    // 2. Second call without domain changes: returns cached result
    // 3. After domain update: recomputes correctly
    // 4. After backtrack: cache invalidated and recomputed
}
```

### Integration Tests

Run existing full test suite:
```bash
cargo test --all --all-features  # All 26 tests must pass
cargo clippy --all-targets       # Zero warnings
cargo fmt --check                # Format compliant
```

### Regression Tests

Benchmark comparison:
```bash
# Before: Record baseline
cargo bench --bench solver_smoke 2>&1 | tee /tmp/before_tier22.txt

# After: Compare
cargo bench --bench solver_smoke 2>&1 | tee /tmp/after_tier22.txt
diff /tmp/before_tier22.txt /tmp/after_tier22.txt
```

---

## Implementation Sequence

### Phase 1: Core Implementation (1-2 hours)

1. Add MrvCache struct definition
2. Extend State with mrv_cache field
3. Initialize MrvCache in State::new
4. Replace choose_mrv_cell with incremental version
5. Add dirty flag tracking in domain updates
6. Test compilation and basic correctness

### Phase 2: Integration (1-2 hours)

1. Verify dirty flag set at all domain change locations
2. Verify cache invalidation at backtrack points
3. Run full test suite: `cargo test --all`
4. Fix any test failures
5. Verify no clippy warnings

### Phase 3: Validation (1 hour)

1. Run benchmarks: `cargo bench --bench solver_smoke`
2. Re-profile with flamegraph
3. Measure actual vs. expected improvement
4. Document results in commit message

### Phase 4: Documentation (30 minutes)

1. Update CLAUDE.md with Tier 2.2 results
2. Commit with detailed message
3. Create analysis document

**Total Estimated Time**: 3-5 hours development + validation

---

## Success Criteria

1. **Correctness**: All 26 tests passing
2. **Code Quality**: Zero clippy warnings, format compliant
3. **Performance**: 20-40% overall improvement measured
4. **choose_mrv_cell**: Reduced from 39% to <10% of solver time
5. **Profiling**: Flamegraph shows clear reduction in choose_mrv_cell width

---

## Alternative Approaches Considered

### Option 1: Adaptive Cell Ordering (Rejected)
- Pre-compute MRV before backtracking
- Problem: Domains change during propagation, ordering becomes stale
- Not pursued

### Option 2: Skip MRV Entirely (Rejected)
- Use arbitrary cell order
- Problem: Severely impacts search efficiency, counterproductive
- Not pursued

### Option 3: Full Recompute with Optimizations (Rejected)
- Optimize the full scan loop (SIMD, early exit)
- Problem: Still O(n^2) per call, doesn't eliminate redundancy
- Incremental (chosen) is better

### Option 4: Incremental Dirty Tracking (Chosen)
- Maintain cache, invalidate when needed
- Pros: Scales with actual changes, not cell count
- Cons: Requires disciplined dirty marking
- **Selected** - best balance of improvement and complexity

---

## Next Steps

1. **Implement Tier 2.2** using this plan
2. **Run benchmarks** to measure improvement
3. **Re-profile** to verify bottleneck reduction
4. **Plan Tier 2.3** (LCV optimization) based on new profiling data
5. **Reconsider Tier 1.3** - if MRV optimization changes enumeration ratio

---

## Appendix: MRV Heuristic Explanation

**Minimum Remaining Values (MRV)** is a cell selection heuristic for constraint satisfaction:
- At each backtracking step, choose the cell with fewest possible values
- Rationale: More constrained cells lead to faster failure detection
- Impact: Dramatically reduces search tree size (10-100x)

**Why optimization matters**:
- This "good" heuristic is computed frequently
- Current implementation doesn't account for domain stability
- Incremental approach leverages this stability

