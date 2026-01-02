# Phoronix Test Suite - Practical Implementation Guide
## Step-by-Step for rustykeen KenKen Solver

---

## Phase 1: Prepare rustykeen CLI

### Step 1.1: Verify CLI Output Format

First, ensure the CLI outputs results in a format compatible with PTS parsing.

**Test current output:**
```bash
cd /home/eirikr/Github/rustykeen
cargo build --release -p kenken-cli --all-features
cargo run --release --bin kenken-cli -- count --n 4 --tier normal --count 10
```

**Desired output format:**
```
Solving 10 puzzles of size 4x4 (tier: normal)...
Progress: [████████████████████] 100%
Total time: 2.345 seconds
Puzzles solved: 10
Puzzles/second: 4.263
```

**If current format differs**, modify CLI output or adjust `results-definition.xml` to match.

### Step 1.2: Add Performance Output to CLI (if needed)

Modify `kenken-cli/src/main.rs` to output metrics:

```rust
// Pseudocode example
let start = std::time::Instant::now();

// Solve puzzles here
let solved_count = solve_puzzles(puzzles, &mut solver);

let elapsed = start.elapsed().as_secs_f64();
let rate = solved_count as f64 / elapsed;

println!("Puzzles/second: {:.3}", rate);
eprintln!("Metadata: {}", json!({
    "puzzles": solved_count,
    "time_secs": elapsed,
    "rate_per_sec": rate
}));
```

### Step 1.3: Test Argument Passing

Ensure CLI correctly handles benchmark parameters:

```bash
# Test basic invocation
./target/release/kenken-cli count --n 3 --tier easy --count 5

# Test with various sizes
for size in 3 4 5 6; do
  echo "Testing size $size..."
  ./target/release/kenken-cli count --n $size --tier normal --count 3
done

# Test with different tiers
for tier in easy normal hard; do
  echo "Testing tier $tier..."
  ./target/release/kenken-cli count --n 4 --tier $tier --count 3
done
```

---

## Phase 2: Create PTS Profile Structure

### Step 2.1: Create Profile Directory

```bash
# Create profile directory
mkdir -p ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0

# Verify
ls -la ~/.phoronix-test-suite/test-profiles/pts/ | grep kenken
```

### Step 2.2: Create test-definition.xml

```bash
cat > ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/test-definition.xml << 'EOF'
<?xml version="1.0"?>
<PhoronixTestSuite>
  <TestInformation>
    <Title>KenKen Solver</Title>
    <AppVersion>0.0.1</AppVersion>
    <Description>Deterministic KenKen puzzle solver from rustykeen. Benchmarks constraint propagation and backtracking performance across puzzle sizes and difficulty levels.</Description>
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
EOF

# Verify created
cat ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/test-definition.xml | head -20
```

### Step 2.3: Create results-definition.xml

```bash
cat > ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/results-definition.xml << 'EOF'
<?xml version="1.0"?>
<PhoronixTestSuite>
  <ResultsParser>
    <OutputTemplate>Puzzles/second: #_RESULT_#</OutputTemplate>
    <LineHint>Puzzles/second</LineHint>
  </ResultsParser>
</PhoronixTestSuite>
EOF

# Verify
cat ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/results-definition.xml
```

### Step 2.4: Create install.sh

```bash
cat > ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/install.sh << 'EOF'
#!/bin/sh

# Clone rustykeen repository
git clone https://github.com/eirikr/rustykeen.git
cd rustykeen

# Build release binary with all features
cargo build --release -p kenken-cli --all-features

# Capture build exit status
BUILD_RESULT=$?
echo $BUILD_RESULT > ~/install-exit-status

# Exit if build failed
if [ $BUILD_RESULT -ne 0 ]; then
  exit 1
fi

cd ~

# Create wrapper script for PTS to call
echo "#!/bin/sh
cd rustykeen
./target/release/kenken-cli count \$@ > \$LOG_FILE 2>&1
echo \$? > ~/test-exit-status" > kenken-benchmark

chmod +x kenken-benchmark
EOF

# Make install.sh executable
chmod +x ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/install.sh

# Verify
cat ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/install.sh
```

### Step 2.5: Verify Profile Structure

