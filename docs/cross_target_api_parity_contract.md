# Cross-target API parity contract (`kenken-uniffi` vs `kenken-wasm`)

Last updated: 2026-04-06

Status: **Published and enforced** by `kenken-wasm/tests/cross_target_api_parity_contract.rs`.

## Scope and audited surfaces

This contract defines parity expectations for core operations across the two binding crates:

- `kenken-uniffi/src/keen.udl`
- `kenken-uniffi/src/lib.rs`
- `kenken-wasm/src/lib.rs`

The focus is the core operation family (`solve`, `count`, `generate`) plus argument/result and error-shape semantics at the public boundary.

## Required parity operations

| Operation family | `kenken-uniffi` surface | `kenken-wasm` surface | Contract status |
|---|---|---|---|
| Solve | `solve_sgt_desc(u8 n, string desc, DeductionTier tier) -> Option<Grid>` | `solve_puzzle_handle(handle: u32) -> Result<String, JsValue>` | **Required now** |
| Count | `count_solutions_sgt_desc(u8 n, string desc, DeductionTier tier, u32 limit) -> u32` | No direct count endpoint | **Accepted intentional divergence** |
| Generate | `generate_sgt_desc(u8 n, u64 seed, DeductionTier tier) -> Option<Generated>` | No direct generate endpoint | **Accepted intentional divergence** |

### Required now

1. Both targets must continue to expose a solving capability.
2. Both targets must preserve `DeductionTier` semantics (`None`, `Easy`, `Normal`, `Hard`) for solve behavior, even if represented through different transport APIs.

## Accepted intentional divergences

1. `kenken-uniffi` is `sgt-desc`-first, while `kenken-wasm` currently uses opaque puzzle handles (`create_test_puzzle` + `solve_puzzle_handle`).
2. `count_solutions_sgt_desc` is not currently exported by `kenken-wasm`.
3. `generate_sgt_desc` is not currently exported by `kenken-wasm`.
4. `kenken-uniffi::generate_sgt_desc` is feature-dependent (`gen`) and may return `None` when generation support is unavailable.

These are intentional for now; adding wasm `count`/`generate` entry points should update this contract and tests in the same change.

## Error-shape expectations

### `kenken-uniffi`

- Solve failure shape: `Option<Grid>` (`None` on parse error, solver error, or no solution).
- Generate failure shape: `Option<Generated>` (`None` on disabled feature or generation/encoding failure).
- Count failure shape: `u32`, where parse/solve failures collapse to `0`.

### `kenken-wasm`

- Transport/runtime failures use `Result<String, JsValue>` error channel (e.g., lock failures, invalid handle).
- Domain-level solve failures are encoded in `SolveResultData` with:
  - `success = false`
  - `solution = None`
  - non-empty `error` text
- Successful solves must set:
  - `success = true`
  - `solution = Some(Vec<u32>)`
  - `error = None`

## Drift-detection checks

Contract drift is validated by:

- `kenken-wasm/tests/cross_target_api_parity_contract.rs`
- `cargo test --all-targets --all-features` (includes the parity contract test target)

These checks enforce:

1. Core operation names remain discoverable in `kenken-uniffi` UDL/source.
2. Current wasm intentional divergences (`count`/`generate` absence) remain explicit and documented.
3. Error-shape expectations remain synchronized with source-level signatures and result fields.
