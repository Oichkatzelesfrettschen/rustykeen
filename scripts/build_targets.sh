#!/usr/bin/env bash
set -euo pipefail

# Convenience build entrypoints for “portable baseline” vs “tuned” artifacts.
#
# This script does NOT install toolchains/targets for you; it assumes you've done:
#   rustup target add aarch64-unknown-linux-gnu aarch64-linux-android
# and for Android you have the NDK + cargo-ndk installed.

cmd="${1:-}"
shift || true

case "$cmd" in
  linux-x86-v1)
    RUSTFLAGS="-C target-cpu=x86-64-v1 ${RUSTFLAGS_EXTRA:-}" \
      cargo build --release -p kenken-cli --all-features
    ;;

  linux-x86-v3)
    RUSTFLAGS="-C target-cpu=x86-64-v3 ${RUSTFLAGS_EXTRA:-}" \
      cargo build --release -p kenken-cli --all-features
    ;;

  linux-aarch64)
    cargo build --release -p kenken-cli --all-features --target aarch64-unknown-linux-gnu
    ;;

  android-arm64)
    if ! command -v cargo-ndk >/dev/null 2>&1; then
      echo "missing cargo-ndk; install with: cargo install cargo-ndk" >&2
      exit 2
    fi
    cargo ndk -t arm64-v8a build --release -p kenken-cli --all-features
    ;;

  *)
    echo "usage: $0 {linux-x86-v1|linux-x86-v3|linux-aarch64|android-arm64}" >&2
    exit 2
    ;;
esac

