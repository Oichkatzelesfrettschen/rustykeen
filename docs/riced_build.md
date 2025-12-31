# Riced Build Configuration (2026)

## Cargo.toml (profiles)
```toml
[profile.release]
codegen-units = 1
lto = "fat"
opt-level = 3
panic = "abort"
strip = "symbols"
debug = false
overflow-checks = false

[profile.bench]
inherits = "release"
debug = true

[profile.dev]
opt-level = 1
debug = 0
split-debuginfo = "unpacked"
```

## .cargo/config.toml
```toml
[build]
rustflags = ["-W","future-incompatible","-W","rust_2024_compatibility","--cfg","tokio_unstable"]

[target.x86_64-unknown-linux-gnu]
rustflags = ["-C","target-cpu=native","-C","link-arg=-fuse-ld=mold"]

[target.aarch64-linux-android]
rustflags = ["-C","target-feature=+neon,+fp16","-C","link-arg=-Wl,--emit-relocs","-C","force-frame-pointers=yes"]
```

## lib.rs Clippy gates
```rust
#![deny(unsafe_code)]
#![deny(clippy::all, clippy::correctness, clippy::suspect_xor_variant)]
#![warn(clippy::perf, clippy::large_stack_arrays, clippy::inline_always, clippy::pedantic, clippy::cargo)]
#![allow(clippy::module_name_repetitions, clippy::cast_possible_truncation)]
```

## PGO + BOLT Makefile
```makefile
pgo-instrument:
	cargo pgo build --release
pgo-train:
	./target/release/keen_engine_cli --generate-batch 1000
pgo-optimize:
	cargo pgo optimize --release
bolt: pgo-optimize
	llvm-bolt ./target/release/keen_engine -o ./target/release/keen_engine.bolt.inst -instrument
	./target/release/keen_engine.bolt.inst --generate-batch 1000
	perf2bolt -p perf.data -o perf.fdata ./target/release/keen_engine
	llvm-bolt ./target/release/keen_engine -o ./target/release/keen_engine_final -data perf.fdata -reorder-blocks=ext-tsp -reorder-functions=hfsort -split-functions -split-all-cold -dyno-stats
```
