# Android Build Skeleton: Gradle + Cargo + UniFFI (2025-12-31T22:02:45.744Z)

## Gradle (build.gradle.kts snippet)
plugins {
  id("org.mozilla.rust-android-gradle") version "0.9.3"
}

rustAndroid {
  module("keen-core") {
    crateDir.set(file("../../rust"))
    targets.set(listOf("arm64-v8a","x86_64"))
    profile.set("release")
    extraCargoArgs.set(listOf("--features","android"))
  }
}

dependencies {
  implementation("net.java.dev.jna:jna:5.14.0")
}

## Cargo.toml (workspace excerpt)
[workspace]
members = ["kenken-core","kenken-solver","kenken-gen","kenken-io","android-uniffi"]

[profile.release]
lto = "thin"
codegen-units = 1

## UniFFI build
- Generate Kotlin bindings in Gradle task:
  - run uniffi-bindgen generate docs/uniffi.udl --language kotlin --out-dir app/src/main/java
- Package .so from rustAndroid plugin outputs into AAB.

## NDK targets
- Use cargo-ndk: cargo ndk -t arm64-v8a -t x86_64 -o ./app/src/main/jniLibs build --release

## Manifest
- Enable MTE: <application android:memtagMode="async" />
