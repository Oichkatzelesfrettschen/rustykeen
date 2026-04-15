# ISA benchmark artifact registry

Last updated: 2026-04-06

Status: **Published** baseline registry contract (referenced by `docs/target_matrix.md`).

This document defines a reproducible artifact registry for ISA-tiered `kenken-cli` build and benchmark runs.

## Scope

The registry covers three required profiles:

1. `linux-x86-64-v1` (`x86_64-unknown-linux-gnu`, `-C target-cpu=x86-64-v1`)
2. `linux-x86-64-v3` (`x86_64-unknown-linux-gnu`, `-C target-cpu=x86-64-v3`)
3. `linux-aarch64-generic` (`aarch64-unknown-linux-gnu`, `-C target-cpu=generic`)

Toolchain pin is taken from `rust-toolchain.toml` (currently `nightly-2026-04-06`).

## Registry invariants

- Run from repo root.
- Capture one immutable `run_id` per publication: `<YYYYMMDDTHHMMSSZ>_<commit_short>`.
- Capture exact commit SHA (`git rev-parse HEAD`) and UTC timestamp (`date -u`).
- Use a dedicated `CARGO_TARGET_DIR` per profile to avoid binary overwrite between v1/v3.
- Keep command lines in metadata exactly as executed.

## Artifact layout (required)

```text
artifacts/isa-bench/<run_id>/
├── registry.json
├── linux-x86-64-v1/
│   ├── build/
│   │   ├── kenken-cli
│   │   ├── kenken-cli.sha256
│   │   ├── build.log
│   │   └── metadata.json
│   └── bench/
│       ├── benchmark_n4_count100_tier-normal.txt
│       ├── benchmark_n6_count50_tier-normal.txt
│       ├── benchmark_n9_count10_tier-normal.txt
│       └── metadata.json
├── linux-x86-64-v3/
│   └── (same as above)
└── linux-aarch64-generic/
    └── (same as above)
```

`registry.json` is the top-level index for consumers; per-profile `metadata.json` files carry execution details.

## Required metadata fields

Both `build/metadata.json` and `bench/metadata.json` must include:

| Field | Required | Description |
|---|---|---|
| `schema_version` | yes | Metadata schema version (`1`) |
| `run_id` | yes | Publication run id |
| `timestamp_utc` | yes | ISO-8601 UTC timestamp |
| `git_commit` | yes | Full commit SHA |
| `toolchain_pin` | yes | Rust channel pin from `rust-toolchain.toml` |
| `target_triple` | yes | Rust target triple |
| `target_flags` | yes | ISA flags (`-C target-cpu=...`) |
| `command` | yes | Exact command executed |
| `host_uname` | recommended | `uname -a` capture |

`build/metadata.json` should also include `binary_path` and `binary_sha256`.
`bench/metadata.json` should also include `binary_path`, `benchmark_cases`, and `result_files`.

## Canonical commands per profile

Use these exact commands (adjust only `<run_id>` and output redirection paths):

### 1) linux-x86-64-v1

```bash
CARGO_TARGET_DIR=target/isa/x86_64_v1 \
RUSTFLAGS="-C target-cpu=x86-64-v1" \
cargo +nightly-2026-04-06 build --release -p kenken-cli --all-features

./target/isa/x86_64_v1/release/kenken-cli benchmark --n 4 --count 100 --tier normal
./target/isa/x86_64_v1/release/kenken-cli benchmark --n 6 --count 50 --tier normal
./target/isa/x86_64_v1/release/kenken-cli benchmark --n 9 --count 10 --tier normal
```

### 2) linux-x86-64-v3

```bash
CARGO_TARGET_DIR=target/isa/x86_64_v3 \
RUSTFLAGS="-C target-cpu=x86-64-v3" \
cargo +nightly-2026-04-06 build --release -p kenken-cli --all-features

./target/isa/x86_64_v3/release/kenken-cli benchmark --n 4 --count 100 --tier normal
./target/isa/x86_64_v3/release/kenken-cli benchmark --n 6 --count 50 --tier normal
./target/isa/x86_64_v3/release/kenken-cli benchmark --n 9 --count 10 --tier normal
```

### 3) linux-aarch64-generic

```bash
CARGO_TARGET_DIR=target/isa/aarch64_generic \
RUSTFLAGS="-C target-cpu=generic" \
cargo +nightly-2026-04-06 build --release -p kenken-cli --all-features \
  --target aarch64-unknown-linux-gnu

./target/isa/aarch64_generic/aarch64-unknown-linux-gnu/release/kenken-cli benchmark --n 4 --count 100 --tier normal
./target/isa/aarch64_generic/aarch64-unknown-linux-gnu/release/kenken-cli benchmark --n 6 --count 50 --tier normal
./target/isa/aarch64_generic/aarch64-unknown-linux-gnu/release/kenken-cli benchmark --n 9 --count 10 --tier normal
```

Note: the aarch64 benchmark commands must run on an aarch64 Linux host (or emulator/device that can execute the produced binary).

## `registry.json` minimum shape

```json
{
  "schema_version": 1,
  "run_id": "20260406T210000Z_abcdef12",
  "toolchain_pin": "nightly-2026-04-06",
  "git_commit": "abcdef1234567890abcdef1234567890abcdef12",
  "profiles": [
    {
      "profile": "linux-x86-64-v1",
      "target_triple": "x86_64-unknown-linux-gnu",
      "target_flags": "-C target-cpu=x86-64-v1",
      "build_metadata": "linux-x86-64-v1/build/metadata.json",
      "bench_metadata": "linux-x86-64-v1/bench/metadata.json"
    }
  ]
}
```

## Verification checks (must pass before publication)

1. **Toolchain pin check**
   ```bash
   grep -F 'channel = "nightly-2026-04-06"' rust-toolchain.toml
   ```
2. **Commit capture check**
   ```bash
   git rev-parse HEAD
   git status --short
   ```
3. **Binary checksum check**
   ```bash
   sha256sum -c artifacts/isa-bench/<run_id>/<profile>/build/kenken-cli.sha256
   ```
4. **ISA/arch sanity check**
   ```bash
   file artifacts/isa-bench/<run_id>/<profile>/build/kenken-cli
   ```
   - x86 profiles must report `x86-64`
   - aarch64 profile must report `ARM aarch64`
5. **Benchmark output check**
   ```bash
   rg -n "^Puzzles/second:" artifacts/isa-bench/<run_id>/<profile>/bench/*.txt
   ```
6. **Metadata field presence check**
   ```bash
   for k in schema_version run_id timestamp_utc git_commit toolchain_pin target_triple target_flags command; do
     rg -n "\"$k\"" artifacts/isa-bench/<run_id>/<profile>/*/metadata.json
   done
   ```

If any check fails, do not publish the run as a baseline.
