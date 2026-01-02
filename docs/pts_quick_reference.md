# Phoronix Test Suite - Quick Reference Card

## File Structure

```
~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/
├── test-definition.xml      [REQUIRED] Metadata + CLI options
├── results-definition.xml   [REQUIRED] Output parser rules
├── install.sh               [REQUIRED] Build/setup script
├── downloads.xml            [OPTIONAL] Remote binaries
└── install_windows.sh       [OPTIONAL] Windows-specific build
```

## XML Essentials

### test-definition.xml - Required Elements

```xml
<TestInformation>
  <Title>                     Human name
  <AppVersion>                App version string
  <Description>               Full description
  <ResultScale>               Unit (e.g., "Puzzles/second")
  <Proportion>                HIB (higher=better) or LIB (lower=better)
  <TimesToRun>                Iterations per test (e.g., 3)
</TestInformation>

<TestProfile>
  <Version>                   Profile version (e.g., 1.0.0)
  <SupportedPlatforms>        Linux, BSD, MacOSX, Windows
  <TestType>                  Processor, System, Disk, Graphics
  <Status>                    Verified, Deprecated, In Development
  <ExternalDependencies>      build-utilities, cargo
</TestProfile>

<TestSettings>
  <Default>
    <Arguments>               CLI args to pass to executable
  </Default>
  <Option>                    Create user menu for parameter
    <DisplayName>             Shown to user
    <Identifier>              Internal ID
    <ArgumentPrefix>          Prepended to selected value
    <Menu>
      <Entry>
        <Name>                Display name
        <Value>               Passed to CLI
```

### results-definition.xml - Simple Parser

```xml
<ResultsParser>
  <OutputTemplate>            Pattern with #_RESULT_# placeholder
  <LineHint>                  [OPT] Text to find correct line
  <StripFromResult>           [OPT] Chars to remove (commas, %)
</ResultsParser>
```

## Shell Script Essentials

### install.sh Template

```bash
#!/bin/sh

# Download/clone
git clone [repo-url]
cd [project]

# Build (use $NUM_CPU_CORES for parallelism)
cargo build --release -p [binary] --all-features
BUILD_RESULT=$?
echo $BUILD_RESULT > ~/install-exit-status
[ $BUILD_RESULT -ne 0 ] && exit 1

cd ~

# Create wrapper
echo "#!/bin/sh
cd [project]
./target/release/[binary] \$@ > \$LOG_FILE 2>&1
echo \$? > ~/test-exit-status" > [executable-name]
chmod +x [executable-name]
```

## PTS Commands

| Command | Purpose |
|---------|---------|
| `inspect-test-profile [test]` | Show parsed profile structure |
| `debug-install [test]` | Run installation step-by-step |
| `debug-run [test]` | Execute test without saving |
| `debug-result-parser [test]` | Test output parsing |
| `benchmark [test]` | Install, run, save results |
| `compare-results [r1] [r2]` | Side-by-side comparison |
| `graph-results [result]` | Generate PNG graphs |
| `upload-result [result]` | Submit to OpenBenchmarking.org |
| `show-result [result]` | Display result details |

## Output Format Examples

### Single Metric
```
Solving puzzles...
Puzzles/second: 4.263
```

### Multiple Metrics
```
Metric_A: 100
Metric_B: 23.456
```

### With Extraction
```
== Results ==
Performance (Puzzles/sec): 4.263
Memory: 256 MB
```

## Environment Variables (in scripts)

| Variable | Value |
|----------|-------|
| `$NUM_CPU_CORES` | Detected CPU count |
| `$OS_TYPE` | Linux, Windows, BSD, MacOSX, Solaris |
| `$OS_ARCH` | x86_64, aarch64, arm, etc. |
| `$LOG_FILE` | Output file path |
| `$BENCHMARK_CACHE` | Cache directory |

## Workflow

```
1. Create profile structure (~/.phoronix-test-suite/test-profiles/pts/name-version/)
2. Write XML metadata files (test-definition.xml, results-definition.xml)
3. Write install.sh (build + wrapper creation)
4. Test: phoronix-test-suite debug-install [test]
5. Test: phoronix-test-suite debug-result-parser [test]
6. Run: phoronix-test-suite benchmark [test]
7. Analyze: phoronix-test-suite compare-results / graph-results
8. Share: phoronix-test-suite upload-result [result-id]
```

## Test-Definition.xml Options Block

```xml
<Option>
  <DisplayName>Puzzle Size</DisplayName>
  <Identifier>puzzle-size</Identifier>
  <ArgumentPrefix>--n </ArgumentPrefix>
  <Menu>
    <Entry>
      <Name>3x3</Name>
      <Value>3</Value>
    </Entry>
    <Entry>
      <Name>4x4</Name>
      <Value>4</Value>
    </Entry>
  </Menu>
</Option>
```

