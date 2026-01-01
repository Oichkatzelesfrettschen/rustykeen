# ADR 0002: Supported puzzle formats (v0)

## Status
Proposed

## Context
We need formats for:
- interoperability/corpus ingestion
- stable app-facing serialization
- deterministic round-tripping for tests

Upstream sgt-puzzles “Keen” uses a compact string format (“desc”) that encodes:
- cage boundaries via a run-length scheme
- a clue stream of op + target per cage

Our docs also propose JSON/TOML models for app integration.

## Decision
- v0 supports two format families:
  1) **Upstream-compatible “desc” string** (for tests, corpus tools, and optional CLI import/export).
  2) **Versioned structured format** (JSON/TOML) for app integration and long-term stability.
- The structured format includes:
  - grid size
  - explicit cage cell lists
  - op as an enum (not a string)
  - schema version
  - optional metadata (seed, difficulty, generator config)

## Alternatives
- Only structured format (loses easy corpus interoperability).
- Only upstream format (harder to evolve and less app-friendly).

## Consequences
- Requires two parsers/printers (but upstream format is high value for golden tests).
- Requires explicit schema versioning from day one.

