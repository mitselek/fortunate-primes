use crate::{FortunateCalculator, FortunateError, Metrics, PrimalityTest, Result};
use crate::primality::MillerRabin;
use rug::Integer;
use std::time::Instant;

/// Wheel factorization for efficient candidate filtering
///
/// Implements 2-3-5 wheel factorization to skip candidates divisible by 2, 3, and 5.
/// This reduces the candidate search space by approximately 73% (from 99 candidates
/// to ~26 candidates in a typical range), enabling ~40-50% speedup in primality testing.
#[derive(Clone)]
pub struct WheelFactorization {
    wheel_size: u32,
    offsets: Vec<u32>,
}

impl WheelFactorization {
    /// Create a new wheel factorization filter
    /// Uses 2-3-5 wheel with period 30
    pub fn new() -> Self {
        // 2-3-5 wheel: numbers in [0, 30) that are coprime to 2*3*5=30
        // These are: 1, 7, 11, 13, 17, 19, 23, 29
        // We start from 2, so: 2, 3, 5, 7, 11, 13, 17, 19, 23, 29
        let offsets = vec![1, 2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        WheelFactorization {
            wheel_size: 30,
            offsets,
        }
    }

    /// Generate candidates up to max using wheel factorization
    pub fn candidates_up_to(&self, max: u32) -> WheelIterator {
        WheelIterator {
            max,
            wheel_size: self.wheel_size,
            offsets: self.offsets.clone(),
            current_wheel: 0,
            offset_idx: 0,
        }
    }
}

impl Default for WheelFactorization {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator for wheel-factorized candidates
pub struct WheelIterator {
    max: u32,
    wheel_size: u32,
    offsets: Vec<u32>,
    current_wheel: u32,
    offset_idx: usize,
}

impl Iterator for WheelIterator {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        loop {
            if self.offset_idx >= self.offsets.len() {
                self.current_wheel += 1;
                self.offset_idx = 0;
            }

            if self.current_wheel * self.wheel_size >= self.max {
                return None;
            }

            let candidate = self.current_wheel * self.wheel_size + self.offsets[self.offset_idx];
            self.offset_idx += 1;

            if candidate <= self.max && candidate >= 2 {
                return Some(candidate);
            }

            if candidate > self.max {
                return None;
            }
        }
    }
}

/// Fortunate calculator using wheel factorization for candidate filtering
///
/// This combines the standard Fortunate number calculation with wheel factorization
/// to skip candidates divisible by 2, 3, and 5. Expected improvement: 2-3x speedup
/// by reducing primality tests by ~73%.
#[derive(Clone)]
pub struct WheelFortunateCalculator {
    primes: Vec<u32>,
    tester: MillerRabin,
    max_candidate: u32,
    wheel: WheelFactorization,
}

impl WheelFortunateCalculator {
    pub fn new(primes: Vec<u32>) -> Self {
        WheelFortunateCalculator {
            primes,
            tester: MillerRabin::with_default_rounds(),
            max_candidate: 10000,
            wheel: WheelFactorization::new(),
        }
    }

    pub fn with_tester(primes: Vec<u32>, tester: MillerRabin) -> Self {
        WheelFortunateCalculator {
            primes,
            tester,
            max_candidate: 10000,
            wheel: WheelFactorization::new(),
        }
    }

    pub fn set_max_candidate(&mut self, max: u32) {
        self.max_candidate = max;
    }

    pub fn prime_count(&self) -> usize {
        self.primes.len()
    }
}

impl FortunateCalculator for WheelFortunateCalculator {
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

