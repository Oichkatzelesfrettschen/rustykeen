# Repo Debt Audit And Remediation Plan

Date: 2026-04-08

## Scope

This document audits repo debt across:

- technical debt
- repository and structural debt
- `.gitignore` debt
- build infrastructure debt
- build-folder and artifact debt
- organizational and documentation debt

The goal is to define a practical recovery plan aligned with Rust repository best practices as of April 2026:

- one clear source of truth for build and quality gates
- deterministic local workflows
- local-only merge protection via canonical gate discipline
- clean artifact boundaries
- repo-root hygiene
- canonical docs separated from historical research
- small, explainable workspace topology

## Executive Summary

The codebase is in better shape than the repo shape.

- The Rust workspace is largely current on dependency freshness and has documented local quality gates.
- The repository operational surface is fragmented across many docs and ad hoc scripts.
- Build, profiling, and artifact workflows do not yet have one canonical entrypoint.
- The repo root contains tracked operational residue that should be treated as artifacts, not source.
- `.gitignore` mixes good ignores with broad path rules that can hide intended source trees.
- Hosted CI is intentionally disabled, so merge protection must come from explicit local gate discipline and contributor policy.
- The docs tree is too large and mixed-purpose to serve as a reliable source of truth without pruning and categorization.

## Findings

### 1. Workspace And Dependency Posture

- The main workspace dependency surface is largely up to date.
- The excluded fuzz package is not covered by the root workspace freshness posture.
- Duplicate dependency lines still exist in the resolved graph, especially for `rand`, `thiserror`, and `toml`.
- The workspace excludes `fuzz`, which makes the top-level repo health story incomplete.

Why this matters:

- best practice in 2026 is to audit the whole repo, not only the primary workspace
- excluded packages drift silently
- duplicate major lines increase compile surface and review complexity

### 2. Root-Level Repo Hygiene

- The repo root contains tracked build or crash residue:
  - `baseline_results_v2.txt`
  - `pgo_results.txt`
  - `rustc-ice-*`
- There is a live `logs/` directory at repo root.
- There is a live `target/` directory at repo root, plus workflow-specific subtrees under it.
- There is no dedicated `artifacts/` top-level convention for reproducible generated outputs.

Why this matters:

- repo root should contain source, policy, and entrypoints, not ephemeral operational debris
- tracked crash artifacts are almost never appropriate as root-level permanent files
- artifact placement should be intentional and documented

### 3. Build Infrastructure Debt

- There is no `justfile`, `Makefile`, `Taskfile.yml`, or equivalent task runner at repo root.
- Build and profiling workflows are split across shell scripts in `scripts/`.
- Script quality is inconsistent:
  - some scripts use `bash` strict mode well
  - some use plain `#!/bin/bash` and looser conventions
  - some write into `target/`, others imply different artifact layouts
- There is no unified command taxonomy such as `fmt`, `lint`, `test`, `audit`, `bench`, `profile`, `dist`.

Why this matters:

- modern Rust repos benefit from one discoverable task entrypoint
- scripts should be implementation detail, not the primary public interface
- reproducibility gets weaker when output locations and prerequisites vary by script

### 4. Local Gate Governance Debt

- The only tracked workflow file is `.github/workflows/ci.yml`, and it is currently deleted in the worktree.
- Documentation states CI was removed to control runner quota.
- The repo currently relies on local discipline, but that discipline was not yet fully consolidated into one canonical local entrypoint before this cleanup.

Why this matters:

- with hosted workflows off-limits, the repo needs stronger local governance:
  - one canonical pre-merge command
  - clear contributor policy
  - reproducible local task entrypoints

### 5. `.gitignore` Debt

- Root `.gitignore` is large and mixes several classes of rules.
- It ignores `/third_party/` and `/vendor/` broadly, while the repo already contains a `third_party/` tree.
- It ignores `/logs/`, but the repo root currently contains a `logs/` directory.
- It contains path-specific legacy rules tied to directories that do not appear canonical anymore.
- Ignore policy is not documented as a design decision, only as raw patterns.

