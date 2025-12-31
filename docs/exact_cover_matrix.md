# Exact Cover Matrix Logic for Keen/KenKen (2025-12-31T22:02:21.880Z)

## Overview
- Exact Cover maps candidate assignments to a sparse binary matrix where each row is an assignment and each column is a constraint. A solution selects rows so every column is covered exactly once.

## Indices
- N: grid size (4..9). Index cells by i = r*N + c, 0-based.
- Column blocks:
  1) Cell(r,c) — size N*N: exactly one number per cell.
  2) RowNum(r,n) — size N*N: each number n appears once in row r.
  3) ColNum(c,n) — size N*N: each number n appears once in col c.
  4) Cage constraints — encoded via tuple columns or hybrid SAT (see dlx_mapping.md, cnf_templates.md).

## Row construction (Latin core)
- For each (r,c,n) candidate (1..N): emit a row covering three primary columns:
  - Cell(r,c)
  - RowNum(r,n)
  - ColNum(c,n)
- Row payload: (r,c,n) and optional cage linkage metadata.

## Column numbering (example)
- Let offsets:
  - C0 = 0 (Cell block), size N*N.
  - C1 = C0 + N*N (RowNum block), size N*N.
  - C2 = C1 + N*N (ColNum block), size N*N.
  - C3 = C2 + N*N (Cage columns), variable size.
- Column index helpers:
  - col_cell(r,c) = C0 + r*N + c
  - col_rownum(r,n) = C1 + r*N + (n-1)
  - col_colnum(c,n) = C2 + c*N + (n-1)

## Cage integration options
- DLX-secondary columns:
  - For each cage k and valid tuple t: add CageTuple(k,t) column; for each position i in t, add CageMember(k,t,i) column.
  - Extend row (r,c,n) to also cover CageTuple/CageMember when (r,c,n) matches tuple position.
- Hybrid DLX+SAT:
  - Keep Latin DLX matrix minimal (3 columns per row).
  - Maintain SAT side-constraints to prune invalid (r,c,n) before/inside search.

## Randomization
- Shuffle column selection order to diversify solutions; use seedable RNG.

## Uniqueness check
- Run DLX search; count up to 2 solutions; stop if >1 found. Use fast early exits.

## Pseudocode
- BuildMatrix(N, cages):
  - cols = 3*N*N + cage_cols(cages)
  - rows = Vec<Row>{}
  - for r in 0..N: for c in 0..N: for n in 1..=N:
    - if cage_prunes((r,c,n)) { continue }
    - row = bitset(cols)
    - row.set(col_cell(r,c)); row.set(col_rownum(r,n)); row.set(col_colnum(c,n))
    - if use_tuple_columns { add cage tuple/member hits }
    - rows.push(row with payload (r,c,n))
  - return Matrix{cols, rows}

## Complexity
- Rows: N*N*N (e.g., 729 for 9x9). Columns: 3*N*N + cage extras. DLX operates in microseconds at N≤9.

## Notes
- Use bumpalo for row/node allocations; fixedbitset/bitvec for row storage.
- Expose deterministic build order; keep mapping functions in core for reuse.
