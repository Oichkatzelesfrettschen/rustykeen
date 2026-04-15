# Final Consolidated Debt Register (Modernization Cycle)

Last updated: 2026-04-06

This is the final cross-class debt register for the modernization cycle. It summarizes disposition and evidence without duplicating detailed analysis in `docs/dependency_audit.md` and `docs/adr/0005-major-upgrade-decision-record.md`.
Session todo snapshot before publication: all modernization todos except this register were already marked `done` in the session tracker (evidence query listed below).

## Evidence baseline

Commands executed from repo root for this final register:

```bash
# session progress evidence
SELECT id, status FROM todos ORDER BY created_at;

# modernization validation evidence
cargo audit
cargo outdated --workspace --root-deps-only
cargo outdated --workspace
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
./docs/literature/verify_corpus.sh
```

Primary evidence files:

- `docs/dependency_audit.md`
- `docs/adr/0005-major-upgrade-decision-record.md`
- `docs/literature/README.md`, `docs/literature/sources.csv`, `docs/literature/provenance_manifest.csv`
- `rust-toolchain.toml`, `Dockerfile`, `Cargo.lock`
- `kenken-verify/src/z3_interface.rs`, `kenken-verify/src/sat_interface.rs`
- `kenken-solver/src/z3_verify.rs`, `kenken-solver/src/sat_cages.rs`

## Consolidated debt register

| Debt class | Item | Disposition | Evidence references (files + commands) | Notes |
|---|---|---|---|---|
| Engineering | Strict all-feature quality gates are enforced in local validation | **Resolved** | `cargo fmt --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-targets --all-features` | Local gate discipline retained after CI/CD removal |
| Engineering | CI/CD workflows are intentionally disabled to control account quota consumption | **Resolved** | `test ! -f .github/workflows/ci.yml`; `git status --short` | Hosted runner usage removed; local validation remains mandatory |
| Architecture | ISA target guidance now includes a reproducible artifact/result registry | **Resolved** | `docs/isa_benchmark_artifact_registry.md`; `docs/target_matrix.md`; `rg -n "artifact registry|linux-x86-64-v1|linux-x86-64-v3|linux-aarch64-generic" docs/isa_benchmark_artifact_registry.md docs/target_matrix.md` | Registry now defines layout, metadata, and verification checks |
| Design | Cross-target API parity contract (`kenken-uniffi` vs `kenken-wasm`) is formalized and regression-tested | **Resolved** | `docs/cross_target_api_parity_contract.md`; `kenken-wasm/tests/cross_target_api_parity_contract.rs`; `cargo test --all-targets --all-features` | Contract + drift checks are now part of the test suite |
| Implementation | `kenken-verify` SAT/Z3 verification APIs now include CNF/SMT2 proof artifact export surfaces | **Resolved** | `kenken-verify/src/z3_interface.rs`; `kenken-verify/src/sat_interface.rs`; `rg -n "generate_cnf|generate_z3_smt2|CnfExportFailed|Smt2ExportFailed" kenken-verify/src/sat_interface.rs kenken-verify/src/z3_interface.rs` | Export API surface now supports external reproducible verification workflows |
| Implementation | `z3_verify` now encodes full cage constraints (Eq/Add/Mul/Sub/Div) and checks alternate models | **Resolved** | `kenken-solver/src/z3_verify.rs`; `kenken-solver/tests/z3_golden_verify.rs`; `rg -n "Full cage arithmetic constraints|Op::Eq|Op::Add|Op::Mul|Op::Sub|Op::Div" kenken-solver/src/z3_verify.rs` | Removes prior vacuous-uniqueness risk in Z3 path |
| Implementation | SAT verifier now supports strict fail-closed certification mode while preserving permissive fallback mode | **Resolved (strict) / Accepted (permissive)** | `kenken-solver/src/sat_cages.rs`; `rg -n "puzzle_uniqueness_via_sat_strict|SAT_TUPLE_THRESHOLD|native fallback" kenken-solver/src/sat_cages.rs` | Certification mode no longer depends on native fallback |
| Toolchain/dependency/security | Toolchain pin normalization completed (`nightly-2026-04-06`) | **Resolved** | `rust-toolchain.toml:2`; `Dockerfile:4`; `rg -n "nightly-2026-04-06" rust-toolchain.toml Dockerfile README.md CONTRIBUTING.md` | Reproducibility baseline established |
| Toolchain/dependency/security | RustSec + freshness debt cleared (`rkyv` on `0.8.15`, outdated checks clean) | **Resolved** | `docs/dependency_audit.md:31-33,136`; `Cargo.lock:1895-1896`; `cargo audit`; `cargo outdated --workspace --root-deps-only`; `cargo outdated --workspace` | Security/freshness debt closed for this cycle |
| Toolchain/dependency/security | Criterion overlap consolidation completed (`criterion 0.8.2` only) | **Resolved** | `Cargo.lock:356-377`; `cargo tree -d --all-features`; `cargo tree -d --all-features | rg -n "criterion v0\\.8\\.2|criterion v0\\.5"` | Removes dual-major benchmark stack debt |
| Toolchain/dependency/security | `varisat` upstream velocity remains low | **Accepted** | `docs/dependency_audit.md:51,123,142`; `cargo metadata --format-version 1 --all-features` | Monitor quarterly as supply-chain risk |
| Toolchain/dependency/security | Literature/PDF provenance corpus now present and integrity-verifiable | **Resolved** | `docs/literature/README.md:23,42`; `docs/literature/sources.csv`; `docs/literature/provenance_manifest.csv`; `./docs/literature/verify_corpus.sh` | Closes prior provenance/evidence gap |

## Explicit open items remaining after this modernization cycle

### Blocked

None.

### Deferred

None.

### Accepted (monitor)

1. Keep CI/CD disabled while account quota limits remain; re-enable only with an explicit budgeted workflow policy.
2. Keep SAT native fallback behavior only for performance mode; keep verification/certification callsites on strict mode.
3. Monitor low-velocity `varisat` upstream health in recurring dependency audits.

## Consistency check

- Dependency and security dispositions align with `docs/dependency_audit.md`.
- Major-upgrade and defer/monitor decisions align with `docs/adr/0005-major-upgrade-decision-record.md`.
- Literature evidence/provenance status aligns with `docs/literature/README.md`, `docs/literature/sources.csv`, and `docs/literature/provenance_manifest.csv`.
