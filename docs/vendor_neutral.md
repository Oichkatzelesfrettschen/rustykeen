# Vendor-Neutral Strategy (2025-12-31T22:07:20Z)

## Principles
- Core is platform-agnostic: pure Rust, no_std-ready, deterministic, stable API; const-generics Grid, SIMD-friendly.
- Adapters per platform: JVM/Swift via UniFFI, C ABI for native toolkits, WASM for web; feature-gated modules.
- No platform logic in core; UI state consumed as plain data.

## Targets
- Linux/BSD/illumos/Windows: CLI/TUI or native UI via C ABI.
- Android: Kotlin/Compose via UniFFI; optional Vulkan.
- iOS/macOS/tvOS: Swift via UniFFI.
- Web: WASM via wasm-bindgen.

## Packaging & CI
- Feature gates per adapter (see feature_gating.md); CI matrices build all; reproducible outputs; rkyv for cross-platform states; global allocator via mimalloc feature.
