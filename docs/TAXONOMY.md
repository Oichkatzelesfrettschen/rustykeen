# Documentation Taxonomy

This taxonomy is the starting point for turning `docs/` into a clearer source
of truth.

## Intended classes

- `canonical`
  - current policy, architecture, contributor workflow, and support contracts
- `runbooks`
  - step-by-step operational guides for supported workflows
- `research`
  - exploratory analysis, design investigations, and historical performance work
- `archive`
  - deprecated or superseded material retained only for reference

## Current transition policy

Until files are physically moved into subdirectories, classify docs by intent:

- Canonical examples:
  - `docs/canonical/dev_workflow.md`
  - `docs/canonical/api_stability.md`
  - `docs/architecture.md`
  - `docs/target_matrix.md`
  - `docs/cross_target_api_parity_contract.md`
- Runbook examples:
  - `docs/runbooks/android_build.md`
  - `docs/runbooks/android_deployment.md`
  - `docs/riced_build.md`
- Research examples:
  - tiered optimization notes
  - PGO and profiling analyses
  - exploratory dependency and design studies
- Archive examples:
  - `.deprecated` files
  - superseded planning documents once replacements are confirmed

## Placement rules

- New policy or support-contract docs should be written as canonical docs.
- New operational instructions should be written as runbooks.
- New exploratory or one-off investigations should be written as research docs.
- Superseded docs should not remain mixed into canonical entrypoints.

## Review rule

If a docs change affects how contributors build, test, release, or understand
supported targets, update the canonical entrypoints at the same time:

- `README.md`
- `CONTRIBUTING.md`
- `docs/README.md`
- `docs/canonical/dev_workflow.md`
