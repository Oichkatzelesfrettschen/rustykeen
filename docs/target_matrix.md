# Target matrix (2025/2026) + gaps in assumptions

This project targets **portable, vendor-neutral Rust** with optional performance-tuned builds.

## Key clarification: Rust target triples vs micro-architecture levels
Rust uses *target triples* like:
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `aarch64-linux-android`

Your statements mention “`x86_64v1` / `x86_64v3`”. Those are **x86-64 micro-architecture levels** (often written `x86-64-v1` / `x86-64-v3`),
not Rust target triples. In Rust these are selected via codegen flags, e.g.:
- `-C target-cpu=x86-64-v1`
- `-C target-cpu=x86-64-v3`

Implication (important):
- A binary compiled for `x86-64-v3` may crash with `Illegal instruction` on `x86-64-v1` machines.
- Therefore, “support v1 + use v3” requires **either**:
  1) shipping multiple artifacts (v1 and v3), **or**
  2) compiling a v1 baseline and using **runtime dispatch** (`is_x86_feature_detected!`) for v3 hotpaths.

## Linux: x86_64
Recommended:
- Baseline artifact: `x86_64-unknown-linux-gnu` with `-C target-cpu=x86-64-v1`
- Optional tuned artifact: same triple, but `-C target-cpu=x86-64-v3`

Why not set `target-cpu=native` by default?
- It produces non-portable binaries (depends on the builder machine).

## Linux: aarch64 (“armv8/armv9”)
Lacuna in the original phrasing:
- “armv8” and “armv9” are **architecture generations**, but Rust/LLVM tuning typically uses either:
  - a specific CPU (`-C target-cpu=neoverse-n1`, `cortex-a76`, etc.), or
  - specific features (`-C target-feature=+sve,+sve2,+dotprod`, etc.).

Reality constraint:
- On aarch64, many features are baseline (NEON), but **SVE/SVE2 is not universal** across all ARMv8/ARMv9 devices.
- So a single “armv9 optimized” binary is not universally safe unless you can guarantee that deployment CPUs support those features.

Recommendation:
- Baseline artifact: `aarch64-unknown-linux-gnu` with `-C target-cpu=generic`
- Optional tuned artifacts: per-datacenter/board CPU (e.g. Neoverse) or feature-tiered builds (if you have a deployment contract).

## Android: aarch64
Primary Rust target:
- `aarch64-linux-android`

Best practice:
- Use `cargo-ndk` (or the NDK toolchain directly) to ensure correct linker/AR selection and ABI.
- Keep baseline assumptions conservative; Android devices vary widely.

Note on BOLT:
- BOLT is primarily a Linux ELF post-link optimizer that depends on perf profiles; it is typically most practical on Linux/x86_64.
- For Android, PGO is usually the higher-return, easier path (instrument → run on device/emulator → merge → rebuild).

## Determinism vs “fast math”
We intentionally avoid enabling LLVM “fast-math” globally:
- It can change floating semantics and break cross-platform determinism.
- Difficulty scoring can use fixed-point (`fixed`) if we need deterministic non-integer math.

Vectorization:
- `opt-level=3` enables LLVM’s vectorizers by default.
- Enabling `target-cpu`/`target-feature` increases what SIMD instructions LLVM is allowed to emit.

## Practical build commands (today)
Portable release:
- `cargo build --release --all-features`

Linux x86_64 baseline v1:
- `RUSTFLAGS="-C target-cpu=x86-64-v1" cargo build --release -p kenken-cli --all-features`

Linux x86_64 tuned v3 (separate artifact):
- `RUSTFLAGS="-C target-cpu=x86-64-v3" cargo build --release -p kenken-cli --all-features`

Android arm64 (requires NDK + cargo-ndk):
- `cargo ndk -t arm64-v8a build --release -p kenken-cli --all-features`

