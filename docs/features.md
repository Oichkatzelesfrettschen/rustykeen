# Feature strategy (workspace)

This repo keeps optional capabilities behind Cargo features so the core remains portable and predictable.

## `kenken-core`
- `serde` (default off): derives `Serialize/Deserialize` for `Op` and `Ruleset`.
- `format-sgt-desc` (default on): upstream sgt-puzzles “desc” parser/printer (`kenken_core::format::sgt_desc`).
- `core-bitvec` (default off): enables `kenken_core::BitDomain` based on `bitvec` (currently not used by solver).
- `perf-assertions` (default off): enables `static_assertions` layout checks for key core structs.
- `std` (default on): placeholder for eventual `no_std` story.

## `kenken-solver`
- `tracing` (default on): enables `tracing::trace!` in hot loops (no runtime initialization required).
- `perf-likely` (default off): enables `likely_stable::likely(...)` hints in hot branches.
- `alloc-bumpalo` (default off): enables `bumpalo` arena allocations for propagation scratch buffers (reduces heap churn).
- `solver-dlx` (default off): enables `dlx-rs` Latin-square exact-cover solver utilities.
- `sat-varisat` (default off): enables `varisat` SAT encoding utilities (uniqueness hooks).
- `std` (default on): placeholder for eventual `no_std` story.

## `kenken-gen`
- `parallel-rayon` (default off): enables `rayon` for batch uniqueness/counting (foundation for parallel generation).
- `gen-dlx` (default off): enables DLX-backed Latin generation via `kenken-solver/solver-dlx`.
- `verify-sat` (default off): enables SAT uniqueness helpers via `kenken-solver/sat-varisat` (hybrid verification).
- `telemetry-tracing` (default off): enables `tracing::trace!` emission from generator loops.
- `std` (default on): placeholder for eventual `no_std` story.

## `kenken-cli`
- `alloc-mimalloc` (default off): enables `mimalloc` global allocator.
- `telemetry-subscriber` (default on): installs a default `tracing-subscriber` for local runs (override with `RUST_LOG`).

## `kenken-io`
- `io-rkyv` (default off): enables `rkyv` snapshot encode/decode (`kenken_io::rkyv_snapshot`).
- `std` (default on): placeholder for eventual `no_std` story.

## `kenken-uniffi`
- `ffi-uniffi` (default on, crate-local): enables UniFFI scaffolding and the exported API surface in `kenken-uniffi`.
- `gen` (default off): enables generator bindings via `kenken-gen/gen-dlx`.
- `std` (default on): placeholder for eventual `no_std` story.

## Composition rules
- Adapters/corpora tooling can depend on `kenken-core/format-sgt-desc`.
- Embedded consumers can disable features (e.g., `default-features = false`) and enable only what they need.

## Upstream crate feature policy
When we add a third-party crate, we record two things:
1) **our workspace gate** (e.g. `kenken-solver/sat-varisat`)
2) **upstream crate features** we intentionally enable/disable

Guidelines:
- Prefer `default-features = false` for libraries unless the upstream defaults are required.
- Avoid enabling `std` in the core if a crate offers `alloc`-only; keep the `no_std` path viable.
- Avoid “test/example” features unless we use them in our own tooling.
- Record any platform constraints (iOS allocator restrictions, Android logging, etc.) in docs and gate by `cfg`.

See `docs/crate_feature_plan.md` for the per-crate intended feature selections.
