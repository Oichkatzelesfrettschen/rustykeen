#!/bin/bash
set -euo pipefail

# Comprehensive Tier 2.2 profiling across puzzle sizes 2x2 through 32x32
# Outputs timing and memory data to assess cache effectiveness at scale

OUTPUT_DIR="${1:-target/tier22_profiling_$(date +%Y%m%d_%H%M%S)}"
mkdir -p "$OUTPUT_DIR"

echo "Tier 2.2 Scaling Profile: 2x2 through 32x32"
echo "Output directory: $OUTPUT_DIR"
echo ""

# Build release binary
echo "Building release binaries..."
cargo build --release -p kenken-solver --bin heap_profile --features dhat-heap >/dev/null 2>&1
cargo build --release -p kenken-cli --all-features >/dev/null 2>&1

# Simple test puzzles for different sizes
declare -A PUZZLES=(
    [2]="b__,a3a3"                          # 2x2 trivial
    [3]="f_6,a6a6a6"                        # 3x3 with some cages
    [4]="e__e,a10a10,b3b3,d__d"             # 4x4 mixed
    [6]="e___e_,a12a12,b10b10a2,c__c,d__d"  # 6x6 medium
)

# For sizes without predefined puzzles, use single-cage puzzles
get_puzzle() {
    local n=$1
    local cells=$((n * n))
    case $n in
        2) echo "b__,a3a3" ;;
        3) echo "f_6,a6a6a6" ;;
        4) echo "e__e,a10a10,b3b3,d__d" ;;
        6) echo "e___e_,a12a12,b10b10a2,c__c,d__d" ;;
        *)
            # Single cage covering all cells (simple but tests worst-case MRV)
            printf "a%.0s" $(seq 1 $cells)
            echo ",$(( n * (n + 1) / 2 ))a$(printf '%.0s' $(seq 1 $cells))"
            ;;
    esac
}

# Profile each size
for SIZE in 2 3 4 6 8 12 16; do
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Profiling ${SIZE}x${SIZE} puzzle..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    PUZZLE=$(get_puzzle $SIZE)
    ITERATIONS=$(( 1000 / (SIZE * SIZE / 4) ))  # Scale iterations inversely

    # Heap profile
    echo "  Running heap profiler (${ITERATIONS} iterations)..."
    HEAP_FILE="$OUTPUT_DIR/heap_${SIZE}x${SIZE}.json"
    ./target/release/heap_profile \
        --n "$SIZE" \
        --desc "$PUZZLE" \
        --tier normal \
        --iterations "$ITERATIONS" \
        2>&1 | grep -E "Total:|Peak:|gmax:" | tee -a "$OUTPUT_DIR/profile_${SIZE}x${SIZE}.log"

    # CPU benchmark via criterion (lighter weight)
    echo "  Running CPU benchmark..."
    cargo bench -p kenken-solver --bench solver_smoke 2>&1 \
        | grep -E "solve_${SIZE}x${SIZE}|time:|change:" \
        | head -20 >> "$OUTPUT_DIR/benchmark_${SIZE}x${SIZE}.log" || true

    echo "  ${SIZE}x${SIZE} complete"
    echo ""
done

# Special handling for larger sizes (32x32 might be unavailable with Domain32)
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Profiling 24x24 (requires solver-u64 feature)..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Try building with u64 support if available
if cargo build --release -p kenken-solver --features solver-u64 --bin heap_profile --features dhat-heap 2>/dev/null; then
    PUZZLE=$(get_puzzle 24)
    ./target/release/heap_profile \
        --n 24 \
        --desc "$PUZZLE" \
        --tier normal \
        --iterations 10 \
        2>&1 | grep -E "Total:|Peak:" | tee -a "$OUTPUT_DIR/profile_24x24.log"
fi

# Generate summary report
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Profiling Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cat > "$OUTPUT_DIR/summary.txt" << 'SUMMARY'
Tier 2.2 Scaling Analysis: Cache Effectiveness vs Grid Size
============================================================

Key Metrics by Grid Size:
- Memory overhead percentage
- Benchmark time change (vs baseline)
- Estimated choose_mrv_cell time

Expected pattern:
- 2x2-4x4: Cache overhead dominates (negative benefit)
- 6x6-8x8: Breakeven zone
- 12x12+: Cache benefits should emerge

Decision criteria:
- If net benefit at 6x6: enhance cache efficiency
- If mixed results: add size threshold (cache only for n>=8)
- If overhead persists: pivot to Tier 2.1/2.3
SUMMARY

# Analyze results
echo ""
echo "Results saved to: $OUTPUT_DIR"
echo "Check following files for detailed analysis:"
ls -lh "$OUTPUT_DIR"/*.log "$OUTPUT_DIR"/*.json 2>/dev/null | tail -20 || true

echo ""
echo "To generate comparison report, run:"
echo "  python3 scripts/analyze_tier22_profile.py $OUTPUT_DIR"
