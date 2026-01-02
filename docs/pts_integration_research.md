# Phoronix Test Suite (PTS) Integration Research
## For KenKen Solver Benchmark Profile

---

## Executive Summary

The Phoronix Test Suite (v10.8.4) is a mature, open-source benchmarking platform that can integrate Rust CLI applications as custom test profiles. Integration requires creating XML metadata, shell installation/execution scripts, and result parsers. The system supports parameterized tests, multi-metric results, and database submission to OpenBenchmarking.org.

---

## 1. PTS Test Profile Structure & Requirements

### Directory Layout
A complete test profile consists of:

```
~/.phoronix-test-suite/test-profiles/
├── pts/
│   └── kenken-solver-1.0.0/          # Profile directory (namespace/name-version)
│       ├── test-definition.xml        # [REQUIRED] Metadata & test configuration
│       ├── results-definition.xml     # [REQUIRED] Output parsing rules
│       ├── install.sh                 # [REQUIRED] Installation script
│       ├── downloads.xml              # [OPTIONAL] External dependencies
│       ├── install_windows.sh         # [OPTIONAL] Windows variant
│       ├── changelog.json             # [OPTIONAL] Version history
│       ├── generated.json             # [AUTO] Metadata (auto-generated)
│       └── pre-install-message        # [OPTIONAL] Pre-install instructions
```

Alternative locations for local/private profiles:
- `~/.phoronix-test-suite/test-profiles/local/` (personal tests)
- `~/.phoronix-test-suite/test-profiles/git/` (git-sourced tests)
- System-wide: `/usr/share/phoronix-test-suite/` (if installed via package manager)

### Discovery & Naming
- Profiles are discovered via directory structure: `{namespace}/{name}-{version}/`
- Referenced as `namespace/name` in PTS commands (e.g., `pts/xsbench`, `local/kenken-solver`)
- Version automatically extracted from directory name

---

## 2. Required XML Files

### 2.1 test-definition.xml (Core Metadata)

**Location:** `test-definition.xml` in profile root

**Purpose:** Defines test metadata, platform support, execution parameters, and test options.

**Minimal Example (primesieve-style):**

```xml
<?xml version="1.0"?>
<PhoronixTestSuite>
  <TestInformation>
    <Title>KenKen Solver</Title>
    <AppVersion>0.0.1</AppVersion>
    <Description>Benchmark of the rustykeen KenKen puzzle solver across puzzle sizes 3-8 with varying difficulty levels.</Description>
    <ResultScale>Puzzles/second</ResultScale>
    <Proportion>HIB</Proportion>
    <TimesToRun>3</TimesToRun>
  </TestInformation>
  <TestProfile>
    <Version>1.0.0</Version>
    <SupportedPlatforms>Linux, BSD, Solaris, MacOSX, Windows</SupportedPlatforms>
    <SoftwareType>Utility</SoftwareType>
    <TestType>Processor</TestType>
    <License>Free</License>
    <Status>Verified</Status>
    <ExternalDependencies>build-utilities, cargo</ExternalDependencies>
    <EnvironmentSize>3</EnvironmentSize>
    <ProjectURL>https://github.com/eirikr/rustykeen</ProjectURL>
    <RepositoryURL>https://github.com/eirikr/rustykeen</RepositoryURL>
    <InternalTags>SMP, Solver, Constraint</InternalTags>
    <Maintainer>Eirikr</Maintainer>
  </TestProfile>
  <TestSettings>
    <Default>
      <Arguments>--n 4 --tier normal --count 10</Arguments>
    </Default>
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
        <Entry>
          <Name>6x6</Name>
          <Value>6</Value>
        </Entry>
      </Menu>
    </Option>
    <Option>
      <DisplayName>Difficulty</DisplayName>
      <Identifier>difficulty</Identifier>
      <ArgumentPrefix>--tier </ArgumentPrefix>
      <Menu>
        <Entry>
          <Name>Easy</Name>
          <Value>easy</Value>
        </Entry>
        <Entry>
          <Name>Normal</Name>
          <Value>normal</Value>
        </Entry>
        <Entry>
          <Name>Hard</Name>
          <Value>hard</Value>
        </Entry>
      </Menu>
    </Option>
  </TestSettings>
</PhoronixTestSuite>
```

