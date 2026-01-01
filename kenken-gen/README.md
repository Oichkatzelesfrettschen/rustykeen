# `kenken-gen`

Parallel-friendly generation utilities and orchestration.

Current focus (Phase B scaffolding):
- Batch solve / uniqueness checking APIs, optionally parallel via `rayon`.
- Deterministic RNG plumbing (seed → `ChaCha20Rng`) for cross-platform reproducibility.
- Experimental generator MVP behind `kenken-gen/gen-dlx` (Latin via DLX, random cage partition, target assignment, reject-until-unique loop).

This crate will eventually contain the full generator pipeline:
Latin solution → cage partition → clue assignment → uniqueness proof → minimization → difficulty scoring.
