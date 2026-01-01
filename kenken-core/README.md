# `kenken-core`

Library-first data model and validation for the “Keen” (KenKen-style) engine.

This crate is intentionally small and portable:
- Defines `Puzzle`/`Cage`/`Ruleset` and the invariants that make a puzzle well-formed.
- Provides import/export for the upstream sgt-puzzles “desc” format for corpus/regression testing.
- Keeps “heavy” functionality (search, generation, certification, FFI) in other crates.

## Key types
- `kenken_core::Puzzle`: grid size `n` and cage list.
- `kenken_core::Cage`: set of cells + operation + target.
- `kenken_core::rules::{Ruleset, Op}`: rule switches and operations.

## Feature flags
- `format-sgt-desc` (default): enables `kenken_core::format::sgt_desc`.
- `serde` (default off): derives `Serialize/Deserialize` for `Op` and `Ruleset`.
- `core-bitvec` (default off): enables `kenken_core::BitDomain` (bitvec-backed domains).
- `perf-assertions` (default off): enables compile-time layout checks via `static_assertions`.