**Key Elements:**

| Element | Purpose | Example |
|---------|---------|---------|
| `<Title>` | Human-readable test name | "KenKen Solver" |
| `<Description>` | Detailed test description | "Benchmark of the rustykeen KenKen puzzle solver..." |
| `<ResultScale>` | Unit of measurement | "Puzzles/second", "Seconds", "GFLOPS" |
| `<Proportion>` | Result direction: HIB = Higher Is Better, LIB = Lower Is Better | HIB (faster = better) |
| `<TimesToRun>` | Iterations per test execution | 3 |
| `<Version>` | Profile version (not app version) | "1.0.0" |
| `<TestType>` | Classification | "Processor", "System", "Disk", "Graphics" |
| `<Status>` | "Verified", "Deprecated", "In Development" | "Verified" |
| `<ExternalDependencies>` | Build requirements (pkgmgr names) | "build-utilities, cargo" |
| `<EnvironmentSize>` | Estimated disk space (GB) | 3 |
| `<Arguments>` in `<Default>` | CLI arguments passed to executable | "--n 4 --tier normal --count 10" |

**Test Options (Parameters):**
- Each `<Option>` creates a user-selectable menu
- `<ArgumentPrefix>` prepended to selected value when building command
- Users can choose variants at runtime or via CLI flags

---

### 2.2 results-definition.xml (Output Parsing)

**Location:** `results-definition.xml` in profile root

**Purpose:** Defines how to extract numeric results from test stdout.

**Example - Single Result:**

```xml
<?xml version="1.0"?>
<PhoronixTestSuite>
  <ResultsParser>
    <OutputTemplate>Puzzles/second: #_RESULT_#</OutputTemplate>
    <LineHint>Puzzles/second</LineHint>
    <StripFromResult>,</StripFromResult>
  </ResultsParser>
</PhoronixTestSuite>
```

**Example - Multiple Results (from HPCC):**

```xml
<?xml version="1.0"?>
<PhoronixTestSuite>
  <ResultsParser>
    <OutputTemplate>HPL_Tflops=#_RESULT_#</OutputTemplate>
    <MatchToTestArguments>HPL</MatchToTestArguments>
    <ResultScale>GFLOPS</ResultScale>
    <ResultProportion>HIB</ResultProportion>
    <ResultPrecision>5</ResultPrecision>
  </ResultsParser>
  <ResultsParser>
    <OutputTemplate>PTRANS_GBs=#_RESULT_#</OutputTemplate>
    <MatchToTestArguments>PTRANS</MatchToTestArguments>
    <ResultScale>GB/s</ResultScale>
    <ResultProportion>HIB</ResultProportion>
    <ResultPrecision>5</ResultPrecision>
  </ResultsParser>
</PhoronixTestSuite>
```

**Key Elements:**

| Element | Purpose |
|---------|---------|
| `<OutputTemplate>` | Regex-like pattern with `#_RESULT_#` placeholder for numeric value |
| `<LineHint>` | [OPTIONAL] String to identify correct output line (speeds up parsing) |
| `<MatchToTestArguments>` | [OPTIONAL] Match result to specific test arguments (for multi-result profiles) |
| `<StripFromResult>` | [OPTIONAL] Characters to remove (commas, percent signs, etc.) |
| `<ResultScale>` | [OPTIONAL] Unit override (if different from test-definition.xml) |
| `<ResultProportion>` | [OPTIONAL] HIB/LIB override |
| `<ResultPrecision>` | [OPTIONAL] Decimal places to preserve |

**Parsing Rules:**
- First `<ResultsParser>` block extracts first result
- Multiple blocks extract multiple metrics from single run
- `#_RESULT_#` is replaced with captured numeric value
- Whitespace and punctuation flexible

---

### 2.3 downloads.xml (Optional: External Downloads)

**Location:** `downloads.xml` in profile root (only if needed)

**Purpose:** Specify remote binaries or source code to download during installation.

**Example:**

