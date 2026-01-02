# Difficulty Grading Architecture

This document reconciles different approaches to puzzle difficulty grading and
presents a unified architecture for the rustykeen KenKen solver.

## 1. Problem Statement

Puzzle difficulty grading can be approached in multiple ways:

1. **Search-based metrics**: Backtracking nodes, assignments, search depth
2. **Technique-based grading**: Which deduction techniques are required
3. **Human-model correlation**: Computational models of human solving time

These approaches can conflict:
- A puzzle may require few backtrack nodes but advanced techniques
- A puzzle may be solvable with basic techniques but have deep search
- Human perception correlates better with technique complexity than search cost

## 2. Research Findings

### 2.1 Upstream sgt-puzzles Approach

The upstream "Keen" implementation uses **technique-based difficulty**:
- **Easy**: Coarse cage digit enumeration (which digits can appear in cage)
- **Normal**: Per-cell cage possibilities (refined tuple pruning)
- **Hard**: Cross-cage row/column constraints (must-appear elimination)

The generator rejects puzzles solvable at a tier lower than the target.
This ensures puzzles labeled "Hard" genuinely require hard deductions.

### 2.2 Sudoku Difficulty Research

Academic research (e.g., Pelanek 2014) shows:
- Technique-based grading correlates 0.95 with human difficulty perception
- Pure search cost correlates poorly (0.4-0.6)
- Best predictors combine technique complexity with branching factor

Common technique hierarchy for Sudoku (applicable to Latin squares):
1. Naked single (forced cell)
2. Hidden single (only position for digit in unit)
3. Naked pair/triple (locked candidates)
4. Hidden pair/triple
5. X-Wing, Swordfish (fish patterns)
6. Forcing chains, guess-and-check

### 2.3 Rust Ecosystem Patterns

Relevant crates with technique-based solving:
- `sudoku`: Strategy solver with naked/hidden singles, pairs, fish patterns
- `sudoko`: Explicit `Difficulty` enum, multiple grid sizes
- `hudoku`: Human-style solving with technique tracking

Pattern: Track which techniques fire during solving, classify based on hardest
technique required.

## 3. Reconciliation

### 3.1 Current State

The rustykeen solver has:
- `DeductionTier` enum: None, Easy, Normal, Hard (propagation control)
- `DifficultyTier` enum: Easy, Normal, Hard, Extreme, Unreasonable
- `classify_difficulty(stats)`: Placeholder using assignment count

Gap: No "tier required" classifier that determines minimum tier to solve.

### 3.2 Unified Architecture

We adopt a **two-phase difficulty classification**:

**Phase 1: Tier Required (primary signal)**
- Try solving at each `DeductionTier` in order (Easy -> Normal -> Hard -> None)
- Record the minimum tier that yields a solution
- This matches upstream's technique-based approach

**Phase 2: Search Cost (secondary signal)**
- For puzzles requiring backtracking (tier == None), use search cost
- Maps to Extreme/Unreasonable based on node count thresholds
- Provides finer granularity within the "requires guessing" category

### 3.3 Mapping

```
Tier Required     Search Cost        -> DifficultyTier
---------------------------------------------------------
Easy              (any)              -> Easy
Normal            (any)              -> Normal
Hard              (any)              -> Hard
None              <= 50k nodes       -> Extreme
None              > 50k nodes        -> Unreasonable
```

## 4. Implementation Design

### 4.1 New API

```rust
/// Result of tier-required classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TierRequiredResult {
    /// Minimum deduction tier needed to solve without guessing.
    /// `None` means guessing (backtracking) was required.
    pub tier_required: Option<DeductionTier>,

    /// Search statistics from the successful solve attempt.
    pub stats: SolveStats,
}

/// Determine the minimum deduction tier required to solve the puzzle.
///
/// Tries solving at progressively weaker deduction tiers until success.
/// This is the primary difficulty signal matching upstream behavior.
pub fn classify_tier_required(
    puzzle: &Puzzle,
    rules: Ruleset,
) -> Result<TierRequiredResult, SolveError>
```

