#!/usr/bin/env bash
set -euo pipefail

# BOLT workflow for Rust (post-link optimization).
#
# Requires: `llvm-bolt`, `perf`, `perf2bolt` on PATH.
#
# This script assumes you've already built a release binary and have a training
# workload you can run under `perf record`.
#
# Usage (example):
#   cargo build --release -p kenken-cli
#   ./scripts/bolt.sh record -- ./target/release/kenken-cli count --n 2 --desc b__,a3a3 --limit 2
#   ./scripts/bolt.sh optimize ./target/release/kenken-cli

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BOLT_DIR="${BOLT_DIR:-$ROOT/target/bolt}"
PERF_DATA="$BOLT_DIR/perf.data"
FDATA="$BOLT_DIR/perf.fdata"

cmd="${1:-}"
shift || true

mkdir -p "$BOLT_DIR"

case "$cmd" in
  record)
    if [[ "$#" -eq 0 ]]; then
      echo "usage: $0 record -- <training command>" >&2
      exit 2
    fi
    perf record -o "$PERF_DATA" --call-graph lbr -- "$@"
    ;;

  optimize)
    bin="${1:-}"
    if [[ -z "$bin" ]]; then
      echo "usage: $0 optimize <path-to-binary>" >&2
      exit 2
    fi
    if [[ ! -f "$PERF_DATA" ]]; then
      echo "missing $PERF_DATA; run: $0 record -- <cmd>" >&2
      exit 2
    fi
    perf2bolt -p "$PERF_DATA" -o "$FDATA" "$bin"
    out="$BOLT_DIR/$(basename "$bin").bolt"
    llvm-bolt "$bin" -o "$out" -data="$FDATA" \
      -reorder-blocks=ext-tsp -reorder-functions=hfsort \
      -split-functions -split-all-cold -dyno-stats
    echo "$out"
    ;;

  *)
    echo "usage: $0 {record|optimize}" >&2
    exit 2
    ;;
esac