```xml
<?xml version="1.0"?>
<PhoronixTestSuite>
  <Downloads>
    <Package>
      <URL>https://github.com/kimwalisch/primesieve/archive/refs/tags/v7.7.tar.gz</URL>
      <MD5>0ce76e78eb1111cdcb1e96856ff39e63</MD5>
      <SHA256>fcb3f25e68081c54e5d560d6d1f6448d384a7051e9c56d56ee0d65d6d7954db1</SHA256>
      <FileName>primesieve-7.7.tar.gz</FileName>
      <FileSize>101558</FileSize>
    </Package>
    <Package>
      <URL>https://github.com/eirikr/rustykeen/archive/refs/tags/v0.0.1.tar.gz</URL>
      <FileName>rustykeen-0.0.1.tar.gz</FileName>
      <FileSize>50000</FileSize>
      <PlatformSpecific>Linux, MacOSX</PlatformSpecific>
    </Package>
  </Downloads>
</PhoronixTestSuite>
```

**For rustykeen:**
- Can reference GitHub releases or master branch tarball
- PTS validates checksums and caches downloads
- If omitted, build via `cargo` (see install.sh)

---

## 3. Shell Scripts

### 3.1 install.sh (Installation & Setup)

**Location:** `install.sh` in profile root

**Purpose:** Download/build the application and create a wrapper executable.

**Execution Environment:**
- Runs in home directory (~)
- Provides environment variables:
  - `$NUM_CPU_CORES` - detected CPU count
  - `$OS_TYPE` - Linux, Windows, BSD, Solaris, MacOSX
  - `$OS_ARCH` - aarch64, x86_64, etc.
  - `$BENCHMARK_CACHE` - cache directory path

**Exit Status:**
- Write `echo $? > ~/install-exit-status` at end to signal success/failure

**Example 1: Source Build (primesieve):**

```bash
#!/bin/sh

version=7.7
tar xvf primesieve-$version.tar.gz
cd primesieve-$version

cmake . -DBUILD_SHARED_LIBS=OFF
make -j $NUM_CPU_CORES
echo $? > ~/install-exit-status
cd ~

echo "#!/bin/sh
primesieve-$version/./primesieve \$@ > \$LOG_FILE 2>&1
echo \$? > ~/test-exit-status" > primesieve-test
chmod +x primesieve-test
```

**Example 2: Rust Cargo Build (for rustykeen):**

```bash
#!/bin/sh

# Clone or extract rustykeen
git clone https://github.com/eirikr/rustykeen.git
cd rustykeen
cargo build --release -p kenken-cli --all-features
echo $? > ~/install-exit-status

cd ~

# Create wrapper script
echo "#!/bin/sh
cd rustykeen
./target/release/kenken-cli count \$@ > \$LOG_FILE 2>&1
echo \$? > ~/test-exit-status" > kenken-benchmark
chmod +x kenken-benchmark
```

**Key Conventions:**
1. Build artifacts compiled in subdirectory (e.g., `rustykeen/target/release/`)
2. Final step: Create executable script in `~` with fixed name (e.g., `kenken-benchmark`)
3. Executable receives CLI args via `$@` and outputs to `$LOG_FILE`
4. Always capture exit status to `~/test-exit-status`
5. Report installation exit status to `~/install-exit-status`

**Error Handling:**
```bash
# Handle build failures gracefully
make -j $NUM_CPU_CORES || { echo "1" > ~/install-exit-status; exit 1; }
```

---

### 3.2 install_windows.sh (Optional: Windows Build)

**Location:** `install_windows.sh` (if Windows build differs)

**Purpose:** Platform-specific installation for Windows.

For rustykeen: Likely identical to install.sh if Cargo handles cross-compilation.

---

## 4. Test Output and Result Format

### Result Output Requirements

The test executable (wrapper script) must output results in a format parseable by `results-definition.xml`.

**Example Valid Output:**

```
Solving 10 puzzles of size 4x4...
[████████████████████] 100% | 2.345 seconds

Puzzles/second: 4.263
```

PTS will:
1. Capture stdout
2. Search for line matching `LineHint` (if provided): "Puzzles/second"
3. Extract number matching `OutputTemplate` pattern
4. Run multiple iterations (3 times, per `<TimesToRun>`)
5. Aggregate results (median or mean, configurable)

