# `smallvec` (audit)
'Small vector' optimization: store up to a small number of items on the stack
## Upstream
- crates.io: `https://crates.io/crates/smallvec`
- latest observed: `2.0.0-alpha.12`
- repository: `https://github.com/servo/rust-smallvec`
- documentation: `https://docs.rs/smallvec/`

## Why we care (engine mapping)
- Intended role: Small, stack-backed vectors (cage cell lists)
- Planned gate: `core-smallvec`
- Adoption status: `now`

## Notable features (from upstream docs, heuristic)
- (no Features section detected in README)

## Cargo features (from crates.io metadata)
- `extract_if`
- `may_dangle`
- `serde`
- `specialization`
- `std`
