# Performance

The “performance core” is staged behind feature flags to keep the default build lean.

Integrated today:
- `bumpalo`: propagation scratch arena in `kenken-solver`
- `rayon`: parallel batch solving foundation in `kenken-gen`
- `dlx-rs`: DLX Latin exact-cover utilities in `kenken-solver`

