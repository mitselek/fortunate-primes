# Fortunate Primes Calculator

A high-performance Rust tool to calculate Fortunate numbers beyond n=3000, with **benchmarking** and **performance analysis** capabilities.

## What are Fortunate Numbers?

A **Fortunate number** is the smallest integer $m > 1$ such that $p_n\# + m$ is prime, where $p_n\#$ is the primorial (product of the first $n$ primes).

A **Fortunate prime** is a Fortunate number that is also itself prime.

As of 2017, all known Fortunate numbers up to n=3000 have been prime (Fortune's conjecture).

## Features

✓ **10,000 primes** hardcoded (supports n up to 1224)  
✓ **Timing instrumentation** — measure primorial calculation & primality testing  
✓ **Multiple algorithms** — Compare Fast (20 rounds), Standard (40 rounds), Thorough (64 rounds)  
✓ **Interactive CLI** — Find single values or run full benchmarks  
✓ **Type-safe architecture** — Traits, error handling, zero runtime type errors

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

Run full benchmark suite:

```bash
./benchmark.sh
```

**Sample Results** (on modern hardware):

| n   | Fortunate # | Time   | Candidates Tested |
| --- | ----------- | ------ | ----------------- |
| 100 | 641         | ~40ms  | 640               |
| 200 | 1619        | ~987ms | 1,618             |
| 300 | 5641        | ~13.3s | 5,640             |
| 400 | 5051        | ~30.3s | 5,050             |

**Observations:**

- Time grows exponentially with n (primorial size grows extremely fast)
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
**Risk:** Minimal (immutable Integer cloning is safe)

### 2. **Candidate Pre-filtering with Wheel Factorization** (HIGH PRIORITY - Expected: 2-3x speedup)

**Current bottleneck:** Testing candidates divisible by 2, 3, 5, 7, etc.

**Strategy:**
Skip candidates where $(p_n\# + m) \mod p \neq 0$ for small primes $p$:

```rust
// Skip if candidate divisible by 2, 3, 5, 7
const SMALL_PRIMES: &[u32] = &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29];

fn is_candidate_viable(m: u32, p_n_sharp: &Integer) -> bool {
    for &p in SMALL_PRIMES {
        if (p_n_sharp + m as i32) % p == 0 {
            return false;
        }
    }
    true
}
```

**Expected impact:** n=400: Skip ~80-90% of candidates before expensive Miller-Rabin  
**Implementation effort:** Low  
**Risk:** Minimal (mathematical proof: all multiples of p can be skipped)

### 3. **Segmented Sieve for Candidates** (MEDIUM PRIORITY - Expected: 1.5x speedup)

**Current bottleneck:** No information about which candidates are likely prime

**Strategy:**
Use Sieve of Eratosthenes on candidate range to identify probable primes first:

```rust
// Pre-sieve candidates [2, max_candidate] for primality hints
fn sieve_candidates(max: u32) -> Vec<bool> {
    let mut is_prime = vec![true; max as usize + 1];
    is_prime[0] = is_prime[1] = false;
    for i in 2..=(max as f64).sqrt() as u32 {
        if is_prime[i as usize] {
            for j in (i*i..=max).step_by(i as usize) {
                is_prime[j as usize] = false;
            }
        }
    }
    is_prime
}
```

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

```
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
