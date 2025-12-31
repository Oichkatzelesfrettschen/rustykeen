# Design Specification

## Models
- Grid<N>: const generic N (4–9); cells have domain bitsets; cages: cell set, op ∈ {Add, Sub, Mul, Div, Eq}, target.
- Constraints: cage op semantics; row/column all-different; domain bounds; optional inequalities.

## Algorithms
- Propagation: AC-3 for binary constraints; generalized arc-consistency for cages; forward checking.
- Search: DFS with MRV/LCV; domain splitting; fail-first; iterative deepening optional; uniqueness by alternative assignment search.
- Generator: cage partitioning heuristics; target assignment consistent with solution; minimization preserving uniqueness; difficulty via search metrics.

## IO
- JSON/TOML schema: {size, cages:[{cells:[(r,c)...], op, target}], metadata:{seed, difficulty, version}}; versioned.

## APIs
- core: types/constraints/domain ops; no_std feature; immutable views.
- solver: solve(puzzle, config) → Solution/None; step API.
- gen: generate(config) → Puzzle; minimize(puzzle) → Puzzle.
- io: parse_json/parse_toml; to_json/to_toml.

## Performance
- SoA domains; bitvec constraints; portable_simd; cache-friendly iteration; avoid allocations in hot paths.

## Android FFI
- Expose kenken_init, kenken_load_puzzle, kenken_solve_step, kenken_solve_all, kenken_generate, kenken_shutdown; shim dispatches to workers; main thread only for callbacks.
