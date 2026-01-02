#!/bin/bash
# Profile benchmarks using cargo-flamegraph and flamegraph tools

set -e

BENCH_NAME="${1:-simd_effectiveness}"
OUTPUT_DIR="target/flamegraphs"

mkdir -p "$OUTPUT_DIR"

echo "Profiling benchmark: $BENCH_NAME"
echo "Output directory: $OUTPUT_DIR"
echo "This may take a minute or two..."
echo ""

# Use cargo-flamegraph to profile the benchmark
cargo flamegraph \
    --bench "$BENCH_NAME" \
    --output "$OUTPUT_DIR/flamegraph_${BENCH_NAME}.svg" \
    --freq 997 \
    -- --bench

echo ""
echo "Flamegraph generated: $OUTPUT_DIR/flamegraph_${BENCH_NAME}.svg"
echo ""
echo "To view the flamegraph, open in a web browser or use:"
echo "  firefox $OUTPUT_DIR/flamegraph_${BENCH_NAME}.svg"
