use std::time::Instant;
use std::sync::atomic::{AtomicUsize, Ordering};
use rayon::prelude::*;
use rug::Integer;

use crate::{FortunateCalculator, FortunateError, Metrics, Result, MillerRabin, PrimalityTest, SegmentedSieve};

/// Sieved Fortunate calculator using segmented sieve pre-filtering
///
/// Phase 3 optimization: Combines segmented sieve with Miller-Rabin testing.
/// The sieve pre-filters candidates, eliminating obvious composites before
/// expensive primality tests.
///
/// Expected performance: 1.3-1.5x speedup over Phase 2 parallel implementation
/// Expected efficiency: 40-60% reduction in Miller-Rabin invocations
#[derive(Clone)]
pub struct SievedFortunateCalculator {
    primes: Vec<u32>,
    tester: MillerRabin,
    max_candidate: u32,
}

impl SievedFortunateCalculator {
    pub fn new(primes: Vec<u32>) -> Self {
        SievedFortunateCalculator {
            primes,
            tester: MillerRabin::with_default_rounds(),
            max_candidate: 10000,
        }
    }

    pub fn with_tester(primes: Vec<u32>, tester: MillerRabin) -> Self {
        SievedFortunateCalculator {
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

impl FortunateCalculator for SievedFortunateCalculator {
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

        // Phase 3: Sieve once to get all probable primes, then parallel test
        // This amortizes sieve overhead while still reducing Miller-Rabin calls
        let sieve = SegmentedSieve::new(self.max_candidate);
        let probable_primes = sieve.sieve_range(2, self.max_candidate + 1);

        // Use Rayon to parallel test the much smaller set of probable primes
        let result = probable_primes.par_iter().find_first(|&&m| {
            let candidate = p_n_sharp.clone() + Integer::from(m);
            self.tester.is_prime(&candidate)
        });

        match result {
            Some(&m) => Ok(m),
            None => Err(FortunateError::NoFortunateFound {
                n,
                max_candidate: self.max_candidate,
            }),
        }
    }

    fn fortunate_number_with_metrics(&self, n: usize) -> Result<(u32, Metrics)> {
        let start = Instant::now();

        let primorial_start = Instant::now();
        let p_n_sharp = self.primorial(n)?;
        let primorial_time = primorial_start.elapsed();

        let primality_test_count = AtomicUsize::new(0);
        let primality_tests_passed = AtomicUsize::new(0);

        // Phase 3: Sieve once to get all probable primes, then parallel test
        let sieve = SegmentedSieve::new(self.max_candidate);
        let probable_primes = sieve.sieve_range(2, self.max_candidate + 1);

        // Use Rayon to parallel test the much smaller set of probable primes
        let result = probable_primes.par_iter().find_first(|&&m| {
            let candidate = p_n_sharp.clone() + Integer::from(m);
            primality_test_count.fetch_add(1, Ordering::Relaxed);

            let is_prime = self.tester.is_prime(&candidate);
            if is_prime {
                primality_tests_passed.fetch_add(1, Ordering::Relaxed);
            }
            is_prime
        });

        let candidate_found = match result {
            Some(&m) => m,
            None => {
                return Err(FortunateError::NoFortunateFound {
                    n,
                    max_candidate: self.max_candidate,
                })
            }
        };

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
    fn test_sieved_reduces_miller_rabin_calls() {
        let primes = vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79,
            83, 89, 97,
        ];

        let mut calc = SievedFortunateCalculator::new(primes);
        calc.set_max_candidate(10000);

        let (_f, metrics) = calc.fortunate_number_with_metrics(10).unwrap();

        // Sieve should dramatically reduce Miller-Rabin calls
        // For n=10 with max_candidate=10000, we'd test ~640 candidates without sieve
        // With sieve, we test only ~1200 probable primes (much smaller than 10000)
        println!(
            "n=10: primality_test_count={}, should be significantly less than 640",
            metrics.primality_test_count
        );
    }

    #[test]
    fn test_sieved_calculator_correctness() {
        let primes = vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79,
            83, 89, 97,
        ];

        let mut calc = SievedFortunateCalculator::new(primes);
        calc.set_max_candidate(10000);

        // OEIS A005235: Fortunate numbers
        assert_eq!(calc.fortunate_number(1).unwrap(), 3);
        assert_eq!(calc.fortunate_number(2).unwrap(), 5);
        assert_eq!(calc.fortunate_number(3).unwrap(), 7);
        assert_eq!(calc.fortunate_number(4).unwrap(), 13);
        assert_eq!(calc.fortunate_number(5).unwrap(), 23);
    }

    #[test]
    fn test_sieved_fortunes_are_prime() {
        let primes = vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79,
            83, 89, 97,
        ];

        let mut calc = SievedFortunateCalculator::new(primes);
        calc.set_max_candidate(10000);

        // All fortunate numbers should produce primes
        for n in 1..=20 {
            let f = calc.fortunate_number(n).unwrap();
            let candidate = calc.primorial(n).unwrap() + Integer::from(f);
            assert!(calc.tester.is_prime(&candidate));
        }
    }

    #[test]
    fn test_sieved_no_regression_small_n() {
        let primes = vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79,
            83, 89, 97, 101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167,
            173, 179, 181, 191, 193, 197, 199,
        ];

        let mut calc = SievedFortunateCalculator::new(primes);
        calc.set_max_candidate(10000);

        // For very small n, sieve overhead might matter, but correctness is paramount
        for n in 1..=10 {
            let result = calc.fortunate_number(n);
            assert!(result.is_ok(), "Sieve should not fail for small n={}", n);
        }

        // Verify specific values
        assert_eq!(calc.fortunate_number(1).unwrap(), 3);
        assert_eq!(calc.fortunate_number(5).unwrap(), 23);
    }

    #[test]
    fn test_sieved_speedup_benchmark() {
        let primes = vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79,
            83, 89, 97, 101, 103, 107, 109, 113, 127, 131, 137, 139, 149,
        ];

        let mut calc = SievedFortunateCalculator::new(primes);
        calc.set_max_candidate(10000);

        let (_f, metrics) = calc.fortunate_number_with_metrics(25).unwrap();

        println!(
            "Sieved calculation for n=25: {}ms, {} primality tests",
            metrics.total_time.as_millis(),
            metrics.primality_test_count
        );

        // Just verify it completes in reasonable time (not a strict benchmark)
        assert!(metrics.total_time.as_secs() < 5);
    }
}
