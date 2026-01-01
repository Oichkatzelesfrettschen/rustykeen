# Cleanroom Policy (operational)

This repo is a cleanroom/reverse-engineering port effort. “Cleanroom” must be an **operational workflow**, not just an intent.

## Allowed artifacts (recommended)
- Public puzzle instances (cage definitions + solutions) sourced from legal/public distributions.
- Black-box observations of behavior (input → output), recorded as tests and notes.
- High-level algorithm descriptions and math facts (Latin squares, constraint solving, etc).

## Prohibited artifacts (recommended)
- Copy/paste of upstream source code into this repository.
- Copy/paste of upstream heuristics/thresholds/constants without re-derivation and independent justification.

## Upstream study workflow
- Upstream code snapshots may be downloaded locally for study under `third_party/` and must remain **git-ignored**.
- Distill any upstream findings into:
  - behavior/format descriptions
  - test vectors (puzzles and expected outcomes)
  - independently re-derived implementation strategies
- Record provenance for any non-trivial rule/format decision (ADR recommended).

## Evidence trail
- For each compatibility feature (e.g., parsing an upstream “desc” format), include:
  - a doc describing the format
  - a corpus of example strings and expected parse results
  - round-trip tests (parse → print → parse)

