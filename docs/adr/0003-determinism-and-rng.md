# ADR 0003: Determinism and RNG policy

## Status
Proposed

## Context
Determinism is a core requirement for:
- reproducible generation (seeded)
- benchmark comparability
- cross-platform consistency (desktop/Android/iOS/WASM)
- debugging and regression bisection

Rust’s default RNG APIs can introduce nondeterminism across versions/platforms if the algorithm is not pinned.

## Decision
- Use `rand_chacha::ChaCha20Rng` as the canonical RNG for generation and randomized solver tie-breaks.
- Store and surface seeds in all generator outputs (and in structured formats as metadata).
- Define deterministic ordering rules:
  - never rely on hash-map iteration ordering in core logic
  - explicitly sort cages/cells where order affects outputs
  - keep a stable “cell id” ordering (row-major)
- Any “parallel generation” mode must preserve determinism by:
  - deriving per-job RNG streams deterministically from the root seed, or
  - being clearly labeled as nondeterministic and excluded from golden tests.

## Alternatives
- `rand_pcg` (also fine, but conflicts with earlier docs recommending ChaCha).
- `getrandom` / OS RNG (not deterministic).

## Consequences
- A pinned RNG becomes part of the compatibility contract.
- Parallelism requires careful seeding strategy to avoid run-to-run drift.