When user selects "4x4", PTS executes:
```
./executable --n 4 [other-args]
```

## Result Parsing Examples

### Parse Single Number
```xml
<OutputTemplate>Puzzles/second: #_RESULT_#</OutputTemplate>
<LineHint>Puzzles/second</LineHint>
```
Extracts from: `Puzzles/second: 4.263` → 4.263

### Parse Multiple Results
```xml
<ResultsParser>
  <OutputTemplate>Total: #_RESULT_# puzzles</OutputTemplate>
</ResultsParser>
<ResultsParser>
  <OutputTemplate>Time: #_RESULT_# seconds</OutputTemplate>
</ResultsParser>
```
Extracts both from same output.

### Clean Up Formatting
```xml
<StripFromResult>,</StripFromResult>
```
Converts: `1,234.56` → `1234.56`

## Multi-Result Profile Pattern

```xml
<!-- results-definition.xml with MatchToTestArguments -->
<ResultsParser>
  <OutputTemplate>HPL_Result=#_RESULT_#</OutputTemplate>
  <MatchToTestArguments>HPL</MatchToTestArguments>
</ResultsParser>
<ResultsParser>
  <OutputTemplate>PTRANS_Result=#_RESULT_#</OutputTemplate>
  <MatchToTestArguments>PTRANS</MatchToTestArguments>
</ResultsParser>
```

Each `<ResultsParser>` matched to test arguments to extract multiple metrics.

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| XML not well-formed | `xmllint file.xml` to validate |
| Output format mismatch | Debug with `debug-result-parser` |
| install.sh not executable | `chmod +x install.sh` |
| Exit status not captured | Add `echo $? > ~/install-exit-status` |
| Wrong argument format | Verify `ArgumentPrefix` matches CLI syntax |
| Results not aggregated | Check `TimesToRun` value |
| Profile not found | Verify directory structure: `pts/[name]-[version]/` |

## Result Output Locations

```
Results: ~/.phoronix-test-suite/test-results/[timestamp]-[test]-[id]/
- results.json                    Raw result data
- result-definition.xml           Parser metadata
- installation-logs/              Build output
- system-logs/                    Hardware info
- composite.png                   Generated graph
```

## Upload to OpenBenchmarking.org

```bash
phoronix-test-suite upload-result \
  ~/.phoronix-test-suite/test-results/[result-id]

# Prompts for:
# - Title
# - Description
# - System tags
# - Visibility (public/private)

# Returns URL: https://openbenchmarking.org/result/[hash]
```

## Quick Test

After creating profile:

```bash
# 1. Inspect
phoronix-test-suite inspect-test-profile pts/kenken-solver

# 2. Debug install
phoronix-test-suite debug-install pts/kenken-solver
# Check: ls ~/kenken-benchmark

# 3. Debug result parsing
echo "Puzzles/second: 4.263" | \
  phoronix-test-suite debug-result-parser pts/kenken-solver

# 4. Run benchmark
phoronix-test-suite benchmark pts/kenken-solver

# 5. Compare
phoronix-test-suite compare-results result1 result2

# 6. Graph
phoronix-test-suite graph-results result1
```

## Feature Flags

### For Rust projects
```xml
<ExternalDependencies>build-utilities, cargo</ExternalDependencies>
```

### Custom build in install.sh
```bash
cargo build --release \
  --all-features \
  --no-default-features \
  --features simd-dispatch
```

## Version Format

Profile location: `namespace/name-X.Y.Z/`
- Extracted automatically from directory
- Used in result tracking
- Updated when profile changes

Example: `kenken-solver-1.0.0/` → version "1.0.0"

## Proportion: HIB vs LIB

| Type | Meaning | Example |
|------|---------|---------|
| HIB | Higher Is Better | Throughput (ops/sec) |
| LIB | Lower Is Better | Time (seconds), Latency |

Affects result interpretation and graphing.

## PTS Features Used

- **Parameterized tests**: User-selectable options at runtime
- **Multi-result extraction**: Extract multiple metrics from single run
- **Automatic aggregation**: Multi-iteration averaging
- **Cross-platform**: Linux, macOS, Windows, BSD, Solaris
- **Result comparison**: Statistical comparison across runs
- **Automated graphing**: Generate publication-quality graphs
- **Public database**: OpenBenchmarking.org integration
- **Exit status tracking**: Detect build/test failures
- **Environment variables**: Adapt to system configuration

## Helpful Files

- GitHub: https://github.com/phoronix-test-suite/test-profiles
- Docs: https://github.com/phoronix-test-suite/phoronix-test-suite/blob/master/documentation/
- OpenBenchmarking: https://openbenchmarking.org/tests/pts
