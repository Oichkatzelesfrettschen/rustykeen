# Difficulty Classification

This document describes the difficulty grading system for KenKen puzzles.

## Overview

Difficulty is determined by **which deduction tier is required** to solve the
puzzle without guessing. This matches the upstream sgt-puzzles approach where
difficulty reflects the complexity of reasoning needed, not raw search cost.

## Deduction Tiers

The solver supports four deduction levels (controlled by `DeductionTier`):

- **None**: No deductions; pure backtracking search
- **Easy**: Coarse cage digit enumeration (which digits can appear in a cage)
- **Normal**: Per-cell cage possibilities (refined tuple pruning)
- **Hard**: Cross-cage row/column constraints (must-appear elimination)

## Difficulty Tiers

Puzzles are classified into five difficulty levels (`DifficultyTier`):

| Tier         | Meaning                                        |
|--------------|------------------------------------------------|
| Easy         | Solvable with Easy deductions only             |
| Normal       | Requires Normal deductions                     |
| Hard         | Requires Hard deductions                       |
| Extreme      | Requires guessing, moderate search cost        |
| Unreasonable | Requires guessing, high search cost            |

## Classification API

### Primary: Tier-Required Classification

```rust
use kenken_solver::{classify_tier_required, classify_difficulty_from_tier};

let result = classify_tier_required(&puzzle, rules)?;
let difficulty = classify_difficulty_from_tier(result);

// result.tier_required: Some(DeductionTier) or None if guessing required
// result.stats: SolveStats with backtracked flag
```

### Legacy: Stats-Only Classification

```rust
use kenken_solver::{solve_one_with_stats, classify_difficulty};

let (solution, stats) = solve_one_with_stats(&puzzle, rules)?;
let difficulty = classify_difficulty(stats);  // Based on assignment count
```

The legacy API is deprecated but retained for backwards compatibility.

## Implementation Details

### Backtracking Detection

The `SolveStats` struct includes a `backtracked: bool` field that indicates
whether the solver tried multiple values at any cell during the search.

When `backtracked == false`, the puzzle was solvable using only deductions
at the given tier without any guessing.

### Classification Algorithm

1. Attempt solve with Easy deductions
2. If solved without backtracking -> Easy
3. Attempt solve with Normal deductions
4. If solved without backtracking -> Normal
5. Attempt solve with Hard deductions
6. If solved without backtracking -> Hard
7. Otherwise, classify as Extreme or Unreasonable based on node count

### Threshold for Extreme vs Unreasonable

Puzzles requiring backtracking are classified as:
- **Extreme**: <= 50,000 nodes visited
- **Unreasonable**: > 50,000 nodes visited

These thresholds are provisional and may be adjusted based on calibration.

## Calibration Corpus

The `kenken-solver/tests/corpus_difficulty.rs` test validates difficulty
classification against a corpus of puzzles with known tier requirements.

See `docs/difficulty_grading_design.md` for the full architectural design.

## References

- Simon Tatham's Puzzles "Keen" difficulty logic
- Pelanek, R. (2014). "Difficulty Rating of Sudoku Puzzles"
