// docs/build.rs
// Inject Android-specific flags for BOLT compatibility (emit-relocs, frame pointers) and linker.
fn main() {
    let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let triple = std::env::var("TARGET").unwrap_or_default();
    if triple.contains("aarch64-linux-android") {
        println!("cargo:rustc-link-arg=-Wl,--emit-relocs");
        println!("cargo:rustc-cfg=android_build");
        println!("cargo:rustc-env=RUSTFLAGS=-C force-frame-pointers=yes");
    }
    // Prefer lld on release
    let profile = std::env::var("PROFILE").unwrap_or_default();
    if profile == "release" {
        println!("cargo:rustc-link-arg=-fuse-ld=lld");
    }
}
