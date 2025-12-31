# MRV Column Ordering in DLX (2025-12-31T22:08:24Z)

## MRV (Minimum Remaining Values)
- Strategy: at each search step, choose the column (constraint) with the fewest rows remaining to minimize branching.
- In node-based DLX: iterate active column headers; pick the one with minimal `Column.size`.

## Benefits
- Dramatically reduces search tree size, particularly on Latin/SAT hybrids.

## Implementation Notes
- Maintain `size` counts during cover/uncover.
- Tie-breakers: prefer cell columns first, or randomize for diversity in generation.
