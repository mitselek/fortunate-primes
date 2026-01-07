# Node.js/TypeScript Implementation Prototype

## Overview

Implement parallelized Fortunate number calculator using **Node.js + TypeScript** with native BigInt and external primality libraries to evaluate accessibility vs performance trade-off.

## Motivation

After successful PARI/GP implementation (Issue #11) showing **1.25-1.67x speedup** over Rust orchestration, Node.js is worth exploring for:

### Potential Advantages

- ✅ **Maximum accessibility**: JavaScript most popular language, TypeScript adds safety
- ✅ **Native BigInt**: Built-in arbitrary precision integers (ES2020+)
- ✅ **Rich ecosystem**: npm packages for primality testing
- ✅ **Strong typing**: TypeScript prevents bugs Rust/PARI/GP would catch
- ✅ **Familiar tooling**: VS Code, debuggers, profilers well-integrated
- ✅ **Worker threads**: `worker_threads` module for parallelism

### Key Question

Can Node.js match Python accessibility while achieving **reasonable performance** (within 2-3x of PARI/GP)?

## Expected Architecture

### Core Components

1. **Primorial computation**: BigInt factorial of primes (no native primorial function)
2. **Primality testing**: External library (bigint-crypto-utils, primality, or custom Miller-Rabin)
3. **Parallel workers**: `worker_threads.Worker` pool for batch distribution
4. **Batch strategy**: Reuse optimal batch sizes from PARI/GP (100 for n~500, 150 for n~1000)

### Implementation Pattern

```typescript
import { Worker } from "worker_threads";
import { isProbablyPrime } from "bigint-crypto-utils";

interface WorkerMessage {
  n: number;
  start: bigint;
  batchSize: number;
}

// Worker file: worker.ts
function testBatch(n: number, start: bigint, batchSize: number): bigint | null {
  const pn = computePrimorial(n);

  for (let m = start; m < start + BigInt(batchSize); m++) {
    if (isProbablyPrime(pn + m, 25)) {
      // 25 Miller-Rabin rounds
      return m;
    }
  }
  return null;
}

// Main coordinator
async function fortunateBatch(
  n: number,
  batchSize: number = 100
): Promise<bigint> {
  const numWorkers = os.cpus().length;
  const workers: Worker[] = [];

  // Create worker pool
  for (let i = 0; i < numWorkers; i++) {
    workers.push(new Worker("./worker.js"));
  }

  let round = 0;
  while (true) {
    round++;
    const promises = workers.map((worker, i) => {
      return new Promise((resolve) => {
        worker.once("message", resolve);
        worker.postMessage({
          n,
          start: BigInt(round * numWorkers * batchSize + i * batchSize),
          batchSize,
        });
      });
    });

    const results = await Promise.all(promises);
    const found = results.find((r) => r !== null);
    if (found) {
      workers.forEach((w) => w.terminate());
      return found as bigint;
    }
  }
}
```

## Prototype Tasks

- [ ] Research primality testing libraries (bigint-crypto-utils, primality, custom Miller-Rabin)
- [ ] Implement `fortunate-batch.ts` with `worker_threads`
- [ ] Optimize primorial computation (memoization vs recomputation)
- [ ] Unit tests: F(5)=23, F(10)=61, F(20)=103
- [ ] Benchmark F(500) with batch_size=[50, 100, 150]
- [ ] Benchmark F(1000) with batch_size=[100, 150, 200]
- [ ] Compare vs PARI/GP, Python, and Rust
- [ ] Measure memory usage and worker overhead
- [ ] Document findings in `implementations/node-ts/BENCHMARKS.md`

## Expected Benchmarks

**Target System**: AMD Ryzen 7 2700 (16 threads)

| n    | F(n) | PARI/GP (baseline) | Node.js (target) | Notes                        |
| ---- | ---- | ------------------ | ---------------- | ---------------------------- |
| 500  | 5167 | 6.8s               | TBD (~15-30s?)   | BigInt primality may be slow |
| 1000 | 8719 | 68.9s              | TBD (~150-250s?) | Acceptable if < 3x PARI/GP   |
| 4602 | TBD  | TBD                | Not recommended  | Too slow for large n         |

**Realistic expectations**: Node.js unlikely to match PARI/GP performance (no native GMP), but may offer best accessibility/maintainability trade-off.

## Dependencies

- **Node.js**: v18+ (for best BigInt performance)
- **TypeScript**: 5.0+
- **Primality library**: TBD (evaluate options)

```bash
# Setup
npm install
npm install --save-dev @types/node typescript

# Primality options (evaluate):
npm install bigint-crypto-utils  # Most popular, uses Miller-Rabin
npm install primality             # Alternative
```

## Library Evaluation Criteria

Compare primality testing libraries:

| Library             | Stars | Algorithm         | Performance | Maintenance  |
| ------------------- | ----- | ----------------- | ----------- | ------------ |
| bigint-crypto-utils | ~700  | Miller-Rabin      | TBD         | Active       |
| primality           | ~50   | Multiple          | TBD         | Inactive?    |
| Custom Miller-Rabin | N/A   | Miller-Rabin (25) | TBD         | Full control |

**Test**: Benchmark `isPrime(primorial(500) + 5167)` for each library.

## Open Questions

1. **Primality testing performance**:

   - How much slower is JavaScript BigInt Miller-Rabin vs GMP?
   - Can we match Python+gmpy2 if Python is also slow?

2. **Worker threads overhead**:

   - Does `worker_threads` have similar overhead to Python's `multiprocessing`?
   - Serialization cost for BigInt messages?

3. **Primorial optimization**:

   - Cache primorial in main thread, serialize to workers?
   - Recompute per worker (PARI/GP approach)?
   - Shared memory for BigInt values?

4. **Native addon consideration**:

   - Should we use N-API bindings to GMP for performance?
   - Defeats accessibility purpose if requiring C++ compilation

5. **V8 BigInt optimizations**:
   - Does Node.js v20+ have better BigInt performance?
   - Turbo-fan JIT optimizations for BigInt arithmetic?

## Success Criteria

### Minimum Viable (Worth keeping)

- ✅ F(500) < 30s (within 5x of PARI/GP)
- ✅ F(1000) < 200s (within 3x of PARI/GP)
- ✅ Clean TypeScript code (~150-200 lines)
- ✅ Best-in-class documentation and developer experience

### Realistic Goal

- 🎯 F(500) ~15-20s (2-3x PARI/GP)
- 🎯 F(1000) ~150-180s (2-3x PARI/GP)
- 🎯 "Accessible alternative" recommendation for education/learning

### Stretch (Unlikely without native bindings)

- 🚀 F(500) < 10s (approaching PARI/GP)
- 🚀 Viable for production use

### Documentation

- 📝 Library comparison and recommendations
- 📝 Performance analysis vs PARI/GP, Python, Rust
- 📝 When to use Node.js (education) vs PARI/GP (production)
- 📝 Lessons learned about JavaScript for numerical computing

## Related Issues

- Issue #11: Pure PARI/GP implementation ✅ (completed, 1.25-1.67x faster than Rust)
- Issue #12: Repository restructure ✅ (completed)
- Issue #13: Python (gmpy2) implementation 🚧 (pending)

## References

- Node.js BigInt: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt
- Worker threads: https://nodejs.org/api/worker_threads.html
- bigint-crypto-utils: https://github.com/juanelas/bigint-crypto-utils
- PARI/GP implementation: `implementations/pari-gp/`
- Performance targets: `implementations/pari-gp/BENCHMARKS.md`

## Notes

**Philosophy**: This implementation prioritizes **accessibility and learning** over raw performance. If performance is critical, recommend PARI/GP. If Python knowledge exists, recommend Python+gmpy2. Node.js fills the gap for JavaScript developers who want to explore numerical computing.
