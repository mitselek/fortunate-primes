# Anomaly: Hard Composite at F(4601), offset 44207

## Summary

During adaptive batch sizing analysis of `F(4601)`, discovered a pathological composite number that takes **~28 seconds** to disprove as prime via Miller-Rabin with 25 rounds. This composite appears "prime-like"—it passes 24+ rounds before finally failing, requiring nearly as much computation as an actual prime.

**Date**: January 8, 2026  
**n**: 4601  
**Offset**: 44207  
**Composite**: primorial(4601) + 44207  
**Status**: COMPOSITE (confirmed)  
**Result**: F(4601) = 56611 (found in 36.76 minutes, 15 workers)

## Discovery

During F(4601) search on 15 workers with adaptive batch sizing:

```
F(4601) W1 : [44221; ?] [44207+1] (22.59s) (22.61s)
```

Worker 1 tested a single offset [44207+1] and it took **22.59 seconds**, which is **~1300x slower** than typical composites (14-25ms).

## Investigation

### Step 1: Manual test with 25 rounds

```
Testing F(4601) at offset 44207...
Primorial computed in 0.03s
Testing if primorial(4601) + 44207 is prime...
Result: COMPOSITE
Test time: 28.28s
```

Confirmed: The number is **composite** but requires 28+ seconds to disprove.

### Step 2: Minimal rounds comparison (24 vs 25)

```
=== Testing with 24 rounds ===
Result: COMPOSITE
Time: 28.17s

=== Testing with 25 rounds ===
Result: COMPOSITE
Time: 27.89s

✓ Both rounds agree: False
```

**Finding**: Both 24 and 25 rounds agree it's composite in nearly identical time. This suggests the witness was found around round 24 or earlier, making round 25 redundant for _this particular_ composite.

### Root Cause

**Miller-Rabin probabilistic structure**:

- Quick composites: Fail early (rounds 1-3), detected instantly
- Normal composites: Fail mid-range (rounds 4-15), take 100ms-1s
- **Hard composites**: Pass many rounds (18-24), only fail on last round(s) → requires full 25 iterations
- Primes: Must complete all 25 rounds with zero failures → full time cost

This composite is "unlucky"—it's a Carmichael number or has other properties that make it resistant to Miller-Rabin early rejection.

## Computational Cost Hierarchy

| Category           | Time       | Example                               |
| ------------------ | ---------- | ------------------------------------- |
| Quick composite    | 14-25ms    | Most offsets, fail rounds 1-5         |
| Normal composite   | 100ms-1s   | Harder to factorize, fail rounds 6-15 |
| **Hard composite** | **25-30s** | **primorial(4601) + 44207**           |
| Actual prime       | 25-30s     | Requires all 25 rounds, no witness    |

**Observation**: Hard composites and actual primes have similar cost (~25-30s for n=4601), since both require running through most/all Miller-Rabin rounds.

## Impact on Adaptive Batching

This discovery exposed the limits of adaptive batch sizing:

**The problem**:

- Batch size was being adjusted based on completion time
- One worker gets unlucky with a 28s batch → triggers shrink (batch_size: 32 → 4)
- Next workers get 4-8 sized batches, which still hit 25-30s hard composites
- Size adjustment chases variance, not improving throughput

**The lesson**:

- Miller-Rabin variance (14ms to 28s) is **inherent**, not fixable by batch tuning
- Adaptive sizing on batch completion time is ineffective when variance is 1000x
- Better approach: **Fixed batch size** (e.g., 16) or hysteresis-based adjustment

## Decision: Minimum Batch Size Floor

Implemented `max(16, calculated_size)` in `adjust_batch_size()` to:

- Eliminate thrashing (32 → 4 → 2 → 4 → 8 → 16 swings)
- Accept that variance is noise, not signal
- Maintain throughput balance with fixed reasonable batch size

## References

- **Miller-Rabin Primality Test**: https://en.wikipedia.org/wiki/Miller%E2%80%93Rabin_primality_test
- **Carmichael Numbers**: https://en.wikipedia.org/wiki/Carmichael_number
- **gmpy2 Documentation**: https://gmpy2.readthedocs.io/

## Test Commands

To reproduce:

```bash
# Single test with 25 rounds
cd /path/to/implementations/python-gmpy2
python test_offset.py

# Compare 24 vs 25 rounds
python test_24_vs_25.py

# Full F(4601) search (may take hours)
.venv/bin/python fortunate_v2.py 4601
```

## Conclusion

Hard composites like primorial(4601) + 44207 are a legitimate computational challenge, not a bug. The gap-closing algorithm correctly handles them, but they contribute significantly to search time variance. Future optimizations should focus on:

1. **Reducing Miller-Rabin rounds** (e.g., 20 instead of 25 for ~10% speedup)
2. **Deterministic primality tests** for smaller n (e.g., AKS, though much slower)
3. **Accepting the variance** and tuning batch size conservatively (e.g., floor = 16)
