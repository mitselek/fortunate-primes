# Python + gmpy2 Implementation

**Status**: ✅ **Production (Winner!)** 🏆

## Overview

Python implementation using gmpy2 (GMP bindings) for big integer arithmetic and primality testing. **Fastest implementation tested** - 9-22x faster than Rust, 2-5x faster than PARI/GP.

## Why Python Wins

1. **gmpy2 efficiency**: GMP backend (same as PARI/GP) with minimal Python overhead
2. **Multiprocessing advantage**: Separate processes avoid GIL entirely, no thread contention
3. **Optimal batch sizing**: Smaller batches (50) better suited to Python's IPC model
4. **Process isolation**: Handles concurrent system load better than threading

## Setup

```bash
# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt

# Or manually
pip install gmpy2  # Requires libgmp-dev system package
```

### System Requirements

**Debian/Ubuntu:**

```bash
sudo apt-get install python3 python3-venv libgmp-dev
```

**macOS:**

```bash
brew install python gmp
```

**Python Version**: 3.9+ (recommended 3.11+ for performance)

## Usage

```bash
# Activate environment
source venv/bin/activate

# Run calculator
python fortunate.py 500     # Calculate F(500)
python fortunate.py 1000    # Calculate F(1000)

# Run tests
pytest test_fortunate.py -v

# Deactivate when done
deactivate
```

## Benchmarks (Clean System)

| n    | F(n)  | Time        | vs Rust   | vs PARI/GP | OEIS ✓ |
| ---- | ----- | ----------- | --------- | ---------- | ------ |
| 500  | 5167  | **1.25s**   | **9.0x**  | 5.4x       | ✅     |
| 1000 | 8719  | **3.86s**   | **22.2x** | 17.8x      | ✅     |
| 1500 | 14281 | **22.27s**  | -         | -          | ✅     |
| 2000 | 51137 | **12m 5s**  | -         | -          | ✅     |
| 2500 | 25643 | **2m 52s**  | **9.6x**  | -          | ✅     |
| 3000 | 27583 | **45.2s**   | **65x**   | -          | ✅     |
| 4601 | 56611 | **36m 46s** | -         | -          | ✅     |

See [BENCHMARKS.md](BENCHMARKS.md) for detailed analysis.

**System**: 16 cores, clean system (load ~1-2), Python 3.12.3, gmpy2 2.1.5  
**Latest**: F(4601) computed with 15 workers, adaptive batch sizing (floor=16), Firoozbakht optimization

## Benefits

- ✅ **Fastest implementation tested** (9-22x faster than Rust)
- ✅ Excellent code clarity (~130 lines vs Rust's 200+)
- ✅ Rich ecosystem (pytest, type hints, OEIS validation)
- ✅ Easy to install (`pip install gmpy2`)
- ✅ Process isolation handles concurrent load well

## Trade-offs

- ⚠️ Requires system with adequate RAM for multiple processes
- ⚠️ Performance degrades ~2.5x under heavy system load
- ⚠️ Requires Python runtime (vs Rust static binary)

## Implementation Details

**Architecture**: Parallel batch search using `multiprocessing.Pool`

```text
                    ┌─ Worker 1 (m=3-52) ──────┐
Main ─► primorial ─►├─ Worker 2 (m=53-102) ───├──► First prime found
                    ├─ Worker 3 (m=103-152) ──┤
                    └─ Worker N (m=...) ──────┘
```

**Key components**:

- `compute_primorial()`: Calculate primorial(n) using gmpy2.mpz
- `test_batch()`: Test range [start, end) for primality
- `find_fortunate()`: Orchestrate parallel search with early termination

## Dependencies

```txt
# requirements.txt
gmpy2>=2.1.5
```

Development tools:

```txt
pytest>=7.0.0    # Testing (19 tests)
```

## References

- Detailed benchmarks: [BENCHMARKS.md](BENCHMARKS.md)
- Main project: [../../README.md](../../README.md)
- gmpy2 documentation: <https://gmpy2.readthedocs.io/>
- GMP library: <https://gmplib.org/>
- OEIS A005235: <https://oeis.org/A005235>