**Exit Status Handling:**
- `~/test-exit-status` file with value `0` = success
- Any non-zero = failure (will be marked in results)

---

## 5. Test Parameters and Customization

### Default Arguments

In `test-definition.xml` `<TestSettings>`:

```xml
<TestSettings>
  <Default>
    <Arguments>--n 4 --tier normal --count 10</Arguments>
  </Default>
</TestSettings>
```

These become the baseline CLI invocation. Users can:
1. Select options from menus (each modifies arguments)
2. Override via command-line flags to PTS

### Dynamic Options

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

When user selects "4x4", PTS appends `--n 4` to arguments.

**For KenKen Solver**, useful options:
- **Puzzle Size**: 3, 4, 5, 6, 7, 8
- **Difficulty**: easy, normal, hard
- **Puzzle Count**: 5, 10, 20, 50
- **Algorithm**: baseline, dlx, sat (if exposed via CLI)
- **Optimization Level**: (can be set via separate test variants)

---

## 6. Multi-Metric and Multi-Run Profiles

### Multiple Metrics from Single Run

Use multiple `<ResultsParser>` blocks in `results-definition.xml`:

```xml
<?xml version="1.0"?>
<PhoronixTestSuite>
  <ResultsParser>
    <OutputTemplate>Total_Puzzles_Solved: #_RESULT_#</OutputTemplate>
    <ResultScale>Puzzles</ResultScale>
  </ResultsParser>
  <ResultsParser>
    <OutputTemplate>Solve_Time_Seconds: #_RESULT_#</OutputTemplate>
    <ResultScale>Seconds</ResultScale>
    <ResultProportion>LIB</ResultProportion>
  </ResultsParser>
</PhoronixTestSuite>
```

If output is:
```
Total_Puzzles_Solved: 100
Solve_Time_Seconds: 23.456
```

PTS extracts two results: 100 and 23.456 seconds.

### Multiple Test Variants (Test Suites)

Create separate test-definition.xml + results-definition.xml for each variant:
- `kenken-solver-3x3-easy/` - 3x3 puzzles, easy tier
- `kenken-solver-6x6-hard/` - 6x6 puzzles, hard tier
- `kenken-solver-simd/` - with SIMD optimizations

Then create a test suite (XML file) grouping them:

```xml
<?xml version="1.0"?>
<PhoronixTestSuite>
  <Suite>
    <Name>KenKen Solver Suite</Name>
    <Version>1.0.0</Version>
    <Description>Comprehensive KenKen solver benchmark across sizes and difficulties</Description>
    <Test>kenken-solver-3x3-easy</Test>
    <Test>kenken-solver-4x4-normal</Test>
    <Test>kenken-solver-6x6-hard</Test>
  </Suite>
</PhoronixTestSuite>
```

---

## 7. Result Aggregation and Comparison

### PTS Result Storage

After running a benchmark:
```bash
phoronix-test-suite benchmark pts/kenken-solver
```

Results are stored in:
- `~/.phoronix-test-suite/test-results/` (JSON + XML formats)
- Timestamped: `[timestamp]-kenken-solver-...-result-...-*/`

### Result Comparison

Compare two test runs:
```bash
phoronix-test-suite compare-results result-1 result-2
```

Generates side-by-side tables with:
- Average performance
- Variance/standard deviation
- Performance delta (% improvement/regression)

### Graphing

```bash
phoronix-test-suite graph-results result-name
```

Creates PNG/SVG graphs with:
- Bar charts across multiple test runs
- Trend analysis
- Statistical comparisons

---

## 8. Registering and Submitting Results

### OpenBenchmarking.org Integration

Upload results to public database:
```bash
phoronix-test-suite upload-result [result-dir]
```

**Steps:**
1. Run benchmark and save results
2. `phoronix-test-suite upload-result ~/.phoronix-test-suite/test-results/[result-id]`
3. PTS prompts for:
   - Title (e.g., "Intel i7-13700K - KenKen Solver Benchmark")
   - Optional description/notes
   - System tags (hardware, OS)
   - Visibility (public/private)
4. Results uploaded to OpenBenchmarking.org
5. Returns public URL for sharing