Why this matters:

- broad path ignores can hide intended source additions
- tracked-but-ignored directories create confusing git behavior
- `.gitignore` should reflect a deliberate repo layout, not accumulated history

### 6. Artifact And Build-Folder Debt

- Profiling and optimization outputs are split across:
  - `target/bolt`
  - `target/pgo`
  - `target/flamegraphs`
  - `target/profiling_results/...`
  - root text files
  - `logs/`
- The docs define one artifact registry for ISA benchmarking, but that discipline is not generalized repo-wide.
- There is no cross-workflow artifact retention policy.

Why this matters:

- artifacts should be either:
  - ephemeral and ignored under `target/` or `.tmp/`
  - durable and stored under a canonical `artifacts/` or `evidence/` tree with metadata
- mixing the two causes clutter and weakens reproducibility

### 7. Organizational And Structural Debt

- The docs tree is very large and mixed:
  - canonical guidance
  - historical optimization journals
  - sketches
  - deprecated material
  - research notes
  - implementation plans
- `docs/README.md` still presents itself as a coherent index, but the volume now exceeds what a flat catalog can support.
- `docs/README.md` says `docs/` contains 81 files in one place of the repo narrative, but the actual docs tree is much larger.
- `docs/` contains example `.rs` files such as `docs/build.rs`, `docs/global_allocator.rs`, `docs/keen_engine.rs`, and `docs/solve_dlx.rs`, which blurs the line between canonical source and illustrative sketches.
- There are many root and crate README files, but no clear content ownership model.

Why this matters:

- documentation should distinguish:
  - canonical policy
  - current architecture
  - operational runbooks
  - historical research
  - deprecated archive
- mixing these increases onboarding cost and causes stale guidance

### 8. Build Reproducibility And Container Debt

- `Dockerfile` starts from `rust:latest` instead of a pinned toolchain image or pinned digest.
- The Docker flow installs tools eagerly and builds all targets in the image build stage.
- The Dockerfile is useful, but not yet minimal, reproducible, or cache-efficient by current best practices.

Why this matters:

- best practice in 2026 is pinned base images, multi-stage builds, and explicit goals:
  - dev container
  - CI container
  - release builder
- `latest` weakens supply-chain reproducibility

## Target State

The desired repo shape is:

- repo root contains only source entrypoints, policy, and a small set of canonical files
- one task runner is the public interface for common developer workflows
- shell scripts remain, but only as lower-level helpers
- all quality gates are callable through one command set
- local gate policy is explicit and enforced socially through contributor workflow
- every generated file is either ephemeral under `target/` or durable under a documented `artifacts/` tree
- `.gitignore` is short, principled, and layout-aware
- docs are split into `canonical`, `runbooks`, `research`, and `archive`
- fuzz, examples, wasm, mobile, and profiling workflows are all represented in the repo health story

## Remediation Plan

## Phase 0: Stop Creating More Debt

Time horizon: 1 to 2 days

Actions:

- Freeze new root-level artifact additions unless they go under a canonical artifact location.
- Stop committing crash outputs, ad hoc benchmark dumps, and profiling residue at repo root.
- Treat `docs/` additions as requiring category placement: canonical, runbook, research, or archive.
- Require all new build automation to hang off one top-level task entrypoint.

Acceptance criteria:

- no new root-level operational files appear in normal development
- contributor guidance explicitly bans root artifact sprawl

## Phase 1: Establish Canonical Repo Operations

Time horizon: 2 to 4 days

Actions:

- Introduce a root `justfile` as the canonical developer entrypoint.
- Keep shell scripts in `scripts/`, but invoke them via named `just` tasks.
- Standardize task names:
  - `just fmt`
  - `just lint`
  - `just test`
  - `just audit`
  - `just ci`
  - `just bench`
  - `just profile`
  - `just fuzz`
  - `just wasm`
  - `just android`
- Make `just ci` the canonical local pre-merge command.
- Make `scripts/local_quality_gates.sh` either:
  - the implementation behind `just ci`, or
  - obsolete and removable

Acceptance criteria:

