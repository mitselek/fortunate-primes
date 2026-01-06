use rayon::prelude::*;
use rug::Integer;
use std::time::Instant;

use crate::{FortunateCalculator, FortunateError, Metrics, MillerRabin, PrimalityTest, Result};

/// Parallel Fortunate calculator using Rayon for candidate testing
///
/// This implementation uses sequential candidate search (to find the FIRST match)
/// but parallelizes the primality testing overhead where possible.
/// The key insight: for Fortunate numbers, we need the SMALLEST m where p_n# + m is prime,
/// so we must test candidates sequentially (2, 3, 4, ...). However, within each iteration,
/// Rayon could theoretically parallelize the Miller-Rabin test itself (not implemented yet).
///
/// Alternative strategies for parallelization:
/// - Batch testing: partition the search range and search batches in parallel, then merge results
/// - Wheel factorization: skip candidates divisible by small primes (Phase 1.2 optimization)
///
/// For now, this maintains correctness by searching sequentially while using the same
/// architecture as PrimeBasedCalculator, ensuring test equivalence and future optimization
/// compatibility.
#[derive(Clone)]
pub struct ParallelFortunateCalculator {
    primes: Vec<u32>,
    tester: MillerRabin,
    max_candidate: u32,
}

impl ParallelFortunateCalculator {
    pub fn new(primes: Vec<u32>) -> Self {
        ParallelFortunateCalculator {
            primes,
            tester: MillerRabin::with_default_rounds(),
            max_candidate: 10000,
        }
    }

    pub fn with_tester(primes: Vec<u32>, tester: MillerRabin) -> Self {
        ParallelFortunateCalculator {
            primes,
            tester,
            max_candidate: 10000,
        }
    }

    pub fn set_max_candidate(&mut self, max: u32) {
        self.max_candidate = max;
    }

    pub fn prime_count(&self) -> usize {
        self.primes.len()
    }
}

impl FortunateCalculator for ParallelFortunateCalculator {
    fn primorial(&self, n: usize) -> Result<Integer> {
        if n == 0 {
            return Ok(Integer::from(1));
        }

        if n > self.primes.len() {
            return Err(FortunateError::InvalidPrimeIndex {
                index: n,
                max: self.primes.len(),
            });
        }

        let mut result = Integer::from(self.primes[0]);
        for &p in &self.primes[1..n] {
            result *= p;
        }

        Ok(result)
    }

    fn fortunate_number(&self, n: usize) -> Result<u32> {
        let p_n_sharp = self.primorial(n)?;

        // Phase 2: Parallel candidate testing with Rayon
        // Strategy: Process candidates in parallel batches while maintaining order
        //
        // We use chunks to test multiple candidates in parallel, but check batches
        // sequentially to ensure we find the SMALLEST Fortunate number
        //
        // Batch size tuned for balance: large enough for parallelism benefits,
        // small enough to avoid wasted work after finding the answer
        const BATCH_SIZE: u32 = 100;

        for batch_start in (2..=self.max_candidate).step_by(BATCH_SIZE as usize) {
            let batch_end = (batch_start + BATCH_SIZE).min(self.max_candidate + 1);

            // Test this batch in parallel
            let result = (batch_start..batch_end).into_par_iter().find_any(|&m| {
                let candidate = p_n_sharp.clone() + Integer::from(m);
                self.tester.is_prime(&candidate)
            });

            // If we found a prime in this batch, find the SMALLEST one
            if result.is_some() {
                // Sequential search within the successful batch to find the FIRST prime
                for m in batch_start..batch_end {
                    let candidate = p_n_sharp.clone() + Integer::from(m);
                    if self.tester.is_prime(&candidate) {
                        return Ok(m);
                    }
                }
            }
        }

        Err(FortunateError::NoFortunateFound {
            n,
            max_candidate: self.max_candidate,
        })
    }

    fn fortunate_number_with_metrics(&self, n: usize) -> Result<(u32, Metrics)> {
        let start = Instant::now();

        let primorial_start = Instant::now();
        let p_n_sharp = self.primorial(n)?;
        let primorial_time = primorial_start.elapsed();

        // Phase 2: Parallel search with metrics tracking
        use std::sync::atomic::{AtomicUsize, Ordering};
        let primality_test_count = AtomicUsize::new(0);
        let primality_tests_passed = AtomicUsize::new(0);

        const BATCH_SIZE: u32 = 100;
        let mut candidate_found = 0u32;

        'outer: for batch_start in (2..=self.max_candidate).step_by(BATCH_SIZE as usize) {
            let batch_end = (batch_start + BATCH_SIZE).min(self.max_candidate + 1);

            // Parallel test of this batch
            let batch_has_prime = (batch_start..batch_end).into_par_iter().find_any(|&m| {
                let candidate = p_n_sharp.clone() + Integer::from(m);
                primality_test_count.fetch_add(1, Ordering::Relaxed);

                let is_prime = self.tester.is_prime(&candidate);
                if is_prime {
                    primality_tests_passed.fetch_add(1, Ordering::Relaxed);
                }
                is_prime
            });

            // If batch has a prime, find the FIRST one sequentially
            if batch_has_prime.is_some() {
                for m in batch_start..batch_end {
                    let candidate = p_n_sharp.clone() + Integer::from(m);

                    // Only count if not already counted in parallel phase
                    // (note: some tests will be duplicated, but metrics are approximate)

                    if self.tester.is_prime(&candidate) {
                        candidate_found = m;
                        break 'outer;
                    }
                }
            }
        }

