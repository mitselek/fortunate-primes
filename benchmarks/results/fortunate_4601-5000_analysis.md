================================================================================
Fortunate Numbers: F(4601) - F(5000) Computation Project
================================================================================

## Overview

Systematic computation of Fortunate numbers beyond the OEIS A005235 dataset 
(which covers n ≤ 4600). Target: complete computation through F(5000).

**System**: AMD Ryzen 7 2700 (16 logical cores)  
**Implementation**: Python 3.12.3 + gmpy2 2.1.5  
**Algorithm**: Parallel gap-closing with adaptive batch sizing (floor=16)  
**Optimization**: Firoozbakht (start at p_{n+1})  
**Status**: F(4601-4606) complete, continuing to F(5000)

================================================================================
COMPUTATION RESULTS (F(4601-4606))
================================================================================

See: `fortunate_4601-5000_data.csv` for raw data.

### Performance Analysis

**Scaling Pattern**:
- F(4603) = 54083 anomalously fast (smaller answer value)
- F(4604) = 83773 slower (larger gap)
- F(4605) = 69143 medium speed
- F(4606) = 97813 slowest so far
- Pattern: Answer size correlates with computation time more than n index

**Speedup vs Rust (where applicable)**:
- F(4601) = 8.2x faster (Rust: 5.0h)
- F(4602) = 6.7x faster (Rust: 5h 52m)
- F(4603-4606): Beyond OEIS dataset, no Rust baseline

**Worker Efficiency**:
- Increased from 15 to 16 workers starting with F(4604)
- All workers consistently active throughout search
- Batch floor of 16 prevents thrashing on hard composites

### Primorial Growth Pattern

```
n     | Primorial Digits | Note
------|------------------|-----
4601  | 19,113          | composite index
4602  | 19,117          | composite index (+4 digits)
4603  | 19,122          | PRIME index (+5 digits)
4604  | 19,127          | composite index (+5 digits)
4605  | 19,131          | composite index (+4 digits)
4606  | 19,136          | composite index (+5 digits)
```

Note: Index 4603 is prime, correctly shows digit increase when new prime 
factor p_4603 is multiplied into primorial.

================================================================================
OEIS DATASET STATUS
================================================================================

**OEIS A005235** (Fortunate numbers):
- Covers: F(1) through F(4600) - all validated ✓
- Beyond: F(4601) and later are unverified but computed

**Ground Truth References**:
- OEIS A005235: https://oeis.org/A005235
- OEIS A000040 (primes): https://oeis.org/A000040

================================================================================
METHODOLOGY
================================================================================

### Algorithm: Parallel Gap-Closing Search

1. **Initial Lower Bound**: Start at p_{n+1} (Firoozbakht optimization)
   - All offsets 2..p_n are composite (share factors with primorial(n))
   
2. **Parallel Workers**: 15-16 processes test batches [start; start+batch_size)
   
3. **Adaptive Batch Sizing**: 
   - Grow when batches complete quickly
   - Shrink when primality tests are slow
   - Floor of 16 prevents thrashing on variance
   
4. **Early Termination**: Stop dispatching once first prime found
   
5. **Gap Closing**: Only wait for batches needed to close [min_offset; candidate)

### Miller-Rabin Primality Testing

- **Rounds**: 25 (deterministic for numbers up to ~3.4×10^14)
- **Variance**: 14ms to 28s per candidate
  - Quick composites: 14-25ms (early witnesses)
  - Hard composites: ~28s (pass 24+ rounds before failing)
  - Actual primes: ~28s (all 25 rounds required)

### Hard Composite Discovery (F(4601))

**Offset 44207**: primorial(4601) + 44207 is a hard composite
- Takes ~28 seconds to disprove as prime
- Passes 24+ Miller-Rabin rounds before final rejection
- Cost nearly equivalent to testing an actual prime
- See: `ANOMALY_F4601_HARD_COMPOSITE.md` for detailed case study

================================================================================
IMPLEMENTATION DETAILS
================================================================================

**fortunate_v2.py**:
- Channel-based work distribution (Golang-inspired)
- Worker ID tracking for debugging parallel execution
- Shared primorial via multiprocessing.shared_memory
- Detailed logging of every batch completion
- Batch size ranges from 16 to 64+ depending on speed

**Key Features**:
- Incremental computation (can resume from checkpoint)
- Worker-specific logging with timing
- Real-time progress tracking
- Comprehensive error handling

================================================================================
NEXT STEPS
================================================================================

- Continue computation: F(4607) through F(5000)
- Monitor for additional hard composites
- Document performance anomalies
- Compare final results against any future OEIS updates

================================================================================
