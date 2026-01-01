# Engineering Plan

Note: This file is a supporting sketch. The synthesized, reconciled plan lives in `docs/plan.md`.

## Scaling beyond 6x6: Haywire diagnosis & fixes
- Root cause: naive DFS/backtracking explodes (O(n!)), causing hangs/non-unique puzzles.
- Fix: exact-cover DLX for Latin core + SAT for cage arithmetic; parallel batch generation via rayon; uniqueness checked by counting solutions (early exit >1).

## Nightly stack additions
- Integrated today: `bumpalo`, `rayon`, `dlx-rs`, `varisat`, `rkyv`, `uniffi` (all feature-gated; see `docs/dependency_matrix.md`).
- Next: cage SAT encoding (varisat), generator topology/difficulty work, then higher-level caches/locks (`dashmap`, `parking_lot`).

## Performance guardrails
- Arena allocation for solver nodes; SoA data; SIMD row/col checks; avoid heap in hot loops; Simpleperf + Perfetto instrumentation points.