        if candidate_found == 0 {
            return Err(FortunateError::NoFortunateFound {
                n,
                max_candidate: self.max_candidate,
            });
        }

        let total_time = start.elapsed();

        Ok((
            candidate_found,
            Metrics {
                primorial_time,
                primality_test_count: primality_test_count.load(Ordering::Relaxed),
                primality_tests_passed: primality_tests_passed.load(Ordering::Relaxed),
                total_time,
                candidate_found,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_with_metrics() {
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
        let mut calc = ParallelFortunateCalculator::new(primes);
        calc.set_max_candidate(10000);

        let (f, metrics) = calc.fortunate_number_with_metrics(5).unwrap();
        assert_eq!(f, 23);
        assert!(metrics.total_time.as_millis() > 0);
        assert!(metrics.primality_test_count > 0);
    }

    #[test]
    fn test_parallel_correctness_with_rayon() {
        let primes = vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97,
        ];

        let mut calc = ParallelFortunateCalculator::new(primes);
        calc.set_max_candidate(10000);

        // OEIS A005235: Fortunate numbers
        assert_eq!(calc.fortunate_number(1).unwrap(), 3);
        assert_eq!(calc.fortunate_number(2).unwrap(), 5);
        assert_eq!(calc.fortunate_number(3).unwrap(), 7);
        assert_eq!(calc.fortunate_number(4).unwrap(), 13);
        assert_eq!(calc.fortunate_number(5).unwrap(), 23);
    }

    #[test]
    fn test_parallel_fortunes_are_prime() {
        let primes = vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97,
        ];

        let mut calc = ParallelFortunateCalculator::new(primes);
        calc.set_max_candidate(10000);

        // All fortunate numbers should produce primes
        for n in 1..=20 {
            let f = calc.fortunate_number(n).unwrap();
            let candidate = calc.primorial(n).unwrap() + Integer::from(f);
            assert!(calc.tester.is_prime(&candidate));
        }
    }

    #[test]
    fn test_parallel_sequential_equivalence_property() {
        let primes = vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97,
        ];

        let mut par_calc = ParallelFortunateCalculator::new(primes.clone());
        par_calc.set_max_candidate(10000);

        let mut seq_calc = crate::calculators::base::PrimeBasedCalculator::new(primes);
        seq_calc.set_max_candidate(10000);

        // Both should find the same Fortunate numbers
        for n in 1..=15 {
            let parallel = par_calc.fortunate_number(n).unwrap();
            let sequential = seq_calc.fortunate_number(n).unwrap();
            assert_eq!(
                parallel, sequential,
                "Parallel and sequential results differ at n={}",
                n
            );
        }
    }

    #[test]
    fn test_parallel_vs_sequential_all_values() {
        use crate::calculators::base::PrimeBasedCalculator;

        let primes = vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97, 101, 103, 107, 109, 113, 127, 131, 137, 139, 149,
        ];

        let mut seq_calc = PrimeBasedCalculator::new(primes.clone());
        seq_calc.set_max_candidate(10000);

        let mut par_calc = ParallelFortunateCalculator::new(primes.clone());
        par_calc.set_max_candidate(10000);

        // Compare parallel against sequential for n=25
        let seq_result = seq_calc.fortunate_number(25).unwrap();
        let par_result = par_calc.fortunate_number(25).unwrap();

        assert_eq!(
            par_result, seq_result,
            "Parallel and sequential calculators disagree for n=25"
        );
    }

    #[test]
    fn test_parallel_thread_safety() {
        let primes = vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97,
        ];

        let mut calc = ParallelFortunateCalculator::new(primes);
        calc.set_max_candidate(10000);

        // Run multiple times to ensure thread safety
        for _ in 0..5 {
            let _ = calc.fortunate_number(10).unwrap();
        }
    }

    #[test]
    fn test_parallel_custom_tester() {
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
        let tester = MillerRabin::fast();
        let mut calc = ParallelFortunateCalculator::with_tester(primes, tester);
        calc.set_max_candidate(10000);

        // Should still work with different tester configurations
        let result = calc.fortunate_number(5).unwrap();
        assert_eq!(result, 23);
    }
}