```bash
# List all files in profile directory
ls -la ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/

# Expected output:
# -rw-r--r-- test-definition.xml
# -rw-r--r-- results-definition.xml
# -rwxr-xr-x install.sh
```

---

## Phase 3: Debug and Validate Profile

### Step 3.1: Inspect Profile

```bash
# PTS will parse and display the profile structure
phoronix-test-suite inspect-test-profile pts/kenken-solver

# Expected output shows:
# - Test title: KenKen Solver
# - Supported platforms
# - Test options (Puzzle Size, Difficulty Tier, etc.)
```

### Step 3.2: Dry-Run Installation

```bash
# This will download/build without running the test
phoronix-test-suite debug-install pts/kenken-solver

# Monitor output:
# - Clone rustykeen
# - cargo build progress
# - Exit status written to ~/install-exit-status

# After completion, verify wrapper exists:
ls -la ~/kenken-benchmark
./kenken-benchmark --help  # Should work
```

### Step 3.3: Debug Result Parsing

```bash
# Generate test output and verify parser
phoronix-test-suite debug-result-parser pts/kenken-solver

# Simulate CLI output:
echo "Solving 10 puzzles of size 4x4...
Puzzles/second: 4.263" | phoronix-test-suite debug-result-parser pts/kenken-solver

# Expected: extracts 4.263 as result
```

### Step 3.4: Dry-Run Test Execution

```bash
# Run test without saving results
phoronix-test-suite debug-run pts/kenken-solver

# This will:
# 1. Use installed ~/kenken-benchmark
# 2. Invoke with default args: --n 4 --tier normal --count 10
# 3. Capture output
# 4. Parse results
# 5. Display to console (no saving)
```

---

## Phase 4: Run and Analyze

### Step 4.1: Run Benchmark (Single Option Set)

```bash
# Run with default options (3 iterations)
phoronix-test-suite benchmark pts/kenken-solver

# PTS will:
# - Prompt for test title (e.g., "Intel i7 - Baseline")
# - Run 3 times automatically
# - Save results with timestamp

# Results saved to:
# ~/.phoronix-test-suite/test-results/[timestamp]-kenken-solver-...
```

### Step 4.2: Run with Custom Options

```bash
# Run with 6x6 puzzles, hard difficulty
phoronix-test-suite benchmark pts/kenken-solver
# Select menu options when prompted:
# - Puzzle Size: 6x6
# - Difficulty Tier: Hard
# - Puzzle Count: 20 puzzles
```

### Step 4.3: View Results

```bash
# List all saved results
ls ~/.phoronix-test-suite/test-results/

# View specific result details
phoronix-test-suite show-result [result-id]

# Generate comparison table
phoronix-test-suite compare-results [result-1-id] [result-2-id]
```

### Step 4.4: Generate Graphs

```bash
# Create PNG graphs
phoronix-test-suite graph-results [result-id]

# Output saved to:
# ~/.phoronix-test-suite/test-results/[result-id]/composite.png
# (Can view with image viewer or browser)
```

---

## Phase 5: Create Test Variants (Optional)

For comprehensive benchmarking, create separate profiles for specific configurations:

### Step 5.1: Create Size-Specific Variant

```bash
# Create 6x6 variant
mkdir -p ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-6x6-1.0.0

# test-definition.xml (with fixed 6x6)
cat > ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-6x6-1.0.0/test-definition.xml << 'EOF'
<?xml version="1.0"?>
<PhoronixTestSuite>
  <TestInformation>
    <Title>KenKen Solver 6x6</Title>
    <AppVersion>0.0.1</AppVersion>
    <Description>KenKen solver benchmarking 6x6 puzzles across difficulty levels.</Description>
    <ResultScale>Puzzles/second</ResultScale>
    <Proportion>HIB</Proportion>
    <SubTitle>Fixed 6x6, Scalable Difficulty</SubTitle>
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
    <ProjectURL>https://github.com/eirikr/rustykeen</ProjectURL>
    <InternalTags>SMP, Solver, Constraint</InternalTags>
    <Maintainer>Eirikr</Maintainer>
  </TestProfile>
  <TestSettings>
    <Default>
      <Arguments>--n 6 --tier normal --count 5</Arguments>
    </Default>
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
  </TestSettings>
</PhoronixTestSuite>
EOF

# Copy install.sh and results-definition.xml (same as base)
cp ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/install.sh \
   ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-6x6-1.0.0/
cp ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/results-definition.xml \
   ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-6x6-1.0.0/
```

