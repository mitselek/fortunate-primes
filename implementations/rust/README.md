# Rust Implementation (Baseline)

**Status**: ✅ Production  
**Performance**: 100% (reference baseline)

## Overview

High-performance parallel Fortunate number calculator using Rust for orchestration and PARI/GP for primality testing.

## Architecture

- **Coordinator**: Rust (2 MB)
- **Workers**: 15 PARI/GP subprocesses (~13 MB each)
- **Parallelism**: Work queue pattern with adaptive batching
- **Optimization**: Worker-count-aware adaptive threshold (60s/num_workers)

## Setup

```bash
# Install dependencies
sudo apt-get install pari-gp  # Debian/Ubuntu
# or
brew install pari             # macOS

# Build release binary
cargo build --release
```

## Usage

```bash
# Calculate F(n)
./target/release/fortunate-primes 500

# Output:
# F(500) : [931; ?] [867+64] (2.13s)
# ...
# F(500) = 5167 (11.31s)
```

## Benchmarks

| n    | F(n)  | Time    | Details                   |
| ---- | ----- | ------- | ------------------------- |
| 5    | 23    | <1ms    | Unit test                 |
| 10   | 61    | <1ms    | Unit test                 |
| 20   | 103   | <1ms    | Unit test                 |
| 500  | 5167  | 11.31s  | 15 workers                |
| 1000 | 8719  | 85.8s   | 15 workers                |
| 2500 | 25643 | 27.35m  | Verified against OEIS     |
| 3000 | 27583 | 48.97m  | Verified against OEIS     |
| 4601 | 56611 | 4.96h   | First beyond OEIS dataset |
| 4602 | 62207 | 5h 40m  | Clean system, 15 workers  |

**Hardware**: AMD Ryzen 7 2700 (8 cores, 16 logical CPUs)

## Testing

```bash
cargo test              # All tests pass
cargo fmt -- --check    # Code formatted
cargo clippy            # No warnings
```

## Key Features

- **Adaptive Batching**: Initial batch size 1, doubles when batch completes <60s/num_workers
- **Early Termination**: Cooperative cancellation stops wasted work
- **Progress Tracking**: Interval notation `[lower; upper]` with batch ranges
- **Lower Bound Tracking**: BTreeMap for contiguous ranges
- **Memory Efficient**: Primorials stay in PARI/GP memory (no serialization)

## Performance Optimizations

1. **Worker-count-aware threshold** (fcfc8b4): 2x speedup on F(500)
2. **Early termination** (previous): Stops dispatch and worker processing
3. **Cooperative cancellation**: Workers check flags, skip irrelevant batches
4. **Batch timing**: Per-batch measurement for adaptive sizing

## Files

- `src/main.rs` - CLI interface
- `src/lib.rs` - Core library with tests
- `src/search.rs` - Parallel batch coordinator
- `src/pari.rs` - PARI/GP subprocess interface
- `src/progress.rs` - Terminal progress reporting
- `Cargo.toml` - Dependencies and metadata

## Documentation

See root [README.md](../../README.md) for project overview and [DEVELOPMENT.md](../../archived/DEVELOPMENT.md) for architecture details.

## References

- PARI/GP: <https://pari.math.u-bordeaux.fr/>
- Baillie-PSW primality test: Used by `ispseudoprime()`
- Fortune's conjecture: All computed F(n) are prime
