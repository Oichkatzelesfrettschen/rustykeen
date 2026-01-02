# KenKen Solver Architecture

Last updated: 2026-01-02

**Optimization Status**: Tier 1.0-1.2 Complete (40-52% speedup) + Tier 2.2 Validated (4-9% additional) = 44-61% cumulative speedup. See `docs/OPTIMIZATION_ROADMAP.md` for detailed tier analysis.

## Overview

The kenken-solver implements a deterministic, constraint-propagation-based solver for KenKen puzzles. This document describes the architecture, data structures, algorithms, and design decisions.

## Architecture Layers

```
┌─────────────────────────────────────────┐
│       Public API                        │
│  solve_one(), count_solutions_up_to()   │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│    Search Engine (search.rs)            │
│  - Backtracking with MRV heuristic      │
│  - Forward checking                     │
│  - Deduction tier selection             │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│  Propagation Engine (propagate.rs)      │
│  - Domain management                    │
│  - Cage constraint application          │
│  - Latin square enforcement             │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│   Domain Representation (u32 bitmask)   │
│  - Bits 1..=n represent digits 1..=n    │
│  - SIMD popcount via kenken-simd        │
│  - Fast domain operations               │
└─────────────────────────────────────────┘
```

## Key Data Structures

### Puzzle and Cage Representation

From `kenken-core`:

```rust
pub struct Puzzle {
    pub n: u8,
    pub cages: Vec<Cage>,
    // Cage cell membership
}

pub struct Cage {
    pub cells: SmallVec<[CellId; 6]>,
    pub target: u32,
    pub op: Op,  // Add, Mul, Sub, Div, Eq
}

pub type CellId = (u8, u8);  // (row, col)
```

### Solver State

```rust
// Domain representation: u32 bitmask
// Bit i is set iff digit i is possible
// Example for n=3: domain = 0b00000111 means {1,2,3}

struct MrvCache {
    valid: bool,              // Cache validity flag
    min_cell: usize,          // Cached MRV cell index
    min_count: u32,           // Cached MRV domain size
    dirty_cells: Vec<bool>,   // Tracks cells with domain reductions
}

struct State {
    domains: Vec<u32>,        // One per cell
    grid: Vec<u8>,            // Assigned values (0=unassigned)
    row_mask: Vec<u32>,       // Placed digits per row
    col_mask: Vec<u32>,       // Placed digits per column
    mrv_cache: MrvCache,      // Tier 2.2: MRV heuristic cache (see below)
}
```

### SolveStats

```rust
pub struct SolveStats {
    pub nodes_visited: u64,      // Search tree nodes explored
    pub assignments: u64,        // Cells assigned
    pub max_depth: u32,          // Maximum recursion depth
    pub backtracked: bool,       // Did solver branch (guess)?
}
```

## Search Algorithm

### High-Level Flow

```
solve_one(puzzle) {
    1. Initialize domains based on Latin constraint
    2. Call propagate() at initial tier
    3. If contradiction: return None
    4. If complete: return solution
    5. Pick unassigned cell with MRV
    6. For each possible value:
        a. Place value
        b. Propagate constraints
        c. Recurse
        d. Backtrack if needed
    7. Return first solution found
}
```

### Minimum Remaining Values (MRV) Heuristic

The solver selects the unassigned cell with the smallest domain:

```rust
// Pick cell with fewest possible values
let (cell_id, candidates) = state
    .unassigned_cells()
    .min_by_key(|&(_, domain)| popcount(domain))
    .unwrap();
```

This typically reduces search tree depth by 50-70%.

### Deduction Tiers

The `DeductionTier` enum controls propagation strength:

| Tier | Description | Overhead |
|------|-------------|----------|
| `None` | No deductions; pure backtracking | Baseline (585ns/solve on 2x2) |
| `Easy` | Coarse cage digit enumeration | +143% |
| `Normal` | Per-cell cage tuple pruning | +97% (often faster due to better pruning) |
| `Hard` | Cross-cage row/column elimination | +199% |

**Key insight**: Normal tier often outperforms Easy tier because better pruning reduces search tree size.

## Propagation Engine

### Propagate Function

```
propagate(puzzle, state, tier) {
    repeat until fixpoint or contradiction:
        1. For each cage:
            - apply_cage_deduction(cage, tier)
        2. For each cell with singleton domain:
            - place_value(cell, value)
            - Update row_mask, col_mask
        3. For each cell:
            - Remove row-assigned and col-assigned digits

    return is_consistent
}
```