- one README section documents the full contributor command surface
- every major script is reachable through `just`
- no operational doc tells contributors to memorize many separate script names

## Phase 2: Clean Up Repo Root And Artifact Placement

Time horizon: 1 to 3 days

Actions:

- Remove or relocate tracked root operational artifacts:
  - `baseline_results_v2.txt`
  - `pgo_results.txt`
  - `rustc-ice-*`
- Move durable benchmark/profiling evidence to a canonical tree such as:
  - `artifacts/benchmarks/...`
  - `artifacts/profiling/...`
  - `artifacts/crash/...`
- Keep ephemeral outputs under:
  - `target/...`
  - `.tmp/...`
- Decide whether `logs/` should exist at all.
- If logs are useful, move them under `artifacts/logs/` or `.tmp/logs/` based on retention intent.

Acceptance criteria:

- repo root contains no generated `.txt` evidence files except explicitly approved canonical files
- artifact placement rules are documented and enforced in `.gitignore`

## Phase 3: Rationalize `.gitignore`

Time horizon: 1 to 2 days

Actions:

- Rewrite `.gitignore` around layout ownership rather than historical accretion.
- Split sections clearly:
  - cargo and rust outputs
  - profiling and fuzz outputs
  - editor and OS noise
  - local env files
  - temporary evidence outputs
- Remove broad ignores that conflict with intended tracked trees, especially:
  - `/third_party/`
  - `/vendor/`
  - any path that is intended to contain curated source or provenance material
- Add comments for each top-level path rule explaining the retention policy.
- Keep fuzz-specific ignores local to `fuzz/.gitignore` when possible.

Acceptance criteria:

- tracked directories are not also broadly ignored
- `git status --ignored` is understandable to a new contributor
- `.gitignore` reflects the intended repo layout, not old experiments

## Phase 4: Bring Build Infrastructure Up To Current Practice

Time horizon: 3 to 5 days

Actions:

- Define build classes explicitly:
  - dev
  - release
  - verification
  - profiling
  - platform builds
- Document which workflows are supported and stable versus experimental.
- Normalize script behavior:
  - `#!/usr/bin/env bash`
  - `set -euo pipefail`
  - consistent output directories
  - prerequisite checks up front
  - machine-readable metadata where artifacts are durable
- Ensure all profiling scripts write either to `target/` or canonical `artifacts/`.
- Consider moving complex operational logic into:
  - `xtask`
  - or a small Rust tool crate
  when shell becomes brittle

Recommendation:

- Use `just` for orchestration.
- Use shell only for narrow OS-facing glue.
- Use `xtask` for logic-heavy repo automation.

Acceptance criteria:

- build and profile workflows have one predictable interface
- every script has a declared output contract
- no workflow silently writes durable outputs into ad hoc locations

## Phase 5: Strengthen Local-Only Merge Discipline

Time horizon: 2 to 4 days

Actions:

- Make `just ci` the required pre-merge command in contributor-facing docs.
- Keep local gate scope explicit:
  - `just fmt`
  - `just lint`
  - `just test`
  - `just audit`
- Add optional local spot-check tasks for fuzz and supported peripheral targets.
- Document that no hosted GitHub Actions workflows should be added without an explicit quota-policy change.
- Prefer lightweight local automation and reproducible scripts over remote workflow expansion.

Acceptance criteria:

- contributor-facing docs all point to the same canonical local gate
- no documentation recommends adding hosted workflows by default
- local gate discipline is explicit and consistent across the repo

## Phase 6: Fold Fuzz And Peripheral Targets Into Repo Health

Time horizon: 2 to 3 days

Actions:

- Decide whether `fuzz/` should remain excluded from the workspace.
- If excluded, add explicit maintenance tasks:
  - `just fuzz-check`
  - `just fuzz-update`
  - `just fuzz-test`
- Audit fuzz dependencies separately as part of the debt program.
- Ensure WASM, SDL2, GTK4, UniFFI, and Android are represented as named support tiers:
  - required
  - supported but optional
  - experimental

Acceptance criteria:

