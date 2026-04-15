# Bitvec Decision Record

Last updated: 2026-04-09

## Purpose

This document defines the exit conditions for the current `bitvec` decision
debt in the repository.

Today:

- `kenken-core` exposes `BitDomain` behind `core-bitvec`
- `kenken-solver` exposes `solver-bitdomain`
- the main solver still uses fixed-width integer masks in hot paths

This is not an automatic "finish the integration" backlog item. It is an
architectural choice that must be resolved by explicit scope and benchmark
evidence.

## Current facts

- The current architecture targets grids up to 16x16 in the core model.
- Practical solver performance goals are centered on 4x4 through 9x9.
- The existing solver hot path is built around fixed-width integer masks.
- `BitDomain` currently serves as a viable capability probe, not a committed
  mainline representation.

## Decision outcomes

Exactly one of these outcomes should eventually be chosen.

### 1. Promote

`bitvec` becomes a supported solver-path domain representation rather than a
side capability.

Promotion requires all of the following:

1. Product scope expands to require supported puzzle sizes that exceed what the
   fixed-width integer strategy can represent cleanly.
2. `criterion` benchmarks show that using `BitDomain` in the solver causes no
   more than a 5% regression on standard workloads (4x4 through 9x9), or
   produces a clear win on supported larger-domain workloads that the project
   actually commits to shipping.
3. The resulting solver code is not materially harder to reason about than the
   fixed-width baseline.

Promotion follow-through:

- document the supported large-grid rationale in architecture docs
- benchmark the promoted path in the regular domain-representation bench suite
- stop describing `bitvec` as decision debt

### 2. Demote

`bitvec` remains in the repository, but only as an explicitly non-default
research or experimental path.

Demotion is the correct choice when:

1. The current supported product scope still centers on 4x4 through 9x9 and
   the broader core target remains at or below 16x16.
2. Benchmarks show that fixed-width integer masks remain clearly better on the
   supported hot path.
3. There is still credible future value in retaining a larger-domain or
   alternative-domain experiment lane.

Demotion follow-through:

- keep `core-bitvec` and `solver-bitdomain` non-default
- describe the lane as experimental/research-only in docs
- remove any wording that implies migration is expected

### 3. Remove

`bitvec` is removed from the repository entirely.

Removal is the correct choice when:

1. The project formally caps supported grid sizes within a fixed-width mask
   envelope that the chosen solver representations already cover.
2. Fixed-width integer masks dominate the relevant benchmark suite and raw
   solver throughput remains the primary priority.
3. The team no longer expects a realistic near-term requirement for arbitrary
   larger domains.

Removal follow-through:

- remove `bitvec` from `kenken-core/Cargo.toml`
- remove `core-bitvec` and `solver-bitdomain`
- remove `BitDomain` and related references from docs

## Benchmark gate

No promote/remove decision should be made without a reproducible benchmark pass.

At minimum, benchmark:

- domain creation
- candidate insertion/removal
- candidate counting
- iteration over set bits
- representative solve workloads at 4x4, 6x6, and 9x9

Use `criterion` and the existing domain-representation bench lane as the
baseline evidence source.

## Default recommendation under current scope

Given the current documented scope:

- core target: up to 16x16
- primary performance focus: 4x4 through 9x9

the default recommendation is to **demote**, not promote.

Promotion should require a conscious scope change. Removal should require a
conscious statement that larger-domain experimentation is no longer valuable.
