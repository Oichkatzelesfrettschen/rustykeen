# Latin squares in Keen (engineering notes)

Keen/KenKen puzzles are “Latin square + arithmetic cages”.

This repo treats the Latin constraint as a first-class performance target because:
- it is *global* (every row/col), and
- it has a well-known, extremely fast exact-cover formulation (DLX / Algorithm X).

Related internal docs:
- `docs/exact_cover_matrix.md` (exact-cover encoding notes)
- `docs/dlx_mapping.md` (mapping from Latin constraints to DLX rows/cols)
- `kenken-solver/src/dlx_latin.rs` (current implementation)

## Definitions (minimal)
- A **Latin square** of order `N` is an `N×N` grid filled with symbols `1..=N` such that:
  - each symbol appears exactly once per row
  - each symbol appears exactly once per column
- A **KenKen/Keen** puzzle adds **cages** (regions) with arithmetic constraints over the values in the region.

## Why DLX fits the Latin core
The Latin constraint can be expressed as an **exact cover** problem:
- Each possible assignment “(row=r, col=c) = v” is a candidate choice.
- Each choice satisfies exactly three constraint families:
  1) one value per cell
  2) one instance of value `v` in row `r`
  3) one instance of value `v` in column `c`

An exact-cover solver selects one choice per cell such that all constraints are covered exactly once.
DLX (“dancing links”) is a classic data structure that makes Algorithm X fast in practice.

## Clean separation: Latin vs cages
The current architecture keeps a deliberate boundary:
- DLX solves *only* the Latin part (fast, predictable, tight encoding).
- Cages are handled by:
  - the native solver (propagation + DFS), and/or
  - SAT-based uniqueness verification (tuple allowlists) when we want a proof-oriented check.

This helps us:
- keep encodings simple and auditable
- avoid mixing two very different constraint families prematurely
- benchmark the impact of each layer independently

## Practical generation trick (planned)
If DLX generation returns a deterministic “first” Latin square, we can still get variety cheaply:
- apply random permutations (seeded) to:
  - the symbols (rename 1..N)
  - rows (permute row indices)
  - columns (permute column indices)

This preserves Latin validity and is easy to make deterministic across platforms.