**Benefits:**
- Compare results across systems globally
- Build public performance database
- Track regressions over versions
- Community benchmarking

### Private Results

If not uploading, results remain local:
```bash
phoronix-test-suite run pts/kenken-solver  # saves locally only
```

### Result Format for Upload

PTS automatically includes:
- CPU info, RAM, kernel version
- Test parameters
- Raw measurement data
- Timestamp

No manual JSON structuring required.

---

## 9. Custom Test Profiles in Ecosystem

### Similar Algorithm/Solver Benchmarks

**asmFish (Chess Engine):**
- Location: `~/.phoronix-test-suite/test-profiles/pts/asmfish-1.1.2/`
- Metrics: Nodes/second
- Parameterization: Hash memory, search depth
- Output: `Nodes/second    : [number]`

**Primesieve (Prime Generator):**
- Location: `~/.phoronix-test-suite/test-profiles/pts/primesieve-1.8.0/`
- Metrics: Seconds (LIB - lower better), Primes generated
- Parameterization: Number range (1e12)
- Multi-result: Extraction of both time and prime count

**XSBench (Monte Carlo Kernel):**
- Location: `~/.phoronix-test-suite/test-profiles/pts/xsbench-1.0.0/`
- Metrics: Lookups/second (HIB)
- Parameterization: Thread count, lookup count, data size
- Simple single-metric extraction

**HPCC (HPC Challenge):**
- Multi-metric (8+ benchmarks in single run)
- Each metric matched to test arguments (HPL, PTRANS, etc.)
- Demonstrates advanced result parsing

---

## 10. Build and Execution Requirements

### Installation Requirements

PTS handles:
- Dependency resolution via system package manager
- Source code download + checksum verification
- Compilation with native toolchain
- Parallel build (respects `$NUM_CPU_CORES`)

### For Rust Projects

**ExternalDependencies** in test-definition.xml:
```xml
<ExternalDependencies>build-utilities, cargo</ExternalDependencies>
```

PTS resolves to:
- Linux (apt): `build-essential`, `cargo`
- macOS (brew): `rustup` or direct cargo
- Windows (choco/manual): Rust toolchain

**Cargo Detection:**
- PTS checks if `cargo` is in PATH
- Uses `cargo build` as standard build mechanism
- Respects `RUSTFLAGS`, `CARGO_BUILD_JOBS`, etc.

### Execution Requirements

**CLI Invocation Pattern:**
```bash
# User runs:
phoronix-test-suite benchmark pts/kenken-solver

# PTS internally runs (for each iteration):
~/kenken-benchmark --n 4 --tier normal --count 10 > /tmp/test-output.txt

# Script must:
1. Accept CLI arguments ($@)
2. Output results to file specified in $LOG_FILE env var
3. Write exit status to ~/test-exit-status
```

### Resource Management

- `<EnvironmentSize>` = disk space estimate (setup)
- `<EnvironmentTestingSize>` = disk space during test (larger puzzles)
- PTS warns if disk space insufficient

---

## 11. Advanced Features

### Sensor Monitoring

Capture CPU usage, temperature, memory during test:
```bash
# PTS can monitor system sensors if configured
# Results shown alongside benchmark metrics
```

### Reboot Handling

Tests can trigger system reboot (v10.6+):
```bash
# In test script:
if [ condition ]; then
  touch ~/reboot-needed  # Signal to reboot
fi

# After reboot, detect:
if [ ! -z "$TEST_RECOVERING_FROM_REBOOT" ]; then
  # Cleanup/resume logic
fi
```

### Pre/Post Install Messages

Create file: `pre-install-message`:
```
This test requires a GPU driver.
Please ensure CUDA/ROCm is installed before proceeding.
```

---

## 12. Debugging and Development

### PTS Commands for Profile Development

```bash
# Inspect parsed profile structure
phoronix-test-suite inspect-test-profile pts/kenken-solver

# Debug installation step-by-step
phoronix-test-suite debug-install pts/kenken-solver

# Debug test execution
phoronix-test-suite debug-run pts/kenken-solver

# Debug result parsing
phoronix-test-suite debug-result-parser pts/kenken-solver

# View exported environment variables
phoronix-test-suite diagnostics

# Create new test profile skeleton
phoronix-test-suite create-test-profile
```