### 4.2 Classification Flow

```
Input: Puzzle + Ruleset
                |
                v
    +------------------------+
    | Try solve with Hard    |---> Success? -> Return Hard
    | deductions only        |
    +------------------------+
                |
                v (failed/needed guess)
    +------------------------+
    | Try solve with Normal  |---> Success? -> Return Normal
    | deductions only        |
    +------------------------+
                |
                v (failed/needed guess)
    +------------------------+
    | Try solve with Easy    |---> Success? -> Return Easy
    | deductions only        |
    +------------------------+
                |
                v (failed/needed guess)
    +------------------------+
    | Solve with full        |---> Return None (guessing required)
    | backtracking           |
    +------------------------+
```

Wait - this is backwards. We should try EASY first, then escalate:

```
    +------------------------+
    | Try solve with Easy    |---> Success? -> Return Easy
    | deductions only        |
    +------------------------+
                |
                v (failed)
    +------------------------+
    | Try solve with Normal  |---> Success? -> Return Normal
    | deductions only        |
    +------------------------+
                |
                v (failed)
    +------------------------+
    | Try solve with Hard    |---> Success? -> Return Hard
    | deductions only        |
    +------------------------+
                |
                v (failed)
    +------------------------+
    | Solve with full        |---> Return None
    | backtracking           |
    +------------------------+
```

### 4.3 Deduction-Only Solving

The key insight is that "deduction-only" means no guessing/backtracking.
We need a solver mode that:
1. Applies deductions until fixed point
2. Checks if puzzle is solved
3. Returns success only if fully solved without branching

This requires tracking whether backtracking occurred during solve.

### 4.4 Integration with Existing Code

Current `search_with_stats_deducing` already has deduction tiers, but it
still allows backtracking. We need to either:

**Option A**: Add a "deduction-only" mode that fails instead of branching
**Option B**: Track whether any branching occurred and use that signal

We choose **Option B** as it's less invasive:
- Add `backtracked: bool` to `SolveStats`
- After solving, if `stats.backtracked == false`, the tier was sufficient

### 4.5 Updated SolveStats

```rust
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SolveStats {
    pub nodes_visited: u64,
    pub assignments: u64,
    pub max_depth: u32,
    /// True if any backtracking (undoing an assignment) occurred.
    pub backtracked: bool,
}
```

## 5. Calibration Strategy

### 5.1 Golden Corpus

Create a corpus of puzzles with known tier requirements:
- Source from upstream sgt-puzzles with known difficulty labels
- Include edge cases (borderline Easy/Normal, borderline Normal/Hard)
- Store as JSON with puzzle desc, expected tier, expected solution

### 5.2 Validation

1. Parse each corpus puzzle
2. Run `classify_tier_required`
3. Compare result against expected tier
4. Report mismatches for investigation

### 5.3 Regression Testing

Add corpus validation to CI to detect deduction regressions.

## 6. Future Extensions

### 6.1 Finer Technique Tracking

For puzzles requiring Hard tier, we could track which specific Hard
techniques were needed:
- Cross-cage row elimination
- Cross-cage column elimination
- Cage interaction patterns

### 6.2 Human Time Estimation

Map tier + grid size to estimated solve time:
```
Time(n, tier) = base_time(n) * tier_multiplier(tier)
```

### 6.3 Difficulty Scoring

Continuous difficulty score combining:
- Tier required (discrete)
- Search cost within tier (continuous)
- Grid size (continuous)

## 7. Implementation Plan

1. Add `backtracked: bool` to `SolveStats`
2. Set `backtracked = true` in backtracking code path
3. Implement `classify_tier_required` function
4. Update `classify_difficulty` to use tier-required as primary signal
5. Create calibration corpus with 20+ known-tier puzzles
6. Add corpus validation test
7. Update difficulty.md documentation

## References

- Pelanek, R. (2014). "Difficulty Rating of Sudoku Puzzles"
- Simon Tatham's Puzzles source: `keen.c` difficulty logic
- Sudoku crate documentation: https://docs.rs/sudoku
