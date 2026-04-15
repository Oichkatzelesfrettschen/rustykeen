set shell := ["bash", "-euo", "pipefail", "-c"]

default:
  @just --list

fmt:
  cargo fmt --check

lint:
  cargo clippy --all-targets --all-features -- -D warnings

test:
  cargo test --all-targets --all-features

audit:
  ./scripts/local_quality_gates.sh --audit-only

ci:
  ./scripts/local_quality_gates.sh

fuzz-check:
  cargo test --manifest-path fuzz/Cargo.toml

fuzz-outdated:
  cargo outdated --manifest-path fuzz/Cargo.toml --root-deps-only

fuzz-audit:
  cargo audit -f fuzz/Cargo.lock

bench:
  cargo bench -p kenken-solver --bench solver_smoke

profile-bench bench="simd_effectiveness":
  ./scripts/profile_benchmarks.sh {{bench}}

profile-solver:
  ./scripts/profile_solver.sh

profile-tier22 out="":
  if [[ -n "{{out}}" ]]; then ./scripts/profile_tier22_scaling.sh "{{out}}"; else ./scripts/profile_tier22_scaling.sh; fi

build-target target:
  ./scripts/build_targets.sh {{target}}

pgo cmd:
  ./scripts/pgo.sh {{cmd}}

bolt cmd *args:
  ./scripts/bolt.sh {{cmd}} {{args}}
