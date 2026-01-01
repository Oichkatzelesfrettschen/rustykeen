# `varisat` (audit)
A CDCL based SAT solver (library)
## Upstream
- crates.io: `https://crates.io/crates/varisat`
- latest observed: `0.2.2`
- repository: `https://github.com/jix/varisat`

## Why we care (engine mapping)
- Intended role: SAT solver for uniqueness proofs
- Planned gate: `sat-varisat`
- Adoption status: `now` (Latin + staged cage encoding behind feature)

## Where it is used today
- `kenken-solver/src/sat_latin.rs` (Latin-only SAT uniqueness helper)
- `kenken-solver/src/sat_cages.rs` (Latin + Eq/Sub/Div + tuple-allowlist Add/Mul with overflow threshold)

## Notable features (from upstream docs, heuristic)
- (no Features section detected in README)

## Cargo features (from crates.io metadata)
- (no feature metadata available)
