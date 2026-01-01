# Upstream Analysis: sgt-puzzles `keen.c` (Keen / KenKen)

This document is a **technical distillation** of the upstream implementation shipped in Simon Tatham’s Portable Puzzle Collection (“sgt-puzzles”), used here as reference material for format/behavior/algorithm study.

Upstream snapshot location (local, not committed): `third_party/upstream/puzzles-HEAD-*/`.

## 1) What upstream implements
- Game: “Keen” (Times) / “KenKen” style puzzle; also mentions “Inshi No Heya”.
- Rules:
  - Fill an `N×N` grid with digits `1..=N`.
  - Each row/column must contain each digit exactly once (Latin square constraint).
- Cages (regions) have an operation and target. Upstream allows:
  - `ADD` (`a`), `MUL` (`m`) for any cage size (capped by generator).
  - `SUB` (`s`), `DIV` (`d`) **only for 2-cell cages**.
  - Implementation note: upstream’s internal clue encoding relies on an operation ordering (ADD/MUL before SUB/DIV, DIV last) for bitflag packing; we only care about the observable external letters and semantics.

External cross-check:
- Many published KenKen variants restrict subtraction/division clues to 2-cell cages due to non-associativity; this matches upstream’s rule choice.

## 2) Upstream puzzle description format (“desc”)

Upstream represents a puzzle as a compact ASCII string:

1) **Block structure encoding** (cage region boundaries)
- Encodes the pattern of internal grid lines using a run-length-ish scheme over letters:
  - `_` means “0 non-lines between lines” (adjacent cage boundaries).
  - `a..y` mean 1..25 non-lines.
  - `z` is a special-case “25 non-lines and no following line” enabling larger counts (`za` = 26, etc).
- After that, the string is further compressed by replacing runs of the same letter with a single copy plus a repeat count (when shorter).

2) A literal comma `,`

3) **Clue stream** (one per cage root, concatenated):
- For each cage root (DSF minimal representative in scan order):
  - A clue type letter: `a`/`m`/`s`/`d`
  - Immediately followed by decimal digits of the target value.

Validation rules include:
- The number of clues must match the number of cages.
- `s` and `d` clues must correspond to cages of area 2.

Implication for this project:
- Decide whether to support this upstream string as an import/export format (high value for corpus + compatibility).

Current repo status:
- Implemented in `kenken-core` as `kenken_core::format::sgt_desc::{parse_keen_desc, encode_keen_desc}` for corpus tooling and golden tests.

## 3) Upstream generator (high-level)

### 3.1 Generate a Latin square solution grid
- Uses `latin_generate(w, rs)` (in `latin.c`), which constructs the Latin square row-by-row using bipartite matching.
- It relies on a theorem: any `r×n` Latin rectangle (`r<n`) can be extended to `(r+1)×n` without backtracking.

### 3.2 Partition the grid into cages (“blocks”)
- Uses a DSF/union-find over `N*N` cells.
- Two-stage cage formation:
  1) Place many **dominoes** (2-cages) first, probabilistically, to enable SUB/DIV clues.
  2) Then fold remaining singletons into neighboring cages subject to `MAXBLK` size cap (upstream uses `MAXBLK = 6`).
- If any singleton cannot be merged (due to size cap or topology), generator restarts.

### 3.3 Choose clue operation + target per cage
Upstream uses heuristics to avoid low-quality clues:
- For 2-cages:
  - Addition always allowed, but “too easy” sums are deprioritized (sums with only one possible pair in `1..=N`).
  - Multiplication is preferred when it leaves multiple options (especially above Normal difficulty).
  - SUB/DIV only when arithmetically viable.
- For >2 cages:
  - Only ADD/MUL (unless in “multiplication only” mode).

### 3.4 Difficulty selection loop
Generator repeatedly samples solutions + cage layouts until:
- Puzzle can be solved at difficulty `diff`, and
- (If `diff > 0`) puzzle is **not** solvable at `diff-1`.

Also: upstream includes explicit difficulty exceptions (e.g., `3×3` at high difficulty can cause the generator to spin forever, so it clamps difficulty).

## 4) Upstream solver (high-level)

Upstream does **not** implement DLX/SAT. Instead it reuses a general Latin-square solver framework (`latin.c`) and plugs in cage-specific deduction and final validation.

### 4.1 Representation
- Candidate cube: per-cell possible digits.
- A “grid” array storing assigned digits, plus row/col bookkeeping inside the Latin solver.
- Cages are represented by DSF, then transformed into a packed “boxlist/boxes” representation for efficient iteration.

### 4.2 Cage-based deduction (difficulty-tiered)
Upstream enumerates candidate digit layouts for each cage consistent with:
- Current per-cell candidate sets, and
- Row/column uniqueness constraints for cells that share a row or column within the cage.

It then derives eliminations differently depending on difficulty tier:
- **Easy**: only learns which digits can appear in the cage at all (coarse).
- **Normal**: learns per-cell possible digits within the cage (stronger).
- **Hard**: derives *cross-cage* constraints by identifying digits that must appear in a specific row/column segment of the cage across all cage-layout candidates, and then eliminates those digits from other cells in that row/column outside the cage.

### 4.3 Add/Mul enumeration strategy
For ADD/MUL cages, upstream brute-enumerates combinations with pruning:
- Maintains a “remaining” total (sum remaining or quotient remaining).
- Backtracks over cells in the cage, skipping candidates that can’t fit the remaining total or violate row/col constraints within the cage.

### 4.4 Uniqueness / ambiguity handling
The solver returns statuses:
- Impossible (no solutions)
- Ambiguous (multiple solutions)
- Solved (and can output a full solution string prefixed with `S`)

Implication for this project:
- Upstream’s difficulty and deduction structure is a concrete baseline spec for “what Normal/Hard means” in at least one well-known implementation.

## 5) Immediate gaps (relative to this repo’s docs)
- Our docs describe a future DLX/SAT hybrid; upstream “baseline correctness” can be achieved without that, and the upstream difficulty rubric is deeply tied to the deduction structure.
- Upstream has a de-facto compatibility format (“desc”) that is valuable for tests and corpus interchange and should be explicitly planned for (even if not identical to this project’s long-term JSON/TOML).
