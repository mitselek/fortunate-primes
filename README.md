# Fortunate Primes

Calculate Fortunate numbers F(n) using PARI/GP.

## Definition

F(n) = smallest m > 1 such that primorial(n) + m is prime, where primorial(n) = p₁ × p₂ × ... × pₙ

## Requirements

- PARI/GP: `sudo apt install pari-gp`

## Usage

```bash
cargo build --release
./target/release/fortunate-primes <n>
```

## Examples

```text
$ ./target/release/fortunate-primes 10
F(10) = 61
time: 16ms

$ ./target/release/fortunate-primes 500
F(500) = 5167
time: 23.36s
```

Progress is shown after 2 seconds of computation.

## Architecture

### CPU-Based Parallel Design

- **pari.rs**: PARI/GP subprocess interface
- **search.rs**: Parallel batch coordinator with cooperative cancellation
- **progress.rs**: Terminal progress reporting with interval notation

**PARI/GP Command:**

Each worker executes PARI/GP script for batch `[start, end]`:

```gp
pn=prod(i=1,n,prime(i));              # Compute primorial(n)
for(m=start,end,                       # Search range
  if(ispseudoprime(pn+m),              # Test primorial(n) + m
    print(m); break))                  # Print and exit on first prime
```

**Primality Testing:**

The `ispseudoprime()` function uses the Baillie-PSW test:

- **Deterministic for n ≤ 15**: `primorial(15) = 614889782588491410 < 2⁶⁴`
- **Probabilistic for n ≥ 16**: Numbers exceed 2⁶⁴, but no Baillie-PSW counterexamples are known
- For n=4601: Testing numbers with ~1000 digits, far beyond deterministic range
- **Reliability**: No false positives found despite extensive research; considered trustworthy for practical purposes

**Key Design Insight:**

The Rust process **never handles the massive primorial numbers** - they stay entirely within PARI/GP!

- **Rust → PARI/GP**: Sends small integers (`n`, `start`, `end`) and script text
- **PARI/GP internal**: Computes primorial(n) with 1000s of digits, performs all big integer arithmetic
- **PARI/GP → Rust**: Returns only the small offset `m` (a u64)

This explains the memory efficiency: Rust coordinator uses only 2 MB, while each PARI/GP worker needs ~13 MB to store the primorial and arithmetic workspace. No serialization overhead - PARI/GP acts as a specialized math coprocessor.

**Threading Model:**

- Main coordinator thread (minimal CPU usage)
- `num_cpus - 1` worker threads (15 on 16-core system)
- Each worker spawns PARI/GP subprocess for primality testing
- Workers distributed across CPU cores at ~99% utilization

**Memory Efficiency:**

- Main process: ~2 MB RSS
- Each PARI/GP worker: ~13 MB RSS
- Total footprint: ~200 MB for 15 workers

### GPU Considerations

While GPU acceleration might seem attractive for parallelism, primality testing for Fortunate numbers faces significant challenges:

**Current CPU Approach:**

- 15 candidates tested in parallel
- Mature big integer libraries (PARI/GP, GMP)
- Efficient for numbers with 100s-1000s of digits

**Hypothetical GPU Approach:**

- Could test 1000s of candidates simultaneously
- **Problem**: Limited big integer arithmetic support on GPU
- Complex modular arithmetic doesn't map well to GPU architecture
- Memory constraints for multi-precision integers per thread

**Expected Reality:**

- Theoretical: 10-100x speedup from massive parallelism
- Practical: 2-5x speedup due to big integer overhead
- GPU libraries (cuBigInt, cgbn) less mature than CPU counterparts

**Conclusion:** CPU + PARI/GP remains optimal for this workload. GPUs excel at simple operations on massive datasets, but primality testing requires complex arithmetic on huge numbers where CPU libraries dominate.

### Design Evolution: Batch vs Interleaved Strategy

The current batch-based approach (dynamic work queue) replaced an earlier interleaved/strided strategy (static worker assignments). Performance comparison reveals an interesting crossover:

**Interleaved Strategy (archived):**

With N workers (e.g., N=15), each worker k tests candidates at stride N:

- Worker 0: tests m=1, 16, 31, 46... (start at 1, stride 15)
- Worker 1: tests m=2, 17, 32, 47... (start at 2, stride 15)
- Worker k: tests m=(k+1), (k+1)+N, (k+1)+2N... (start at k+1, stride N)
- Static assignment, no coordination overhead

**Batch Strategy (current):**

- Workers pull consecutive-candidate batches from dynamic queue
- Adaptive batch sizing (starts 100, doubles if <30s completion)
- Cooperative cancellation when candidate found
- Contiguous lower bound tracking for progress

**Performance Crossover:**

| n    | F(n)  | Interleaved | Batch-based | Winner      |
| ---- | ----- | ----------- | ----------- | ----------- |
| 600  | 16187 | 16.7s       | 19.80s      | Interleaved |
| 2000 | 51137 | 1775s       | ~1634s      | Batch-based |

**Why interleaved wins at n=500-600:**

- F(n) found quickly (~16K tests), runtime dominated by parallel coverage
- No coordination overhead - workers stride independently
- Process spawn cost (10-15ms) negligible compared to 16-20s runtime

**Why batch-based wins at n≥2000:**

- Cache locality: testing consecutive numbers vs jumping by stride-16
- Early termination: stops dispatching + cooperative worker exit when candidate found
- Dynamic load balancing: adapts to variable primality test costs (larger numbers = slower)
- Lower bound tracking: enables progress monitoring and gap closure detection
- Long runtime (27+ minutes) amortizes coordination overhead

**Crossover point:** Around n=1000-1500, where runtime becomes long enough that the batch optimizations (early termination, cache locality, adaptive sizing) outweigh the interleaved approach's zero-coordination advantage.

## Testing

```bash
cargo test
```

## Performance

Using PARI/GP backend is significantly faster than pure Rust implementations, especially for larger `n`.

Some sample results:

|    n |  F(n) | Batch size |   Time |
| ---: | ----: | ---------: | -----: |
|  500 |  5167 |        800 |  5.70s |
|  600 | 16187 |        800 | 19.80s |
|  700 | 12853 |       1600 | 30.00s |
| 1079 |  8929 |        800 | 57.28s |
| 1300 | 13457 |       1600 |  3.20m |
| 1800 | 16229 |       1600 |  8.30m |
| 2000 | 51137 |        200 | 27.23m |
| 2500 | 25643 |        200 | 27.35m |
| 3000 | 27583 |        200 | 48.97m |
| 4601 | 56611 |        200 |  4.96h |
