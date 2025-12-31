# Rust-Native Build System (Cargo) Audit and Solutions (2025-12-31T22:20:44Z)

## Answer: Makefile is not Rust-native
- Rust’s native build system is Cargo (package manager + build orchestrator). Makefiles are auxiliary for tooling (PGO/BOLT), not primary.

## Lacunae
- Cross-platform targets (Android/iOS) not first-class in Cargo; need cargo-ndk/mobile2.
- Feature gating for vendor-neutral adapters inconsistently defined.
- Profiles and toolchain flags scattered; missing .cargo/config.toml guidance.
- Codegen/perf tooling (PGO/BOLT/mold) outside Cargo defaults.
- Telemetry initialization not standardized across crates/workspace.

## Resolutions
- Use Cargo workspace to centralize members, profiles, features.
- Add .cargo/config.toml for per-target rustflags (mold, emit-relocs, NEON).
- Gate adapters with features: android, ios, wasm, c_abi, simd, mimalloc.
- Integrate cargo-ndk and cargo-mobile2 for platform builds; keep core Cargo-native.
- Provide build.rs for per-target linker args (lld, emit-relocs).
- Standardize tracing init via a shared crate module.

## Sample Workspace Layout
- kenken-core, kenken-solver, kenken-gen, kenken-io, kenken-cli, android-uniffi, wasm.

## Guidance
- Use Cargo profiles from riced_build.md.
- Use feature_gating.md patterns.
- Keep Makefile optional for PGO/BOLT only.
