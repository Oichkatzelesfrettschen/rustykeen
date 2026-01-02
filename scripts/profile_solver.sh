#!/bin/bash
# Comprehensive KenKen solver profiling script
#
# Profiles the solver across multiple grid sizes (2x2 through 32x32) and
# deduction tiers using both tracing-flame and CPU flamegraph techniques.
#
# Output: target/profiling_results/<timestamp>/ containing HTML and SVG flamegraphs
#
# Requires: cargo, flamegraph (via `cargo install flamegraph`)

set -eu

# Configuration
PROFILING_DIR="${PROFILING_DIR:-target/profiling_results/$(date +%Y%m%d_%H%M%S)}"
PROFILE_EXECUTABLE="${PROFILE_EXECUTABLE:-profile_spans}"

# Puzzle sizes to profile (must have known test puzzles)
SIZES=(2 4 6)

# Deduction tiers to test
TIERS=(none easy normal hard)

# Known test puzzles (sgt-desc format)
declare -A PUZZLES
PUZZLES[2]="b__,a3a3"
PUZZLES[4]="c_4,a12a12a12d_3,b3b3"
PUZZLES[6]="i_,f6,b_,c_5,a_,h_,e6,d5,g_,j_"

mkdir -p "$PROFILING_DIR"

echo "=========================================="
echo "KenKen Solver Profiling"
echo "=========================================="
echo "Output directory: $PROFILING_DIR"
echo ""

# Function to run profiling for a given size and tier
profile_solver() {
    local size=$1
    local tier=$2
    local puzzle=${PUZZLES[$size]:-""}

    if [ -z "$puzzle" ]; then
        echo "⊘ Skipping ${size}x${size} (no test puzzle available)"
        return
    fi

    echo "→ Profiling ${size}x${size} with tier=${tier}..."

    # Generate output filenames
    local trace_output="$PROFILING_DIR/trace_${size}x${size}_${tier}.html"
    local cpu_output="$PROFILING_DIR/cpu_${size}x${size}_${tier}.svg"

    # Run tracing-flame profiler
    if cargo run --release --bin "$PROFILE_EXECUTABLE" --features prof-flame \
        -- --n "$size" --desc "$puzzle" --tier "$tier" --output "$trace_output" 2>/dev/null; then
        echo "  ✓ Tracing flamegraph: $(basename "$trace_output")"
    else
        echo "  ✗ Tracing flamegraph failed (invalid puzzle?)"
        return
    fi

    # Run CPU flamegraph profiler (if cargo-flamegraph installed)
    if command -v cargo-flamegraph >/dev/null 2>&1; then
        if cargo flamegraph --release --bin "$PROFILE_EXECUTABLE" --features prof-flame \
            --output "$cpu_output" -- --n "$size" --desc "$puzzle" --tier "$tier" >/dev/null 2>&1; then
            echo "  ✓ CPU flamegraph: $(basename "$cpu_output")"
        else
            echo "  ✗ CPU flamegraph failed (flamegraph tool missing?)"
        fi
    else
        echo "  ⊘ CPU flamegraph skipped (cargo-flamegraph not installed)"
    fi
}

# Main profiling loop
for size in "${SIZES[@]}"; do
    for tier in "${TIERS[@]}"; do
        profile_solver "$size" "$tier"
    done
done

echo ""
echo "=========================================="
echo "Profiling Complete"
echo "=========================================="
echo "Results location: $PROFILING_DIR"
echo ""
echo "Analysis steps:"
echo "1. Open HTML flamegraphs in browser:"
echo "   firefox $PROFILING_DIR/trace_*.html"
echo ""
echo "2. Convert SVG flamegraphs to interactive HTML:"
echo "   # Install flamegraph tools: cargo install flamegraph"
echo "   # SVG files can be opened directly in browser"
echo ""
echo "3. Key metrics to extract from flamegraphs:"
echo "   - Width (time spent) in backtrack vs propagate vs cage_feasible"
echo "   - Deduction tier impact on propagation time"
echo "   - Choose_mrv_cell overhead"
echo "   - Cage tuple enumeration cost (enumerate_cage_tuples)"
