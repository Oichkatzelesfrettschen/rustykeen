#!/usr/bin/env bash
set -euo pipefail

# PGO workflow for Rust (safe-by-default).
#
# Requires: `llvm-profdata` on PATH.
#
# Usage:
#   ./scripts/pgo.sh gen
#   ./scripts/pgo.sh train -- cargo run -p kenken-cli -- count --n 2 --desc b__,a3a3 --limit 2
#   ./scripts/pgo.sh use
#
# Notes:
# - This does NOT enable "fast-math" (we keep floating semantics safe/deterministic).
# - For local CPU-tuned builds, set: PGO_RUSTFLAGS_EXTRA="-C target-cpu=native"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGO_DIR="${PGO_DIR:-$ROOT/target/pgo}"
PROFILE_DIR="$PGO_DIR/profiles"
MERGED="$PGO_DIR/merged.profdata"
ARTIFACT_DIR="$PGO_DIR/artifacts"

cmd="${1:-}"
shift || true

case "$cmd" in
  gen)
    rm -rf "$PGO_DIR"
    mkdir -p "$PROFILE_DIR" "$ARTIFACT_DIR"
    (
      cd "$ROOT"
      export LLVM_PROFILE_FILE="$PROFILE_DIR/%m-%p.profraw"
      export RUSTFLAGS="-C profile-generate=$PROFILE_DIR ${PGO_RUSTFLAGS_EXTRA:-}"
      cargo build --release --all-targets --all-features
    )
    ;;

  train)
    if [[ ! -d "$PROFILE_DIR" ]]; then
      echo "missing $PROFILE_DIR; run: $0 gen" >&2
      exit 2
    fi
    if [[ "$#" -eq 0 ]]; then
      echo "usage: $0 train -- <training command>" >&2
      exit 2
    fi
    (
      cd "$ROOT"
      export LLVM_PROFILE_FILE="$PROFILE_DIR/%m-%p.profraw"
      "$@"
    )
    llvm-profdata merge -o "$MERGED" "$PROFILE_DIR"/*.profraw
    ;;

  use)
    if [[ ! -f "$MERGED" ]]; then
      echo "missing $MERGED; run: $0 train -- <cmd>" >&2
      exit 2
    fi
    (
      cd "$ROOT"
      export RUSTFLAGS="-C profile-use=$MERGED -C llvm-args=-pgo-warn-missing-function ${PGO_RUSTFLAGS_EXTRA:-}"
      cargo build --release --all-targets --all-features
    )
    ;;

  *)
    echo "usage: $0 {gen|train|use}" >&2
    exit 2
    ;;
esac

