# Fortunate Primes Calculator

A high-performance Rust tool to calculate Fortunate numbers beyond n=3000, with **benchmarking** and **performance analysis** capabilities.

**Contributing?** → Read [CONTRIBUTING.md](CONTRIBUTING.md) first (5-minute guide to the workflow)
**Detailed workflow?** → See [DEVELOPMENT.md](DEVELOPMENT.md) (architecture, strategy, examples)
**Want to contribute code?** → Check [SYSTEM_PROMPT.md](SYSTEM_PROMPT.md) (for AI agents)

## What are Fortunate Numbers?

A **Fortunate number** is the smallest integer m > 1 such that p_n# + m is prime, where p_n# is the primorial (product of the first n primes).

A **Fortunate prime** is a Fortunate number that is also itself prime.

As of 2017, all known Fortunate numbers up to n=3000 have been prime (Fortune's conjecture).

## Features

✓ **10,000 primes** hardcoded (supports n up to 1224)
✓ **Wheel Factorization** — 40-50% speedup via candidate pre-filtering (Phase 1.2)
✓ **Parallel Candidate Testing** — 2-3x speedup using Rayon multi-core execution (Phase 2)
✓ **Timing instrumentation** — measure primorial calculation & primality testing
✓ **Multiple algorithms** — Compare Fast (20 rounds), Standard (40 rounds), Thorough (64 rounds)
✓ **Interactive CLI** — Find single values or run benchmarks comparing optimizations
✓ **Type-safe architecture** — Traits, error handling, zero runtime type errors
✓ **TDD-First development** — 39 unit tests, integration tests, OEIS A005235 validation

## Testing & Quality

This project follows **Test-Driven Development (TDD)** to ensure correctness and safe optimization:

```bash
# Run all tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

**Test coverage**: Primality tests validated against [OEIS A005235](https://oeis.org/A005235); Fortunate calculations verified against known values (n=5→23, n=100→641, etc.).

## Building

```bash
cargo build --release
```

## Running

```bash
./target/release/fortunate
```

**Interactive menu:**

1. Find Fortunate number (with metrics)
2. Benchmark different algorithms
3. Exit

## Performance Benchmarks

### Before Phase 1 Optimizations

| n   | Fortunate # | Time   | Candidates Tested |
| --- | ----------- | ------ | ----------------- |
| 100 | 641         | ~43ms  | 640               |
| 200 | 1619        | ~994ms | 1,618             |
| 300 | 5641        | ~13.3s | 5,640             |
| 400 | 5051        | ~30.3s | 5,050             |

### After Phase 1 Optimizations (v0.2.0)

| n   | Standard Time | Wheel Factorization | Speedup   | Test Reduction     |
| --- | ------------- | ------------------- | --------- | ------------------ |
| 100 | 43.2ms        | 30.6ms              | **1.41x** | 640 → 236 (-63%)   |
| 200 | 993.8ms       | 656.8ms             | **1.51x** | 1,618 → 593 (-63%) |

**Phase 1 Results:**

- ✅ **Wheel Factorization**: Pre-filters candidates divisible by 2, 3, 5 (reduces search space by ~63%)
- ✅ **Parallel Infrastructure**: `ParallelFortunateCalculator` ready for future Rayon optimizations (Phase 1.3+)
- ✅ **35 Tests**: All passing with 100% OEIS A005235 validation through n=31
- ✅ **Consistent Speedup**: 40-50% improvement across test range

### After Phase 2 Optimizations (v0.3.0)

| n   | Wheel Factorization | Parallel Rayon | Speedup   | vs Original  |
| --- | ------------------- | -------------- | --------- | ------------ |
| 100 | 35.9ms              | 19.1ms         | **1.87x** | **2.26x**    |
| 200 | 675ms               | 235ms          | **2.87x** | **4.23x**    |

**Phase 2 Results:**

- ✅ **Rayon Parallelization**: Batch-based parallel candidate testing leverages multi-core CPUs
- ✅ **2-3x Speedup**: Achieved target performance improvement over Phase 1 wheel factorization
- ✅ **Correctness Maintained**: Still finds SMALLEST Fortunate number (OEIS A005235 validated)
- ✅ **39 Tests**: All passing including 4 new parallel performance/correctness tests
- ✅ **Thread-Safe**: Atomic counters for metrics tracking across parallel threads

**Observations:**

- Wheel factorization consistently reduces candidates by ~63% (matching theoretical 2×3×5 elimination rate)
- Actual speedup is ~40-50% (limited by non-candidate work: primorial calculation, setup)
- Primorial calculation time unchanged (accounts for ~30-40% of total time)
- Miller-Rabin rounds (20/40/64) have negligible impact on total time (candidate finding dominates)

- Algorithm choice (20/40/64 rounds) has minimal impact on total time
  - Finding the candidate dominates computation time
  - Primality test rounds matter less than primorial magnitude

## Architecture

### Traits (Type-Safe Interfaces)

```rust
trait PrimalityTest {
  fn is_prime(&self, n: &Integer) -> bool;
  fn name(&self) -> &'static str;
}

trait FortunateCalculator {
  fn primorial(&self, n: usize) -> Result<Integer>;
  fn fortunate_number(&self, n: usize) -> Result<u32>;
  fn fortunate_number_with_metrics(&self, n: usize) -> Result<(u32, Metrics)>;
}
```

### Error Handling

Custom `FortunateError` enum with explicit error types:

- `InvalidPrimeIndex` — n exceeds prime list
- `NoFortunateFound` — no Fortunate within search range
- `InvalidPrimorial` — calculation error

### Performance Metrics

Track and report:

- Primorial calculation time
- Primality test count
- Tests passed
- Total execution time

## Dependencies

- **rug** — GMP bindings for arbitrary-precision arithmetic
- **rayon** — Ready for parallel implementations (future)
- **serde** — Serialization support (future)

## Optimization Exploration

The benchmarks reveal **exponential time growth**. Here are evidence-based optimization strategies:

### 1. **Parallel Candidate Testing** (HIGH PRIORITY - Expected: 4-8x speedup)

**Current bottleneck:** Sequential primality testing of 5,000+ candidates

**Strategy:**

```rust
use rayon::prelude::*;

// Test candidates in parallel batches
let result = (2..=max_candidate)
    .into_par_iter()
    .find_any(|m| is_prime(&(p_n_sharp.clone() + m)));
```

**Expected impact:** n=400 from 30s → 4-8s (CPU-bound, scales with cores)
**Implementation effort:** Low (Rayon handles threading)

## Optimization Roadmap

### Phase 1: Completed ✅ (v0.2.0)

#### 1.1 Parallel Infrastructure (TDD Foundation)

- ✅ Introduced `ParallelFortunateCalculator` struct
- ✅ Full trait compatibility with sequential implementation
- ✅ Ready for Rayon parallelization in Phase 1.3+

#### 1.2 Wheel Factorization (COMPLETED - 40-50% speedup)

- ✅ Implemented 2-3-5 wheel with period 30
- ✅ Reduces candidate search space by ~63%
- ✅ Verified correct results: OEIS A005235 validation through n=31
- ✅ Actual speedup: n=100 (1.41x), n=200 (1.51x)
- ✅ 7 comprehensive tests all passing

**Results:** Wheel factorization consistently reduces candidates by 63% and cuts execution time by 40-50%.

### Phase 2: Planned

#### 2.1 **Parallel Candidate Testing with Rayon** (Expected: 2-4x speedup)

**Current bottleneck:** Primality testing dominates after wheel filtering

**Strategy:**

```rust
use rayon::prelude::*;

let result = wheel.candidates_up_to(max)
    .into_par_iter()
    .find_any(|m| is_prime(&(p_n_sharp.clone() + m)));
```

**Expected impact:** Linear scaling with CPU cores (2-8 cores → 2-4x speedup)
**Implementation effort:** Medium (Integer cloning overhead manageable)
**Validation:** Test equivalence guaranteed by existing 35 tests

#### 2.2 **Segmented Sieve for Candidates** (Expected: 1.5x speedup)

**Strategy:**
Pre-sieve candidates [2, max] to identify probable primes first, then test in order of likelihood:

```rust
fn sieve_candidates(max: u32) -> Vec<bool> {
    let mut is_prime = vec![true; max as usize + 1];
    for i in 2..=(max as f64).sqrt() as u32 {
        if is_prime[i as usize] {
            for j in ((i * i)..=max).step_by(i as usize) {
                is_prime[j as usize] = false;
            }
        }
    }
    is_prime
}
```

### Phase 3: Future Exploration

- **GPU Acceleration**: CUDA for Miller-Rabin on large n
- **Batch Processing**: Optimize for multiple n values simultaneously
- **Extended Prime List**: Generate primorials beyond n=1224 via segmented sieve

### Phase 2.2 Details: Segmented Sieve

**Expected impact:** Test sieved candidates first, massive speedup on partial matches
**Implementation effort:** Medium
**Risk:** Low (sieve is proven algorithm)

### 4. **Batch Miller-Rabin with Shared Modular Arithmetic** (LOW PRIORITY - Expected: 1.2x speedup)

**Current bottleneck:** Each candidate re-computes $2^r \mod n$ independently

**Strategy:**

```rust
// For batch of candidates, reuse exponentiation setup
fn batch_is_prime(candidates: &[Integer], rounds: usize) -> Vec<bool> {
    // ... compute common witness bases once, apply to all
}
```

**Expected impact:** Cache effects, ~20% faster Miller-Rabin loop
**Implementation effort:** High (complex modular arithmetic)
**Risk:** Moderate (easy to introduce bugs in witness scheduling)

### 5. **Lucas Primality Test + Miller-Rabin Hybrid** (MEDIUM PRIORITY - Expected: 1.3x speedup)

**Current bottleneck:** Miller-Rabin alone requires many rounds for certainty

**Strategy:**
Combine deterministic Lucas test with fast Miller-Rabin:

```rust
fn is_prime_hybrid(n: &Integer) -> bool {
    // Fast reject with trial division
    if trial_division(n, 30) { return false; }

    // Miller-Rabin (20 rounds)
    if !miller_rabin(n, 20) { return false; }

    // Lucas test (deterministic for small witnesses)
    lucas_test(n)
}
```

**Expected impact:** Fewer false positives, potentially fewer rounds needed
**Implementation effort:** High (Lucas test is complex)
**Risk:** Moderate (need proven witness parameters)

### 6. **GPU-Accelerated Primality Testing** (ADVANCED - Expected: 10-100x speedup)

**Current bottleneck:** CPU-only Miller-Rabin loop can run on GPU

**Strategy:**
Use `rust-gpu` or CUDA bindings to test multiple candidates simultaneously:

```text
GPU: Test 1000 candidates in parallel
vs CPU: Test 1 candidate at a time
```

**Expected impact:** n=400 from 30s → 1-3s (if GPU available)
**Implementation effort:** Very High (GPU programming, memory management)
**Risk:** High (GPU availability, portability issues)

---

## Recommended Optimization Path

### **Phase 1: Quick Wins (2-3 hours)**

1. Implement parallel candidate testing with Rayon → **4-8x speedup**
2. Add wheel factorization pre-filtering → **2-3x speedup**
3. **Combined: n=400 from 30s → 1-2s**

### **Phase 2: Medium Effort (4-6 hours)**

1. Add segmented sieve for candidate range
2. Implement Lucas test as fallback
3. **Combined: Additional 1.5-2x speedup** → **n=400 ≈ 0.5s**

### **Phase 3: Advanced (1-2 days)**

1. Benchmark GPU options
2. Consider C/C++ FFI for critical loops
3. **Potential: 10-100x total speedup**

---

## Benchmark Targets

After each optimization, measure:

```bash
# Single run
echo "2\n400\n" | ./target/release/fortunate

# Full suite
./benchmark.sh
```

Track speedup ratio:

- **Phase 1 goal:** 4-8x faster
- **Phase 2 goal:** 8-16x faster total
- **Phase 3 goal:** 80-160x faster with GPU

## Contributing

Want to contribute? **Read this first:**

### For Contributors (Start Here)

👉 **[CONTRIBUTING.md](CONTRIBUTING.md)** — Complete workflow guide (5-10 minutes)

- TDD workflow
- Quality gates
- PR checklist
- Common patterns

### For Detailed Architecture

👉 **[DEVELOPMENT.md](DEVELOPMENT.md)** — In-depth guide (30 minutes)

- Project architecture
- Feature request strategy
- Testing patterns
- Design discussion template

### For AI Agents

👉 **[SYSTEM_PROMPT.md](SYSTEM_PROMPT.md)** — System instructions for development

- Guiding principles
- Workflow validation
- Red flags and scenarios

### Quick Summary

This project follows **Test-Driven Development (TDD)** with **OEIS validation** and **benchmark proof**:

1. **Create issue** with clear description
2. **Discuss design** (critical for algorithms)
3. **Create feature branch** (`feature/N-description`)
4. **Write test first** (TDD)
5. **Implement** and make test pass
6. **Quality gates**: `cargo fmt`, `cargo clippy`, `cargo test`
7. **Benchmark**: `./benchmark.sh` (no regressions)
8. **Merge & tag** (bump version, update CHANGELOG)

### Key Rules

- ✅ **OEIS validation** — Fortune's conjecture must hold
- ✅ **Benchmark proof** — Optimizations show >5% improvement
- ✅ **No regressions** — All tests pass, benchmarks don't slow down
- ❌ **No shortcuts** — TDD is mandatory, not optional

### Questions?

See [CONTRIBUTING.md](CONTRIBUTING.md) for workflow or [DEVELOPMENT.md](DEVELOPMENT.md) for deep dives.
