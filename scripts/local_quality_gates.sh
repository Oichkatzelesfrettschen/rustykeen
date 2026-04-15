#!/usr/bin/env bash
set -euo pipefail

# Local correctness/quality gate bundle used in place of hosted CI.
# Run from repo root. This script is intentionally the canonical local gate
# because hosted GitHub Actions workflows are disabled for quota control.

usage() {
  cat <<'EOF'
usage: ./scripts/local_quality_gates.sh [--audit-only]

Runs the canonical local gate bundle for rustykeen.

Options:
  --audit-only   run only dependency/security freshness checks
EOF
}

require_cmd() {
  local cmd="$1"
  local hint="$2"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "missing required command: $cmd" >&2
    echo "hint: $hint" >&2
    exit 2
  fi
}

mode="full"
case "${1:-}" in
  "")
    ;;
  --audit-only)
    mode="audit"
    ;;
  -h|--help)
    usage
    exit 0
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac

require_cmd cargo "install Rust and ensure cargo is on PATH"

if [[ "$mode" == "full" ]]; then
  cargo fmt --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-targets --all-features
fi

if cargo audit --version >/dev/null 2>&1; then
  cargo audit
else
  echo "missing cargo-audit; install with: cargo install cargo-audit" >&2
  exit 2
fi

if cargo outdated --version >/dev/null 2>&1; then
  cargo outdated --workspace --root-deps-only
  cargo outdated --workspace
else
  echo "missing cargo-outdated; install with: cargo install cargo-outdated" >&2
  exit 2
fi
