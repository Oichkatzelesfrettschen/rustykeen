# cargo-mobile2 Setup (2025-12-31T22:17:59.005Z)

## Install
```bash
cargo install cargo-mobile2
```

## Initialize Android project
```bash
cargo mobile init android --package com.example.kenken --app-name "KenKen"
```

## Initialize iOS project
```bash
cargo mobile init ios --bundle-id com.example.kenken --app-name "KenKen"
```

## Build and run
```bash
# Android
cargo mobile run android --device auto --release
# iOS (simulator)
cargo mobile run ios --device simulator --release
```

## UniFFI hooks (Android/iOS)
- Define a pre-build hook to generate bindings:
```bash
uniffi-bindgen generate docs/uniffi.udl --language kotlin --out-dir android/app/src/main/java
uniffi-bindgen generate docs/uniffi.udl --language swift  --out-dir ios/Sources/KeenBindings
```
- Ensure Gradle/Xcode include generated sources; add build phase in Xcode:
  - Run Script: `uniffi-bindgen generate docs/uniffi.udl --language swift --out-dir "$SRCROOT"/Sources/KeenBindings`
- Link Rust outputs:
  - Android: cargo-ndk builds .so into app/src/main/jniLibs; Gradle picks up libs.
  - iOS: cargo-mobile2 produces staticlib; link in Xcode target.

## Notes
- Use feature gates (android/ios) to select platform sinks for tracing.
- Combine with cargo-ndk for fine-grained Android ABIs.
