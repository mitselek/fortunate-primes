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
✓ **Live progress reporting** — Real-time updates with auto-scaling time units  
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

**Final Result:**

```text
F(123) = 2087
time: 1.23s
per_iteration: 0.61ms
```

**Live Progress (shown during calculation):**
While running, progress updates appear on stderr with human-readable time formatting:

```text
F(2000) > 5000 | time: 5.15s | per_iteration: 1.03ms
```

Progress line updates every ~1 second and overwrites itself (no scrolling). Final result replaces progress line when complete.

## Performance

### Sequential PARI/GP Benchmarks (v0.5.1)

Baseline sequential performance on Intel 16-core system with PARI/GP 2.15.4:

| n   | F(n) | Time    | Per Iteration |
| --- | ---- | ------- | ------------- |
| 100 | 641  | 39.5ms  | 0.062ms       |
| 200 | 1619 | 427.3ms | 0.264ms       |
| 300 | 5641 | 4.7s    | 0.826ms       |
| 400 | 5051 | 10.4s   | 2.061ms       |
| 500 | 5167 | 21.0s   | 4.071ms       |

### Parallel PARI/GP Performance (v0.5.2+)

Multi-core sequential benchmarks with interleaved candidate search (16 workers):

| n    | F(n)  | Time    | Time/F(n) | Per Iteration |
| ---- | ----- | ------- | --------- | ------------- |
| 600  | 16187 | 16.7s   | 1.032ms   | 16.482ms      |
| 700  | 13259 | 21.3s   | 1.606ms   | 25.719ms      |
| 800  | 6473  | 18.0s   | 2.779ms   | 44.363ms      |
| 900  | 7547  | 48.1s   | 6.371ms   | 101.808ms     |
| 1000 | 8719  | 75.9s   | 8.703ms   | 139.268ms     |
| 2000 | 51137 | 1775.1s | 34.704ms  | 555.247ms     |

**Metric Explanation:**

- **Time**: Total wall-clock execution time to calculate F(n) with 16 parallel workers
- **Time/F(n)**: Total time divided by the Fortunate number value (time per unit of result magnitude). Less mathematically meaningful but shows execution scaling relative to result size. Note: increases from 1.0ms to 34.7ms, indicating F(n) grows slower than computation cost.
- **Per Iteration**: Total time divided by number of candidates tested (≈ F(n) - 1). This is the actual computational cost of each primality test. Shows the true algorithm bottleneck: primality testing becomes exponentially more expensive as n increases (16.5ms → 555.2ms), because PARI/GP must verify increasingly large numbers for primality.

**Usage:**

```bash
# Sequential (default)
./target/release/fortunate 500

# Parallel with auto-detected workers
./target/release/fortunate --parallel 500

# Parallel with specific worker count
./target/release/fortunate --parallel --workers 8 500
```

**Algorithm: Interleaved Candidate Search**  

Each worker thread spawns a PARI/GP process and tests candidates at stride N:

- Worker 0: primorial(n) + 1, +N+1, +2N+1, ...
- Worker 1: primorial(n) + 2, +N+2, +2N+2, ...
- Worker k: primorial(n) + k + 1, +N+k+1, +2N+k+1, ...

All workers are guaranteed to find the same F(n) - whichever finds the prime first returns it.

**Performance Notes:**

- Speedup peaks at 8 workers (3.5x) then diminishes due to process overhead
- Process creation (~10-15ms) and synchronization costs become significant
- Optimal configuration: 4-8 workers for most systems
- Scales linearly up to physical core count, then sublinearly

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
