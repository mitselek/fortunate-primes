# Fortunate Primes: Architectural Comparison

Research project comparing four implementations for computing Fortunate numbers F(n).

## What are Fortunate Numbers?

**Definition**: F(n) = smallest m > 1 such that primorial(n) + m is prime

Where primorial(n) = product of first n primes (p₁ × p₂ × ... × pₙ)

**Examples**:

- F(5) = 23 (primorial(5) = 2×3×5×7×11 = 2310, and 2310+23 = 2333 is the first prime)
- F(10) = 61
- F(500) = 5167
- F(4601) = 56611 (beyond OEIS dataset)

**Fortune's Conjecture**: All F(n) are prime (verified computationally, unproven)

**Reference**: [OEIS A005235](https://oeis.org/A005235)

## Research Question

**Which architecture is optimal for computing Fortunate numbers?**

We compare four implementations across multiple dimensions:

- **Performance**: Runtime for F(500) and F(1000)
- **Complexity**: Lines of code, maintainability
- **Accessibility**: Developer familiarity, setup ease
- **Trade-offs**: Memory, dependencies, distribution

## Implementations

| Implementation                          | Version | Status        | Language     | Strategy                        | Performance (F(500)) |
| --------------------------------------- | ------- | ------------- | ------------ | ------------------------------- | -------------------- |
| [Python](implementations/python-gmpy2/) | v5      | ✅ Production | Python 3.12  | Queue-based + visualization     | **1.25s (fastest!)** |
| [Python](implementations/python-gmpy2/) | v3      | ✅ Legacy     | Python 3.12  | Queue-based worker assignment   | 1.25s                |
| [Python](implementations/python-gmpy2/) | v2      | ✅ Legacy     | Python 3.12  | Batch-based with early exit     | 2.65s                |
| [PARI/GP](implementations/pari-gp/)     | -       | ✅ Production | PARI/GP 2.15 | Native thread parallelism       | 6.8s                 |
| [Rust](implementations/rust/)           | -       | ✅ Production | Rust 1.92.0  | Orchestration + PARI/GP workers | 11.3s                |
| [Node.js](implementations/node-ts/)     | -       | 🚧 Planned    | TypeScript   | Native BigInt or WASM+GMP       | Not implemented      |

### Quick Comparison

**🏆 Python + gmpy2 v5 (Winner!)**

- ✅ **Fastest implementation**: 9-22x faster than Rust, 2-5x faster than PARI/GP
- ✅ **Clean architecture**: Workers compute, main orchestrates, print reports (no race conditions)
- ✅ **Real-time visualization**: Markdown table showing worker assignments per result
- ✅ **Queue-based load balancing**: Natural handling of 150x variance within batches
- ✅ Most accessible language
- ✅ Excellent clean-system scaling (8.5x CPU parallelism)
- ✅ Rich ecosystem (pytest, type hints, OEIS validation)
- ✅ Process isolation handles system load well
- ⚠️ Performance degrades 2.5x under heavy load (18-26)
- 📊 **Best for**: Range computations n ≤ 2500 (production-ready)

**Python + gmpy2 v3/v2 (Legacy)**

- v3: Queue-based, minimal output (36% faster than v2 for ranges)
- v2: Batch-based with early termination (good for single large n)
- 📊 **Status**: Superseded by v5, kept for comparison

**PARI/GP (Runner-up)**

- ✅ Second fastest: 1.7x faster than Rust on F(500)
- ✅ Architectural simplicity (~50 lines)
- ✅ Single binary, no orchestration overhead
- ✅ Good load resilience (maintains 8-9x parallelism)
- ⚠️ GP scripting less familiar to developers
- 📊 **Best for**: Standalone scripts, prototyping

**Rust (Baseline)**

- ✅ Strong type safety and memory safety
- ✅ Best for very large n (F(4602) = 5h 52m)
- ✅ Adaptive batching with early termination
- ⚠️ Slowest for small-medium n (9-22x slower than Python)
- ⚠️ Requires Rust toolchain + PARI/GP
- ⚠️ More complex (200+ lines, subprocess orchestration)
- 📊 **Best for**: Production systems, n > 3000

**Node.js + TypeScript (Not Implemented)**

- ❌ Native BigInt 10-50x slower than GMP
- ⚠️ WASM+GMP needed for competitive performance

## Getting Started

Each implementation has its own directory with setup instructions:

```bash
# Rust (current baseline)
cd implementations/rust
cargo build --release
./target/release/fortunate-primes 500

# PARI/GP (when implemented)
cd implementations/pari-gp
gp -q fortunate.gp

# Python + gmpy2 (recommended)
cd implementations/python-gmpy2
python3 -m venv venv && source venv/bin/activate
pip install -r requirements.txt
python fortunate_v5.py 500 510 --md output.md

# Node.js + TypeScript (when implemented)
cd implementations/node-ts
npm install && npm run build
npm start -- 500
```

## Benchmarking

Cross-implementation comparison infrastructure:

```bash
# Run all implementations with standard test cases
./benchmarks/compare-all.sh

# View results
cat benchmarks/results/*.log
```

See [benchmarks/README.md](benchmarks/README.md) for details.

## Performance Comparison (Clean System)

| n    | F(n)  | Python v3   | Python v2 | PARI/GP    | Rust       | v3 Speedup vs Rust |
| ---- | ----- | ----------- | --------- | ---------- | ---------- | ------------------ |
| 500  | 5167  | **1.25s**   | 2.65s     | 6.8s       | 11.3s      | **9.0x**           |
| 1000 | 8719  | **3.86s**   | 8.83s     | 68.9s\*    | 85.8s      | **22.2x**          |
| 2000 | 51137 | **12m 5s**  | -         | Not tested | Not tested | -                  |
| 2500 | 25643 | **2m 52s**  | -         | Not tested | 27.4m      | **9.6x**           |
| 3000 | 27583 | **45.2s**   | -         | Not tested | 49.0m      | **65x**            |
| 4601 | 56611 | **36m 46s** | -         | Not tested | 5.0h       | **8.2x**           |
| 4602 | 62207 | **52m 52s** | -         | Not tested | 5h 52m     | **6.7x**           |
| 4603 | 54083 | **29m 56s** | -         | Not tested | -          | -                  |
| 4604 | 83773 | **1h 49m**  | -         | Not tested | -          | -                  |
| 4608 | 74717 | **1h 27m**  | -         | Not tested | -          | -                  |

\* PARI/GP F(1000) under heavy load (30-36); would be faster on clean system

**Range Performance** (F(500-550), 51 values, 16 workers):

| Implementation | Total Time | Avg per F(n) | Architecture             |
| -------------- | ---------- | ------------ | ------------------------ |
| Python v3      | **79s**    | **1.55s**    | Queue-based              |
| Python v2      | 108s       | 2.11s        | Batch-based (sequential) |

**Hardware**: AMD Ryzen 7 2700 (8 physical cores, 16 logical CPUs with SMT)

## Research Status

- ✅ **Python + gmpy2 v3**: Production-ready, fastest implementation with queue-based architecture (Issue #16)
- ✅ **Python + gmpy2 v2**: Legacy batch-based implementation (36% slower than v3)
- ✅ **PARI/GP**: Production-ready, native parallelism (Issue #11)
- ✅ **Rust baseline**: Optimized with worker-count-aware adaptive batching
- ✅ **Issue #12**: Project restructured for parallel comparison
- 📊 **Benchmarking**: v2 vs v3 architectural comparison complete (ranges 500-550, 600-650)
- 🎯 **Winner**: Python v3 queue-based (36% faster than v2, 9-65x faster than Rust)

## Key Findings

### Python + gmpy2 v3: Queue-Based Architecture Wins! 🏆

**Latest Breakthrough**: v3 queue-based architecture is **36% faster** than v2 batch-based:

- **F(500-550) range**: 79s (v3) vs 108s (v2) = **1.37x speedup**
- **Architecture**: Dynamic queue assignment vs static batch distribution
- **Load balancing**: Workers pull next index when finished (natural work distribution)
- **Variance handling**: 150x variance within batches (F(509)=439ms vs F(531)=66s)

**Why Queue-Based Wins**:

1. **Natural load balancing**: Fast workers (F(509)=439ms) pull more indices, slow workers don't block
2. **No redundant computation**: Each primorial(n) computed once per index vs 16× per batch in v2
3. **Optimal for ranges**: v3 excels at sequential n computations (production use case)
4. **Simpler architecture**: Fewer moving parts than batch coordination

**v2 vs v3 Trade-off**:

- **v2 (batch-based)**: Better for single large n (early termination), redundant primorial computation
- **v3 (queue-based)**: Better for ranges (load balancing), compute once per index

**Overall Performance**: Python v3 is **9-22x faster** than Rust for small-medium n:

- F(500): 1.25s (Python v3) vs 11.3s (Rust) = **9.0x speedup**
- F(1000): 3.86s (Python v3) vs 85.8s (Rust) = **22.2x speedup**
- F(2500): 2m 50s (Python v3) vs 27.4m (Rust) = **9.6x speedup**

**Why Python Wins**:

1. **GMP efficiency**: gmpy2 provides direct GMP bindings with minimal overhead
2. **Process isolation**: Multiprocessing avoids GIL, handles system load better than threading
3. **Clean-system scaling**: Achieves 8.3-8.7x CPU parallelism (16 workers)
4. **Queue-based work distribution**: Natural load balancing (v3)
5. **No subprocess overhead**: Unlike Rust's PARI/GP orchestration, Python runs primality tests in-process

**Load Sensitivity**: Python performance degrades 2.4-2.5x under heavy load (18-26), but remains fastest overall.

**Sweet Spot**: Python v3 excels for range computations n ≤ 2500. Beyond n=2000, primality testing (Miller-Rabin on ~7800+ digit numbers) dominates and scaling becomes superlinear.

### PARI/GP: Simplicity + Performance

**Design**: Native PARI/GP with thread parallelism

**Performance**: Second fastest (1.7x faster than Rust on F(500))

**Advantages**:

- Architectural simplicity (~50 lines vs Rust's 200+)
- Load resilience (maintains 8-9x parallelism under heavy load)
- Single binary, no orchestration overhead

### Rust Architecture

**Design**: Rust orchestration + 15 PARI/GP subprocesses

**Key Insight**: Primorials never leave PARI/GP memory

- Rust sends: Small integers (n, start, end) + script text
- PARI/GP computes: Primorial with 1000s of digits internally
- PARI/GP returns: Small offset m (u64)
- Result: Zero serialization overhead

**Memory Efficiency**:

- Rust coordinator: ~2 MB
- Each PARI/GP worker: ~13 MB
- Total: ~197 MB for 15 workers

**Optimizations**:

1. Worker-count-aware adaptive batching (60s/num_workers threshold)
2. Early termination with cooperative cancellation
3. Contiguous lower bound tracking
4. Progress reporting with interval notation

### Design Trade-offs: Batch vs Interleaved

Performance crossover around n=1000-1500:

- **Interleaved wins** (n < 1000): Zero coordination, static assignment
- **Batch wins** (n ≥ 1000): Cache locality, early termination, adaptive sizing

See [implementations/rust/README.md](implementations/rust/README.md) for full analysis.

## Contributing

We welcome implementations in other languages and architecture experiments!

1. Create directory under `implementations/`
2. Add README with setup, usage, benchmarks
3. Implement standard test cases (n=5, 10, 20, 500, 1000)
4. Run benchmarks and document findings
5. Update comparison tables

See [archived/CONTRIBUTING.md](archived/CONTRIBUTING.md) for development workflow.

## Documentation

- [Implementations Overview](implementations/README.md) - Detailed comparison matrix
- [Benchmarking Guide](benchmarks/README.md) - How to run performance tests
- [Rust Implementation](implementations/rust/README.md) - Current baseline details
- [PARI/GP Design](implementations/pari-gp/README.md) - Architectural simplicity approach
- [Python Implementation](implementations/python-gmpy2/README.md) - Accessibility focus
- [Node.js Implementation](implementations/node-ts/README.md) - Maximum reach

## References

- **OEIS A005235**: <https://oeis.org/A005235>
- **Fortune's Conjecture**: R. K. Guy, "Unsolved Problems in Number Theory"
- **PARI/GP**: <https://pari.math.u-bordeaux.fr/>
- **Baillie-PSW Test**: Used by `ispseudoprime()`, no known counterexamples
- **GMP**: <https://gmplib.org/> (underlying big integer library)

## License

This is a research project for architectural comparison and education.

## Related Issues

- [#11: Pure PARI/GP implementation](https://github.com/mitselek/fortunate-primes/issues/11)
- [#12: Restructure for parallel comparison](https://github.com/mitselek/fortunate-primes/issues/12)