### Common Issues

| Issue | Solution |
|-------|----------|
| Results not parsed | Check `OutputTemplate` regex; verify actual output matches |
| Exit status mismatch | Ensure script writes to `~/test-exit-status` |
| Installation fails silently | Check `install-exit-status`; use `debug-install` |
| Multi-result not extracted | Verify `<ResultsParser>` blocks order matches output order |
| Test runs sequentially on SMP | Check `<InternalTags>SMP</InternalTags>` if parallelizable |
| Parameters not applied | Verify `<ArgumentPrefix>` matches CLI syntax |

---

## 13. Example: Complete KenKen Solver Profile

### Directory Structure
```
~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/
├── test-definition.xml
├── results-definition.xml
├── install.sh
└── downloads.xml (optional)
```

### test-definition.xml
```xml
<?xml version="1.0"?>
<PhoronixTestSuite>
  <TestInformation>
    <Title>KenKen Solver</Title>
    <AppVersion>0.0.1</AppVersion>
    <Description>Deterministic KenKen puzzle solver from rustykeen. Tests constraint propagation and backtracking performance across puzzle sizes 3-8.</Description>
    <ResultScale>Puzzles/second</ResultScale>
    <Proportion>HIB</Proportion>
    <SubTitle>Puzzle Size 4x4, Normal Difficulty</SubTitle>
    <TimesToRun>3</TimesToRun>
  </TestInformation>
  <TestProfile>
    <Version>1.0.0</Version>
    <SupportedPlatforms>Linux, BSD, MacOSX, Windows</SupportedPlatforms>
    <SoftwareType>Benchmark</SoftwareType>
    <TestType>Processor</TestType>
    <License>MIT</License>
    <Status>Verified</Status>
    <ExternalDependencies>build-utilities, cargo</ExternalDependencies>
    <EnvironmentSize>2</EnvironmentSize>
    <EnvironmentTestingSize>0.1</EnvironmentTestingSize>
    <ProjectURL>https://github.com/eirikr/rustykeen</ProjectURL>
    <RepositoryURL>https://github.com/eirikr/rustykeen</RepositoryURL>
    <InternalTags>SMP, Solver, Constraint</InternalTags>
    <Maintainer>Eirikr</Maintainer>
  </TestProfile>
  <TestSettings>
    <Default>
      <Arguments>--n 4 --tier normal --count 10</Arguments>
    </Default>
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
        <Entry>
          <Name>5x5</Name>
          <Value>5</Value>
        </Entry>
        <Entry>
          <Name>6x6</Name>
          <Value>6</Value>
        </Entry>
      </Menu>
    </Option>
    <Option>
      <DisplayName>Difficulty Tier</DisplayName>
      <Identifier>difficulty-tier</Identifier>
      <ArgumentPrefix>--tier </ArgumentPrefix>
      <Menu>
        <Entry>
          <Name>Easy</Name>
          <Value>easy</Value>
        </Entry>
        <Entry>
          <Name>Normal</Name>
          <Value>normal</Value>
        </Entry>
        <Entry>
          <Name>Hard</Name>
          <Value>hard</Value>
        </Entry>
      </Menu>
    </Option>
    <Option>
      <DisplayName>Puzzle Count</DisplayName>
      <Identifier>puzzle-count</Identifier>
      <ArgumentPrefix>--count </ArgumentPrefix>
      <Menu>
        <Entry>
          <Name>5 puzzles</Name>
          <Value>5</Value>
        </Entry>
        <Entry>
          <Name>10 puzzles</Name>
          <Value>10</Value>
        </Entry>
        <Entry>
          <Name>20 puzzles</Name>
          <Value>20</Value>
        </Entry>
      </Menu>
    </Option>
  </TestSettings>
</PhoronixTestSuite>
```

### results-definition.xml
```xml
<?xml version="1.0"?>
<PhoronixTestSuite>
  <ResultsParser>
    <OutputTemplate>Puzzles/second: #_RESULT_#</OutputTemplate>
    <LineHint>Puzzles/second</LineHint>
  </ResultsParser>
</PhoronixTestSuite>
```

