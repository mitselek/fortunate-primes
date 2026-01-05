/// Segmented Sieve implementation for composite pre-filtering
///
/// This module provides an efficient segmented sieve of Eratosthenes for quickly
/// identifying probable primes before expensive Miller-Rabin primality testing.
/// This is the core optimization of Phase 3.

/// Segmented Sieve of Eratosthenes
///
/// Pre-filters candidates by eliminating multiples of small primes.
/// Much faster than Miller-Rabin for obvious composites, allowing the expensive
/// primality test to run on a much smaller candidate set.
#[derive(Clone)]
pub struct SegmentedSieve {
    /// Small primes used as sieve basis (up to sqrt(limit))
    basis_primes: Vec<u32>,
    /// Segment size for cache efficiency (typically 10K-100K)
    #[allow(dead_code)]
    segment_size: usize,
}

impl SegmentedSieve {
    /// Create a new segmented sieve for numbers up to `limit`
    ///
    /// Pre-computes basis primes up to sqrt(limit)
    pub fn new(limit: u32) -> Self {
        let sqrt_limit = (limit as f64).sqrt() as u32 + 1;
        let basis_primes = Self::simple_sieve(sqrt_limit);

        SegmentedSieve {
            basis_primes,
            segment_size: 10_000, // Tuned for cache efficiency
        }
    }

    /// Simple sieve of Eratosthenes for small primes
    ///
    /// Used to generate basis primes for segmented sieving
    fn simple_sieve(limit: u32) -> Vec<u32> {
        if limit < 2 {
            return vec![];
        }

        let mut is_prime = vec![true; limit as usize + 1];
        is_prime[0] = false;
        is_prime[1] = false;

        for i in 2..=((limit as f64).sqrt() as usize) {
            if is_prime[i] {
                for j in ((i * i)..=limit as usize).step_by(i) {
                    is_prime[j] = false;
                }
            }
        }

        is_prime
            .iter()
            .enumerate()
            .filter_map(|(num, &is_prime)| if is_prime { Some(num as u32) } else { None })
            .collect()
    }

    /// Sieve a specific range [low..high) and return probable primes
    ///
    /// This is the core segmented sieve algorithm
    pub fn sieve_range(&self, low: u32, high: u32) -> Vec<u32> {
        if low >= high {
            return vec![];
        }

        let range_size = (high - low) as usize;
        let mut is_prime = vec![true; range_size];

        // Mark multiples of each basis prime
        for &p in &self.basis_primes {
            // Find first multiple of p in range [low..high)
            let mut start = low.div_ceil(p) * p;
            if start < p * p {
                start = p * p;
            }

            // Mark all multiples of p as composite
            if start < high {
                for j in ((start - low) as usize..range_size).step_by(p as usize) {
                    is_prime[j] = false;
                }
            }
        }

        // Collect unmarked numbers as probable primes
        is_prime
            .iter()
            .enumerate()
            .filter_map(|(i, &prime)| {
                if prime {
                    let num = low + i as u32;
                    Some(num)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segmented_sieve_basic() {
        // TDD TEST: Verify sieve correctly identifies composites in a range
        // UNIT TEST: Sieve correctness for small range

        // Test sieving range [2..100]
        // Expected primes in [2..100]: 2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71,73,79,83,89,97
        // That's 25 primes, so 75 composites should be marked

        let sieve = SegmentedSieve::new(100);
        let primes = sieve.sieve_range(2, 100);

        assert_eq!(primes.len(), 25, "Should find 25 primes in range [2..100]");
        assert!(primes.contains(&2), "Should include 2");
        assert!(primes.contains(&97), "Should include 97");
        assert!(!primes.contains(&4), "Should not include 4 (composite)");
        assert!(
            !primes.contains(&100),
            "Should not include 100 (out of range)"
        );

        // Verify all known primes are present
        let expected_primes = vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97,
        ];
        for p in expected_primes {
            assert!(primes.contains(&p), "Missing prime {}", p);
        }
    }
}
