# ADR 0004: Crate layout and naming (workspace strategy)

## Status
Proposed

## Context
The docs describe a multi-crate workspace (`kenken-core`, `kenken-solver`, `kenken-gen`, etc.), but the repo currently has no `Cargo.toml`.

We need a layout that:
- keeps the engine library-first
- keeps platform adapters optional
- allows incremental bootstrapping (compile early, grow gradually)

## Decision
- Use a Cargo workspace with these initial members:
  - `kenken-core` (model, ruleset, formats, validation)
  - `kenken-solver` (solver + counting + difficulty metrics)
- Defer `kenken-gen`, `kenken-io`, and adapters (`kenken-cli`, `kenken-uniffi`, `kenken-wasm`, `kenken-cabi`) until M1/M2 unless needed earlier.
- Repo name can remain `rustykeen`; crates use `kenken-*` prefix for clarity and discoverability.

## Alternatives
- Single crate only (simpler initially, but mixes concerns and complicates adapter isolation).
- Full workspace immediately (more scaffolding before correctness exists).

## Consequences
- Faster bootstrap with a clean separation boundary between core modeling and solver mechanics.
- Adding generator/adapters later is straightforward without moving core APIs.

