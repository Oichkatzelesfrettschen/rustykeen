# ADR 0005: Major upgrade decision record (toolchain + dependency modernization)

## Status
Accepted

## Context
The workspace completed a modernization cycle covering toolchain pinning and major-version crate upgrades. We need an explicit, durable decision record that captures:

- what changed (`prior -> current`)
- why we accepted/deferred each major migration
- implementation/refactor impact
- introduced risk and mitigation
- evidence from reproducible command gates

This ADR is the canonical decision log for this cycle and is aligned with:

- `docs/dependency_audit.md`
- `docs/lacunae_audit.md`

## Decision
Adopt major upgrades that improve security, compatibility, and maintenance posture, while explicitly tracking deferred consolidation work where overlap remains intentional.

| Decision area | Prior -> current | Rationale | Implementation impact / refactor summary | Risks introduced | Mitigation | Disposition |
|---|---|---|---|---|---|---|
| Rust toolchain pin | `nightly` -> `nightly-2026-04-06` | Reproducibility across CI/local and stable diagnostics | Pinned in `rust-toolchain.toml` and CI lane updates | Pin can become stale | Periodic bump policy + strict gate reruns on each bump | **Adopted** |
| `rand` + `rand_chacha` | `0.9` line -> runtime `0.10.0` line (`Cargo.lock` also retains `0.9` via dev path) | Keep deterministic RNG stack current while preserving ChaCha policy | Workspace dependency bump in `Cargo.toml`; determinism policy unchanged (`ChaCha20Rng`) | Dual rand lines increase graph complexity | Accepted as dev-only overlap; monitor via `cargo tree -d --all-features` and debt register | **Adopted** (runtime), **Monitor** (dev overlap) |
| `z3` | `0.12.1` -> `0.20.0` (+ full cage-constraint uniqueness encoding) | Compatibility with current `z3` ecosystem and correctness-complete puzzle semantics in verification path | `kenken-solver` dependency bump; `z3_verify` refactored to newer API style (`with_z3_config`, contextless constructors), now encoding Eq/Add/Mul/Sub/Div cages and alternate-model checks | Backend still optional by feature and may return `UNKNOWN` in edge conditions | Keep feature-gated and retain regression tests (`kenken-solver/tests/z3_golden_verify.rs`) | **Adopted** (upgrade + hardening) |
| `criterion` benchmark stack | `0.5.1/0.8.2` overlap -> `0.8.2` only | Eliminate benchmark major-version duplication and keep maintained harness | Workspace benchmark path consolidated on `criterion 0.8`; overlap-inducing integration removed | Remaining dependency overlap elsewhere (`rand`, `thiserror`, `syn`) still exists | Continue duplicate-graph monitoring via `cargo tree -d --all-features` | **Adopted** (primary), **Resolved** (single-major consolidation) |
| GTK desktop stack (`gtk4`, `gdk4`, `glib`, `pango`, `cairo-rs`) | `0.9/0.20` family -> `0.11/0.22` family | Platform compatibility and maintenance updates | `kenken-ui/Cargo.toml` upgraded; UI modules adjusted/cleaned to satisfy strict gates on new stack | Potential runtime/packaging drift on host systems | Continue strict CI gates and UI-target smoke validation | **Adopted**, **Monitor** |
| `uniffi` | `0.30.0` -> `0.31.0` | Keep mobile binding generator/runtime aligned with maintained upstream | `kenken-uniffi` runtime + build dependency bumps | Generated binding differences across targets | Keep feature-gated (`ffi-uniffi`) and validate via full quality gates | **Adopted** |
| `wasm-bindgen` + `serde-wasm-bindgen` | `0.2.106` -> `0.2.117`; `0.4.5` -> `0.6.5` | Web target compatibility and maintenance | `kenken-wasm` dependency updates; existing handle-based API retained | JS glue/tooling behavior may shift with ecosystem updates | Keep wasm API boundary narrow and monitor through workspace gates | **Adopted**, **Monitor** |
| `rkyv` security patch | `0.8.12` -> `0.8.15` | Security response (`RUSTSEC-2026-0001`) | Lock/workspace now on patched release; dependency audit reflects clean RustSec state | Snapshot/runtime behavior changes possible after patch | Keep `io-rkyv` tests and re-run audit/outdated gates each cycle | **Adopted** |
| Cross-target API parity contract | Undocumented/unenforced parity assumptions -> explicit contract + drift tests | Make UniFFI/WASM overlap intentional, reviewable, and regression-tested | Added `docs/cross_target_api_parity_contract.md`; added `kenken-wasm/tests/cross_target_api_parity_contract.rs` | API divergence risk reappears when new wasm endpoints are added | Require contract+test updates in the same change as surface expansion | **Adopted** |
| ISA artifact registry publication | ISA guidance only -> published artifact registry contract | Ensure reproducible, auditable ISA-tier benchmark publication | Added `docs/isa_benchmark_artifact_registry.md` and linked from target docs | Publication drift if metadata/checks are skipped | Keep explicit verification checklist in registry doc | **Adopted** |
| `kenken-verify` stub burndown | Partial interface stubs -> solver-backed SAT/Z3 agreement APIs | Convert verification crate from placeholder surface to usable integration layer | Implemented `verify_with_sat(...)` and `verify_with_z3(...)`; retained explicit unsupported CNF/SMT2 export errors | External proof-artifact export still unavailable | Keep unsupported behavior explicit until export API is implemented | **Adopted** (core), **Deferred** (export) |

## Command evidence (2026-04-06)
Executed in workspace root:

```bash
cargo audit
cargo outdated --workspace --root-deps-only
cargo outdated --workspace
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Observed results:

- `cargo audit`: no active advisories.
- `cargo outdated --workspace --root-deps-only`: all up to date.
- `cargo outdated --workspace`: all up to date.
- Quality gates (`fmt`, strict `clippy`, full-feature `test`): passing.

## Consistency notes
- Matches `docs/dependency_audit.md`:
  - `rkyv` advisory resolved on `0.8.15`.
  - `criterion` dual-major overlap is resolved (`0.8.2` only).
  - dev-only dual `rand` line accepted/monitored.
  - Z3 cage-constraint hardening and parity-contract/test publication are reflected in debt dispositions.
- Matches `docs/lacunae_audit.md`:
  - modernization outcomes (toolchain pin, clean outdated/audit, strict gates) are now backed by an explicit ADR record.

## Consequences
- Major upgrades are now traceable with explicit tradeoffs and dispositions.
- This cycle closed criterion overlap, parity-contract formalization/testing, and Z3 cage-constraint correctness debt.
- Remaining deferred debt is explicit (notably verification export APIs and SAT strict-certification policy).
- Future re-audits must update this ADR or supersede it with a newer migration ADR.
