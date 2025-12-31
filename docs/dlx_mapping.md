# DLX Exact Cover Mapping for Keen/KenKen

## Columns (constraints)
- Cell(r,c): exactly one number per cell.
- Row(r,n): each number n appears once in row r.
- Col(c,n): each number n appears once in col c.
- CageTuple(k,t): choose exactly one valid tuple t for cage k (DLX extension via secondary columns), or encode via SAT side-constraints.
- CageMember(k,t,i): link cell membership index i of tuple t in cage k to its (r,c,n) assignment rows.

## Rows (assignments)
- A row represents assigning value n to cell (r,c) AND (optionally) selecting cage tuple t where this (r,c,n) appears.
- Primary columns: Cell/Row/Col; secondary columns: CageTuple/CageMember to constrain arithmetic without over-constraining Latin.

## Cage tuple matrix construction
- For each cage k with cells S and op/target:
  1) Enumerate all digit tuples T ⊆ [1..N]^{|S|} satisfying op(target) and Latin domain (no repeats if same row/col required).
  2) Create a secondary column CageTuple(k,t) per tuple t.
  3) For each position i in tuple t mapped to cell s_i ∈ S, add CageMember(k,t,i) secondary column.
  4) For each (r,c,n) matching s_i and n == t[i], create assignment row covering: Cell(r,c), Row(r,n), Col(c,n), CageTuple(k,t), CageMember(k,t,i).
- Uniqueness: DLX exact cover ensures one tuple per cage and consistent cell assignments.

## Hybrid DLX+SAT approach
- If tuple explosion is large (Mul/Add with many combos), use SAT to prune: encode tuple feasibility in CNF and filter DLX rows before search.

## Randomization & diversity
- Shuffle column order; randomize tuple selection seeds; parallelize multiple DLX runs to sample diverse full grids and cage partitions.
