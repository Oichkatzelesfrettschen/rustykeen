# CNF Templates for KenKen/Keen Cage Constraints (2025-12-31T22:00:49.082Z)

## Variables
- X_{r,c,n}: cell (r,c) takes value n (1..N). Latin constraints covered by DLX or CNF separately.
- For cages, define helper variables when needed:
  - T_{k,t}: tuple t (assignment of digits to cells of cage k) selected.
  - S_{k,s,v}: cage k cell slot s uses value v (link to X).
  - A_{k,u}: sum/product accumulator bits (for ladder/sequential encodings).

## Equality (Eq)
- Single-cell cage k at (r,c) with target v:
  - Clause: (X_{r,c,v}) and for n≠v add (¬X_{r,c,n}).

## Subtraction (Sub) two-cell cage
- Cells (r1,c1),(r2,c2), target d>0: allowed pairs (a,b) with |a−b|=d.
- For each disallowed pair (n1,n2) add clause: (¬X_{r1,c1,n1} ∨ ¬X_{r2,c2,n2}).
- Or enumerate allowed pairs and use auxiliary S slots:
  - Introduce S_{k,0,a} and S_{k,1,b} linking to X; enforce exactly-one per slot via pairwise at-most-one + at-least-one.
  - Add implication: S_{k,0,a} ∧ S_{k,1,b} → allowed; encode by blocking disallowed pairs.

## Division (Div) two-cell cage
- Target q>1: allowed pairs (a,b) with a/b=q or b/a=q and integral.
- Same encoding as Sub using allowed/disallowed pair blocking.

## Addition (Add) k-cell cage sum = T
- Approach 1: Enumerate tuples (v1..vk) with sum T; small k (≤4) feasible.
  - Introduce T_{k,t}; Exactly-one over tuples: at-least-one (∨_t T_{k,t}), at-most-one via pairwise (¬T_{k,t} ∨ ¬T_{k,t'}) or ladder encoding.
  - Link to cells: T_{k,t} → X_{r_i,c_i,v_i} for each i; encode as (¬T_{k,t} ∨ X_{r_i,c_i,v_i}).
- Approach 2: Cardinality/Ladder (no tuple enumeration):
  - Use slot variables S_{k,i,v} for each cell i and value v, with exactly-one per cell.
  - Sum constraint via binary adder or sequential counter:
    - Sequential counter: introduce A_{k,u} for partial sums; encode transitions ensuring total equals T (min-cost for small domains).
    - Binary adder: encode k adders of log2(T) bits; enforce final sum equals T using equality bits; more clauses but structured.

## Multiplication (Mul) k-cell cage product = P
- Approach 1: Enumerate tuples by factorization (best when k≤3 and P small): same as Add tuple method with T_{k,t} and links.
- Approach 2: Prime-exponent sum encoding:
  - Factor P = ∏ p_j^{e_j}. For each cell i and prime p_j, define E_{i,j,e} variables for exponent choice of digit at cell i.
  - Constrain per-cell consistency: E_{i,j,e} → X_{r_i,c_i,v} where v has exponent e on p_j.
  - Sum exponents across cells with sequential counters to equal e_j for each prime; avoids full tuple enumeration.

## Exactly-one encodings
- At-least-one: single clause (∨ v_i).
- At-most-one:
  - Pairwise: for all i<j add (¬v_i ∨ ¬v_j); simple, O(k^2).
  - Ladder (Sinz): introduce y_i chain variables; clauses: (v_1 → y_1), (v_i → y_i), (y_{i-1} → y_i), (¬v_i ∨ ¬y_{i-1}), (¬y_k); O(k) auxiliaries, O(k) clauses.
  - Binary encoding + commander variables for large k.

## Linking X and cage helpers
- X_{r,c,n} ↔ S_{k,s,n} for the cage slot s corresponding to (r,c): encode with two implications.
- When using T_{k,t}, enforce: T_{k,t} → S_{k,s_i,v_i} per position i; and prevent conflicts via exactly-one on T.

## Uniqueness check optimization
- Early-exit CDCL: run solver counting up to 2 solutions by adding a blocking clause for found model and stopping if a second is found.
- Use assumptions to test removal/minimization quickly.

## Clause budget guidance
- Prefer tuple enumeration for k≤3 and small T/P; switch to ladder/counters beyond threshold.
- Use prime-exponent encoding for products with large factorization space.

## Example: 2-cell Add (sum=10), N=9, cells a=(r1,c1), b=(r2,c2)
- Allowed pairs: (1,9)(2,8)(3,7)(4,6)(5,5) and permutations.
- Encode disallowed pairs with (¬X_{a,n1} ∨ ¬X_{b,n2}) for all (n1,n2) not in allowed.
- Or exactly-one over allowed with auxiliaries A_t and implications.

## Example: 3-cell Mul (product=24), N=9, cells a,b,c
- Factor 24=2^3*3^1. For prime 2: sum exponents =3; for prime 3: sum exponents =1.
- Define exponent vars for digits: e2(v)∈{0,1,3?} etc; link to X; sequential counters enforce totals.
