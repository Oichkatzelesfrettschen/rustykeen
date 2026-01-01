# `ringbuf` (audit)
Lock-free SPSC FIFO ring buffer with direct access to inner data
## Upstream
- crates.io: `https://crates.io/crates/ringbuf`
- latest observed: `0.4.8`
- repository: `https://github.com/agerasev/ringbuf.git`
- documentation: `https://docs.rs/ringbuf`

## Why we care (engine mapping)
- Intended role: Lock-free SPSC telemetry queue
- Planned gate: `telemetry-ringbuf`
- Adoption status: `planned`

## Notable features (from upstream docs, heuristic)
- (no Features section detected in README)

## Cargo features (from crates.io metadata)
- `alloc`
- `bench`
- `default`
- `portable-atomic`
- `std`
- `test_local`
