FROM rust:latest

# Set build arguments
ARG RUST_VERSION=nightly-2026-04-06
ARG PROFILE=release

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set Rust toolchain
RUN rustup default ${RUST_VERSION}

# Add targets for cross-compilation
RUN rustup target add \
    wasm32-unknown-unknown \
    aarch64-linux-gnu \
    armv7-unknown-linux-gnueabihf

# Install tools
RUN cargo install wasm-pack cargo-build-all

# Create workspace directory
WORKDIR /workspace

# Copy project files
COPY . .

# Build all crates
RUN cargo build --${PROFILE} --all-features --all-targets

# Create entrypoint script for flexible usage
RUN echo '#!/bin/bash\n\
if [ $# -eq 0 ]; then\n\
  cargo run --release -p kenken-cli -- --help\n\
else\n\
  cargo run --release -p kenken-cli -- "$@"\n\
fi' > /entrypoint.sh && chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]

# Default command
CMD ["solve", "--help"]

# Build metadata
LABEL version="0.1.0"
LABEL description="Rustykeen: Production-Grade KenKen Solver"
LABEL maintainer="Eirik Rasmusen"
LABEL org.opencontainers.image.source="https://github.com/eirikr/rustykeen"
