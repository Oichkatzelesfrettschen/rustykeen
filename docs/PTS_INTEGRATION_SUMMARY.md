# Phoronix Test Suite Integration - Research Summary

## Overview

This directory contains comprehensive research on integrating the rustykeen KenKen solver into the Phoronix Test Suite (PTS), a mature open-source benchmarking platform used globally for performance testing and comparison.

## What is Phoronix Test Suite?

**Phoronix Test Suite (PTS)** is a comprehensive, open-source benchmarking platform (v10.8.4 tested) that:
- Automates test installation, execution, and reporting
- Supports 600+ built-in tests across processors, systems, graphics, and storage
- Integrates with OpenBenchmarking.org for public result sharing and comparison
- Runs on Linux, macOS, Windows, BSD, and Solaris
- Provides result aggregation, graphing, and statistical comparison
- Requires minimal setup for custom benchmarks (3 required files)

## Key Integration Points for rustykeen

### Minimal Overhead
- Only 3 files required: XML metadata (2) + shell script (1)
- PTS handles: build, iteration, aggregation, graphing, database submission
- No code changes to rustykeen needed (CLI already suitable)

### Parameterization
- User-selectable menus for puzzle size, difficulty, puzzle count
- Same profile supports 3x3 through 8x8 with easy/normal/hard tiers
- No script modifications needed per variant

### Public Database
- Optional result upload to OpenBenchmarking.org
- Enables cross-system performance comparison
- Community-driven benchmarking via public URL sharing

### Multi-Metric Support
- Extract multiple metrics from single run (e.g., time + throughput)
- Track different algorithm backends (baseline, DLX, SAT, SIMD)
- Statistical aggregation across multiple iterations

## Document Guide

### 1. pts_integration_research.md (27 KB)
**Comprehensive technical reference** covering all aspects:

- **Sections 1-4**: Profile structure, XML file requirements, output parsing
- **Sections 5-7**: Parameterization, multi-metric profiles, ecosystem examples
- **Sections 8-9**: Build/execution requirements, advanced features
- **Sections 10-16**: Debugging, real-world examples, integration checklist

**Best for**: Understanding the big picture, designing profile architecture, learning from similar benchmarks (asmFish, primesieve, HPCC)

**Key examples**:
- Complete KenKen profile structure (Section 13)
- Result parsing patterns (simple + multi-metric)
- Platform-specific build handling
- Database registration and result submission

---

### 2. pts_practical_guide.md (20 KB)
**Step-by-step implementation guide** organized in phases:

- **Phase 1**: Prepare rustykeen CLI (verify output format)
- **Phase 2**: Create profile directory structure and files
- **Phase 3**: Debug and validate with PTS commands
- **Phase 4**: Run benchmarks and analyze results
- **Phase 5**: Create test variants (optional)
- **Phase 6**: Submit results to OpenBenchmarking.org
- **Phase 7**: Advanced features (SIMD variants, regression tracking)
- **Phase 8**: CI/CD integration (GitHub Actions example)
- **Phase 9**: Documentation updates

**Best for**: Following concrete implementation steps, copy-paste ready commands, troubleshooting issues

**Quick wins**:
- Create profile in <5 minutes
- First benchmark run in <15 minutes
- Result comparison and graphing immediately after
- Public database submission with one command

---

### 3. pts_quick_reference.md (9 KB)
**Cheat sheet and lookup table** for common tasks:

- File structure overview
- XML element reference (required/optional)
- Shell script template
- PTS command reference (8 essential commands)
- Output format examples and parsing patterns
- Environment variables available in scripts
- Common mistakes and fixes
- Result locations and upload workflow

**Best for**: Quick lookups while implementing, troubleshooting, remembering syntax

**Saved searches**:
- "How do I capture exit status?" → Look for `echo $? >`
- "Where are results saved?" → Result Output Locations section
- "What does HIB/LIB mean?" → Proportion table
- "How do I parse multiple metrics?" → Multi-Result Profile Pattern

---

## File Structure Created

```
~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/
├── test-definition.xml        [Metadata, CLI options, parameters]
├── results-definition.xml     [Output parsing rules]
└── install.sh                 [Build + wrapper script creation]
```

## Minimal Example (Single File Changes)

**test-definition.xml** (metadata):
```xml
<Title>KenKen Solver</Title>
<Description>Rust KenKen puzzle solver benchmark</Description>
<ResultScale>Puzzles/second</ResultScale>
<Proportion>HIB</Proportion>  <!-- Higher is better -->
<Arguments>--n 4 --tier normal --count 10</Arguments>
```

**results-definition.xml** (parsing):
```xml
<OutputTemplate>Puzzles/second: #_RESULT_#</OutputTemplate>
```

**install.sh** (build):
```bash
git clone https://github.com/eirikr/rustykeen.git
cd rustykeen
cargo build --release -p kenken-cli --all-features
# Create wrapper that PTS will call
echo "#!/bin/sh
cd rustykeen
./target/release/kenken-cli count \$@ > \$LOG_FILE 2>&1
echo \$? > ~/test-exit-status" > ~/kenken-benchmark
```

**Running it:**
```bash
phoronix-test-suite benchmark pts/kenken-solver
```

---

## Key Findings

### 1. Similar Ecosystem Benchmarks

Examined real PTS profiles for reference:

| Profile | Type | Output | Use Case |
|---------|------|--------|----------|
| **asmFish** | Chess engine | Nodes/second | Algorithm throughput |
| **Primesieve** | Prime generator | Seconds, Primes | Multi-metric extraction |
| **XSBench** | Monte Carlo kernel | Lookups/second | Parameterized testing |
| **HPCC** | HPC Challenge | 8+ metrics | Advanced multi-result |

