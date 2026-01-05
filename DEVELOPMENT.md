# Development Guide

## Project Setup

This is a high-performance Rust calculator for Fortunate numbers with benchmarking capabilities.

## Prerequisites

- **Rust 1.70+** — Install via [rustup](https://rustup.rs/)
- **GMP** — Automatically downloaded via `rug` crate
- **Linux/macOS/Windows** — Cross-platform compatible

## Getting Started

```bash
# Navigate to project
cd projects/fortunate-primes

# Build release binary
cargo build --release

# Run tests
cargo test

# Run the calculator
./target/release/fortunate
```

## Project Structure

```text
fortunate-primes/
├── src/
│   ├── lib.rs              # Core library with traits and implementations
│   ├── main.rs             # CLI interface
│   └── primes.rs           # 10,000 hardcoded primes
├── Cargo.toml              # Rust manifest
├── Cargo.lock              # Dependency lock (generated)
├── README.md               # User-facing documentation
├── DEVELOPMENT.md          # This file
├── .gitignore              # Git exclusions
├── .editorconfig           # Editor configuration
├── Makefile                # Development shortcuts
└── benchmark.sh            # Performance benchmarking script
```

## Development Workflow

### Running Tests

```bash
# All tests
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test test_fortunate_numbers
```

### Benchmarking

```bash
# Interactive (menu-driven)
./target/release/fortunate

# Full benchmark suite
./benchmark.sh

# Release optimizations
cargo build --release  # LTO + codegen-units=1
```

### Code Quality

```bash
# Format check
cargo fmt -- --check

# Auto-format
cargo fmt

# Lint
cargo clippy -- -D warnings

# Check without building
cargo check
```

## Architecture

### Core Traits

**`PrimalityTest`** — Abstraction for primality testing algorithms

```rust
trait PrimalityTest {
  fn is_prime(&self, n: &Integer) -> bool;
  fn name(&self) -> &'static str;
}
```

**`FortunateCalculator`** — Contract for calculating Fortunate numbers

```rust
trait FortunateCalculator {
  fn primorial(&self, n: usize) -> Result<Integer>;
  fn fortunate_number(&self, n: usize) -> Result<u32>;
  fn fortunate_number_with_metrics(&self, n: usize) -> Result<(u32, Metrics)>;
}
```

### Error Handling

Custom error enum ensures exhaustive handling:

```rust
pub enum FortunateError {
  InvalidPrimeIndex { index: usize, max: usize },
  NoFortunateFound { n: usize, max_candidate: u32 },
  InvalidPrimorial { reason: String },
}
```

### Performance Metrics

Structured timing data:

```rust
pub struct Metrics {
  pub primorial_time: Duration,
  pub primality_test_count: usize,
  pub primality_tests_passed: usize,
  pub total_time: Duration,
  pub candidate_found: u32,
}
```

## Quick Commands

Use the Makefile for common tasks:

```bash
make build       # Debug build
make release     # Optimized release build
make test        # Run all tests
make fmt         # Auto-format code
make lint        # Run clippy linter
make bench       # Full benchmark suite
make run         # Build and run calculator
make help        # Show all targets
```

## Adding New Features

### Adding a New Primality Test

1. Create a new struct implementing `PrimalityTest`:

   ```rust
   pub struct LucasPrimalityTest;

   impl PrimalityTest for LucasPrimalityTest {
       fn is_prime(&self, n: &Integer) -> bool {
           // Implementation
       }
       fn name(&self) -> &'static str {
           "Lucas"
       }
   }
   ```

2. Add to the calculator options in `main.rs`
3. Add tests to `lib.rs`
4. Update `README.md` with benchmark results

### Adding More Primes

1. Generate primes using external tool (e.g., sieve)
2. Update `src/primes.rs` with new prime list
3. Update `PRIMES_10K` array
4. Update README prime count
5. Test with `cargo test`

## Optimization Work

See **README.md** "Optimization Exploration" section for detailed strategies.

### Profiling

```bash
# Generate flame graph (requires flamegraph tool)
cargo install flamegraph
cargo flamegraph --release -- 2 500

# Linux perf
cargo build --release
perf record -g ./target/release/fortunate
perf report
```

### Benchmarking Against Changes

```bash
# Baseline
./target/release/fortunate  # Note times for n=300, 400

# Make changes
# ...

# Rebuild
cargo build --release

# Compare times
./target/release/fortunate
```

## Common Issues

### Build Errors

**"Cannot find rug"** — Ensure GMP development libraries are installed:

```bash
# macOS
brew install gmp

# Ubuntu/Debian
sudo apt-get install libgmp-dev

# Fedora/RHEL
sudo dnf install gmp-devel
```

**"Integer arithmetic error"** — Check `rug` version in `Cargo.toml` matches docs

### Runtime Issues

**"No Fortunate number found"** — Increase `max_candidate` in code (currently 1,000,000)

**Slow performance** — Ensure release build: `cargo build --release`

## Testing Strategy (TDD-First Approach)

### Test-Driven Development (TDD)

We follow TDD principles: **write tests first, then implementation**. This is especially critical for:

- **Correctness**: Math is unforgiving. Test against [OEIS A005235](https://oeis.org/A005235) before optimization.
- **Optimization Safety**: Before each optimization (parallelization, new algorithm), ensure tests pass.
- **Regression Prevention**: Tests catch performance regressions or accuracy loss.

### Unit Tests

All primality tests and Fortunate calculations have unit tests in `lib.rs`:

```rust
#[test]
fn test_miller_rabin_small_primes() { ... }

#[test]
fn test_fortunate_numbers() { ... }
```

**Validate against known values**:

- OEIS sequence [A005235](https://oeis.org/A005235) (Fortunate numbers)
- Known Fortunate primes: n=5→23, n=100→641, n=200→1619

### Adding Tests for New Features

Before implementing an optimization:

1. Write a test with known input/output
2. Run `cargo test` (should fail)
3. Implement the optimization
4. Run `cargo test` (should pass)
5. Benchmark with `./benchmark.sh` to verify improvement

Example:

```rust
#[test]
fn test_parallel_speedup() {
    // Before parallelization, this establishes baseline
    let calc = PrimeBasedCalculator::new(primes.clone());
    let (result, metrics) = calc.fortunate_number_with_metrics(300).unwrap();

    // Assert correctness first
    assert_eq!(result, 5641);

    // After parallelization, result must stay same but metrics change
    // This ensures we don't trade correctness for speed
}
```

### Integration Testing

CLI tested manually:

```bash
echo -e "1\n5\n2" | ./target/release/fortunate
# Should find Fortunate number for n=5 → 23
```

### Pre-Commit Testing

Run before every commit:

```bash
cargo test && cargo clippy && cargo fmt
```

Or use git hooks to enforce automatically.

## Contributing

We follow **TDD-first** development. When adding code:

1. **Write test first**: Define expected behavior with a test case
2. **Run tests** (they'll fail): `cargo test`
3. **Implement**: Write the minimal code to pass
4. **Format**: `cargo fmt`
5. **Lint**: `cargo clippy`
6. **Test again**: `cargo test` (must pass)
7. **Benchmark**: `./benchmark.sh` (ensure no regression)
8. **Document**: Update README/DEVELOPMENT if needed

**Golden Rule**: No optimization is correct if tests don't pass. No optimization is good if benchmarks regress.

## Feature Request Strategy

### 1. **Issue First, Code Second**

Before starting any work:

```bash
# Create GitHub Issue with:
# - Title: Clear, concise description
# - Body: What? Why? Expected behavior?
# - Labels: enhancement, optimization, bug, etc.
```

**Why?** Catches design problems early. Prevents wasted effort on rejected ideas.

### 2. **Design Discussion** (Critical for Math-Heavy Work)

For algorithmic features, discuss first:

- **Current approach**: "We use Miller-Rabin with N rounds"
- **Proposed change**: "Add Lucas-Lehmer primality test"
- **Expected benefit**: "2x speedup for certain inputs"
- **Risk analysis**: "Changes core primality logic, needs comprehensive tests"
- **Validation plan**: "Test against OEIS A005235, benchmark n=100-300"

Example comment in issue:

```
## Design Proposal

### Current
- Miller-Rabin: O(k * log^3 n) where k = rounds

### Proposed
- Hybrid: Miller-Rabin for small n, Lucas-Lehmer for larger n
- Expected: 1.5-2x speedup on n > 200

### Risk
- Changes proven algorithm
- Mitigation: Comprehensive tests + regression benchmarks

### Acceptance Criteria
- ✓ All existing tests pass
- ✓ OEIS validation through n=100
- ✓ Benchmark shows speedup on target range
- ✓ README updated with new algorithm details
```

### 3. **Feature Branch Workflow**

```bash
# Create issue first (e.g., #12)
git checkout -b feature/12-lucas-lehmer-test
# or: git checkout -b feat/lucas-lehmer

# Work in isolation
cargo test  # After each meaningful change
./benchmark.sh  # Before merging back

# When ready: Create pull request linking issue
# "Closes #12: Add Lucas-Lehmer primality test"
```

### 4. **Acceptance Criteria Template**

Before merging, every feature must satisfy:

**Correctness**:

- ✓ All unit tests pass (existing + new)
- ✓ OEIS A005235 validation through highest n tested
- ✓ No regression on benchmark suite

**Code Quality**:

- ✓ `cargo fmt` passes
- ✓ `cargo clippy` clean (no warnings)
- ✓ Code follows existing patterns

**Documentation**:

- ✓ README updated (features section, benchmarks if changed)
- ✓ DEVELOPMENT.md updated if approach changed
- ✓ Code comments for complex algorithms
- ✓ Changelog entry (see below)

**Performance**:

- ✓ Benchmark numbers documented
- ✓ No regression on slower inputs
- ✓ Improvement validated with `make bench`

### 5. **Versioning & Changelog**

**Semantic Versioning** (MAJOR.MINOR.PATCH):

```
X.Y.Z
│ │ └─ Patch: Bug fixes, internal refactors
│ └──── Minor: New features, non-breaking changes
└────── Major: Breaking changes, algorithm rewrites
```

**Update on each merge**:

1. Increment version in [Cargo.toml](Cargo.toml)
2. Add entry to bottom of README (sample below)
3. Commit: `bump: v1.0.0 → v1.1.0`

**CHANGELOG.md** template (to create):

```markdown
# Changelog

## [1.1.0] - 2026-01-15

### Added

- Lucas-Lehmer primality test (hybrid approach)
- Algorithm selection in CLI menu

### Changed

- Benchmark suite now tests n=100-500

### Fixed

- Integer overflow in primorial calculation for n > 50

---

## [1.0.0] - 2026-01-05

### Added

- Initial release: Miller-Rabin tester + CLI
```

### 6. **Review Checklist** (Before Merge)

Ask yourself:

- [ ] Does this solve the stated problem?
- [ ] Are all tests passing?
- [ ] Did benchmarks improve or stay same (not regress)?
- [ ] Is this the simplest correct solution?
- [ ] Would Fortune's conjecture (n up to 3000) still hold?
- [ ] Can someone understand this 6 months from now?
- [ ] Did I validate against OEIS?

### 7. **Common Feature Categories**

**Type: Optimization**

- Measure baseline: `./benchmark.sh > baseline.txt`
- Implement with tests
- Measure after: `./benchmark.sh > after.txt`
- Compare: `diff baseline.txt after.txt`
- Accept only if improvement ≥ expected

**Type: Algorithm Addition**

- Design discussion (see Lucas-Lehmer example)
- Tests against known values (OEIS)
- Comparison benchmarks (new vs. existing)
- Update README comparison table

**Type: Bug Fix**

- Create failing test first
- Fix implementation
- Test passes
- Check no regression on others

### 8. **Issue Labels**

Use these to organize:

- `enhancement` — New feature
- `optimization` — Speed/memory improvement
- `bug` — Broken behavior
- `docs` — Documentation only
- `blocked` — Waiting on something
- `phase-1`, `phase-2`, `phase-3` — Roadmap stages
- `good-first-issue` — Newcomer-friendly

### 9. **When to Reject Features**

Say "no" (politely) if:

- ❌ Breaks existing tests
- ❌ Causes benchmarks to regress >5%
- ❌ Changes core algorithm without proof it's better
- ❌ Adds complexity with unclear benefit
- ❌ Violates Fortune's conjecture validation

Say "yes" if:

- ✅ Solves real problem
- ✅ Has acceptance criteria
- ✅ Tests prove correctness
- ✅ Benchmarks prove improvement
- ✅ Design is sound

## License

Part of the mitselek project repository.
