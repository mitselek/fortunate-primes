# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned (Phase 1)
- Parallel candidate testing with Rayon (expected 4-8x speedup)
- Wheel factorization pre-filtering (expected 2-3x speedup)
- Extended prime list (beyond current 1,224 primes)

### Planned (Phase 2)
- Segmented sieve optimization
- Lucas-Lehmer primality test (for hybrid approach)

### Planned (Phase 3)
- GPU acceleration exploration
- Batch processing for multiple n values

---

## [0.1.0] - 2026-01-05

### Added
- Initial release: Fortunate prime calculator
- Miller-Rabin primality test (20/40/64 round variants)
- Interactive CLI with menu-driven interface
- Benchmarking infrastructure with timing metrics
- Comprehensive test suite (20 tests with OEIS validation)
- Support for n up to 1,224 (hardcoded primes)
- Performance analysis across three algorithm variants
- TDD-first development practices

### Technical Details
- Language: Rust 1.92.0 (stable)
- Math library: rug 1.28 (GMP bindings)
- Parallelization: rayon 1.8 (available, not yet used)
- Build: Release with LTO and codegen-units=1
- Tested: OEIS A005235 validation through n=10
- Fortune's conjecture: Validated all results prime through n=10

### Documentation
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

