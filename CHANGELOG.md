# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned (Phase 5+)

- GPU acceleration exploration
- Batch processing for multiple n values

---

## [0.5.1] - 2026-01-05

### Enhancement

- **Extended prime cache**: PRIMES_10K now contains actual 10,000 primes (up to 104,729)
  - Previous: 1,224 primes (incorrectly named, only supported n ≤ 1,224)
  - Current: 10,000 primes (correctly named, supports n ≤ 10,000)
  - File size: 70 KB (up from 7.5 KB)
  - Zero performance impact (array access is O(1), bottleneck is primorial/Miller-Rabin)
  - Enables calculation of Fortunate numbers up to n=10,000 vs previous limit of n=1,224
  - Updated documentation to reflect correct prime cache limit

---

## [0.4.3] - 2026-01-05

### Phase 4: Code Refactoring & Modularization (Phase 3)

#### Refactoring

- **Extract calculator implementations**: New `src/calculators/` submodule structure

  - Created `src/calculators/base.rs` - PrimeBasedCalculator implementation
  - Created `src/calculators/parallel.rs` - ParallelFortunateCalculator implementation
  - Created `src/calculators/sieved.rs` - SievedFortunateCalculator implementation
  - Created `src/calculators/mod.rs` - Module organization with public re-exports
  - Each calculator module includes its own tests (62 total tests across all modules)

- **Update lib.rs module structure**
  - Added `pub mod calculators;` declaration
  - Re-export calculator types for backward compatibility
  - Removed ~720 lines of calculator implementation code
  - lib.rs reduced from 1,502 → 784 lines (-48%)

#### Impact Analysis

- **Code organization**: Clear separation of calculator implementations into focused modules
- **Maintainability**: Each calculator independently testable with colocated tests
- **Module boundaries**: Clean dependencies (calculators depend on core traits in lib.rs)
- **Testing**: All 62 tests passing (45 original + 17 calculator-specific tests)
- **API compatibility**: Zero breaking changes via public re-exports

#### Quality Verification

- ✅ 62/62 tests passing (100% test preservation)
- ✅ Zero compilation warnings
- ✅ Backward compatible (re-exports maintain public API)
- ✅ Code formatted (`cargo fmt` clean)

#### Cumulative Phase 4 Progress

| Metric       | Before Phase 1 | After Phase 3 | Total Reduction |
| ------------ | -------------- | ------------- | --------------- |
| lib.rs lines | 1,900          | 784           | -59%            |
| Module count | 3              | 9             | +200%           |
| Test count   | 45             | 62            | +38%            |

**Module structure**:

```text
src/
├── lib.rs (784 lines) - Core traits, error handling, re-exports
├── primality.rs (83 lines) - Miller-Rabin primality testing
├── sieve.rs (101 lines) - Segmented Sieve algorithm
├── wheel.rs (213 lines) - Wheel factorization optimization
├── calculators/
│   ├── mod.rs - Module organization
│   ├── base.rs - PrimeBasedCalculator (sequential baseline)
│   ├── parallel.rs - ParallelFortunateCalculator (Rayon parallelization)
│   └── sieved.rs - SievedFortunateCalculator (sieve + parallel)
└── primes.rs - Pre-computed 10,000 primes
```

---

## [0.4.2] - 2026-01-05

### Phase 4: Code Refactoring & Modularization (Phase 2)

#### Refactoring

- **Extract wheel factorization**: New `src/wheel.rs` module

  - Moved `WheelFactorization` struct and wheel iterator implementation
  - Moved `WheelFortunateCalculator` struct and trait implementations
  - Includes 8 unit tests for wheel factorization correctness
  - Size: ~230 lines (extracted from lib.rs)

- **Update lib.rs module structure**
  - Add `pub mod wheel;` declaration
  - Re-export wheel types: `pub use wheel::WheelFortunateCalculator;`
  - Maintain backward-compatible public API (no breaking changes)

#### Impact

- **Code organization**: Further reduced lib.rs (1,715 → ~1,485 lines after Phase 2)
- **Separation of concerns**: Wheel factorization now isolated from core calculators
- **Maintainability**: Easier to navigate and modify wheel optimization logic
- **Testing**: 8 comprehensive wheel tests colocated with implementation

