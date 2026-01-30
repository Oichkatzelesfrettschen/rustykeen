# Contributing to rustykeen

Thank you for interest in contributing to rustykeen! This guide will help you get started with development.

## Quick Start

### Prerequisites

- Rust: `nightly-2026-01-01` or later (pinned in `rust-toolchain.toml`)
- For Android: Android NDK, cargo-ndk
- For profiling: Optional tools (perf, samply, flamegraph)

### Build & Test

```bash
# Build all crates
cargo build

# Run all tests
cargo test --all-targets

# Run with all optional features
cargo test --all-features

# Build in release mode (LTO, opt-level=3)
cargo build --release

# Check code quality
cargo fmt --check && cargo clippy --all-targets --all-features
```

## Code Style Guide

### Formatting

Use `rustfmt` (automatic via `cargo fmt`):

```bash
cargo fmt
```

The project uses the default Rust style. All code must pass:

```bash
cargo fmt --check
```

### Linting

All clippy warnings must be fixed. The workspace enforces:
- `warnings = "deny"` - Treat all warnings as errors
- `unsafe_code = "forbid"` - No `unsafe` except in `kenken-simd` (FFI crate)

Run locally:

```bash
cargo clippy --all-targets --all-features
```

### Naming Conventions

- **Types**: `PascalCase` (e.g., `Solution`, `SolveStats`)
- **Functions**: `snake_case` (e.g., `solve_one()`, `count_solutions_up_to()`)
- **Constants**: `UPPER_SNAKE_CASE` (e.g., `SAT_TUPLE_THRESHOLD`)
- **Modules**: `snake_case` (e.g., `domain_ops`, `sat_cages`)

### Documentation

Add doc comments to all public items:

```rust
/// Solves a puzzle and returns the first solution.
///
/// Returns `Ok(Some(solution))` if a solution exists, `Ok(None)` if no solution,
/// or `Err(SolveError)` if validation fails.
pub fn solve_one(puzzle: &Puzzle, rules: Ruleset) -> Result<Option<Solution>, SolveError> {
    // ...
}
```

## Testing Requirements

### Unit Tests

- Add unit tests in the same file as the code being tested
- Use `#[test]` for simple tests
- Aim for coverage of happy path and edge cases

### Integration Tests

- Integration tests go in `crates/*/tests/` directories
- Test public API contracts, not implementation details
- Use golden corpus data when available

### Property Tests

- Use `proptest` for property-based testing
- Test invariants and algebraic properties
- Example: `all_permutations` verify uniqueness

### Running Tests

```bash
# Run all tests
cargo test --all-targets

# Run tests with all features
cargo test --all-features

# Run a specific test
cargo test test_solve_one

# Run tests with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored
```

## Feature Flags

### Core Features

**kenken-solver:**
- `solver-dlx`: Enable Dancing Links exact cover solver
- `sat-varisat`: Enable SAT solver backend for uniqueness verification
- `simd-dispatch`: Runtime SIMD instruction dispatch
- `tracing`: Enable structured logging
- `perf-likely`: Enable branch prediction hints

**kenken-gen:**
- `gen-dlx`: DLX-based puzzle generation
- `parallel-rayon`: Parallel batch solving

**kenken-core:**
- `core-bitvec`: Large bitfield support for n > 63

### Development Workflow

When testing a feature:

```bash
# Test with specific features
cargo test --features solver-dlx,sat-varisat

# Test without optional features
cargo test --no-default-features

# Build with all features
cargo build --all-features
```

## Workspace Structure

```
kenken-core/        Core puzzle model and validation
kenken-solver/      Backtracking solver, deduction tiers, SAT/DLX backends
kenken-gen/         Puzzle generator with uniqueness verification
kenken-io/          Snapshot serialization (rkyv)
kenken-simd/        Runtime SIMD dispatch (safe wrapper)
kenken-uniffi/      Kotlin/Swift FFI bindings
kenken-cli/         Command-line reference tool
kenken-verify/      Formal verification utilities
docs/               Documentation (81 files, see docs/INDEX.md)
```

## Common Development Tasks

### Running the CLI

```bash
# Solve a puzzle
cargo run -p kenken-cli -- solve --n 2 --desc b__,a3a3

# Count solutions
cargo run -p kenken-cli -- count --n 2 --desc b__,a3a3 --limit 2

# Generate a puzzle
cargo run -p kenken-cli -- generate --n 4 --difficulty normal

# Benchmark puzzles
cargo run -p kenken-cli -- benchmark --n 4 --count 100
```

### Benchmarking

```bash
# Run solver smoke tests
cargo bench --bench solver_smoke

# Run domain representation comparison
cargo bench --bench domain_repr

# Profile with flamegraph
cargo install flamegraph
cargo flamegraph --bin kenken-cli -- solve --n 6 --desc ...
```

### Memory Profiling

```bash
# Build with dhat support
cargo build --release --features dhat-heap

# Run with memory profiling
cargo run --release --features dhat-heap -- solve --n 6 --desc ...
```

### Android Development

```bash
# Install cargo-ndk
cargo install cargo-ndk --version 3.2.0

# Build for Android arm64
cargo ndk -t arm64-v8a build --release -p kenken-uniffi --all-features

# Copy to Android project
cp target/aarch64-linux-android/release/libkenken_uniffi.so \
   examples/android-app/app/src/main/jniLibs/arm64-v8a/
```

## Pull Request Checklist

Before submitting a pull request:

- [ ] Code builds with `cargo build`
- [ ] Formatting passes: `cargo fmt --check`
- [ ] Linting passes: `cargo clippy --all-targets --all-features`
- [ ] Tests pass: `cargo test --all-targets`
- [ ] Tests pass with all features: `cargo test --all-features`
- [ ] New public APIs have doc comments
- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)
- [ ] PR description explains the what and why

### Commit Message Format

```
feat: add new deduction tier for advanced constraints

More detailed explanation of the change and its motivation.
Can span multiple lines.

Fixes #123
```

Prefixes:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Test additions or fixes
- `perf`: Performance improvement
- `refactor`: Code restructuring
- `chore`: Build, dependency, or configuration changes

## API Stability

The project uses semantic versioning. See [docs/api_stability.md](docs/api_stability.md) for:
- Stable public API items that are covered by semver
- Internal implementation details (never breaking)
- Deprecation policy
- Feature flag stability

## Documentation

All changes should include documentation updates:

- Update [docs/INDEX.md](docs/INDEX.md) if adding new documentation
- Update [docs/work_done.md](docs/work_done.md) with major milestones
- Update [docs/roadmap_2026.md](docs/roadmap_2026.md) if completing planned work
- Add or update doc comments in code

## Project-Specific Guidelines

See [CLAUDE.md](CLAUDE.md) for:
- Core principles and no-shortcuts policy
- Conflict analysis protocol
- Task planning and todo discipline
- Memory file organization
- Build and test quality gates
- Performance principles
- Git workflow guidelines

## Cleanroom Policy

This project follows cleanroom development practices. Do not:
- Copy code from upstream sgt-puzzles
- Reference upstream implementation details directly

All behavior must be derived from the puzzle specification.

## Getting Help

- Check existing documentation in [docs/INDEX.md](docs/INDEX.md)
- Review [docs/architecture.md](docs/architecture.md) for system overview
- Look at existing tests for usage examples
- Open an issue with questions

## License

By contributing, you agree that your contributions will be licensed under the project's GPL-2.0-only license.

---

**Happy contributing!**
