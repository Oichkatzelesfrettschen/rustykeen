# Golden corpus strategy

Goal: maintain a small set of “known puzzles” that anchor:
- parsing/printing (upstream `desc` round-trips)
- validation invariants
- solving and uniqueness behavior (`count_solutions_up_to`)
- (later) difficulty tiers vs upstream expectations

## Source
- Prefer upstream sgt-puzzles “desc” strings (clean, compact, version-stable) as corpus entries.
- Do not vendor upstream code; only puzzle instances and expected outcomes.

## Structure
- Store corpus as text fixtures under `tests/fixtures/` (or per-crate `tests/fixtures/`).
- Each entry should include:
  - `n`
  - `desc`
  - expected: `unique` (bool) or `solutions_up_to_2` (0/1/2)
  - optional: `tier_expected` (later)

