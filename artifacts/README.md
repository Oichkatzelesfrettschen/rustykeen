# Artifacts Policy

This directory holds durable, intentionally curated generated outputs that are
worth versioning or reviewing as evidence.

Rules:

- Durable evidence belongs here, not in the repo root.
- Ephemeral build output belongs under `target/` or `.tmp/`.
- Local logs belong under ignored scratch paths, not as tracked root files.
- Each durable artifact set should include enough context to explain:
  - when it was produced
  - what command produced it
  - what toolchain or environment was used
  - what revision it corresponds to when relevant
- Prefer adding a `metadata.json` file alongside curated artifact sets.
- Use `artifacts/METADATA_TEMPLATE.json` as the starting template.

Suggested subtrees:

- `artifacts/benchmarks/` for curated benchmark summaries
- `artifacts/profiling/` for curated profiling outputs
- `artifacts/crash/` for preserved compiler or runtime crash evidence
- `artifacts/logs/` only when logs are intentionally retained as evidence

This repo intentionally does not use hosted GitHub Actions workflows because of
quota restrictions. Local validation is canonical, and durable output belongs
here only when it is part of that validation or an evidence trail.