### install.sh
```bash
#!/bin/sh

# Clone rustykeen repository
git clone https://github.com/eirikr/rustykeen.git
cd rustykeen

# Build release binary
cargo build --release -p kenken-cli --all-features

# Check build success
BUILD_RESULT=$?
echo $BUILD_RESULT > ~/install-exit-status

if [ $BUILD_RESULT -ne 0 ]; then
  exit 1
fi

cd ~

# Create wrapper script for benchmark execution
echo "#!/bin/sh
cd rustykeen
./target/release/kenken-cli count \$@ > \$LOG_FILE 2>&1
echo \$? > ~/test-exit-status" > kenken-benchmark

chmod +x kenken-benchmark
```

### CLI Output Format (from kenken-cli)
```
Solving 10 puzzles of size 4x4 (difficulty: normal)...
[████████████████████] 100% | 2.345 seconds
Total puzzles solved: 10
Puzzles/second: 4.263
```

---

## 14. Integration Checklist

### Pre-Integration
- [ ] Rust CLI application builds with `cargo build --release`
- [ ] CLI accepts `--help` for argument documentation
- [ ] CLI outputs results in parseable format (numeric + label)
- [ ] Test completes in <30 seconds (recommended for practicality)
- [ ] Install script can build from scratch in <5 minutes

### Profile Structure
- [ ] Create `~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/` directory
- [ ] Create `test-definition.xml` with metadata
- [ ] Create `results-definition.xml` with output parser
- [ ] Create `install.sh` (executable, mode 755)
- [ ] Create `downloads.xml` if using external downloads (optional)

### Testing
- [ ] Run: `phoronix-test-suite debug-install pts/kenken-solver`
- [ ] Verify: `~/install-exit-status` is 0
- [ ] Run: `phoronix-test-suite debug-run pts/kenken-solver`
- [ ] Verify: Results extracted correctly
- [ ] Run: `phoronix-test-suite inspect-test-profile pts/kenken-solver`
- [ ] Run with options: `phoronix-test-suite benchmark pts/kenken-solver --size 6`

### Finalization
- [ ] Register on OpenBenchmarking.org (optional)
- [ ] Submit profile via `phoronix-test-suite upload-test-profile`
- [ ] Document in project README.md
- [ ] Tag release with profile version
- [ ] Update CLAUDE.md with PTS benchmark instructions

---

## 15. Real-World Examples in PTS Repository

### GitHub References
- **Official PTS GitHub:** https://github.com/phoronix-test-suite/phoronix-test-suite
- **Test Profiles Repository:** https://github.com/phoronix-test-suite/test-profiles
- **Documentation:** `/documentation/test-profile-creation.md` in repo

### OpenBenchmarking.org
- **Test Registry:** https://openbenchmarking.org/tests/pts
- **Example Test Page:** https://openbenchmarking.org/test/pts/asmfish
- **Test Suites:** https://openbenchmarking.org/suites/pts

---

## 16. Key Takeaways for rustykeen Integration

1. **Minimal Setup**: Only 3 required files (test-definition.xml, results-definition.xml, install.sh)

2. **Clean Workflow**:
   - User runs: `phoronix-test-suite benchmark pts/kenken-solver`
   - PTS handles: download, build, run iterations, parse results, aggregate, graph, upload

3. **Parameterization**: Menu-driven options for puzzle size/difficulty without script changes

4. **Multi-Metric Support**: Can extract solve time AND puzzles/sec from single run

5. **Community Database**: Optional sharing via OpenBenchmarking.org for cross-system comparison

6. **Reproducibility**: Exit statuses, checksums, versioning ensure repeatable benchmarks

7. **Debugging**: `debug-*` commands help profile development without full benchmark cycle

8. **Platform Agnostic**: Same profile works on Linux, macOS, Windows, BSD with minimal tweaks

---

## Conclusion

PTS provides a production-grade benchmarking framework perfect for rustykeen. The integration is straightforward: write 3 XML/shell files, validate with debug commands, and you have a fully parameterizable, shareable, results-uploadable benchmark profile integrated into a 600+ test ecosystem used by thousands globally.
