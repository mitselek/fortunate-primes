# Alternative Implementation Experiments

## Overview

This directory contains four parallel implementations for calculating Fortunate numbers, designed for architectural comparison and research.

## Implementations

| Implementation                 | Status             | Language             | Dependencies | Performance vs Rust |
| ------------------------------ | ------------------ | -------------------- | ------------ | ------------------- |
| [python-gmpy2/](python-gmpy2/) | ✅ **Winner** 🏆   | Python 3.12+         | gmpy2, GMP   | **900-2220%** ⚡    |
| [pari-gp/](pari-gp/)           | ✅ Production      | PARI/GP              | pari-gp      | 125-167%            |
| [rust/](rust/)                 | ✅ Production      | Rust 1.92.0          | pari-gp      | 100% (baseline)     |
| [node-ts/](node-ts/)           | 🚧 Prototype       | Node.js + TypeScript | none         | Not implemented     |

## Status Legend

- ✅ **Production**: Battle-tested, optimized, documented
- 🚧 **Prototype**: Functional implementation, benchmarking in progress
- ⏸️ **Paused**: On hold pending other findings
- ❌ **Deprecated**: Superseded by better approach

## Evaluation Criteria

Each implementation is evaluated on:

1. **Performance**: Runtime for F(500) and F(1000)
2. **Complexity**: Lines of code, cognitive overhead
3. **Dependencies**: Installation difficulty, platform compatibility
4. **Maintainability**: Code clarity, debugging tools
5. **Accessibility**: Developer familiarity, learning curve
6. **Memory Efficiency**: Peak memory usage during computation
7. **Parallelism**: Effectiveness of multi-core utilization

## Benchmark Results

Standard test cases (all implementations):

| n    | F(n) | Python          | PARI/GP       | Rust   | Node.js         |
| ---- | ---- | --------------- | ------------- | ------ | --------------- |
| 5    | 23   | <1ms            | 0.005s        | <1ms   | TBD             |
| 10   | 61   | <1ms            | 0.005s        | <1ms   | TBD             |
| 20   | 103  | <1ms            | TBD           | <1ms   | TBD             |
| 500  | 5167 | **1.25s** ⚡    | 6.8s          | 11.31s | Not implemented |
| 1000 | 8719 | **3.86s** ⚡    | 68.9s         | 85.8s  | Not implemented |
| 2000 | 51137 | **12m 5s** ⚡   | -             | -      | Not implemented |
| 2500 | 25643 | **2m 52s** ⚡   | -             | 27.4m  | Not implemented |
| 3000 | 27583| **45.2s** ⚡    | -             | 49.0m  | Not implemented |
| 4602 | 62207| -               | -             | 5h 52m | Not implemented |

**Note**: All results on clean system (load ~1-2). See [main README](../README.md#performance-comparison-clean-system) for detailed methodology.

See individual implementation READMEs for detailed setup and benchmark instructions.

## Quick Start

Each implementation directory contains:

- `README.md` - Setup instructions and findings
- `setup.sh` or equivalent - Dependency installation
- `benchmark.sh` - Run standard benchmarks
- Implementation files

## Research Goals

This multi-implementation approach answers:

1. **Can we simplify?** - ✅ PARI/GP eliminates orchestration layer
2. **Can we democratize?** - ✅ Python+gmpy2 is fastest AND most accessible
3. **What's the trade-off?** - Python wins on all fronts for n ≤ 2500
4. **Which is optimal?** - Python for production, PARI/GP for prototyping, Rust for extreme scale

## Contributing

When adding a new implementation:

1. Create directory under `implementations/`
2. Add README with setup, usage, benchmarks
3. Create `setup.sh` for dependencies
4. Create `benchmark.sh` for standard tests
5. Update this comparison table
6. Document findings and trade-offs

## References

- Main README: [../README.md](../README.md) (authoritative benchmark data)
- GitHub Issues: [#11](https://github.com/mitselek/fortunate-primes/issues/11) (PARI/GP), [#12](https://github.com/mitselek/fortunate-primes/issues/12) (Restructure), [#13](https://github.com/mitselek/fortunate-primes/issues/13) (Python)
- OEIS A005235: <https://oeis.org/A005235>
- Benchmark suite: [../benchmarks/](../benchmarks/)