**Recommendation for rustykeen**: Follow primesieve pattern (clean single metric, optional extensions for time + throughput)

### 2. Build Automation

PTS handles:
- Automatic `cargo build --release` compilation
- Parallel builds via `$NUM_CPU_CORES`
- Cross-platform detection (Linux, macOS, Windows, BSD)
- Exit status tracking for failure detection
- Checksum validation for downloads

**Advantage**: Zero custom build logic needed in PTS profile

### 3. Result Submission

OpenBenchmarking.org integration:
```bash
phoronix-test-suite upload-result [result-id]
```

Returns publicly shareable URL for result comparison across systems. Optional but recommended for community adoption.

### 4. Profile Variants

Can create multiple profiles:
- `kenken-solver-1.0.0` - General purpose with options
- `kenken-solver-simd-1.0.0` - SIMD-optimized variant
- `kenken-solver-sat-1.0.0` - SAT solver backend
- `kenken-solver-suite` - Test suite grouping all variants

Enables direct performance comparison of optimization levels.

---

## Integration Checklist

### Pre-Integration (with rustykeen)
- [ ] CLI outputs `Puzzles/second: [number]` format
- [ ] CLI accepts `--n`, `--tier`, `--count` arguments
- [ ] Test runs in <30 seconds (recommended)
- [ ] Build completes in <5 minutes

### Profile Creation (5 minutes)
- [ ] Create directory: `~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/`
- [ ] Add `test-definition.xml` (copy from examples, customize)
- [ ] Add `results-definition.xml` (1 line parser template)
- [ ] Add `install.sh` (git clone + cargo build + wrapper)
- [ ] Run: `chmod +x install.sh`

### Validation (15 minutes)
- [ ] `phoronix-test-suite inspect-test-profile pts/kenken-solver` (parse check)
- [ ] `phoronix-test-suite debug-install pts/kenken-solver` (build check)
- [ ] `phoronix-test-suite debug-result-parser pts/kenken-solver` (parsing check)
- [ ] `phoronix-test-suite debug-run pts/kenken-solver` (end-to-end check)

### First Benchmark (5 minutes)
- [ ] `phoronix-test-suite benchmark pts/kenken-solver`
- [ ] Results automatically saved to `~/.phoronix-test-suite/test-results/`
- [ ] View results: `phoronix-test-suite show-result [id]`

### Optional: Public Sharing
- [ ] Create OpenBenchmarking.org account
- [ ] `phoronix-test-suite upload-result [result-id]`
- [ ] Share public URL

---

## PTS Commands Reference

| Task | Command |
|------|---------|
| Inspect profile | `phoronix-test-suite inspect-test-profile pts/kenken-solver` |
| Debug installation | `phoronix-test-suite debug-install pts/kenken-solver` |
| Debug execution | `phoronix-test-suite debug-run pts/kenken-solver` |
| Debug parsing | `phoronix-test-suite debug-result-parser pts/kenken-solver` |
| Run benchmark | `phoronix-test-suite benchmark pts/kenken-solver` |
| View results | `phoronix-test-suite show-result [id]` |
| Compare runs | `phoronix-test-suite compare-results r1 r2` |
| Generate graphs | `phoronix-test-suite graph-results [id]` |
| Upload results | `phoronix-test-suite upload-result [id]` |

---

## Expected Workflow

```
1. Read pts_integration_research.md (understand big picture)
2. Read pts_practical_guide.md Phase 1 (prepare CLI)
3. Follow pts_practical_guide.md Phase 2-3 (create & validate)
4. Run benchmark (Phase 4)
5. Create variants if needed (Phase 5)
6. Share results (Phase 6) - optional
7. Update project documentation
```

---

## Sources & References

### Official Resources
- **PTS GitHub**: https://github.com/phoronix-test-suite/phoronix-test-suite
- **Test Profiles**: https://github.com/phoronix-test-suite/test-profiles
- **OpenBenchmarking.org**: https://openbenchmarking.org/
- **Documentation**: `/documentation/test-profile-creation.md` in PTS repo

### Key PTS Features for rustykeen
- **Parameterized Testing**: User-selectable puzzle size, difficulty, count
- **Multi-Metric Extraction**: Puzzles/sec, solve time, both simultaneously
- **Automatic Graphing**: PNG graphs for result sets
- **Statistical Comparison**: Cross-system performance delta
- **Cross-Platform**: Linux/macOS/Windows/BSD support
- **Database Integration**: Public result sharing via OpenBenchmarking.org

---

## Next Steps

1. **Quick Start** (15 minutes):
   - Review pts_practical_guide.md Phase 1-2
   - Create profile directory and files
   - Run `phoronix-test-suite debug-install pts/kenken-solver`

2. **First Benchmark** (5 minutes):
   - Run `phoronix-test-suite benchmark pts/kenken-solver`
   - View results with `phoronix-test-suite show-result`

3. **Advanced** (Optional):
   - Create SIMD/DLX/SAT variants for comparison
   - Set up CI/CD integration (GitHub Actions)
   - Submit results to OpenBenchmarking.org

---

## Summary

The Phoronix Test Suite provides a production-grade benchmarking framework with minimal setup overhead:
- **3 files** needed (2 XML, 1 shell script)
- **600+ test ecosystem** available for reference
- **Parameterized** test variants without code changes
- **Public database** for global result comparison
- **Automated** graphing and statistical analysis

For rustykeen, PTS enables reproducible, shareable, and community-integrated performance benchmarking across puzzle sizes, difficulty levels, and optimization strategies.

---

**Documentation Created**: 2026-01-02
**PTS Version Tested**: 10.8.4
**Status**: Research complete, ready for implementation
