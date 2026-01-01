# `kenken-solver`

Deterministic solver for `kenken-core` puzzles:
- Backtracking search with MRV cell selection and cage feasibility pruning.
- Solution counting up to a limit (for uniqueness checks).
- Optional, staged acceleration modules behind feature flags:
  - `alloc-bumpalo`: arena-backed scratch buffers for propagation.
  - `solver-dlx`: Latin-square exact-cover utilities (DLX via `dlx-rs`).
  - `sat-varisat`: Latin-square SAT uniqueness utilities (Varisat).

## Public API
Top-level functions are re-exported from `kenken_solver`:
- `solve_one_with_deductions(...)`
- `count_solutions_up_to_with_deductions(...)`