- the repo health statement covers more than the main workspace
- excluded packages cannot drift unnoticed

## Phase 7: Restructure Docs Around Canonicality

Time horizon: 5 to 8 days

Actions:

- Create explicit doc categories:
  - `docs/canonical/`
  - `docs/runbooks/`
  - `docs/research/`
  - `docs/archive/`
- Move historical optimization journals and one-off analyses into `docs/archive/` or `docs/research/`.
- Keep only durable, current guidance in `docs/canonical/`.
- Move example `.rs` sketches out of the docs root into:
  - `docs/examples/`
  - or `examples/`
  depending on whether they are illustrative or runnable
- Replace the flat docs index with:
  - a short canonical entrypoint
  - a generated or curated topic index
- Assign each canonical doc an owner and review trigger.

Acceptance criteria:

- a new contributor can find build, architecture, and contribution rules within 2 minutes
- historical notes no longer compete with current policy
- `docs/README.md` becomes a gateway, not an exhaustive dump

## Phase 8: Tighten Governance For Durable Outputs

Time horizon: 2 to 3 days

Actions:

- Define what counts as:
  - source
  - generated source
  - reproducible artifact
  - ephemeral output
- Add a short policy file documenting where each class belongs.
- Require durable artifacts to carry metadata:
  - creation date
  - command
  - toolchain
  - source revision
  - checksum when relevant
- Extend the existing ISA benchmark artifact discipline into a repo-wide artifact policy.

Acceptance criteria:

- every durable generated output has a home and metadata expectations
- artifact sprawl stops recurring

## Phase 9: Container And Reproducibility Hardening

Time horizon: 2 to 4 days

Actions:

- Replace `FROM rust:latest` with a pinned base image or digest.
- Convert the Dockerfile into a multi-stage design if it serves more than one purpose.
- Decide whether the Dockerfile is for:
  - local reproducible dev builds
  - CI execution
  - release artifact building
- Avoid installing unused tools in the default build image.
- Add a documented container task such as `just docker-ci`.

Acceptance criteria:

- container purpose is explicit
- build image inputs are pinned
- containerized workflow matches the documented local workflow

## Priority Order

Recommended order of execution:

1. Phase 0
2. Phase 1
3. Phase 2
4. Phase 3
5. Phase 5
6. Phase 6
7. Phase 4
8. Phase 7
9. Phase 8
10. Phase 9

Rationale:

- repo hygiene, ignores, and task entrypoints should be fixed before larger structural reorg
- local gate discipline should return early so debt does not keep compounding
- docs reorganization should happen after the operating model is clearer

## Concrete Backlog

### Sprint A: Repo Hygiene

- add `justfile`
- add `just ci` as canonical local gate
- relocate tracked root artifacts
- normalize `.gitignore`
- document artifact policy

### Sprint B: Verification Surface

- complete the local-only merge-discipline model
- add explicit fuzz maintenance path
- ensure local and remote gates are identical

### Sprint C: Structural Cleanup

- categorize docs
- archive stale or historical material
- relocate doc sketches
- simplify root README and docs gateway

### Sprint D: Build Platform Maturity

- normalize scripts
- decide on `xtask` for complex repo automation
- harden Dockerfile
- define support tiers for non-core targets

## Success Metrics

- root directory contains zero ad hoc generated files
- one task entrypoint covers 90 percent of contributor workflows
- `just ci` is clearly documented and remains practical on a contributor workstation
- `.gitignore` no longer masks tracked or intended source trees
- docs entrypoints shrink while discoverability improves
- fuzz and platform-specific targets have explicit maintenance paths
- artifact generation is intentional and reproducible

## Recommended First Changes

If this work starts immediately, the highest-leverage first set is:

1. Add `justfile` and make `just ci` call the existing quality gates.
2. Clean the root artifact clutter and define `artifacts/` versus `target/`.
3. Rewrite `.gitignore` to match intended repo structure.
4. Complete the local-only merge-discipline model and remove stale hosted-CI guidance.
5. Add a docs taxonomy and begin moving historical notes out of the main docs surface.
