# UniFFI Codegen Tasks (2025-12-31T22:03:44Z)

## Generate Kotlin bindings
- Command: uniffi-bindgen generate docs/uniffi.udl --language kotlin --out-dir app/src/main/java
- Include in Gradle task `generateKeenBindings` before `assemble`.

## Generate Swift bindings (optional)
- Command: uniffi-bindgen generate docs/uniffi.udl --language swift --out-dir ios/Sources/KeenBindings

## Rust integration
- Add build.rs in android-uniffi crate to run codegen if needed; prefer Gradle task for Android.

## Packaging
- Ensure generated Kotlin files are part of source set; .so from cargo-ndk placed in app/src/main/jniLibs.