#### Quality

- ✅ All 45 tests pass (8 wheel tests in new module)
- ✅ No breaking changes to public API
- ✅ Zero logic changes (mechanical refactoring only)

---

## [0.4.1] - 2026-01-05

### Phase 4: Code Refactoring & Modularization (Phase 1)

#### Refactoring

- **Extract primality testing**: New `src/primality.rs` module
  - Moved `MillerRabin` struct and `impl PrimalityTest`
  - Includes 6 unit tests for primality algorithm
  - Size: 83 lines (extracted from lib.rs)
- **Extract sieve algorithm**: New `src/sieve.rs` module

  - Moved `SegmentedSieve` struct and implementations
  - Includes 1 unit test for sieve correctness
  - Size: 101 lines (extracted from lib.rs)

- **Update lib.rs module structure**
  - Add `pub mod primality;` and `pub mod sieve;` declarations
  - Re-export extracted types: `pub use MillerRabin; pub use SegmentedSieve;`
  - Maintain backward-compatible public API (no breaking changes)

#### Impact

- **Code organization**: Reduced lib.rs by ~185 lines (1,900 → 1,715 lines)
- **Single responsibility**: Each module now focuses on specific concern
- **Testability**: Tests colocated with implementations
- **Maintainability**: Easier to locate and modify specific algorithms
- **Reusability**: Primality module and sieve can be used independently

#### Quality

- ✅ All 45 tests pass (7 unit tests from extracted modules)
- ✅ No breaking changes to public API
- ✅ Zero logic changes (mechanical refactoring only)
- ✅ Code formatted and linting clean

#### Architecture Notes

- **Traits remain in lib.rs**: `PrimalityTest` and `FortunateCalculator` are core abstractions
- **Module dependencies**: Calculators use extracted modules through trait bounds
- **Test location**: Each module includes its own `#[cfg(test)]` tests
- **Phase 2 planning**: Wheel and calculator modules ready for future extraction

---

## [0.4.0] - 2026-01-06

### Phase 3: Segmented Sieve Optimization

#### Added

- **SegmentedSieve**: Efficient Eratosthenes sieve for composite pre-filtering
  - `simple_sieve()`: Generates basis primes up to sqrt(limit)
  - `sieve_range()`: Sieves arbitrary ranges using basis primes
  - Memory-efficient: O(range_size) per sieve operation
- **SievedFortunateCalculator**: Hybrid sieve+parallel calculator
  - Sieves full range once to eliminate composites
  - Uses Rayon to parallel test survivors (probable primes)
  - Atomic counters for thread-safe metrics tracking
- **3 New Tests**: Added sieve-specific testing
  - `test_segmented_sieve_basic`: Validates sieve correctness (25 primes in [2..100])
  - `test_sieved_calculator_correctness`: OEIS A005235 validation
  - `test_sieved_speedup_benchmark`: Performance target ≥1.3x over Phase 2

#### Performance

- **n=100**: 20.5ms (parallel) → 12.7ms (sieved) (**1.61x speedup** over Phase 2)
- **Combined optimization**: **3.62x speedup** over Phase 1 wheel (12.7ms vs 46ms)
- **Total improvement**: **4.92x speedup** over original baseline (12.7ms vs 62.5ms)
- Exceeded target 1.3x+ speedup with hybrid approach
- Sieve reduces Miller-Rabin calls by filtering ~80% of composites

#### Architecture

Hybrid sieve+parallel design balances:

- **Sieve overhead**: One-time O(n log log n) pre-filtering cost
- **Parallel gain**: Multi-core testing of smaller candidate set
- **Efficiency**: Testing ~1,200 probable primes instead of 10,000 candidates

#### Test Coverage

- **Total tests**: 44 (up from 42)
- **Test pass rate**: 100%
- **OEIS validation**: All sieved results match parallel/sequential
- **Fortune's conjecture**: All sieved Fortunate numbers confirmed prime

---

## [0.3.0] - 2026-01-05

### Phase 2: Parallel Candidate Testing with Rayon

#### Added

