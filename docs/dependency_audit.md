# Dependency Audit (Cargo + GitHub + GitLab)

Last updated: 2026-04-06

Related decision record: `docs/adr/0005-major-upgrade-decision-record.md`

## Scope and method

This audit covers direct and transitive dependencies for the full workspace (`--all-features`), plus upstream maintenance signals.
Commands were executed with the pinned toolchain (`nightly-2026-04-06` from `rust-toolchain.toml`).

Commands used:

```bash
cargo audit
cargo outdated --workspace --root-deps-only
cargo outdated --workspace
cargo tree -d --all-features
cargo metadata --format-version 1 --all-features
```

External-source checks:
- **GitHub** repository health (archival status, recent push activity, issue volume) for high-impact crates.
- **GitLab** detection for transitive dependencies via `cargo metadata` repository URLs.
- **crates.io** latest stable version checks for key crates.

## Current status

| Check | Result | Notes |
|---|---|---|
| RustSec vulnerabilities | ✅ Clean | `rkyv` updated to `0.8.15`; no active advisories |
| Outdated direct deps | ✅ None | `cargo outdated --workspace --root-deps-only` clean |
| Outdated transitive deps | ✅ None | `cargo outdated --workspace` clean |
| Warnings-as-errors gates | ✅ Passing | `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features` |

## Inventory snapshot (`cargo metadata --format-version 1 --all-features`)

- Resolved package graph: **281** packages total
  - **11** workspace members
  - **270** transitive crates
- Direct external crates declared by workspace manifests: **31**
- Feature-gated optional direct crates (workspace manifests): `bitvec`, `bumpalo`, `dhat`, `fixedbitset`, `mimalloc`, `rayon`, `rkyv`, `serde`, `static_assertions`, `tracing`, `tracing-flame`, `tracing-subscriber`, `varisat`, `z3`
- High-impact feature-gated crates remain: `z3`, `varisat`, `rayon`, `mimalloc`, `rkyv`

## Suitability review of major crates

| Crate | Status | Fit assessment |
|---|---|---|
| `rand` + `rand_chacha` | Keep | Correct choice for deterministic RNG behavior |
| `z3` (feature-gated) | Keep | Appropriate for SMT-backed verification and certification |
| `varisat` (feature-gated) | Keep with watch | API fit is good; upstream is low-velocity, monitor maintenance risk |
| `uniffi` | Keep | Required for Kotlin/Swift bindings; no comparable lower-cost substitute |
| GTK stack (`gtk4`, `glib`, `gdk4`, `pango`, `cairo-rs`) | Keep | Appropriate for desktop UI target |
| `wasm-bindgen` + `serde-wasm-bindgen` | Keep | Correct and standard for web target |
| `rkyv` | Keep (reassess later) | Good for zero-copy snapshots; now on patched release |
| `criterion` | Keep | Consolidated on `0.8.2` with no `0.5.x` overlap in the active graph |

## Focused transitive API map (all-features)

Evidence sources for this pass:

```bash
cargo metadata --format-version 1 --all-features
cargo tree -d --all-features
```

Derived map (from resolved graph):

| Area | Crate(s) | Active version(s) | Transitive closure size* | Reach from workspace members |
|---|---|---|---:|---|
| Solver / verification | `z3` | `0.20.0` | 3 | via `kenken-solver` consumers |
| Solver / verification | `varisat` | `0.2.2` | 34 | via `kenken-solver` consumers |
| Solver / verification | `rkyv` | `0.8.15` | 33 | `kenken-io` only |
| Runtime / platform | `gtk4`/`glib`/`gdk4`/`pango`/`cairo-rs` | `0.11.2`/`0.22.4`/`0.11.2`/`0.22.4`/`0.22.0` | 64 / 40 / 50 / 44 / 42 | `kenken-ui` |
| Runtime / platform | `wasm-bindgen` + `serde-wasm-bindgen` | `0.2.117` + `0.6.5` | 11 / 16 | `kenken-wasm` path |
| Runtime / platform | `uniffi` | `0.31.0` | 90 | `kenken-uniffi` |
| Benchmark / profiling | `criterion` | `0.8.2` | 55 | dev graph (`kenken-solver` benches) |

\*Closure size = count of unique transitive dependencies reachable from that crate node in the resolved graph.

## Overlap and duplication debt

`cargo tree -d --all-features` now shows **no dual-criterion overlap**. Remaining relevant overlap is:

