# Python + gmpy2 Implementation

**Status**: 🚧 Prototype

## Overview

Python implementation using gmpy2 (GMP bindings) for big integer arithmetic and primality testing, eliminating PARI/GP subprocess overhead.

## Motivation

Python with gmpy2 could match PARI/GP performance (both use GMP) while offering:

- More accessible language for contributors
- No subprocess spawning overhead
- Native Python parallelism (multiprocessing/concurrent.futures)
- Rich ecosystem for tooling and visualization

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

## Expected Usage

```bash
# Activate environment
source venv/bin/activate

# Run calculator
python fortunate.py 500

# Deactivate when done
deactivate
```

## Prototype Tasks

- [ ] Implement parallel primorial + search using multiprocessing
- [ ] Use gmpy2.is_prime() for primality testing (Baillie-PSW)
- [ ] Implement progress reporting
- [ ] Benchmark vs Rust baseline (n=500, n=1000)
- [ ] Evaluate code complexity and maintainability
- [ ] Compare memory efficiency

## Expected Benchmarks

| n    | F(n) | Time (estimated) | vs Rust |
| ---- | ---- | ---------------- | ------- |
| 500  | 5167 | TBD              | TBD     |
| 1000 | 8719 | TBD              | TBD     |

## Expected Benefits

- **Performance**: gmpy2 uses GMP (same as PARI/GP), should be equivalent
- **No subprocesses**: Direct function calls instead of process spawning
- **Accessibility**: Python more familiar than Rust for many developers
- **Type safety**: Optional with type hints + mypy
- **Rich tooling**: pytest, black, pylint, mypy

## Expected Trade-offs

- **Startup time**: Python interpreter slower than compiled Rust
- **Memory**: Python overhead (~50 MB) vs Rust (~2 MB)
- **Distribution**: Requires Python runtime vs single static binary
- **GIL concerns**: Mitigated by using multiprocessing (separate processes)

## Dependencies

```txt
# requirements.txt
gmpy2>=2.1.5
```

Optional development tools:

```txt
mypy>=1.0.0      # Type checking
pytest>=7.0.0    # Testing
black>=23.0.0    # Formatting
```

## References

- gmpy2 documentation: <https://gmpy2.readthedocs.io/>
- GMP library: <https://gmplib.org/>
- Python multiprocessing: <https://docs.python.org/3/library/multiprocessing.html>
- Rust baseline: [../rust/](../rust/)

## Status

Awaiting prototype implementation. Directory structure ready.
