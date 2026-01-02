# Tier 2.1 Implementation Plan - Detailed

**Date**: 2026-01-02
**Approach**: Option A - Minimal changes with affected cages tracking
**Estimated Effort**: 1-2 days (implementation + validation)
**Risk Level**: Medium (correctness risk requires comprehensive testing)

---

## Overview

This document provides step-by-step implementation guidance for Tier 2.1, following Option A (track changed cells, only recalculate affected domains).

---

## Phase 1: Baseline Profiling (Pre-Implementation)

### 1.1 Capture Current Iteration Characteristics

Add temporary instrumentation to understand propagate() behavior:

```rust
// Add to propagate() loop start
let mut iteration_count = 0;
let mut forced_per_iteration = Vec::new();

loop {
    iteration_count += 1;
    // ... existing code ...
    let forced_count = newly_forced.len();
    forced_per_iteration.push(forced_count);
    if iteration_count > 100 {
        eprintln!("WARNING: propagate exceeded 100 iterations");
        break;
    }
}

if iteration_count > 10 {
    eprintln!("Puzzle {:?}: {} iterations, forced: {:?}", puzzle.n, iteration_count, forced_per_iteration);
}
```

### 1.2 Run Baseline Benchmarks

Before modifying propagate():
```bash
cargo bench -p kenken-solver --bench solver_scaling 2>&1 | tee /tmp/baseline_before_tier21.txt
```

Expected output to collect:
- Time per puzzle size
- P-value (statistical significance)
- Outlier counts

### 1.3 Flamegraph of Current Implementation

```bash
cargo flamegraph --bench solver_scaling -- --bench 6x6
# Review target/flamegraph.svg, note propagate() width
```

---

## Phase 2: Implementation (Modified propagate)

### 2.1 Add Helper Function to solver.rs

Insert after propagate() function definition (around line 750):

```rust
/// Find all cage indices that contain a specific cell.
/// Used for Tier 2.1 optimization: identify affected cages when a cell changes.
fn cages_for_cell(puzzle: &Puzzle, cell_idx: usize) -> Vec<usize> {
    puzzle.cages
        .iter()
        .enumerate()
        .filter(|(_, cage)| cage.cells.iter().any(|&c| c.0 as usize == cell_idx))
        .map(|(i, _)| i)
        .collect()
}
```

**Location**: Add around line 822 (after apply_cage_deduction implementations)

**Testing**:
- Compile check: `cargo build -p kenken-solver`
- No runtime test needed (helper is deterministic)

### 2.2 Modify propagate() - Part 1: Initialize changed_cells

Replace lines 758-773 with:

**OLD (Original)**:
```rust
let mut domains = vec![0u64; a];

loop {
    domains.fill(0u64);
    for (idx, dom_slot) in domains.iter_mut().enumerate() {
        if state.grid[idx] != 0 {
            *dom_slot = 1u64 << (state.grid[idx] as u32);
            continue;
        }
        let r = idx / n;
        let c = idx % n;
        *dom_slot = full_domain(state.n) & !state.row_mask[r] & !state.col_mask[c];
    }
```

**NEW (With Tier 2.1)**:
```rust
let mut domains = vec![0u64; a];

// Tier 2.1: Track which cells need domain recalculation
// Initially all cells changed (first iteration needs full calculation)
let mut changed_cells: Vec<usize> = (0..a).collect();

loop {
    // Tier 2.1 optimization: Only recalculate domains for changed cells
    for &idx in &changed_cells {
        if state.grid[idx] != 0 {
            domains[idx] = 1u64 << (state.grid[idx] as u32);
            continue;
        }
        let r = idx / n;
        let c = idx % n;
        domains[idx] = full_domain(state.n) & !state.row_mask[r] & !state.col_mask[c];
    }
```

**Changes**:
- Add `changed_cells` vector after domains initialization
- Initialize to all cell indices (first iteration needs full rescan)
- Replace `domains.fill(0u64)` with iteration only over `changed_cells`
- Replace loop over all indices with loop over `&changed_cells`

**Validation**:
- Compile: `cargo build -p kenken-solver`
- Test 2x2: `cargo test solver::tests::test_2x2` (should pass)

