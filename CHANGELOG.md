# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned (Phase 2)

- Parallel candidate testing with Rayon (expected 2-4x speedup)
- Segmented sieve optimization for probable prime pre-filtering (expected 1.5x)

### Planned (Phase 3)

- GPU acceleration exploration
- Batch processing for multiple n values
- Extended prime list (beyond current 1,224 primes)

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
