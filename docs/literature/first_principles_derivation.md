# First-Principles Derivation: KenKen Solving/Generation Foundations

This note defines the mathematical core used by `rustykeen` and ties each statement to implementation anchors.

## Evidence anchors (local corpus paths)

- **[SRC]** `docs/literature/sources.csv`
- **[PROV]** `docs/literature/provenance_manifest.csv`
- **[DLX]** `docs/literature/pdfs/2000_knuth_dancing-links.pdf`
- **[EXACT]** `docs/literature/pdfs/2010_kapanowski_exact-cover-problem.pdf`
- **[LATIN-1]** `docs/literature/pdfs/2012_bartlett_completions-epsilon-dense-partial-latin-squares.pdf`
- **[LATIN-2]** `docs/literature/pdfs/2013_fontana_random-latin-squares-sudoku-designs-generation.pdf`
- **[LATIN-3]** `docs/literature/pdfs/2025_kawase_roy_sanpui_latin-square-constraint.pdf`
- **[AC-1]** `docs/literature/pdfs/2001_cooper_schiex_arc-consistency-soft-constraints.pdf`
- **[AC-2]** `docs/literature/pdfs/2011_chen_dalmau_grussien_arc-consistency-and-friends.pdf`
- **[AC-3]** `docs/literature/pdfs/2013_berkholz_verbitsky_speed-constraint-propagation.pdf`
- **[SAT-1]** `docs/literature/pdfs/2001_moskewicz_et-al_chaff-efficient-sat-solver.pdf`
- **[SAT-2]** `docs/literature/pdfs/2003_een_sorensson_extensible-sat-solver.pdf`
- **[SAT-3]** `docs/literature/pdfs/2024_zhang_chen_cai_revisiting-restarts-cdcl.pdf`
- **[SAT-4]** `docs/literature/pdfs/2026_cai_zhang_et-al_clause-management-cdcl.pdf`
- **[GEN]** `docs/literature/pdfs/2020_nishikawa_toda_generating-strategy-solvable-sudoku-clues.pdf`

## 1) First-principles problem model

Let:

- Grid size: \(n \in \mathbb{N}\), \(n \ge 1\)
- Cell index set: \(C = \{(r,c)\ |\ r,c \in \{0,\dots,n-1\}\}\)
- Value set: \(V = \{1,\dots,n\}\)
- Variables: \(X_{r,c} \in V\) for each \((r,c)\in C\)

### 1.1 Latin-square constraints

For each row \(r\): `AllDiff(X_{r,0},...,X_{r,n-1})`  
For each column \(c\): `AllDiff(X_{0,c},...,X_{n-1,c})`

These are the structural constraints that make KenKen a constrained Latin-square problem ([LATIN-1], [LATIN-2], [LATIN-3]).

### 1.2 Cage arithmetic constraints

Let cages form a partition \(\mathcal{G} = \{g_1,\dots,g_m\}\) of \(C\), with each cage
\(g=(\text{cells}(g), \text{op}(g), \text{target}(g))\).

For a cage with ordered cells \((c_1,\dots,c_k)\), define tuple \(x=(x_1,\dots,x_k)\in V^k\):

- `Eq`: \(k=1 \land x_1 = t\)
- `Add`: \(\sum_{i=1}^{k} x_i = t\)
- `Mul`: \(\prod_{i=1}^{k} x_i = t\)
- `Sub`: \(k=2 \land |x_1-x_2| = t\)
- `Div`: \(k=2 \land \max(x_1,x_2)=t\cdot\min(x_1,x_2)\)

The satisfying tuple set for cage \(g\) is:
\[
T_g = \{x \in V^k \mid \text{arith}_{op(g),target(g)}(x)\}
\]
and cage satisfaction is \( (X_{c_1},...,X_{c_k}) \in T_g\).

## 2) CSP formulation

Define CSP \(P=(X,D,\Gamma)\):

- \(X = \{X_{r,c}\ |\ (r,c)\in C\}\)
- Initial domains \(D_{r,c}=V\)
- Constraints \(\Gamma = \Gamma_{\text{latin}} \cup \Gamma_{\text{cage}}\), where:
  - \(\Gamma_{\text{latin}}\): row/column all-different
  - \(\Gamma_{\text{cage}}\): extensional (table) constraints via \(T_g\)

Equivalently, Latin constraints can be represented as exact-cover rows/columns over `(cell)`, `(row,value)`, `(col,value)` incidence ([DLX], [EXACT]), while cage constraints remain arithmetic table constraints.

## 3) Propagation semantics and deduction tiers

Let \(D\) be the current domain map. A propagation step is a monotone reduction operator:
\[
F_{\text{tier}}(D) = F_{\text{latin}}(D) \cap F_{\text{cage,tier}}(D)
\]
iterated to a fixed point (or failure if some domain becomes empty).  
This is the implementation analogue of local consistency enforcement ([AC-1], [AC-2], [AC-3]).

### 3.1 Tier semantics (as implemented)

