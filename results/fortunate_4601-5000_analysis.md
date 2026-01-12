# Fortunate Numbers: F(4601) - F(5000) Computation Project

## Overview

Systematic computation of Fortunate numbers beyond the OEIS A005235 dataset (which covers n ≤ 4600).

**Target Range**: F(4601) through F(5000)  
**System**: AMD Ryzen 7 2700 (16 logical cores)  
**Implementation**: Python 3.12.3 + gmpy2 2.1.5  
**Algorithm**: Parallel gap-closing with Firoozbakht optimization (start at p\_{n+1})  
**Status**: F(4601-4606) complete

## Results (F(4601-4606))

Raw data: `fortunate_4601-5000_data.csv`

| n    | F(n)  | Time    | Workers | Primorial Digits |
| ---- | ----- | ------- | ------- | ---------------- |
| 4601 | 56611 | 36m 46s | 15      | 19,113           |
| 4602 | 62207 | 52m 52s | 15      | 19,117           |
| 4603 | 54083 | 29m 56s | 15      | 19,122           |
| 4604 | 83773 | 1h 49m  | 16      | 19,127           |
| 4605 | 69143 | 1h 9m   | 16      | 19,131           |
| 4606 | 97813 | 2h 24m  | 16      | 19,136           |

## Performance Notes

**Rust Baseline Comparison** (where applicable):

- F(4601): 8.2× faster than Rust (Rust: 5.0h)
- F(4602): 6.7× faster than Rust (Rust: 5h 52m)
- F(4603-4606): Beyond OEIS validation range, no baseline

**Pattern Observation**: Computation time correlates more strongly with answer magnitude than with index n. For example, F(4603)=54083 completes in 29m 56s, while F(4604)=83773 (larger answer) requires 1h 49m despite only one index increment.

## Algorithm

### Gap-Closing Search

1. **Lower Bound**: Start search at p\_{n+1} (Firoozbakht optimization)
   - Offsets 2..p_n are all composite (share factors with primorial(n))
2. **Parallel Testing**: 15-16 worker processes test candidates in batches
3. **Batch Strategy**: Adaptive sizing based on primality test latency
4. **Early Stop**: Terminate once first prime found
5. **Efficiency**: Only wait for batches needed to close remaining range

### Primality Testing

Miller-Rabin deterministic variant: 25 rounds (covers numbers up to ~3.4×10^14)

**Observed Timing**:

- Fast composites: 14-25ms (early witness found)
- Slow composites: ~28s per candidate
- Actual primes: ~28s per candidate

### Hard Composite: primorial(4601) + 44207

This composite requires ~28 seconds to reject via Miller-Rabin. Case study documented in `ANOMALY_F4601_HARD_COMPOSITE.md` confirms that 24 rounds would not produce a false positive.

## OEIS Dataset Status

**OEIS A005235** (Fortunate numbers):

- Validated coverage: F(1) through F(4600)
- F(4601) and beyond: computed but unverified against external sources

**References**:

- OEIS A005235: https://oeis.org/A005235
- OEIS A000040 (primes): https://oeis.org/A000040

## Continuation

Next: F(4607) through F(5000)  
Expected: ~394 additional results before project completion

## Trivia: Primorial Structure and the Guaranteed Gap

An interesting structural property of primorial numbers:

**Guaranteed No-Prime Zone**: Every offset from 2 through p_n added to primorial(n) yields a composite number. This is because primorial(n) = 2 × 3 × 5 × ... × p_n, so any offset k ∈ [2, p_n] will share a prime factor with primorial(n), making primorial(n) ± k composite. This guaranteed gap is why the Firoozbakht optimization begins the search at p_{n+1}—it avoids testing the mathematically impossible offsets. The first candidate Fortunate number must be at least p_{n+1} by necessity.

**Primorial Twin Primes (OEIS A088256)**: While primorial(n) ± 1 are sometimes both prime, this is extraordinarily rare. A088256 lists primorial numbers where both k-1 and k+1 are prime. Only three are known: primorial(2)=6 (5 and 7), primorial(3)=30 (29 and 31), and primorial(5)=2310 (2309 and 2311). After checking the first 3000 primorials (over 230,000 primorial numbers), no additional terms were found. The sequence is conjectured to be finite, with any further term likely exceeding 10^1,400,000.

**Contrast with Fortunate Numbers**: Unlike primorial twin primes (vanishingly rare), Fortunate numbers are guaranteed to exist for every n. Bertrand's postulate ensures a prime exists in every interval (n, 2n), so there is always a Fortunate number F(n) between primorial(n) and 2·primorial(n). This fundamental difference shows why Fortunate number computation is tractable while primorial twin prime searches hit rapidly diminishing returns.
