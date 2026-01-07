# Pure PARI/GP Implementation

**Status**: 🚧 Prototype  
**Issue**: [#11](https://github.com/mitselek/fortunate-primes/issues/11)

## Overview

Single-process PARI/GP implementation using native parallelism (`parapply`, `parfor`, `pareval`) to eliminate the orchestration layer.

## Motivation

Current Rust implementation spawns 15 PARI/GP subprocesses with IPC overhead. PARI/GP has built-in parallel functions that could handle everything internally, achieving architectural simplicity (askesis).

## Expected Architecture

- Single PARI/GP process (~200 MB)
- Native parallel primitives for work distribution
- Primorial never crosses process boundaries (zero serialization)
- ~30-50 lines of GP code vs 200+ lines of Rust

## Setup

```bash
# Ensure PARI/GP installed
sudo apt-get install pari-gp  # Debian/Ubuntu
brew install pari             # macOS

# Check version (need 2.15+ for parallelism)
gp --version
```

## Expected Usage

```bash
# Run GP script
gp -q fortunate.gp

# Or from GP REPL
\r fortunate.gp
fortunate(500)
```

## Prototype Tasks

- [ ] Implement simple parallel search using `parapply()`
- [ ] Benchmark vs Rust (n=500, n=1000)
- [ ] Evaluate adaptive batching feasibility in GP
- [ ] Compare progress reporting capabilities
- [ ] Document findings and performance comparison

## Expected Benchmarks

| n    | F(n)  | Time (estimated) | vs Rust |
|------|-------|------------------|---------|
| 500  | 5167  | TBD              | TBD     |
| 1000 | 8719  | TBD              | TBD     |

## Benefits (Hypothetical)

- **Simplicity**: No subprocess spawning, no IPC, single binary
- **Performance**: Eliminate subprocess spawn cost (10-15ms × 15)
- **Zero serialization**: Big integers stay in GP memory
- **Accessibility**: No Rust toolchain required

## Trade-offs (Expected)

- **Progress reporting**: More primitive than Rust
- **Adaptive batching**: Harder to implement sophisticated features
- **Debugging**: Less mature tools than Rust ecosystem
- **Familiarity**: GP scripting less common than Rust/Python

## References

- PARI/GP Parallelism: <https://pari.math.u-bordeaux.fr/dochtml/html-stable/Parallelism.html>
- Issue #11: Design discussion and implementation plan
- Rust baseline: [../rust/](../rust/)

## Status

Awaiting prototype implementation. See Issue #11 for detailed design discussion.
