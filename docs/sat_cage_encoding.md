# SAT cage encoding (Varisat) — design notes

Goal: extend the existing Latin SAT encoding (`kenken-solver/src/sat_latin.rs`) to cover **full KenKen cage arithmetic** so that SAT can be used as:
- a *uniqueness certificate hook* (count up to 2 solutions with blocking clauses), and/or
- a *fallback solver* when tuple-table explosion makes DFS/DLX impractical.

This document is intentionally engineering-focused: variable mapping, clause templates, and performance guardrails.

## 1) Variables

We use one-hot Boolean variables for cell values:

`X(r,c,v)` means “cell (r,c) has value v”, where `v ∈ 1..=N`.

Index mapping (0-based value for storage):
- `X_idx = ((r*N + c) * N) + (v-1)`

This matches the Latin-only encoding in `kenken-solver/src/sat_latin.rs`.

## 2) Latin constraints (baseline)

These are already implemented in the Latin SAT helper:
- Exactly-one value per cell:
  - At-least-one: `X(r,c,1) ∨ ... ∨ X(r,c,N)`
  - At-most-one: pairwise `¬X(r,c,i) ∨ ¬X(r,c,j)` for all `i<j`
- Row uniqueness: for each row r and digit v:
  - for all columns `c1<c2`: `¬X(r,c1,v) ∨ ¬X(r,c2,v)`
- Column uniqueness: analogous.

## 3) Cage constraints (engine-owned ruleset baseline)

Ruleset baseline: subtraction/division are 2-cell only (see ADR 0001).

### 3.1 Eq cage (single cell)
For cage `{cell=(r,c), op=Eq, target=t}`:
- Clause: `X(r,c,t)`

### 3.2 Sub/Div (2-cell only)
For cells `a=(r1,c1)`, `b=(r2,c2)` and target `T`:
- Compute allowed ordered pairs `(va, vb)` with:
  - `Sub`: `|va - vb| = T`
  - `Div`: `max(va,vb) = min(va,vb) * T`

Encode as a disjunction of allowed pairs using selector variables:
- Introduce selector vars `S_k` for each allowed pair k.
- Exactly-one selector is true: `∨ S_k` and pairwise `¬S_i ∨ ¬S_j`.
- Selector implies assignments:
  - `S_k → X(a,va_k)` and `S_k → X(b,vb_k)`

Rationale: avoids large disjunction-of-conjunctions patterns and keeps propagation strong.

### 3.3 Add/Mul (k-cell)
For larger cages we use a tuple-table selector encoding (bounded by thresholds):
- Enumerate all satisfying tuples (respecting in-cage row/col uniqueness for cells sharing row/col).
- Create one selector var per satisfying tuple; enforce exactly-one selector.
- Each selector implies all chosen values for all cells.

Guardrails:
- If tuple count exceeds the threshold `SAT_TUPLE_THRESHOLD` (currently 512), do not encode this way:
  - fall back to native solver enumeration (count up to 2 with early exit); or
  - use alternative encodings (sequential counters / pseudo-Boolean) once implemented; or
  - use SMT (Z3) in an optional certification path.

### 3.4 SAT_TUPLE_THRESHOLD = 512 justification

The threshold of 512 is chosen as a balance between:

**Upper bound on selector variables**: Each tuple requires one selector variable plus implications.
For a k-cell cage with T tuples:
- T selector variables
- 2*k*T implication clauses (selector → cell value, both directions)
- T*(T-1)/2 at-most-one clauses for selectors

At T=512 with k=4 (common Add cage), this is ~512 + 4096 + 130816 ≈ 135k clauses.
Beyond this, the CNF size grows rapidly and may dominate solve time.

**Practical cage bounds**: For baseline rulesets (N ≤ 9, cages ≤ 6 cells):
- 2-cell Add: at most (N-1) tuples (e.g., 8 for N=9) — well under threshold
- 3-cell Add: at most ~80 tuples for N=9 — under threshold
- 4-cell Add with high target: can reach hundreds of tuples — near threshold
- 5+ cell Mul: can explode (e.g., 6 cells of {1,2} factors) — may exceed threshold

**Empirical observation**: In testing with N=6 and N=9 puzzles, cages exceeding 512 tuples
are rare (<1% of generated puzzles) and often indicate degenerate or invalid configurations.

**Fallback cost**: When threshold is exceeded, the fallback is `count_solutions_up_to(..., limit=2)`
which uses native backtracking. For small puzzles this is fast; the SAT path is primarily
valuable for larger puzzles where native enumeration would be slow anyway.

The threshold can be adjusted via `SAT_TUPLE_THRESHOLD` in `kenken-solver/src/sat_cages.rs`.

## 4) Uniqueness via SAT

To check uniqueness:
1) Solve once, extract a model, decode a complete grid.
2) Add a blocking clause that forbids that exact assignment:
   - `¬X(r0,c0,v0) ∨ ¬X(r1,c1,v1) ∨ ...` for all cells.
3) Solve again:
   - SAT → multiple solutions
   - UNSAT → unique

This works for the full puzzle encoding, not just Latin.

## 5) Implementation staging

1) Add `sat_cages` module that builds cage clauses on top of the existing Latin variable mapping.
2) Implement Eq + 2-cell Sub/Div first.
3) Add tuple-selector encoding for Add/Mul cages up to a conservative threshold.
4) Add tests comparing SAT uniqueness vs solver enumeration for small puzzles.
5) Add instrumentation counters:
   - number of selector vars per cage
   - tuple counts and threshold cutoffs

Current repo status:
- `kenken-solver/src/sat_common.rs` centralizes the Latin SAT var mapping and model→blocking extraction.
- `kenken-solver/src/sat_cages.rs` implements:
  - Eq cages, 2-cell Sub/Div cages, tuple allowlists for Add/Mul (thresholded)
  - a sound fallback: on tuple overflow, it falls back to native `count_solutions_up_to_with_deductions(..., limit=2)`
  - tracepoints (behind `kenken-solver/tracing`) for tuple counts and selector counts

## Appendix: tuple enumeration helper

The current plan uses a tuple-enumeration strategy instead of encoding arithmetic circuits:
- `kenken_core::Cage::valid_permutations(...)` enumerates satisfying ordered tuples up to `max_tuples`.
- SAT encoders can then use those tuples to build selector allowlists.