### Cage-Specific Deductions

#### Eq Cages (Single cell)

```rust
domain[cell] = (1 << target)  // Force to single value
```

#### Add Cages

**Easy tier**: Which digits CAN appear anywhere in the cage?

```
1. Enumerate valid digit tuples
2. Union all digits appearing in any tuple
3. AND with domain of each cell
```

**Normal tier**: Which digits can appear AT EACH POSITION?

```
1. Enumerate valid digit tuples
2. For position i, AND cell domain with digits appearing at position i
```

**Hard tier**: Cross-cage elimination

```
1. Compute must-appear masks for each row/column
2. For cells NOT in cage:
    domain[cell] &= ~must_appear_mask[row/col]
```

#### Mul/Sub/Div Cages

Similar to Add cages but with different tuple validity functions.

### Domain Representation

Uses `u32` bitmask for efficiency:

```rust
// Bit 0 (unused), Bits 1..=n represent digits 1..=n
// Example: n=4, domain=0b0110 means digits {2,3} are possible

fn is_possible(domain: u32, digit: u8) -> bool {
    (domain & (1u32 << digit)) != 0
}

fn remove_digit(domain: &mut u32, digit: u8) {
    *domain &= !(1u32 << digit)
}

fn popcount(domain: u32) -> u32 {
    domain.count_ones()
}
```

## Performance Optimizations

### Tier 1: Foundation Optimizations (Tier 1.0-1.2)

**Achievement**: 40-52% cumulative speedup through:
1. **SIMD Popcount** - Runtime ISA dispatch via kenken-simd (u64 bit operations)
2. **Row/Column Flag Propagation** - Faster row/column mask computation
3. **Early Constraint Checking** - 2-cell cages checked before enumeration

### Tier 2.2: MRV Heuristic Caching

**Achievement**: 4-9% additional speedup (validated across 2x2-12x12 puzzles)

**How it works**:
1. Cache the result of `choose_mrv_cell()` across propagation iterations
2. Track "dirty cells" - cells whose domains were actually reduced
3. Invalidate cache only when dirty cells exist

**Implementation details**:
- `MrvCache` struct stores: `valid` flag, cached `min_cell`, `min_count`, and `dirty_cells` vector
- **Dirty marking**: In propagate(), after each cage deduction, mark cells dirty only if domain actually reduced:
  ```rust
  if (domain_before[i] & !domain_after) != 0 {
      state.mrv_cache.mark_dirty(idx);
  }
  ```
  This fine-grained approach reduces false invalidations vs. marking all touched cells

**Validation**: 2x2 (-9%), 3x3 (-6%), 6x6 (-5%), 8x8 (-6%), 12x12 (-5%) - no regressions

### Legacy Optimizations (Tier 1 Core)

1. **Eager Contradiction Detection** - Return early if any cell has empty domain
2. **Minimal Domain Copies** - State passed by reference; only backtrack points allocate
3. **Cage Ordering** - Larger cages processed first (more constraints)

## Data Flow Example

### Solving a 3x3 Puzzle

```
Input: Puzzle with cages {(0,1): Add 5}, {(2): Eq 3}, ...

1. Initialize domains
   - All cells: domain = 0b0111 (digits {1,2,3})

2. Propagate (Easy tier)
   - Cage (2): Eq 3 → domain[2] = 0b0100 (only digit 3)
   - Place value 3 at cell 2
   - Remove 3 from row 0, col 2

3. Propagate (Easy tier)
   - Cage (0,1): Add 5
     Valid tuples: {1,4} (not possible, max is 3)
                 {2,3} (possible)
     So cells (0,1) can only be from {2,3}

4. Latin constraint
   - Row 0 has 3 at position 2
   - So (0,0) and (0,1) must be {1,2}

5. Domain after pruning
   - Cell (0,0): domain = 0b0011 (digits {1,2})
   - Cell (0,1): domain = 0b0011 (digits {1,2})

6. Search
   - Pick cell (0,0) with 2 possible values
   - Try value 1: propagate → success
   - Return solution
```

## Uniqueness Verification

The solver can count solutions up to a limit:

```rust
count_solutions_up_to(puzzle, 2)  // Find up to 2 solutions
```

