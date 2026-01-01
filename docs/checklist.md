# Checklist (docs vs implementation)

This checklist distinguishes between:
- **Docs**: design/architecture/spec work captured in `docs/`
- **Code**: implemented, compiled, and tested in the Rust workspace

## Documentation status
- Model + API sketch (types, ops, IO shape): done (`docs/design.md`, `docs/uniffi.udl`)
- DLX mapping + exact cover matrix notes: done (`docs/dlx_mapping.md`, `docs/exact_cover_matrix.md`, `docs/solve_dlx.rs`)
- CNF templates notes: done (`docs/cnf_templates.md`)
- Crates/perf/telemetry/verification notes: done (`docs/crates_audit.md`, `docs/riced_build.md`, `docs/formal_verification.md`, `docs/telemetry_build_assets.md`)
- Synthesized plan tying docs together: done (`docs/plan.md`)

## Implementation status (current repo state)
- Rust workspace with `kenken-*` crates: not started
- DLX matrix builder + solver: not started
- Cage arithmetic constraints + SAT fallback: not started
- Generator + minimizer + difficulty rubric: not started
- IO schema + round-trip tests: not started
- CLI/WASM/Android adapters: not started
- CI (fmt/clippy/test/bench matrix): not started

## Immediate next steps (bootstrap)
- Create Cargo workspace + crate skeletons, pin nightly toolchain, add baseline tests.
- Add a “golden puzzle corpus” (inputs + expected solutions/uniqueness/difficulty).
- Implement minimal `kenken-core` model + parser + solver “solve one” path.
