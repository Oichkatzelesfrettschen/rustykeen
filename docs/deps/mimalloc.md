# `mimalloc` (audit)
Performance and security oriented drop-in allocator
## Upstream
- crates.io: `https://crates.io/crates/mimalloc`
- latest observed: `0.1.48`
- repository: `https://github.com/purpleprotocol/mimalloc_rust`

## Why we care (engine mapping)
- Intended role: High-performance global allocator (non-iOS)
- Planned gate: `alloc-mimalloc`
- Adoption status: `now` (CLI opt-in global allocator behind feature)

## Where it is used today
- `kenken-cli/src/main.rs` (`#[global_allocator]` behind `alloc-mimalloc`)

## Notable features (from upstream docs, heuristic)
- (no Features section detected in README)

## Cargo features (from crates.io metadata)
- `debug`
- `debug_in_debug`
- `default`
- `extended`
- `local_dynamic_tls`
- `no_thp`
- `override`
- `secure`
- `v3`
