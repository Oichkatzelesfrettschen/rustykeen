# ADR 0001: Ruleset and cage semantics baseline

## Status
Proposed

## Context
KenKen-style puzzles vary in how they treat subtraction/division and multi-cell cages. Upstream “Keen” (sgt-puzzles) provides a widely used baseline:
- digits `1..=N`
- Latin row/column constraints
- cages with target + op
- subtraction/division restricted to 2-cell cages

We need a baseline ruleset to anchor solver correctness, generator behavior, difficulty grading, and compatibility formats.

## Decision
- v0 baseline ruleset matches sgt-puzzles “Keen”:
  - Allowed ops: Add, Mul for any cage size; Sub, Div only for 2-cell cages; Eq for 1-cell cages.
  - Cage repeats are allowed iff they do not violate row/column uniqueness.
- The ruleset becomes an explicit type in the core API (so future variants can exist without silent behavior changes).

## Alternatives
- Allow multi-cell Sub/Div (requires permutation semantics and increases solver/generator complexity).
- Support both, but default unspecified (risks ambiguity and incompatibility).

## Consequences
- Simplifies baseline compatibility and reduces edge-case ambiguity.
- Future variants are possible but must be explicit (feature flag or ruleset version).