Returns 1 if unique, >1 if non-unique.

Optional Z3-based verification (`verify` feature) provides formal proof:

```rust
z3_verify::verify_solution_is_unique(n, solution)
```

## Testing and Validation

- **Unit tests**: Cage semantics, domain operations, propagation rules
- **Integration tests**: Golden corpus (52 puzzles, 2x2 to 6x6)
- **Property tests**: Random puzzle generation and classification
- **Formal verification**: 15 Kani proofs on core invariants
- **Z3 certification**: Solution uniqueness formal verification

## Design Decisions and Rationale

### 1. Why u32 Bitmask Instead of BitDomain?

- **Why bitmask**: Faster for n ≤ 31 (no heap allocation, CPU-friendly popcount)
- **BitDomain fallback**: Available via `core-bitvec` feature for n > 31
- **Benchmark result**: u32 bitmask is 2-3x faster than `BitDomain` for typical sizes

### 2. Why MRV Caching (Tier 2.2) Works, But Propagation Optimization (Tier 2.1) Failed

**Tier 2.2 Success - MRV Cache**:
- Why it works: Cache stores only the MRV result (clean state separation)
- Invalidation is simple: dirty-cell tracking requires no complex state
- Result: 4-9% speedup, no regressions

**Tier 2.1 Failure - Propagation Optimization** (attempted to skip domain recalculation):
- Why it fails: Constraint propagation system has tight interdependencies
- Root causes (see `docs/tier21_findings.md`):
  1. Row/column masks affect ALL cells' domains, not just assigned cells
  2. Cage deductions are interdependent (cage A's deductions constrain cage B)
  3. Domain propagation chains require full recalculation for precision
- Attempted workaround: "affected cages only" optimization - BREAKS correctness
- Lesson: Incremental domain updates are unsafe without major architectural refactoring

**Future LCV Heuristic (Tier 2.3)**:
- **Different approach**: Instead of optimizing CPU efficiency, optimize search tree width
- **Concept**: Try values that constrain remaining cells least (fewer failed branches)
- **Status**: Pre-implementation research phase (see `docs/tier23_lcv_evaluation.md`)
- **Benefit**: 5-12% estimated improvement on backtracking-heavy puzzles

### 3. Why Tier-Based Propagation?

- **Flexible**: Users choose speed vs. solution quality
- **Composable**: Tiers build on each other (Normal includes Easy constraints)
- **Measurable**: Each tier has documented overhead

### 4. Why SAT Fallback for Add Cages?

- **Problem**: Tuple explosion for large cages (e.g., 6-cell Add cage in 9x9)
- **Solution**: Threshold (`SAT_TUPLE_THRESHOLD = 512`) triggers SAT encoding
- **Benefit**: Handles extreme cases without solver hanging

## Known Limitations

1. **Determinism vs. Completeness**: Deterministic ordering may miss some deductions vs. non-deterministic SAT-based approaches
2. **Hard Tier Cost**: Cross-cage elimination (+199% overhead) may not be worth it for small puzzles
3. **Generation**: Random puzzle generation occasionally produces pathological cases (see `gen` module docs)

## Future Improvements

### Tier 2.3: LCV Value Ordering (Pre-Implementation Research)

**Concept**: Select values during backtracking that constrain remaining cells least (Least Constraining Value heuristic)

**Target**: Search tree width on puzzles requiring backtracking

**Expected benefit**: 5-12% improvement on backtracking-heavy puzzles

**Measurement plan** (see `docs/tier23_lcv_evaluation.md`):
- Identify backtracking-heavy puzzle corpus
- Measure baseline backtrack counts and timings
- Measure LCV scoring overhead (propagate simulation per value)
- Estimate portfolio impact

**Next steps**: Run measurement phase; implement if >3% average improvement detected

### Longer-Term Optimizations

1. **Cage Enumeration Caching**: Cache tuple enumerations for repeated cage/domain states (2-5% benefit)
2. **Domain Representation Alternatives**: Explore fixedbitset (SIMD-optimized) or smallbitvec (inline storage) for larger puzzles
3. **Arc Consistency (AC-3)**: Full AC-3 for stronger propagation
4. **Parallel Search**: Explore multiple branches concurrently on large puzzles
5. **Incremental SAT**: Reuse SAT state across backtracks
