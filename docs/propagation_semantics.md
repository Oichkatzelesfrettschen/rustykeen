# Propagation and Deduction Semantics

Last updated: 2026-01-01

This document formalizes the constraint propagation and deduction mechanisms used by the kenken-solver.

## Overview

The solver uses a combination of:
1. **Forward checking**: Fail fast when a cell has no valid digits
2. **Domain pruning**: Restrict cell domains based on cage constraints
3. **Arc consistency**: Propagate constraints until fixpoint

## Deduction Tiers

The solver supports four deduction strengths:

| Tier | Name | Description |
|------|------|-------------|
| `None` | Backtracking | No deductions; pure backtracking search |
| `Easy` | Coarse enumeration | Which digits CAN appear in a cage (any position) |
| `Normal` | Per-cell pruning | Which digits can appear at each position |
| `Hard` | Cross-cage elimination | Must-appear elimination across rows/columns |

### Tier Selection

Higher tiers add overhead but may reduce search tree size:
- `None`: ~585ns/solve (2x2), baseline
- `Easy`: ~1.42us/solve, +143% overhead
- `Normal`: ~1.15us/solve, +97% overhead (paradoxically faster due to better pruning)
- `Hard`: ~1.75us/solve, +199% overhead

## Core Functions

### `propagate()`

```
propagate(puzzle, rules, tier, state, forced) -> bool
```

Main propagation loop that:
1. Computes initial domains from Latin constraints
2. Applies cage deductions at the specified tier
3. Forces cells with singleton domains
4. Repeats until fixpoint or contradiction

**Returns**: `true` if consistent, `false` if contradiction detected

### `cage_feasible()`

```
cage_feasible(puzzle, rules, state, cage) -> bool
```

Quick bounds check using partial assignments. Tests whether a cage
CAN be satisfied given current assignments and remaining domains.

### `apply_cage_deduction()`

```
apply_cage_deduction(puzzle, rules, state, cage, tier, domains)
```

Per-cage domain restriction. Behavior varies by cage operation and tier.

## Cage-Specific Bounds

### Eq (Equality) Cages

```
domain[cell] &= (1 << target)
```

Single-cell cages are immediately resolved to their target value.

### Add Cages

**Feasibility bounds**:
```
sum_assigned + min_remaining <= target <= sum_assigned + max_remaining
```

Where:
- `sum_assigned` = sum of already-placed digits
- `min_remaining` = sum of minimum values in remaining cell domains
- `max_remaining` = sum of maximum values in remaining cell domains

**Easy tier**: `domain[cell] &= any_mask` (digits appearing in ANY valid tuple)

**Normal tier**: `domain[cell] &= per_pos[i]` (digits appearing at THIS position)

**Hard tier**: Additionally computes must-appear masks:
```
for each valid tuple:
    must_row[r] &= (bits appearing in row r)
    must_col[c] &= (bits appearing in col c)

for cells NOT in cage:
    domain[cell] &= ~must_row[row_of_cell]
    domain[cell] &= ~must_col[col_of_cell]
```

### Mul Cages

**Feasibility bounds**:
```
prod_assigned * min_remaining <= target <= prod_assigned * max_remaining
AND target % prod_assigned == 0
```

Deduction follows same pattern as Add cages.

### Sub Cages (two-cell only)

```
|a - b| = target
```

Valid pairs: `(a, b)` where `|a - b| = target` and both are in respective domains.

**Feasibility**: Check existence of valid pair.
**Deduction**: `domain[a] &= a_ok`, `domain[b] &= b_ok`

### Div Cages (two-cell only)

```
max(a, b) / min(a, b) = target
```

Valid pairs: `(a, b)` where `max/min = target` and `min != 0`.

**Feasibility/Deduction**: Same pattern as Sub cages.

## Domain Representation

Domains use a `u32` bitmask where bit `d` represents digit `d` (1-indexed):

```rust
fn full_domain(n: u8) -> u32 {
    ((1u32 << (n + 1)) - 1) & !1u32  // bits 1..=n
}

// Domain for cell at (row, col)
let dom = full_domain(n) & !row_mask[row] & !col_mask[col];
```

Key operations:
- `popcount_u32(dom)`: Number of possible values
- `dom.trailing_zeros()`: Minimum value in domain
- `31 - dom.leading_zeros()`: Maximum value in domain
- `dom & (1 << d)`: Check if digit d is in domain

## Latin Square Constraints

Maintained via `row_mask` and `col_mask` arrays:

```rust
fn place(state, row, col, d) {
    state.grid[row * n + col] = d;
    state.row_mask[row] |= 1 << d;
    state.col_mask[col] |= 1 << d;
}

fn unplace(state, row, col, d) {
    state.grid[row * n + col] = 0;
    state.row_mask[row] &= !(1 << d);
    state.col_mask[col] &= !(1 << d);
}
```

## Tuple Enumeration

For Add/Mul cages with partial assignments, the solver enumerates valid tuples:

```
enumerate_cage_tuples(cage, cells, coords, domains, depth, current, per_pos, any_mask)
```

This recursive function:
1. Generates all valid digit assignments to unassigned cells
2. Checks if the tuple satisfies the cage constraint
3. Accumulates valid digits per position (`per_pos`) and globally (`any_mask`)

Complexity: O(n^k) where k = number of unassigned cells. Large cages with many
unassigned cells can explode; the SAT backend provides a fallback for such cases.

## Fixpoint Semantics

Propagation terminates when:
1. **Contradiction**: Any cell has empty domain (`dom == 0`)
2. **Fixpoint**: No singleton domains remain to force

The loop invariant is:
- All forced cells have been placed
- All cage deductions have been applied
- Either contradiction or stable state reached

## Performance Considerations

1. **Early termination**: Check for contradiction after each cage deduction
2. **SIMD popcount**: Use hardware popcount via `kenken-simd` feature
3. **Arena allocation**: Optional bumpalo arena reduces allocation pressure
4. **Bitwise operations**: All domain ops are O(1) bitwise operations

## References

- Solver implementation: `kenken-solver/src/solver.rs`
- Kani proofs for Latin constraints: `solver.rs:kani_verification`
- Benchmark baselines: `docs/benchmark_baselines.md`