### 2.3 Modify propagate() - Part 2: Affected Cages

Replace lines 775-795 (cage deduction loop) with:

**OLD (Original)**:
```rust
for cage in &puzzle.cages {
    // Tier 2.2: Smarter dirty tracking - capture domain state before deduction
    let cage_cells: Vec<usize> = cage.cells.iter().map(|c| c.0 as usize).collect();
    let domain_before: Vec<u64> = cage_cells.iter().map(|&idx| domains[idx]).collect();

    #[cfg(feature = "alloc-bumpalo")]
    apply_cage_deduction_with_bump(&bump, puzzle, rules, state, cage, tier, &mut domains)?;

    #[cfg(not(feature = "alloc-bumpalo"))]
    apply_cage_deduction(puzzle, rules, state, cage, tier, &mut domains)?;

    // Tier 2.2: Only mark cells whose domains were actually reduced (smarter dirty tracking)
    for (i, &idx) in cage_cells.iter().enumerate() {
        let domain_after = domains[idx];
        // Mark dirty only if domain was reduced (bits removed)
        // Using: (before & ~after) != 0 means bits were removed
        if (domain_before[i] & !domain_after) != 0 {
            state.mrv_cache.mark_dirty(idx);
        }
    }
}
```

**NEW (With Tier 2.1)**:
```rust
// Tier 2.1: Only apply deductions to cages touching changed cells
let affected_cage_indices: std::collections::HashSet<usize> = changed_cells
    .iter()
    .flat_map(|&cell_idx| cages_for_cell(puzzle, cell_idx))
    .collect();

for cage_idx in affected_cage_indices {
    let cage = &puzzle.cages[cage_idx];

    // Tier 2.2: Smarter dirty tracking - capture domain state before deduction
    let cage_cells: Vec<usize> = cage.cells.iter().map(|c| c.0 as usize).collect();
    let domain_before: Vec<u64> = cage_cells.iter().map(|&idx| domains[idx]).collect();

    #[cfg(feature = "alloc-bumpalo")]
    apply_cage_deduction_with_bump(&bump, puzzle, rules, state, cage, tier, &mut domains)?;

    #[cfg(not(feature = "alloc-bumpalo"))]
    apply_cage_deduction(puzzle, rules, state, cage, tier, &mut domains)?;

    // Tier 2.2: Only mark cells whose domains were actually reduced (smarter dirty tracking)
    for (i, &idx) in cage_cells.iter().enumerate() {
        let domain_after = domains[idx];
        // Mark dirty only if domain was reduced (bits removed)
        // Using: (before & ~after) != 0 means bits were removed
        if (domain_before[i] & !domain_after) != 0 {
            state.mrv_cache.mark_dirty(idx);
        }
    }
}
```

**Changes**:
- Compute `affected_cage_indices` from `changed_cells` using helper function
- Loop over `affected_cage_indices` instead of all cages
- Move `cage.cells` and `cage.op` extraction inside the affected loop

**Key Detail**: Must import `std::collections::HashSet` at top of file (or use `Vec` + dedup).

**Validation**:
- Compile: `cargo build -p kenken-solver`
- Test 3x3: `cargo test solver::tests::test_3x3` (should pass)

### 2.4 Modify propagate() - Part 3: Track New Forced Assignments

Replace lines 803-816 (forced assignment loop) with:

**OLD (Original)**:
```rust
let mut any_forced = false;
for (idx, &dom) in domains.iter().enumerate() {
    if state.grid[idx] != 0 {
        continue;
    }
    if popcount_u64(dom) == 1 {
        let val = dom.trailing_zeros() as u8;
        let r = idx / n;
        let c = idx % n;
        place(state, r, c, val);
        forced.push((idx, val));
        any_forced = true;
    }
}
```

