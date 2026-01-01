# `rkyv` Snapshot v2 (schema + compatibility plan)

This repo treats persistence as **engine-owned** and **versioned**.
The goal is to support:
- fast save/load (including mmap-friendly layouts later)
- stable cross-platform reproduction
- forward evolution with explicit compatibility tests

Implementation lives in `kenken-io/src/rkyv_snapshot.rs` behind `kenken-io/io-rkyv`.

## Snapshot v1 (current)
Snapshot v1 contains only:
- `Puzzle` (n + cages)

Limitations:
- It does not store the `Ruleset` used when the puzzle was generated/validated.

## Snapshot v2 (added)
Snapshot v2 adds:
- `Ruleset` fields that affect validity/semantics:
  - `sub_div_two_cell_only`
  - `require_orthogonal_cage_connectivity`
  - `max_cage_size`

Rationale:
- A snapshot should be self-describing enough to validate itself and to preserve
  “what rules were intended” even if defaults evolve.

## Version identification strategy
Snapshot v1 is a legacy “unframed” `rkyv` root type (so you can't reliably read a magic prefix from raw bytes).

Snapshot v2+ use an explicit, non-`rkyv` framing header:
- 8-byte envelope magic: `KEENSNAP`
- 2-byte little-endian version (`2` for Snapshot v2)
- 2-byte little-endian header length (`16` for v2, keeping the payload aligned)
- 4 bytes reserved (currently zero)
- followed by the `rkyv` payload bytes

This keeps version detection trivial and avoids guessing root types.

## Compatibility tests (must-have)
- v1 roundtrip: `encode_puzzle_v1` → `decode_puzzle_v1`
- v1 decode via unified entrypoint: `encode_puzzle_v1` → `decode_snapshot` returns `version=V1`
- v2 roundtrip: `encode_puzzle_v2(puzzle, rules)` → `decode_snapshot` returns `version=V2` and preserves `rules`

## Next planned v3+ additions (not implemented yet)
Potential future schema additions (all optional / versioned):
- generator metadata: `seed`, `attempts`, `difficulty tier`
- a canonical solution grid (for corpora / regression)
- stable IDs/hashes for deduping puzzle banks
- compression framing (outside `rkyv` payload) for large archives
