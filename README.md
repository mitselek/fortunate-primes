# Fortunate Primes Calculator

A high-performance calculator using **PARI/GP** to compute Fortunate numbers efficiently.

**Contributing?** → Read [CONTRIBUTING.md](CONTRIBUTING.md) (5-minute workflow guide)  
**Architecture details?** → See [DEVELOPMENT.md](DEVELOPMENT.md) (design and structure)  
**System instructions?** → Check [SYSTEM_PROMPT.md](SYSTEM_PROMPT.md) (for AI agents)

## What are Fortunate Numbers?

A **Fortunate number** F(n) is the smallest integer m > 1 such that p_n# + m is prime, where p_n# is the primorial (product of the first n primes).

A **Fortunate prime** is a Fortunate number that is also itself prime.

**Fortune's conjecture**: All known Fortunate numbers up to n=3000 are prime.

## Features

✓ **PARI/GP backend** — 3-170x faster than pure Rust (depending on n)  
✓ **Simple CLI** — Single command: `./fortunate <n>` (non-interactive, scriptable)  
✓ **10,000 primes hardcoded** — Supports n up to 10,000  
✓ **Type-safe Rust library** — Traits, error handling, tests  
✓ **TDD-first development** — 39 unit tests, OEIS validation

## Dependencies

**Required**: PARI/GP (for calculations)

```bash
# Ubuntu/Debian
sudo apt install pari-gp

# macOS
brew install pari
```

## Building

```bash
cargo build --release
```

## Usage

Non-interactive CLI that requires `n` as argument:

```bash
./target/release/fortunate <n>
```

### Examples

```bash
./target/release/fortunate 100    # F(100) = 641
./target/release/fortunate 500    # Larger calculations
./target/release/fortunate 1000   # Supported up to n=10000
```

### Error Handling

```bash
./target/release/fortunate          # Missing argument
./target/release/fortunate abc      # Invalid input
./target/release/fortunate 0        # Out of range
```

### Output Format

```text
F(123) = 2087
iterations: 2086
```

## Performance

### PARI/GP Benchmarks (Measured v0.5.1+)

Actual measured performance on an Intel system with PARI/GP 2.15.4:

| n   | F(n)  | Total Time  | Per Iteration |
| --- | ----- | ----------- | ------------- |
| 100 | 641   | **39.5ms**  | 0.062ms       |
| 200 | 1619  | **427.3ms** | 0.264ms       |
| 300 | 5641  | **4.7s**    | 0.826ms       |
| 400 | 5051  | **10.4s**   | 2.061ms       |
| 500 | 5167  | **21.0s**   | 4.071ms       |
| 600 | 16187 | **107.5s**  | 6.642ms       |

**Understanding the metrics:**

- **F(n)**: The Fortunate number (always equals iterations + 1)
- **Total Time**: Wall-clock time to find F(n), measured by the program
- **Per Iteration**: Time spent per primality test (F(n) - 1 tests required)

The per-iteration cost increases with n because:

- Larger primorials have more digits
- Baillie-PSW primality test is O(log³ n) in the number's bit length
- FFT multiplication cost dominates for large numbers

**Performance characteristics:**

- Primality testing is the computational bottleneck
- Linear scaling with iteration count for small n, superlinear for larger n
- Baillie-PSW algorithm (Miller-Rabin + Lucas) deterministic for all tested ranges
- Optimized C implementation with 30+ years of mathematical library development

## Architecture

### Current Design (v0.5.0+)

```text
src/
├── main.rs           # CLI interface, argument parsing
├── hybrid.rs         # PARI/GP subprocess wrapper
└── lib.rs            # Rust implementations (reference/testing)
    ├── primality.rs  # Miller-Rabin tests
    ├── sieve.rs      # Candidate generation
    └── primes.rs     # Pre-computed prime list
```

**Data flow:**

```text
Input n → [main.rs] Parse & validate
         → [hybrid.rs] Call PARI/GP subprocess
         → [PARI/GP] Calculate F(n)
         → [hybrid.rs] Parse output
         → Output F(n) and iteration count
```

### Why This Design

1. **PARI/GP is battle-tested** — Decades of mathematical research implementation
2. **Simple and maintainable** — Single responsibility: calculate and format output
3. **Rust library still available** — For testing, benchmarking, educational use
4. **Extensible** — Easy to add CLI features, output formats, etc.

## Testing

```bash
# Run all tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

**Test coverage:**

- Primality tests validated against OEIS A005235
- Fortunate calculations verified (n=5→23, n=100→641, etc.)
- 39 unit tests covering edge cases

## Contributing

### Quick Start

1. **Read** [CONTRIBUTING.md](CONTRIBUTING.md) (5 minutes)
2. **Create feature branch**: `feature/N-description`
3. **Write test first** (TDD)
4. **Implement & make test pass**
5. **Quality gates**: `cargo fmt && cargo clippy && cargo test`
6. **Create PR** with benchmark results

### Key Requirements

- ✅ All tests pass
- ✅ OEIS A005235 validation (Fortune's conjecture holds)
- ✅ Benchmark shows improvement (>5% speedup minimum)
- ✅ No regressions in existing performance

### For More Details

- **Workflow**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **Architecture**: [DEVELOPMENT.md](DEVELOPMENT.md)
- **System prompt** (for AI agents): [SYSTEM_PROMPT.md](SYSTEM_PROMPT.md)

## Project Status

- ✅ PARI/GP integration complete
- ✅ CLI simplified to non-interactive mode
- ✅ Dead code cleaned up
- 📋 Next: Parallel candidate testing (future optimization)

## License

See LICENSE file