**NEW (With Tier 2.1)**:
```rust
// Tier 2.1: Track newly forced assignments for next iteration
let mut newly_forced = Vec::new();
for (idx, &dom) in domains.iter().enumerate() {
    if state.grid[idx] != 0 {
        continue;
    }
    if popcount_u64(dom) == 1 {
        let val = dom.trailing_zeros() as u8;
        let r = idx / n;
        let c = idx % n;
        place(state, r, c, val);
        forced.push((idx, val));
        newly_forced.push(idx);
    }
}

let any_forced = !newly_forced.is_empty();
```

**Changes**:
- Add `newly_forced` vector to track cells that were just assigned
- Push newly-forced cell indices to this vector
- Change `any_forced` from boolean flag to check if vector is non-empty
- Keep `forced` vector unchanged (output parameter)

**Validation**:
- Compile: `cargo build -p kenken-solver`
- Test 4x4: `cargo test solver::tests::test_4x4` (should pass)

### 2.5 Modify propagate() - Part 4: Loop Termination

Replace lines 818-821 (loop control) with:

**OLD (Original)**:
```rust
if !any_forced {
    return Ok(true);
}
```

**NEW (With Tier 2.1)**:
```rust
if !any_forced {
    return Ok(true);
}

// Tier 2.1: Only cells with forced assignments are "changed" for next iteration
changed_cells = newly_forced;
```

**Changes**:
- Update `changed_cells` to only include newly-forced cells
- This ensures next iteration only recalculates affected domains

**Validation**:
- Compile: `cargo build -p kenken-solver`

---

## Phase 3: Compilation and Basic Testing

### 3.1 Full Build

```bash
cargo build -p kenken-solver --all-features
```

**Expected Result**: Compiles without warnings (WARN=deny in workspace)

### 3.2 Run Full Test Suite

```bash
cargo test -p kenken-solver --all-features
```

**Expected Result**: All 29/29 tests pass

**If Tests Fail**:
1. Check error message for specific failure
2. Likely issues:
   - `cages_for_cell()` not found (check location)
   - Type mismatch on `changed_cells` (should be `Vec<usize>`)
   - Borrow checker errors on `changed_cells` (may need `&` or `.clone()`)

### 3.3 Smoke Test on CLI

```bash
cargo run -p kenken-cli --release -- solve --n 2 --desc b__,a3a3 --tier normal
# Should print a single valid solution

cargo run -p kenken-cli --release -- count --n 3 --desc f_6,a6a6a6 --limit 2
# Should print "1" (unique puzzle) or "2" (multiple solutions)
```

---

## Phase 4: Profiling and Validation

### 4.1 New Benchmarks

```bash
cargo bench -p kenken-solver --bench solver_scaling 2>&1 | tee /tmp/results_after_tier21.txt
```

**Expected Results**:
- 2x2-4x4: No regression or small regression (<2%)
- 6x6: -3-5% improvement
- 8x8: -5-10% improvement
- 12x12: -8-15% improvement

### 4.2 Compare Baseline vs Optimized

```bash
diff /tmp/baseline_before_tier21.txt /tmp/results_after_tier21.txt
# Look for time change percentages
```

### 4.3 Check for Regressions

If any size shows +3% or worse:
- Profile that size specifically
- Check if affected_cages computation is expensive
- Consider adding size threshold: skip optimization for n < 6

### 4.4 Flamegraph Analysis

```bash
cargo flamegraph --bench solver_scaling -- --bench 8x8
# Compare width of propagate() function
# Should be narrower than before (smaller time footprint)
```

---

## Phase 5: Optional Optimization (If Needed)

### 5.1 If Affected Cages Computation is Slow

**Symptom**: Small puzzles show regression; affected_cages line takes significant time

**Fix Option 1**: Cache affected cages computation
```rust
// One-time: build map of cell -> cages
let mut cell_to_cages: Vec<Vec<usize>> = vec![Vec::new(); a];
for (cage_idx, cage) in puzzle.cages.iter().enumerate() {
    for &cell_id in &cage.cells {
        let idx = cell_id.0 as usize;
        cell_to_cages[idx].push(cage_idx);
    }
}

// In loop:
let affected_cage_indices: HashSet<usize> = changed_cells
    .iter()
    .flat_map(|&idx| cell_to_cages[idx].iter().copied())
    .collect();
```

