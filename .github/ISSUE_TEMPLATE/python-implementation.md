# Python (gmpy2) Implementation Prototype

## Overview

Implement pure Python parallelized Fortunate number calculator using **gmpy2** (GMP bindings) to evaluate whether Python can match or approach PARI/GP performance while offering better accessibility.

## Motivation

After successful PARI/GP implementation (Issue #11) showing **1.25-1.67x speedup** over Rust orchestration, Python+gmpy2 is worth exploring because:

### Potential Advantages

- ✅ **Same underlying library**: gmpy2 wraps GMP (same as PARI/GP), should match primality test performance
- ✅ **Accessibility**: Python more familiar than GP scripting to most developers
- ✅ **No subprocess overhead**: Direct GMP calls vs PARI/GP subprocess spawning (Rust approach)
- ✅ **Rich ecosystem**: Better tooling, debugging, visualization than GP
- ✅ **Native parallelism**: `multiprocessing.Pool` or `concurrent.futures.ProcessPoolExecutor`

### Key Question

Can Python+gmpy2 match PARI/GP's **6.8s for F(500)** and **68.9s for F(1000)**?

## Expected Architecture

### Core Components

1. **Primorial computation**: `gmpy2.primorial(n)` - efficient factorial of primes
2. **Primality testing**: `gmpy2.is_prime()` or `gmpy2.is_strong_prp()`
3. **Parallel workers**: `multiprocessing.Pool.map()` for batch distribution
4. **Batch strategy**: Reuse optimal batch sizes from PARI/GP (100 for n~500, 150 for n~1000)

### Implementation Pattern

```python
import gmpy2
from multiprocessing import Pool, cpu_count

def test_batch(args):
    """Worker function: test batch [start, start+batch_size)"""
    n, start, batch_size = args
    pn = gmpy2.primorial(n)  # Each worker computes primorial

    for m in range(start, start + batch_size):
        if gmpy2.is_prime(pn + m):
            return m
    return None

def fortunate_batch(n, batch_size=100):
    """Coordinator: distribute work to pool"""
    num_workers = cpu_count()

    with Pool(num_workers) as pool:
        round_num = 0
        while True:
            round_num += 1
            # Generate batch arguments
            args = [(n, i * batch_size, batch_size)
                    for i in range(num_workers)]

            # Execute in parallel
            results = pool.map(test_batch, args)

            # Check for first prime found
            for r in results:
                if r is not None:
                    return r
```

## Prototype Tasks

- [ ] Implement `fortunate_batch(n, batch_size)` with `multiprocessing.Pool`
- [ ] Unit tests: F(5)=23, F(10)=61, F(20)=103
- [ ] Benchmark F(500) with batch_size=[50, 100, 150]
- [ ] Benchmark F(1000) with batch_size=[100, 150, 200]
- [ ] Compare vs PARI/GP and Rust baselines
- [ ] Measure memory usage (Python process + workers)
- [ ] Document findings in `implementations/python-gmpy2/BENCHMARKS.md`

## Expected Benchmarks

**Target System**: AMD Ryzen 7 2700 (16 threads)

| n    | F(n) | PARI/GP (baseline) | Python+gmpy2 (target) | Notes                           |
| ---- | ---- | ------------------ | --------------------- | ------------------------------- |
| 500  | 5167 | 6.8s               | TBD                   | Optimal batch_size=100          |
| 1000 | 8719 | 68.9s              | TBD                   | Optimal batch_size=150          |
| 4602 | TBD  | TBD                | TBD                   | After Rust baseline established |

## Dependencies

- **Python**: 3.9+ (3.11+ recommended for performance)
- **gmpy2**: 2.1.0+ (GMP bindings)
- **System**: libgmp-dev (Debian/Ubuntu) or gmp (macOS via brew)

```bash
# Setup
python3 -m venv venv
source venv/bin/activate
pip install gmpy2

# Verify
python3 -c "import gmpy2; print(gmpy2.version())"
```

## Open Questions

1. **gmpy2.is_prime() vs is_strong_prp()**:

   - Which is faster for large primorials?
   - Does PARI/GP's `ispseudoprime()` have optimizations gmpy2 lacks?

2. **Python GIL impact**:

   - Does `multiprocessing` eliminate GIL concerns?
   - Process spawning overhead vs PARI/GP threads?

3. **Primorial computation overhead**:

   - Is `gmpy2.primorial()` as optimized as PARI/GP's `prod(i=1, n, prime(i))`?
   - Cache primorial vs recompute per worker?

4. **Memory efficiency**:
   - Python process overhead vs PARI/GP's ~13 MB per worker
   - Pickling large integers for IPC?

## Success Criteria

### Minimum Viable (Worth keeping)

- ✅ F(500) < 15s (better than Rust 11.3s)
- ✅ F(1000) < 120s (better than Rust 85.8s)
- ✅ Cleaner code than Rust (~100-150 lines Python)

### Stretch Goal (Matches PARI/GP)

- 🎯 F(500) ~7-10s (within 50% of PARI/GP 6.8s)
- 🎯 F(1000) ~70-100s (within 50% of PARI/GP 68.9s)
- 🎯 Simpler than PARI/GP for Python developers

### Documentation

- 📝 Detailed performance analysis
- 📝 Comparison with PARI/GP and Rust
- 📝 Lessons learned (what works, what doesn't)

## Related Issues

- Issue #11: Pure PARI/GP implementation ✅ (completed, 1.25-1.67x faster than Rust)
- Issue #12: Repository restructure ✅ (completed)
- Issue #13: Node.js/TypeScript implementation 🚧 (pending)

## References

- gmpy2 documentation: <https://gmpy2.readthedocs.io/>
- PARI/GP implementation: `implementations/pari-gp/`
- Rust baseline: `implementations/rust/`
- Performance targets: `implementations/pari-gp/BENCHMARKS.md`
