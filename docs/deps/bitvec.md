# `bitvec` (audit)
Addresses memory by bits, for packed collections and bitfields
## Upstream
- crates.io: `https://crates.io/crates/bitvec`
- latest observed: `1.0.1`
- repository: `https://github.com/bitvecto-rs/bitvec`
- documentation: `https://docs.rs/bitvec/latest/bitvec`

## Why we care (engine mapping)
- Intended role: Bit-level candidate domains / masks
- Planned gate: `core-bitvec`
- Adoption status: `now` (initial `BitDomain` exists; solver migration is planned)

## Where it is used today
- `kenken-core/src/domain.rs` (`BitDomain` under `core-bitvec`)

## Notable features (from upstream docs, heuristic)
- (no Features section detected in README)

## Cargo features (from crates.io metadata)
- `alloc`
- `atomic`
- `default`
- `std`
- `testing`
