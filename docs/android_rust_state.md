# Rust on Android: State of the Union (2025→2026)

## Platform vs App Divide
- AOSP: Rust is first-class; major subsystems (Bluetooth, Keystore, UWB) now primarily Rust; memory safety vulns <20% by late 2025.
- Apps/NDK: Rust is BYO build; integrate Cargo via Gradle plugins; no first-party Android Studio Rust support.

## App Architecture Standard (2026)
- Rust + UniFFI for Kotlin bindings; avoid manual JNI.
- Build: mozilla-rust-android-gradle + cargo-ndk; targets aarch64/x86_64; publish .so into AAB.

## Recommended Crates
- FFI/glue: uniffi, ndk, ndk-glue, jni (fallback), anyhow/thiserror.
- Concurrency/Perf: rayon (optional), parking_lot, smallvec, fixedbitset, bitvec, itertools, rand_chacha, portable-simd (nightly), tracing, tracing-android.
- IO: serde, serde_json, toml; insta (snapshots), proptest, criterion.
- Graphics: wgpu (UI/tool), ash + volk (Vulkan heavy) when needed.

## Performance & Safety
- Throughput parity with C++; borrow checker often improves alias analysis for LLVM.
- Concurrency safety at compile time; sanitize in debug (HWASan/MTE on Android).

## Guidance by Scenario
- New native logic: Rust + UniFFI.
- Game engine: consider C++ AGDK; Rust (Bevy/wgpu) viable but ecosystem thinner.
- Legacy port: keep C++; bridge via cxx.
- UI: Kotlin/Compose; call Rust for logic only.
