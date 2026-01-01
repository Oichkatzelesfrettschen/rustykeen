# Architecture

The workspace is layered:
- `kenken-core`: model + validation + formats
- `kenken-solver`: deterministic solving + counting
- `kenken-gen`: generation pipeline scaffolding + parallel batch hooks
- `kenken-io`: versioned snapshots
- `kenken-uniffi`: UniFFI adapter
- `kenken-cli`: reference CLI tooling

