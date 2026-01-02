// Top-level build file for the KenKen Android example app

plugins {
    id("com.android.application") version "8.1.4" apply false
    kotlin("android") version "1.9.20" apply false
}

tasks.register("clean", Delete::class) {
    delete(rootProject.buildDir)
}

// Configuration notes:
// - Requires Android SDK API 21+ (Android 5.0)
// - Requires NDK r25c or later for native library compilation
// - Uses cargo-ndk for Rust cross-compilation to Android
//
// Build the native libraries before building the app:
//   cargo ndk -t arm64-v8a -t armeabi-v7a build --release \
//     -p kenken-uniffi --all-features