- **None**: no iterative propagation; search uses feasibility checks only.
- **Easy**:
  - Sub/Div: keep values with pair support in \(T_g\).
  - Add/Mul: use cage-wide union mask (`any_mask`) for all cells in cage.
- **Normal**:
  - Sub/Div: same support filtering as Easy.
  - Add/Mul: per-position support masks from tuple enumeration (`per_pos`).
- **Hard**:
  - Normal filtering plus cross-row/column eliminations from mandatory value sets (`must_row`, `must_col`) derived from all currently valid cage tuples.

So tiers are not separate solvers; they are progressively stronger domain-reduction operators over the same CSP.

## 4) Uniqueness and bounded counting

Let \(\mathrm{Sol}(P)\) be the solution set and \(N(P)=|\mathrm{Sol}(P)|\).

- Solve/existence task: decide \(N(P)\ge 1\), optionally produce witness.
- Uniqueness task: decide \(N(P)=1\).
- Counting primitive used in code:
  \[
  \mathrm{count\_up\_to}(P,L)=\min(N(P),L)
  \]
  with DFS search + early stop at \(L\).

Uniqueness is implemented via `count_up_to(..., 2) == 1`, which is the minimal bounded-count check needed for generation loops ([GEN]).

## 5) SAT / SMT / DLX placement and feature-gating rationale

### 5.1 DLX (exact cover)

DLX fits the Latin core because row/column uniqueness is naturally exact-cover over binary incidence constraints ([DLX], [EXACT]).  
In this codebase it is used for Latin-solution construction/counting support, not as the only full KenKen solver.

### 5.2 SAT (CDCL)

SAT fits uniqueness checking and alternative encodings via Boolean variables, clause learning, and model blocking ([SAT-1], [SAT-2], [SAT-3], [SAT-4]).  
Current integration encodes Latin constraints and staged cage encodings (Eq, two-cell Sub/Div, and tuple allowlists for Add/Mul with threshold fallback), then uses a second solve with blocking clause for uniqueness.

### 5.3 SMT (Z3)

SMT fits symbolic verification workflows where constraints are asserted over integer terms and uniqueness is checked by asking for an alternative model. In this repo it is verification-oriented rather than the default runtime solver.

### 5.4 Why feature-gated

Optional backends introduce external dependencies and non-trivial compile/runtime tradeoffs (e.g., Varisat, Z3).  
Feature-gating keeps default builds library-first and deterministic, while enabling SAT/SMT/DLX only when needed for generation or verification workflows.

## 6) Mapping derivation statements to current crates/modules

| Derivation statement | Implementation anchor |
|---|---|
| Puzzle = grid size + partition into cages + cage operation/target | `kenken-core/src/puzzle.rs` (`Puzzle`, `Cage`) |
| Allowed cage tuple semantics \(T_g\) for Eq/Add/Mul/Sub/Div | `kenken-core/src/puzzle.rs` (`Cage::valid_permutations`) |
| Rule restrictions (Sub/Div two-cell, max cage size, connectivity) | `kenken-core/src/puzzle.rs` (`validate_shape`), `kenken-core/src/rules.rs` (`Ruleset::keen_baseline`) |
| CSP search and witness extraction | `kenken-solver/src/solver.rs` (`solve_one`, `backtrack`) |
| Bounded counting for uniqueness | `kenken-solver/src/solver.rs` (`count_solutions_up_to`, `count_solutions_up_to_with_deductions`) |
| Fixed-point propagation over domains | `kenken-solver/src/solver.rs` (`propagate`, `apply_cage_deduction`) |
| Tier interpretation (None/Easy/Normal/Hard) | `kenken-solver/src/solver.rs` (`DeductionTier`, branch logic in `apply_cage_deduction`) |
| Generator’s uniqueness loop via bounded counting (`limit=2`) | `kenken-gen/src/generator.rs` (`count_solutions_up_to_with_deductions(..., 2)`) |
| DLX exact-cover Latin backend | `kenken-solver/src/dlx_latin.rs` |
| SAT uniqueness backends (Latin + staged cages) | `kenken-solver/src/sat_latin.rs`, `kenken-solver/src/sat_cages.rs` |
| SMT uniqueness verifier | `kenken-solver/src/z3_verify.rs` |
| Backend feature gates | `kenken-solver/Cargo.toml`, `kenken-gen/Cargo.toml`, `kenken-verify/Cargo.toml` |

## 7) Directly actionable architecture/debt notes (non-speculative)

1. The core mathematical model is a **single CSP**; deduction tiers are propagation-strength variants, not different correctness models.
2. SAT integration is intentionally **staged/partial** for cage encodings and already includes bounded native fallback for tuple explosion.
3. DLX currently captures the **Latin exact-cover subproblem**; cage arithmetic remains outside pure exact-cover unless additional encoding layers are added.
4. Uniqueness in generation is already grounded in the bounded-count identity `count_up_to(2)==1`.

---

All references above are pinned to local corpus files listed in [SRC] and checksum-tracked in [PROV].