### Step 5.2: Create Test Suite (Groups Multiple Variants)

```bash
# Create suite file
cat > ~/.phoronix-test-suite/test-suites/kenken-comprehensive.xml << 'EOF'
<?xml version="1.0"?>
<PhoronixTestSuite>
  <Suite>
    <Name>KenKen Comprehensive Benchmark</Name>
    <Version>1.0.0</Version>
    <Description>Complete benchmark suite for rustykeen KenKen solver across all puzzle sizes and difficulties.</Description>
    <Test>pts/kenken-solver</Test>
    <Test>pts/kenken-solver-6x6</Test>
  </Suite>
</PhoronixTestSuite>
EOF

# Run entire suite
phoronix-test-suite run-tests-in-suite kenken-comprehensive
```

---

## Phase 6: Submit Results to OpenBenchmarking.org

### Step 6.1: Create OpenBenchmarking.org Account

1. Visit: https://openbenchmarking.org/
2. Click "Create Account"
3. Verify email

### Step 6.2: Upload Result

```bash
# After running benchmark
phoronix-test-suite upload-result

# Or specify result ID
phoronix-test-suite upload-result ~/.phoronix-test-suite/test-results/[result-id]

# PTS prompts for:
# - Title: "Intel Core i9-13900K @ 5.8GHz - KenKen Solver Benchmark"
# - Description: "Baseline performance on x86-64-v3 tuned build"
# - System tags: (OS, CPU, RAM)
# - Visibility: Public / Private

# Returns shareable URL (e.g., https://openbenchmarking.org/result/[hash])
```

### Step 6.3: Create Custom Test Profile on OpenBenchmarking.org

```bash
# Submit profile itself
phoronix-test-suite upload-test-profile

# Allows others to run:
# phoronix-test-suite benchmark pts/kenken-solver
# (without manual profile creation)
```

---

## Phase 7: Advanced Features

### Step 7.1: Performance Variants (SIMD, DLX, SAT)

Create profiles for each optimization:

```bash
# Create SIMD variant
mkdir -p ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-simd-1.0.0

# Modify install.sh to use feature flags
cat > ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-simd-1.0.0/install.sh << 'EOF'
#!/bin/sh
git clone https://github.com/eirikr/rustykeen.git
cd rustykeen
# Enable SIMD dispatch
cargo build --release -p kenken-cli --all-features --no-default-features --features simd-dispatch
echo $? > ~/install-exit-status
# ... rest same as before
EOF
```

Then compare:
```bash
# Run both variants
phoronix-test-suite benchmark pts/kenken-solver
phoronix-test-suite benchmark pts/kenken-solver-simd

# Compare results
phoronix-test-suite compare-results baseline-result simd-result
```

### Step 7.2: Track Regressions Over Commits

```bash
# After each major commit, run benchmark
git tag -a v0.0.2-bench-run -m "Performance baseline for v0.0.2"
phoronix-test-suite benchmark pts/kenken-solver

# Create comparison across versions
phoronix-test-suite compare-results v0.0.1-result v0.0.2-result
# Highlights performance delta (% improvement/regression)
```

### Step 7.3: Stress Testing

```bash
# Run same test repeatedly to detect regressions/variance
phoronix-test-suite stress-run pts/kenken-solver

# Or run with extended iterations
phoronix-test-suite benchmark pts/kenken-solver --runs 10
```

---

## Phase 8: Integration with CI/CD (Optional)

### Step 8.1: GitHub Actions Workflow

Create `.github/workflows/benchmark.yml`:

```yaml
name: KenKen Benchmark

on:
  release:
    types: [published]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y phoronix-test-suite

      - name: Setup PTS profile
        run: |
          mkdir -p ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0
          cp pts-profile/* ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/

      - name: Run benchmark
        run: |
          phoronix-test-suite benchmark pts/kenken-solver \
            --title "KenKen Solver ${{ github.ref_name }}" \
            --run-count 5

      - name: Upload results
        run: |
          phoronix-test-suite upload-result [result-id]
          # Get result URL and post to release notes
```