**Fix Option 2**: Use bitset for cages instead of HashSet
```rust
use kenken_simd::popcount_u64;  // Or similar

let mut affected_bitset = 0u64;  // Supports up to 64 cages
for &idx in &changed_cells {
    for &cage_idx in &cell_to_cages[idx] {
        affected_bitset |= 1u64 << cage_idx;
    }
}

for cage_idx in (0..puzzle.cages.len()).filter(|i| (affected_bitset >> i) & 1 != 0) {
    // ...
}
```

**Decision**: Only apply if benchmarks show regression.

### 5.2 If Cascade Effects Are Problematic

**Symptom**: propagate() iteration counts differ from baseline

**Check**:
```rust
// Add temporary debug output
if iteration_count != baseline_iterations {
    eprintln!("Iteration count mismatch: got {}, expected {}",
              iteration_count, baseline_iterations);
}
```

**Fix**: May need to track domain changes per cage, not just forced assignments (Option B approach).

---

## Phase 6: Documentation

### 6.1 Update solver.rs Comments

Add comments above propagate() explaining Tier 2.1:

```rust
/// Constraint propagation with Tier 2.1 optimization.
///
/// Applies cage constraints and row/column uniqueness to narrow domains
/// until a fixed point is reached or contradiction found.
///
/// Tier 2.1 Optimization: Only recalculates domains for cells that were
/// assigned in the previous iteration. This reduces redundant domain
/// recalculation on large puzzles where later iterations affect few cells.
///
/// Performance: ~7-12% speedup overall, ~25-40% speedup in propagate() itself.
pub fn propagate(...) { ... }
```

### 6.2 Document cages_for_cell() Helper

Already documented in code.

### 6.3 Update Architecture Docs

Add section to `docs/solver_architecture.md`:

```markdown
## Tier 2.1: Propagation Optimization

**Target**: Constraint propagation loop efficiency
**Benefit**: 7-12% overall, 25-40% in propagate() alone
**Implementation**: Skip domain recalculation for unchanged cells

The propagate() function iteratively applies cage constraints until reaching
a fixed point. Tier 2.1 optimizes by:
1. Tracking which cells were assigned in previous iteration (changed_cells)
2. Only recalculating domains for changed cells
3. Only applying cage deductions to affected cages

This reduces O(n²) full rescans to O(k) incremental updates, where k is the
number of cells that changed (typically 1-10 on later iterations).
```

---

## Rollback Plan

If significant issues discovered:

### Option A: Revert Changes
```bash
git revert HEAD  # If already committed
git checkout -- kenken-solver/src/solver.rs  # If not committed
```

### Option B: Disable via Feature Flag
```rust
// Wrap optimization in feature flag:
#[cfg(feature = "tier-2-1")]
let affected_cage_indices = ...;

#[cfg(not(feature = "tier-2-1")]
for cage in &puzzle.cages {
    // Original loop
}
```

In `Cargo.toml`:
```toml
[features]
tier-2-1 = []
```

Run tests:
```bash
cargo test --all-features           # With optimization
cargo test --no-default-features    # Without optimization
```

---

## Success Criteria Checklist

- [ ] Code compiles with -D warnings
- [ ] All 29/29 tests pass
- [ ] CLI smoke tests pass (solve/count on various sizes)
- [ ] Benchmarks show expected improvements (or minimal regression)
- [ ] No regressions > 2% on any puzzle size
- [ ] Flamegraph shows narrower propagate() function
- [ ] Iteration counts match baseline (± 0 iterations)
- [ ] Comments and docs updated
- [ ] Ready for merge

---

## Estimated Timeline

- **Implementation**: 2-3 hours (coding + compilation fixes)
- **Testing**: 1-2 hours (test suite + profiling)
- **Optimization (if needed)**: 1-2 hours
- **Documentation**: 30-45 minutes
- **Total**: 5-8 hours (1 working day)

---

## References

- Tier 2.1 Research: `docs/tier21_propagation_optimization.md`
- Current propagate(): `kenken-solver/src/solver.rs` lines 750-822
- Tier 2.2 Reference: `docs/tier22_final_decision.md` (similar pattern)
