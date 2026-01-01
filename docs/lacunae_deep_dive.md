# Lacunae Deep Dive (design / crates / engineering / algorithms / logic / Latin squares)

This document enumerates the main gaps between:
- our current documentation set (`docs/*`)
- the repository’s implementation state (no Cargo workspace yet)
- and the upstream “Keen” baseline behavior/format as observed in sgt-puzzles (`docs/upstream_sgt_puzzles_keen.md`)

## 1) Design lacunae

### 1.1 “Single source of truth” vs doc sprawl
We now have `docs/plan.md`, but several older docs still contain statements that look like settled decisions without being explicitly recorded as decisions. That leads to drift.

Resolution direction:
- Treat `docs/plan.md` as the root, and introduce ADR-style decision records for anything that affects public API, formats, or solver correctness.

### 1.2 Public API scope is not pinned to a stability policy
The docs sketch APIs, but do not define:
- semver policy
- what constitutes “breaking change”
- which formats are stable vs internal

Resolution direction:
- Define v0: “API unstable, fast iteration”, but still keep *format* versioning and compatibility tests.

### 1.3 Format strategy is under-specified
Docs propose JSON/TOML schema, but upstream has a compact canonical “desc” string format that is extremely valuable for:
- corpus ingestion
- interoperability with existing tools
- “golden tests” that don’t require complex structured encoders

Resolution direction:
- Decide and document: support upstream “desc” as import/export (even if only for tests/CLI), plus a versioned structured format for app integration.

### 1.4 Cage semantics policy needs explicit declaration
Upstream restricts SUB/DIV to 2-cell cages. Some KenKen variants allow larger cages for SUB/DIV with permutation semantics, but that changes both solver and generator complexity.

Resolution direction:
- Decide early: follow upstream restriction (recommended for baseline parity), and treat “multi-cell SUB/DIV” as a future variant behind a feature or ruleset version.

## 2) Cargo crates / dependency lacunae

### 2.1 The dependency set is speculative and not aligned to a staged bootstrapping path
`Cargo.sample.toml` lists a large optimization stack, but the repo has no `Cargo.toml` yet. Even as a planning artifact, it is missing core dependencies implied by the docs:
- `serde`, `serde_json`, `toml` (IO docs)
- `clap` (CLI docs)
- `rand_chacha` (docs mention ChaCha; sample uses `rand_pcg`)

Resolution direction:
- Adopt a two-tier dependency strategy:
  - **Tier 0 (bootstrap)**: minimal crates required for correctness and testability.
  - **Tier 1 (perf/verification/adapters)**: optional, feature-gated.

### 2.2 “Nightly everywhere” is not justified per-dependency
Nightly is useful for `portable_simd`, but most of the core engine can be stable. Locking everything to nightly early increases churn.

Resolution direction:
- Keep nightly toolchain pinned (because you explicitly want it), but design so `kenken-core` compiles on stable where possible; gate nightly-only fast paths behind `simd` (or similar).

### 2.3 DLX/SAT crate choices are not validated
Docs mention DLX (`dlx_rs`) and SAT (`varisat`), but:
- compatibility, maintenance, and performance aren’t validated in-repo
- there’s no fallback plan if a crate is unmaintained or too slow

Resolution direction:
- Treat DLX/SAT as optional “Phase M2” and use upstream-style deduction as the baseline solver first.

## 3) Engineering lacunae

### 3.1 Repo is not buildable
There is no workspace `Cargo.toml`, no pinned toolchain file, and no CI. That blocks iterative optimization entirely.

Resolution direction:
- Bootstrap a minimal compiling workspace and wire `fmt/clippy/test` gates.

### 3.2 No corpus, benchmarks, or regression harness
Docs talk about difficulty grading and performance, but there is no:
- golden puzzle corpus
- benchmark suite
- instrumentation contract wired into tests

Resolution direction:
- Add a seed-based corpus generator and a fixed “known puzzle set” derived from the upstream `desc` format.

### 3.3 Cleanroom workflow is not operationalized
“Cleanroom” is stated as a goal, but there is no evidence workflow:
- where upstream code snapshots live
- what is allowed to be copied vs observed
- how behavioral parity is demonstrated

Resolution direction:
- Add explicit rules and artifact locations (e.g., `third_party/` ignored by git; docs record provenance; tests derived from puzzle instances, not code).

### 3.4 Licensing file is incorrect
Current `LICENSE` is an HTML redirect instead of the actual text. That is a correctness/legal hygiene issue.

## 4) Algorithm lacunae (solver + generator)

### 4.1 Baseline algorithm parity vs planned DLX/SAT
Upstream achieves solvability + difficulty classification without DLX/SAT by:
- Latin-square constraint solver (`latin.c`)
- cage enumeration with pruning
- difficulty-tiered deductions (Easy/Normal/Hard)

Our docs jump directly to DLX/SAT, but do not specify:
- how “difficulty” is derived from those methods
- which solver is canonical when results disagree

Resolution direction:
- Implement upstream-style deduction first for parity and to define difficulty tiers; then add DLX/SAT as optional acceleration/proof tools.

### 4.2 Add/Mul cage enumeration needs a concrete cost model
Tuple explosion is a real issue; upstream mitigates by pruning in enumeration and limiting cage size.
Our docs mention thresholds and SAT fallback but don’t define them.

Resolution direction:
- Start with upstream constraints: cage max size (`MAXBLK`-like cap), and implement pruning enumeration.
- Add SAT fallback only after benchmarks show where enumeration breaks down.

### 4.3 Latin square generation is underspecified
Upstream generates Latin squares via bipartite matching row-by-row, guaranteeing extendibility without backtracking.
Our docs do not specify:
- which Latin generation algorithm we’ll use
- whether uniformity matters
- how determinism interacts with sampling

Resolution direction:
- Decide: port the matching-based generator (for parity) or accept a simpler generator (for speed), and document the tradeoff.

## 5) Logic lacunae (rules, validation, edge-cases)

### 5.1 Duplicate digits inside a cage are conditionally allowed
KenKen allows repeats in a cage as long as they don’t violate row/col uniqueness. Upstream’s cage enumeration explicitly enforces “no equal digits if two cage cells share row or column”.

Resolution direction:
- Make this an explicit invariant in the rules engine, and test it.

### 5.2 Partial evaluation semantics need to be specified
For propagation and UI “is this inconsistent yet?” you need partial semantics:
- ADD: partial sum bounds
- MUL: divisibility / factor bounds
- SUB/DIV (2-cell): partial constraints based on remaining possibilities

Our docs mention AC-3/propagation but don’t spell out the exact partial checks.

## 6) Latin square lacunae (conceptual + implementation)

### 6.1 Canonical representation and symmetry handling
Latin squares have large symmetry groups (row/col/symbol permutations). If generation and difficulty grading ignore symmetry, you can:
- over-sample equivalent puzzles
- bias difficulty metrics

Resolution direction:
- For v0, accept symmetry bias, but document it.
- For later, consider normalizing generated solution grids (e.g., reduced form) before cage partitioning.

### 6.2 Validation and uniqueness need a formal definition
Even for Latin-only:
- Validation is straightforward.
- Uniqueness is subtle once cages are introduced; “unique solution” must be defined with the exact same rule set and parsing semantics.

Resolution direction:
- Make `Ruleset` (operations allowed, SUB/DIV cage size policy, etc.) explicit in the API and include it in serialized formats.

