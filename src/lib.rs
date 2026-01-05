use rug::Integer;
use std::fmt;
use std::time::{Duration, Instant};

pub mod primes;

/// Performance metrics for Fortunate number calculation
#[derive(Debug, Clone)]
pub struct Metrics {
    pub primorial_time: Duration,
    pub primality_test_count: usize,
    pub primality_tests_passed: usize,
    pub total_time: Duration,
    pub candidate_found: u32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FortunateError {
    InvalidPrimeIndex { index: usize, max: usize },
    NoFortunateFound { n: usize, max_candidate: u32 },
    InvalidPrimorial { reason: String },
}

impl fmt::Display for FortunateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FortunateError::InvalidPrimeIndex { index, max } => {
                write!(f, "Prime index {} out of range (max: {})", index, max)
            }
            FortunateError::NoFortunateFound { n, max_candidate } => {
                write!(
                    f,
                    "No Fortunate number found for n={} within range [2, {}]",
                    n, max_candidate
                )
            }
            FortunateError::InvalidPrimorial { reason } => {
                write!(f, "Invalid primorial: {}", reason)
            }
        }
    }
}

impl std::error::Error for FortunateError {}

pub type Result<T> = std::result::Result<T, FortunateError>;

/// Trait for primality testing
pub trait PrimalityTest {
    fn is_prime(&self, n: &Integer) -> bool;
    fn name(&self) -> &'static str;
}

/// Trait for calculating Fortunate numbers
pub trait FortunateCalculator {
    fn primorial(&self, n: usize) -> Result<Integer>;
    fn fortunate_number(&self, n: usize) -> Result<u32>;
    fn fortunate_number_with_metrics(&self, n: usize) -> Result<(u32, Metrics)>;
}

/// Miller-Rabin primality tester
#[derive(Clone)]
pub struct MillerRabin {
    rounds: usize,
}

impl MillerRabin {
    pub fn new(rounds: usize) -> Self {
        MillerRabin { rounds }
    }

    pub fn with_default_rounds() -> Self {
        MillerRabin::new(40)
    }

    pub fn fast() -> Self {
        MillerRabin::new(20)
    }

    pub fn thorough() -> Self {
        MillerRabin::new(64)
    }
}

impl PrimalityTest for MillerRabin {
    fn is_prime(&self, n: &Integer) -> bool {
        if n <= &Integer::from(1) {
            return false;
        }
        if n == &Integer::from(2) || n == &Integer::from(3) {
            return true;
        }
        if n.is_even() {
            return false;
        }

        // Write n-1 as 2^r * d
        let n_minus_1 = n.clone() - 1i32;
        let mut d: Integer = n_minus_1.clone();
        let mut r = 0;
        while d.is_even() {
            d /= 2;
            r += 1;
        }

        // Deterministic witnesses for numbers up to 2^64
        let witnesses = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

        for &w in witnesses.iter().take(self.rounds.min(witnesses.len())) {
            let a = Integer::from(w);
            if a >= *n {
                continue;
            }

            let mut x = a.pow_mod(&d, n).unwrap();
            let one = Integer::from(1);

            if x == one || x == n_minus_1 {
                continue;
            }

            let mut composite = true;
            for _ in 0..r - 1 {
                let x_sq = x.clone() * x.clone();
                x = x_sq % n;
                if x == n_minus_1 {
                    composite = false;
                    break;
                }
            }

            if composite {
                return false;
            }
        }

        true
    }

    fn name(&self) -> &'static str {
        "Miller-Rabin"
    }
}

/// Prime-based Fortunate calculator
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
    fn test_miller_rabin_small_primes() {
        let tester = MillerRabin::with_default_rounds();
        assert!(tester.is_prime(&Integer::from(2)));
        assert!(tester.is_prime(&Integer::from(3)));
        assert!(tester.is_prime(&Integer::from(5)));
        assert!(tester.is_prime(&Integer::from(7)));
        assert!(tester.is_prime(&Integer::from(11)));
    }

    #[test]
    fn test_miller_rabin_composites() {
        let tester = MillerRabin::with_default_rounds();
        assert!(!tester.is_prime(&Integer::from(4)));
        assert!(!tester.is_prime(&Integer::from(6)));
        assert!(!tester.is_prime(&Integer::from(9)));
        assert!(!tester.is_prime(&Integer::from(15)));
    }

    #[test]
    fn test_primorial() {
        let primes = vec![2, 3, 5, 7, 11];
        let calc = PrimeBasedCalculator::new(primes);

        assert_eq!(calc.primorial(1).unwrap(), Integer::from(2));
        assert_eq!(calc.primorial(2).unwrap(), Integer::from(6)); // 2*3
        assert_eq!(calc.primorial(3).unwrap(), Integer::from(30)); // 2*3*5
        assert_eq!(calc.primorial(5).unwrap(), Integer::from(2310)); // 2*3*5*7*11
    }

    #[test]
    fn test_fortunate_numbers() {
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let calc = PrimeBasedCalculator::new(primes);

        // Known Fortunate numbers from OEIS A005235
        assert_eq!(calc.fortunate_number(1).unwrap(), 3);
        assert_eq!(calc.fortunate_number(2).unwrap(), 5);
        assert_eq!(calc.fortunate_number(3).unwrap(), 7);
        assert_eq!(calc.fortunate_number(4).unwrap(), 13);
        assert_eq!(calc.fortunate_number(5).unwrap(), 23);
    }

    #[test]
    fn test_error_handling() {
        let primes = vec![2, 3, 5];
        let calc = PrimeBasedCalculator::new(primes);

        assert!(calc.primorial(10).is_err());
        assert_eq!(
            calc.primorial(10).unwrap_err(),
            FortunateError::InvalidPrimeIndex { index: 10, max: 3 }
        );
    }
}
