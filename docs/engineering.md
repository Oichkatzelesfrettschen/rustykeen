# Engineering Plan

## Scaling beyond 6x6: Haywire diagnosis & fixes
- Root cause: naive DFS/backtracking explodes (O(n!)), causing hangs/non-unique puzzles.
- Fix: exact-cover DLX for Latin core + SAT for cage arithmetic; parallel batch generation via rayon; uniqueness checked by counting solutions (early exit >1).

## Nightly stack additions
- dlx_rs/custom DLX, varisat, bumpalo, dashmap, num-integer, portable-simd; cache difficulty scores across threads.

## Performance guardrails
- Arena allocation for solver nodes; SoA data; SIMD row/col checks; avoid heap in hot loops; Simpleperf + Perfetto instrumentation points.
