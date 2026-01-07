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

| Implementation | Status | Language | Strategy | Performance (F(500)) |
|----------------|--------|----------|----------|----------------------|
| [Rust](implementations/rust/) | ✅ Production | Rust 1.92.0 | Orchestration + PARI/GP workers | 11.31s (baseline) |
| [PARI/GP](implementations/pari-gp/) | 🚧 Prototype | PARI/GP | Native parallelism | TBD ([Issue #11](https://github.com/mitselek/fortunate-primes/issues/11)) |
| [Python](implementations/python-gmpy2/) | 🚧 Prototype | Python 3.9+ | gmpy2 (GMP bindings) | TBD |
| [Node.js](implementations/node-ts/) | 🚧 Prototype | TypeScript | Native BigInt or WASM+GMP | TBD |

### Quick Comparison

**Rust (Current Baseline)**
- ✅ Highest performance (worker-count-aware adaptive batching)
- ✅ Strong type safety and memory safety
- ⚠️ Requires Rust toolchain + PARI/GP
- ⚠️ More complex (200+ lines, subprocess orchestration)

**Pure PARI/GP** ([Issue #11](https://github.com/mitselek/fortunate-primes/issues/11))
- ✅ Architectural simplicity (~30-50 lines)
- ✅ Single binary, no orchestration overhead
- ⚠️ GP scripting less familiar to developers
- ⚠️ Progress reporting more primitive

**Python + gmpy2**
- ✅ Most accessible language
- ✅ Performance should match PARI/GP (both use GMP)
- ✅ Rich ecosystem (pytest, mypy, black)
- ⚠️ Python runtime overhead vs compiled binary

**Node.js + TypeScript**
- ✅ Maximum developer accessibility (most popular language)
- ✅ Strong TypeScript typing
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

# Python + gmpy2 (when implemented)
cd implementations/python-gmpy2
python3 -m venv venv && source venv/bin/activate
pip install -r requirements.txt
python fortunate.py 500

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

## Current Performance (Rust Baseline)

| n    | F(n)  | Time    | Workers | Hardware |
|------|-------|---------|---------|----------|
| 500  | 5167  | 11.31s  | 15      | Ryzen 7 2700 |
| 1000 | 8719  | 85.8s   | 15      | Ryzen 7 2700 |
| 2500 | 25643 | 27.35m  | 15      | Ryzen 7 2700 |
| 3000 | 27583 | 48.97m  | 15      | Ryzen 7 2700 |
| 4601 | 56611 | 4.96h   | 15      | Ryzen 7 2700 |

**Hardware**: AMD Ryzen 7 2700 (8 physical cores, 16 logical CPUs with SMT)

## Research Status

- ✅ **Rust baseline**: Optimized with worker-count-aware adaptive batching
- ✅ **Issue #11**: Design discussion for pure PARI/GP implementation
- ✅ **Issue #12**: Restructure project for parallel comparison
- 🚧 **Prototypes**: PARI/GP, Python, Node.js implementations pending

## Key Findings

### Rust Architecture (Current)

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
