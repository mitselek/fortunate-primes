# Alternative Implementation Experiments

## Overview

This directory contains four parallel implementations for calculating Fortunate numbers, designed for architectural comparison and research.

## Implementations

| Implementation                 | Status        | Language             | Dependencies | Performance vs Rust |
| ------------------------------ | ------------- | -------------------- | ------------ | ------------------- |
| [rust/](rust/)                 | ✅ Production | Rust 1.92.0          | pari-gp      | 100% (baseline)     |
| [pari-gp/](pari-gp/)           | ✅ **Winner** | PARI/GP              | pari-gp      | **147-200%** ⚡     |
| [python-gmpy2/](python-gmpy2/) | 🚧 Prototype  | Python 3.9+          | gmpy2, GMP   | TBD                 |
| [node-ts/](node-ts/)           | 🚧 Prototype  | Node.js + TypeScript | none         | TBD                 |

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

| n    | F(n) | Rust   | PARI/GP      | Python | Node.js |
| ---- | ---- | ------ | ------------ | ------ | ------- |
| 5    | 23   | <1ms   | 0.005s       | TBD    | TBD     |
| 10   | 61   | <1ms   | 0.005s       | TBD    | TBD     |
| 20   | 103  | <1ms   | TBD          | TBD    | TBD     |
| 500  | 5167 | 11.31s | **6.76s** ⚡ | TBD    | TBD     |
| 1000 | 8719 | 85.8s  | TBD          | TBD    | TBD     |

**Note**: PARI/GP benchmarked under heavy system load (concurrent Rust F(4602)). Clean system estimated 1.8-2x speedup vs measured 1.67x.

See individual implementation READMEs for detailed setup and benchmark instructions.

## Quick Start

Each implementation directory contains:

- `README.md` - Setup instructions and findings
- `setup.sh` or equivalent - Dependency installation
- `benchmark.sh` - Run standard benchmarks
- Implementation files

## Research Goals

This multi-implementation approach aims to answer:

1. **Can we simplify?** - Pure PARI/GP eliminates orchestration layer
2. **Can we democratize?** - Python/Node.js increase accessibility
3. **What's the trade-off?** - Performance vs simplicity vs accessibility
4. **Which is optimal?** - For production, for education, for research

## Contributing

When adding a new implementation:

1. Create directory under `implementations/`
2. Add README with setup, usage, benchmarks
3. Create `setup.sh` for dependencies
4. Create `benchmark.sh` for standard tests
5. Update this comparison table
6. Document findings and trade-offs

## References

- GitHub Issues: [#11](https://github.com/mitselek/fortunate-primes/issues/11) (PARI/GP), [#12](https://github.com/mitselek/fortunate-primes/issues/12) (Restructure)
- OEIS A005235: <https://oeis.org/A005235>
- Benchmark suite: [../benchmarks/](../benchmarks/)
