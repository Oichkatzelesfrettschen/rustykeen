# Difficulty (provisional)

Upstream sgt-puzzles “Keen” defines difficulty tiers in terms of **which deduction routines** are required to solve the puzzle (Easy/Normal/Hard), not in terms of raw backtracking search cost.

This repo currently has a baseline deterministic backtracking solver, so any difficulty classification is **provisional** until we implement an upstream-style deduction-tiered solver.

## Current state
- `kenken-solver` records simple solve statistics (`SolveStats`) and provides `classify_difficulty(stats)` as a placeholder rubric.

## Plan
- Implement upstream-style cage deduction tiers and use “which deductions were needed” as the primary difficulty signal.
- Calibrate with a golden corpus of upstream “desc” puzzles and expected tier classifications.

## Status (implementation)
- `kenken-solver` now has a `DeductionTier` and a `solve_one_with_deductions(...)` entrypoint.
- Implemented (v0): `Easy` and `Normal` cage deductions that prune per-cell candidates by enumerating cage-consistent tuples (with lightweight pruning).
- Implemented (v0): `Hard` cross-cage row/column deductions (must-appear digits in a row/col segment of a cage are eliminated from the rest of that row/col).
- Not yet implemented: a true “tier required” classifier that matches upstream’s exact deduction ordering/reporting.
