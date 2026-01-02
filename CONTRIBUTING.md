# Contributing to rustykeen

Thank you for your interest in contributing to rustykeen!

## Development Setup

1. Install Rust nightly (pinned version in `rust-toolchain.toml`)
2. Clone the repository
3. Run `cargo build` to verify setup
4. Run `cargo test` to verify tests pass

## Code Quality Standards

All contributions must:

- Pass `cargo fmt --check` (formatting)
- Pass `cargo clippy --all-targets --all-features -- -D warnings` (linting)
- Pass `cargo test --all-targets` (tests)
- Not introduce `unsafe` code outside of `kenken-simd`

## Commit Guidelines

We use conventional commits:

- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `refactor:` Code refactoring
- `test:` Test additions/changes
- `perf:` Performance improvements
- `chore:` Maintenance tasks

## Pull Request Process

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes
4. Ensure CI passes
5. Submit a pull request with a clear description

## Cleanroom Policy

This project follows cleanroom development practices. Do not:

- Copy code from upstream sgt-puzzles
- Reference upstream implementation details directly

All behavior must be derived from the puzzle specification.

## Questions?

Open an issue for discussion.
