use rug::Integer;
use std::time::Instant;

use crate::{FortunateCalculator, FortunateError, Metrics, MillerRabin, PrimalityTest, Result};

/// Basic Fortunate calculator using sequential candidate testing
///
/// This is the simplest implementation that tests candidates sequentially
/// without any optimizations. It serves as the performance baseline for
/// comparing with wheel factorization and parallel implementations.
pub struct PrimeBasedCalculator {
    primes: Vec<u32>,
    tester: MillerRabin,
    max_candidate: u32,
}

impl PrimeBasedCalculator {
    pub fn new(primes: Vec<u32>) -> Self {
        PrimeBasedCalculator {
            primes,
            tester: MillerRabin::with_default_rounds(),
            max_candidate: 10000,
        }
    }

    pub fn with_tester(primes: Vec<u32>, tester: MillerRabin) -> Self {
        PrimeBasedCalculator {
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

impl FortunateCalculator for PrimeBasedCalculator {
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

        for m in 2..=self.max_candidate {
            let candidate = p_n_sharp.clone() + Integer::from(m);
            if self.tester.is_prime(&candidate) {
                return Ok(m);
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

        let mut primality_test_count = 0;
        let mut primality_tests_passed = 0;
        let mut candidate_found = 0u32;

        for m in 2..=self.max_candidate {
            let candidate = p_n_sharp.clone() + Integer::from(m);
            primality_test_count += 1;

            if self.tester.is_prime(&candidate) {
                primality_tests_passed += 1;
                candidate_found = m;
                break;
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
                primality_test_count,
                primality_tests_passed,
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
    fn test_primorial() {
        let primes = vec![2, 3, 5, 7, 11, 13];
        let calc = PrimeBasedCalculator::new(primes);

        // 0# = 1
        assert_eq!(calc.primorial(0).unwrap(), Integer::from(1));

        // 1# = 2
        assert_eq!(calc.primorial(1).unwrap(), Integer::from(2));

        // 3# = 2 * 3 * 5 = 30
        assert_eq!(calc.primorial(3).unwrap(), Integer::from(30));

        // 6# = 2 * 3 * 5 * 7 * 11 * 13 = 30030
        assert_eq!(calc.primorial(6).unwrap(), Integer::from(30030));
    }

    #[test]
    fn test_primorial_single_prime() {
        let primes = vec![2, 3, 5];
        let calc = PrimeBasedCalculator::new(primes);
        assert_eq!(calc.primorial(1).unwrap(), Integer::from(2));
    }

    #[test]
    fn test_primorial_growth() {
        let primes = vec![2, 3, 5, 7, 11];
        let calc = PrimeBasedCalculator::new(primes);

        let p2 = calc.primorial(2).unwrap();
        let p3 = calc.primorial(3).unwrap();

        // 2# = 6, 3# = 30, so 3# / 2# should equal 5
        let div_result = Integer::from(&p3 / &p2);
        assert_eq!(div_result, Integer::from(5));
    }

    #[test]
    fn test_fortunate_numbers_early_values() {
        let primes = vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97,
        ];

        let mut calc = PrimeBasedCalculator::new(primes);
        calc.set_max_candidate(10000);

        // OEIS A005235: Fortunate numbers
        assert_eq!(calc.fortunate_number(1).unwrap(), 3);
        assert_eq!(calc.fortunate_number(2).unwrap(), 5);
        assert_eq!(calc.fortunate_number(3).unwrap(), 7);
        assert_eq!(calc.fortunate_number(4).unwrap(), 13);
        assert_eq!(calc.fortunate_number(5).unwrap(), 23);
    }

    #[test]
    fn test_fortunate_prime_detection() {
        let primes = vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97,
        ];

        let mut calc = PrimeBasedCalculator::new(primes);
        calc.set_max_candidate(10000);

        // All fortunate numbers up to n=25 should be prime
        for n in 1..=25 {
            let f = calc.fortunate_number(n).unwrap();
            let candidate = calc.primorial(n).unwrap() + Integer::from(f);
            assert!(
                calc.tester.is_prime(&candidate),
                "Fortunate number F({}) = {} should be prime",
                n,
                f
            );
        }
    }
}
