# Developer workflow

## Toolchain
- This repo uses Rust nightly pinned by `rust-toolchain.toml`.

## Quality gates
- Canonical entrypoint: `just ci`
- Format: `just fmt`
- Lint: `just lint`
- Tests: `just test`
- Security/freshness: `just audit`
- Low-level bundle script: `./scripts/local_quality_gates.sh`

## Automation policy
- Hosted CI/CD is intentionally disabled for quota control.
- Do not add GitHub Actions workflows to this repo unless the quota policy changes explicitly.
- Treat `just ci` as mandatory pre-merge validation.

## Upstream study artifacts
- Keep upstream code snapshots under `third_party/upstream/` (git-ignored).
- Distill upstream behavior into docs/tests, not copied code.

## Artifact placement
- Durable, curated evidence belongs under `artifacts/`.
- Ephemeral build and profiling output belongs under `target/` or `.tmp/`.
- Do not commit generated logs, crash outputs, or benchmark dumps at repo root.
