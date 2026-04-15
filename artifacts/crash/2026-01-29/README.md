# 2026-01-29 rustc ICE snapshots

These files were preserved from repo-root crash outputs and relocated under
`artifacts/crash/` as part of repo hygiene cleanup.

Observed failure class:

- rustc LLVM/codegen thread-spawn failures with `Os { code: 11, kind: WouldBlock }`

Preservation note:

- These are historical evidence files.
- New crash outputs should not be committed at repo root.
- If preserved intentionally, store them under `artifacts/crash/<date>/`.
