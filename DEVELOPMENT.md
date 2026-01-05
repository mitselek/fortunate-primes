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

## License

Part of the mitselek project repository.
