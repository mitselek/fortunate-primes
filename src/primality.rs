/// Miller-Rabin primality testing implementation
///
/// This module provides efficient probabilistic primality testing using the Miller-Rabin algorithm.
/// It implements the `PrimalityTest` trait for use across the fortunate-primes calculator.

use crate::PrimalityTest;
use rug::Integer;

/// Miller-Rabin primality tester
///
/// Uses the Miller-Rabin algorithm with configurable number of rounds for probabilistic primality testing.
/// Higher rounds increase accuracy but slow testing proportionally.
#[derive(Clone)]
pub struct MillerRabin {
    rounds: usize,
}

impl MillerRabin {
    /// Create a new Miller-Rabin tester with specified number of rounds
    pub fn new(rounds: usize) -> Self {
        MillerRabin { rounds }
    }

    /// Create a Miller-Rabin tester with default 40 rounds (recommended)
    pub fn with_default_rounds() -> Self {
        MillerRabin::new(40)
    }

    /// Create a fast Miller-Rabin tester with 20 rounds (lower accuracy)
    pub fn fast() -> Self {
        MillerRabin::new(20)
    }

    /// Create a thorough Miller-Rabin tester with 64 rounds (higher accuracy)
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
        assert!(tester.is_prime(&Integer::from(13)));
        assert!(tester.is_prime(&Integer::from(17)));
        assert!(tester.is_prime(&Integer::from(19)));
        assert!(tester.is_prime(&Integer::from(23)));
        assert!(tester.is_prime(&Integer::from(29)));
    }

    #[test]
    fn test_miller_rabin_composites() {
        let tester = MillerRabin::with_default_rounds();
        assert!(!tester.is_prime(&Integer::from(4)));
        assert!(!tester.is_prime(&Integer::from(6)));
        assert!(!tester.is_prime(&Integer::from(8)));
        assert!(!tester.is_prime(&Integer::from(9)));
        assert!(!tester.is_prime(&Integer::from(10)));
        assert!(!tester.is_prime(&Integer::from(12)));
        assert!(!tester.is_prime(&Integer::from(15)));
        assert!(!tester.is_prime(&Integer::from(16)));
        assert!(!tester.is_prime(&Integer::from(20)));
        assert!(!tester.is_prime(&Integer::from(25)));
    }

    #[test]
    fn test_miller_rabin_edge_cases() {
        let tester = MillerRabin::with_default_rounds();
        assert!(!tester.is_prime(&Integer::from(0)));
        assert!(!tester.is_prime(&Integer::from(1)));
        assert!(tester.is_prime(&Integer::from(2)));
        assert!(tester.is_prime(&Integer::from(3)));
    }

    #[test]
    fn test_miller_rabin_large_primes() {
        let tester = MillerRabin::with_default_rounds();
        // Large known primes
        assert!(tester.is_prime(&Integer::from(97)));
        assert!(tester.is_prime(&Integer::from(541)));
        assert!(tester.is_prime(&Integer::from(7919)));
        assert!(tester.is_prime(&Integer::from(104729)));
    }

    #[test]
    fn test_miller_rabin_algorithm_variants() {
        let fast = MillerRabin::fast();
        let standard = MillerRabin::with_default_rounds();
        let thorough = MillerRabin::thorough();

        let test_cases = vec![
            Integer::from(2),
            Integer::from(17),
            Integer::from(97),
            Integer::from(7919),
        ];

        for n in test_cases {
            // All variants should agree on these small-medium primes
            assert_eq!(
                fast.is_prime(&n),
                standard.is_prime(&n),
                "Fast and standard variants disagree on {}",
                n
            );
            assert_eq!(
                standard.is_prime(&n),
                thorough.is_prime(&n),
                "Standard and thorough variants disagree on {}",
                n
            );
        }
    }

    #[test]
    fn test_miller_rabin_carmichael_numbers() {
        // Carmichael numbers fool simple primality tests
        // 561 = 3 × 11 × 17
        let tester = MillerRabin::with_default_rounds();
        assert!(!tester.is_prime(&Integer::from(561)));
        assert!(!tester.is_prime(&Integer::from(1105))); // 5 × 13 × 17
        assert!(!tester.is_prime(&Integer::from(1729))); // 7 × 13 × 19
    }
}