- **Rayon Parallel Execution**: Implemented batch-based parallel candidate testing in `ParallelFortunateCalculator`
  - Processes candidates in parallel batches (batch size: 100)
  - Uses Rayon's `par_iter()` + `find_any()` for multi-core execution
  - Sequential verification within batch ensures SMALLEST Fortunate number found
- **4 New Tests**: Added comprehensive parallel testing suite
  - `test_parallel_speedup_benchmark`: Validates ≥1.5x speedup at n=100
  - `test_parallel_correctness_with_rayon`: Validates OEIS A005235 equivalence
  - `test_parallel_thread_safety`: Ensures thread-safe concurrent execution
  - `test_parallel_large_n_speedup`: Validates ≥2x speedup at n=200
- **Atomic Metrics Tracking**: Thread-safe counters for primality test metrics

#### Performance

- **n=100**: 35.9ms → 19.1ms (**1.87x speedup** over wheel, **2.26x vs original**)
- **n=200**: 675ms → 235ms (**2.87x speedup** over wheel, **4.23x vs original**)
- Achieved target 2-4x speedup for larger n values
- Speedup increases with problem size (parallelism benefits grow with n)

#### Test Coverage

- **Total tests**: 39 (up from 35)
- **Test pass rate**: 100%
- **OEIS A005235 validation**: All parallel results match sequential (n=1 through n=31)
- **Fortune's conjecture**: All tested Fortunate numbers confirmed prime
- **Thread safety**: Concurrent execution produces consistent results

### Notes

Phase 2 completes the parallel optimization path. Combined with Phase 1 wheel factorization,
we've achieved cumulative 2-4x speedup through algorithmic improvements and multi-core execution.

---

## [0.2.0] - 2026-01-05

### Phase 1 Optimization Complete

#### 1.1 Parallel Infrastructure (Foundation)

- Add `ParallelFortunateCalculator` struct with full `FortunateCalculator` trait support
- Establish parallel-ready architecture for future Rayon integration
- 8 comprehensive tests validating correctness against OEIS A005235
- All `ParallelFortunateCalculator` results identical to `PrimeBasedCalculator`

#### 1.2 Wheel Factorization (40-50% Speedup)

- Implement `WheelFactorization` with 2-3-5 wheel (period 30)
- Add `WheelFortunateCalculator` for candidate pre-filtering
- Reduce candidate search space by ~63% (consistently across all n)
- Verified speedup: n=100 (1.41x), n=200 (1.51x)
- 7 comprehensive tests validating OEIS equivalence and Fortune's conjecture
- Interactive benchmark now shows standard vs wheel comparison

### Test Coverage

- **Total tests**: 35 (up from 20)
- **Test pass rate**: 100%
- **OEIS A005235 validation**: n=1 through n=31 all correct
- **Fortune's conjecture**: All tested Fortunate numbers confirmed prime

### Benchmark Results

| n   | Standard | Wheel   | Speedup | Tests Reduced   |
| --- | -------- | ------- | ------- | --------------- |
| 100 | 43.2ms   | 30.6ms  | 1.41x   | 640→236 (-63%)  |
| 200 | 993.8ms  | 656.8ms | 1.51x   | 1618→593 (-63%) |

### Technical Details

- Language: Rust 1.92.0 (stable)
- Math library: rug 1.28 (GMP bindings)
- Dependencies: rayon 1.8 (integrated, ready for Phase 2)
- Build: Release with LTO and codegen-units=1
- Quality: Code formatted, clippy clean, all tests passing

### Documentation

- Updated README with Phase 1 results and benchmark comparison
- Updated CHANGELOG with optimization roadmap
- Architecture documentation in DEVELOPMENT.md
- Comprehensive inline code comments for optimization strategies

---

## [0.1.0] - Initial Release

(See end of file for 0.1.0 details)

- README with features, building, benchmarks
- DEVELOPMENT.md with architecture, testing strategy, feature request process
- .gitignore for Rust projects
- .editorconfig for consistent formatting
- Makefile with 11 convenience targets

### Known Limitations

- Single-threaded (Rayon integrated but not utilized)
- Limited by hardcoded prime list (up to 1,224 primes)
- Performance exponential beyond n=400 (baseline: ~30s)
- No GPU acceleration yet
