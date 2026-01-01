# `rkyv` (audit)
Zero-copy deserialization framework for Rust
## Upstream
- crates.io: `https://crates.io/crates/rkyv`
- latest observed: `0.8.12`
- repository: `https://github.com/rkyv/rkyv`
- documentation: `https://docs.rs/rkyv`

## Why we care (engine mapping)
- Intended role: Zero-copy snapshots / persistence
- Planned gate: `io-rkyv`
- Adoption status: `now` (Snapshot v1 encode/decode behind feature)

## Where it is used today
- `kenken-io/src/rkyv_snapshot.rs` (Snapshot v1 + v2 schemas, version-dispatched decode, roundtrip/compat tests)

## Notable features (from upstream docs, heuristic)
- (no Features section detected in README)

## Cargo features (from crates.io metadata)
- `aligned`
- `alloc`
- `big_endian`
- `bytecheck`
- `default`
- `hashbrown-0_15`
- `indexmap-2`
- `little_endian`
- `pointer_width_16`
- `pointer_width_32`
- `pointer_width_64`
- `std`
- `triomphe-0_1`
- `unaligned`
- `uuid-1`