---

## Phase 9: Documentation

### Step 9.1: Update Project README.md

Add section:

```markdown
## Performance Benchmarking

### Using Phoronix Test Suite

To benchmark the KenKen solver with PTS:

```bash
# Install PTS (if not already installed)
sudo apt install phoronix-test-suite  # or brew, pacman, etc.

# Install and run benchmark
phoronix-test-suite benchmark pts/kenken-solver

# View results
phoronix-test-suite show-result [result-id]

# Generate graphs
phoronix-test-suite graph-results [result-id]

# Compare multiple runs
phoronix-test-suite compare-results result-1 result-2
```

### Benchmark Parameters

- **Puzzle Size**: 3x3 to 6x6 grids
- **Difficulty**: Easy (max 2 cages), Normal (varied), Hard (many 1-cages)
- **Puzzle Count**: 5 to 20 puzzles per run
- **Metrics**: Puzzles/second (higher is better)

### Upload Results

Share performance data publicly on OpenBenchmarking.org:

```bash
phoronix-test-suite upload-result [result-id]
```

Results dashboard: https://openbenchmarking.org/user/[your-account]
```

### Step 9.2: Update CLAUDE.md

Add section:

```markdown
## Benchmark Integration

### Phoronix Test Suite Profile

A PTS benchmark profile is available in `~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/`.

**Key files:**
- `test-definition.xml` - Metadata, options, CLI parameters
- `results-definition.xml` - Output parsing rules
- `install.sh` - Build and test setup

**Usage:**
```bash
phoronix-test-suite benchmark pts/kenken-solver
```

**Options at runtime:**
- Puzzle Size: 3, 4, 5, 6
- Difficulty: Easy, Normal, Hard
- Puzzle Count: 5, 10, 20

**Result Submission:**
```bash
phoronix-test-suite upload-result [result-id]
# Publicly available at: https://openbenchmarking.org/
```

**Comparison:**
```bash
phoronix-test-suite compare-results baseline simd-optimized
```
```

---

## Troubleshooting

### Issue: "test-definition.xml not found"

**Solution:**
```bash
# Verify file exists
ls ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/test-definition.xml

# Check XML syntax
xmllint ~/.phoronix-test-suite/test-profiles/pts/kenken-solver-1.0.0/test-definition.xml
```

### Issue: Results not parsed

**Solution:**
```bash
# Check actual CLI output
./kenken-benchmark --n 4 --tier normal --count 10

# Verify output format matches results-definition.xml template
# Expected: "Puzzles/second: [number]"

# Debug parser
phoronix-test-suite debug-result-parser pts/kenken-solver
```

### Issue: Installation fails

**Solution:**
```bash
# Run debug install
phoronix-test-suite debug-install pts/kenken-solver

# Check install exit status
cat ~/install-exit-status  # Should be 0

# Check build manually
cd ~/rustykeen
cargo build --release -p kenken-cli --all-features

# Check wrapper script
cat ~/kenken-benchmark
```

### Issue: Cargo not found

**Solution:**
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Update PTS dependencies
phoronix-test-suite install-dependencies pts/kenken-solver
```

---

## Verification Checklist

After following all phases:

- [ ] CLI outputs `Puzzles/second: [number]` format
- [ ] `test-definition.xml` is valid XML (xmllint passes)
- [ ] `results-definition.xml` parses expected output format
- [ ] `install.sh` is executable and builds successfully
- [ ] `phoronix-test-suite inspect-test-profile pts/kenken-solver` works
- [ ] `phoronix-test-suite debug-install pts/kenken-solver` completes
- [ ] `phoronix-test-suite debug-run pts/kenken-solver` extracts results
- [ ] `phoronix-test-suite benchmark pts/kenken-solver` runs and saves results
- [ ] Results can be compared and graphed
- [ ] Results can be uploaded to OpenBenchmarking.org (optional)

---

## Next Steps

1. **Implement Phase 1** - Verify CLI output format
2. **Implement Phase 2-3** - Create and validate profile
3. **Implement Phase 4** - Run and analyze results
4. **Implement Phase 5-6** - Create variants and submit
5. **Document Phase 8** - Update project documentation