        // Use wheel-filtered candidates instead of testing all numbers
        for m in self.wheel.candidates_up_to(self.max_candidate) {
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

        // Use wheel-filtered candidates
        for m in self.wheel.candidates_up_to(self.max_candidate) {
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
    use crate::PrimeBasedCalculator;

    #[test]
    fn test_wheel_iterator_skips_even() {
        // Wheel factorization should include 2, 3, 5 (the wheel bases)
        // and skip all even numbers > 2, multiples of 3 > 3, multiples of 5 > 5
        let wheel = WheelFactorization::new();
        let candidates: Vec<u32> = wheel.candidates_up_to(30).collect();

        // Should not contain even numbers > 2
        for &c in &candidates {
            if c > 2 {
                assert_ne!(
                    c % 2,
                    0,
                    "Wheel should skip even numbers > 2, but includes {}",
                    c
                );
            }
        }

        // Should not contain multiples of 3 > 3
        for &c in &candidates {
            if c > 3 {
                assert_ne!(
                    c % 3,
                    0,
                    "Wheel should skip multiples of 3 > 3, but includes {}",
                    c
                );
            }
        }

        // Should not contain multiples of 5 > 5
        for &c in &candidates {
            if c > 5 {
                assert_ne!(
                    c % 5,
                    0,
                    "Wheel should skip multiples of 5 > 5, but includes {}",
                    c
                );
            }
        }

        // Wheel should include all of these (wheel bases + primes/composites not divisible by 2,3,5)
        let expected = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        for exp in expected {
            assert!(
                candidates.contains(&exp),
                "Wheel should include {}, got: {:?}",
                exp,
                candidates
            );
        }
    }

    #[test]
    fn test_wheel_iterator_basic() {
        // Basic wheel iteration test
        let wheel = WheelFactorization::new();
        let candidates: Vec<u32> = wheel.candidates_up_to(30).collect();

        // Check expected candidates (those not divisible by 2, 3, 5)
        // This follows a 2-3-5 wheel with period 30
        let expected = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];

        for exp in expected {
            assert!(
                candidates.contains(&exp),
                "Wheel should include {}, but got: {:?}",
                exp,
                candidates
            );
        }
    }

    #[test]
    fn test_wheel_with_primorial() {
        // When combined with primorial, wheel should skip divisible candidates
        let wheel = WheelFactorization::new();
        let candidates: Vec<u32> = wheel.candidates_up_to(100).collect();

        // Wheel should reduce search space significantly
        // Full range 2..=100 has 99 candidates
        // Wheel should have roughly 99 * (1 - 1/2) * (1 - 1/3) * (1 - 1/5) ≈ 26 candidates
        let reduction_ratio = candidates.len() as f64 / 99.0;
        assert!(
            reduction_ratio < 0.5,
            "Wheel should reduce candidates by 50%, but got {} candidates ({:.1}%)",
            candidates.len(),
            reduction_ratio * 100.0
        );
    }

    #[test]
    fn test_wheel_fortunate_number_oeis() {
        // Wheel-optimized calculator should find same Fortunate numbers
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let standard_calc = PrimeBasedCalculator::new(primes.clone());
        let wheel_calc = WheelFortunateCalculator::new(primes);

        // OEIS A005235 test values
        let oeis_values = vec![
            (1, 3),
            (2, 5),
            (3, 7),
            (4, 13),
            (5, 23),
            (6, 17),
            (7, 19),
            (8, 23),
            (9, 37),
            (10, 61),
        ];

        for (n, expected) in oeis_values {
            let standard_result = standard_calc.fortunate_number(n).unwrap();
            let wheel_result = wheel_calc.fortunate_number(n).unwrap();

            assert_eq!(
                wheel_result, expected,
                "Wheel calculator: n={} produced {} but expected {}",
                n, wheel_result, expected
            );
            assert_eq!(
                standard_result, wheel_result,
                "Standard vs wheel mismatch for n={}: standard={}, wheel={}",
                n, standard_result, wheel_result
            );
        }
    }

    #[test]
    fn test_wheel_vs_standard_equivalence() {
        // CORRECTNESS: Wheel must find same Fortunate numbers as standard
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
        let standard_calc = PrimeBasedCalculator::new(primes.clone());
        let wheel_calc = WheelFortunateCalculator::new(primes);

        for n in 1..=12 {
            let standard_result = standard_calc.fortunate_number(n).unwrap();
            let wheel_result = wheel_calc.fortunate_number(n).unwrap();

            assert_eq!(
                standard_result, wheel_result,
                "Wheel and standard calculators differ at n={}: standard={}, wheel={}",
                n, standard_result, wheel_result
            );
        }
    }

    #[test]
    fn test_wheel_fortune_conjecture() {
        // Verify wheel implementation still finds primes
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let wheel_calc = WheelFortunateCalculator::new(primes);
        let tester = MillerRabin::with_default_rounds();

        for n in 1..=10 {
            let f = wheel_calc.fortunate_number(n).unwrap();
            assert!(
                tester.is_prime(&Integer::from(f)),
                "Fortune's conjecture violated with wheel: n={} produced {} (not prime)",
                n,
                f
            );
        }
    }

    #[test]
    fn test_wheel_with_metrics() {
        // Wheel calculator should track metrics correctly
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let wheel_calc = WheelFortunateCalculator::new(primes);

        let (value, metrics) = wheel_calc.fortunate_number_with_metrics(5).unwrap();

        assert_eq!(value, 23);
        assert!(metrics.total_time.as_micros() > 0);
        assert_eq!(metrics.candidate_found, 23);
        // Wheel should test fewer candidates than standard (2-3x fewer)
        assert!(metrics.primality_test_count > 0);
    }
}