1. `rand`/`rand_chacha`/`rand_core` dual lines (`0.10` runtime path + `0.9` via `proptest` dev path).
2. `thiserror` dual major lines (`1.x` via `varisat`, `2.x` via workspace/uniffi path).
3. `syn` dual major lines (`1.x` and `2.x`) across proc-macro ecosystems.
4. `toml` dual lines (`0.9` and `1.1`) across toolchain ecosystems.

These are not correctness bugs, but they increase compile surface and cognitive load.

## Non-overlap completeness risks (high-impact)

1. **SAT strict certification mode vs permissive mode is now explicit**
   - `kenken-solver/src/sat_cages.rs` exposes:
     - strict fail-closed mode (`puzzle_uniqueness_via_sat_strict`)
     - permissive mode with native fallback (`puzzle_uniqueness_via_sat`)
   - **Action**: keep certification code paths on strict mode and monitor permissive fallback rate in performance workloads.

2. **Proof artifact export API surface is implemented**
   - `kenken-verify/src/sat_interface.rs`: `generate_cnf(...)` emits DIMACS CNF.
   - `kenken-verify/src/z3_interface.rs`: `generate_z3_smt2(...)` emits SMT-LIB2.
   - `kenken-solver` contains export backends and golden equivalence checks.

3. **Serialization backend completeness now includes a non-`rkyv` path**
   - `kenken-io/src/sgt_desc_snapshot.rs` provides SGT text encode/decode as a portable backend.
   - `io-rkyv` remains the binary snapshot path.

## Recently closed completeness items

1. **Z3 uniqueness encoding now includes full cage arithmetic constraints**
   - `kenken-solver/src/z3_verify.rs` encodes Eq/Add/Mul/Sub/Div cage semantics plus alternate-model search, and includes verification tests for each op family.

2. **Cross-target API parity contract is now documented and regression-tested**
   - Contract publication: `docs/cross_target_api_parity_contract.md`
   - Drift checks: `kenken-wasm/tests/cross_target_api_parity_contract.rs`

## Upstream maintenance signals

High-impact repos were checked via GitHub API:

- Actively maintained in 2026 Q1/Q2: `rand`, `thiserror`, `serde`, `tracing`, `criterion-rs/criterion.rs`, `rkyv`, `z3.rs`, `uniffi-rs`, `wasm-bindgen`, `gtk4-rs`, `gtk-rs-core`, `bumpalo`, `rayon`.
- Notably lower velocity: `jix/varisat` (last push 2022-11).

GitLab-hosted transitive deps detected:
- `redox_syscall`
- `version-compare`

No direct workspace dependency currently points to GitLab.

## Dependency debt register

| Class | Finding | Impact | Disposition |
|---|---|---|---|
| Security | `rkyv` advisory previously present (`RUSTSEC-2026-0001`) | UB risk in OOM edge paths | **Resolved** (`rkyv 0.8.15`) |
| Overlap | Criterion overlap consolidation (`criterion 0.8.2` only; no `0.5.x` line) | Reduced dev graph complexity and tooling drift | **Resolved** |
| Overlap | Dual `rand` lines from dev tooling | Dev graph complexity | **Accepted** (low risk, dev-only) |
| Completeness | `kenken-verify` SAT/Z3 uniqueness checks are wired and CNF/SMT2 exports are implemented | Enables external proof artifact workflows | **Resolved** |
| Completeness | `z3_verify` now encodes full cage semantics with alternate-model check | Removed vacuous uniqueness risk in Z3 path | **Resolved** |
| Completeness | Cross-target parity contract + drift tests now present (`kenken-uniffi` vs `kenken-wasm`) | Reduces API drift risk across bindings | **Resolved** |
| Completeness | SAT strict certification mode is implemented while permissive fallback remains available by design | Certification semantics now fail-closed when requested | **Resolved (strict) / Accepted (permissive path)** |
| Completeness | `kenken-io` now includes SGT text backend in addition to `io-rkyv` | Removes single-backend serialization dependency | **Resolved** |
| Supply-chain | `varisat` low-velocity upstream | Medium-term maintenance risk | **Open / monitor** |
| Supply-chain | GitLab transitive nodes | Small governance surface increase | **Accepted** (transitive only) |

## Recommended next actions

1. **SAT mode discipline**: keep verification/certification callsites on strict SAT mode; reserve permissive fallback mode for performance workloads only.
2. **Parity contract maintenance**: require contract+test updates whenever wasm adds `count`/`generate` APIs.
3. **Quarterly supply-chain sweep**: rerun the command set above and refresh this document.

## Reproducible re-audit command set

```bash
cargo audit
cargo outdated --workspace --root-deps-only
cargo outdated --workspace
cargo tree -d --all-features
cargo metadata --format-version 1 --all-features
```
