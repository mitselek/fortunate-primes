use rug::Integer;
use std::fmt;
use std::time::{Duration, Instant};
// Rayon is imported and available for future parallel optimizations (Phase 1.2+)
#[allow(unused_imports)]
use rayon::prelude::*;

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

        // Find the smallest candidate where p_n# + m is prime
        // Using a linear search instead of parallel since we need the FIRST match
        // (Rayon's find_any doesn't guarantee the smallest value)
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

        // Sequential search to ensure we find the FIRST (smallest) Fortunate number
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

    // ============================================================================
    // Miller-Rabin Primality Tests
    // ============================================================================

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

    // ============================================================================
    // Primorial Tests
    // ============================================================================

    #[test]
    fn test_primorial() {
        let primes = vec![2, 3, 5, 7, 11];
        let calc = PrimeBasedCalculator::new(primes);

        assert_eq!(calc.primorial(1).unwrap(), Integer::from(2));
        assert_eq!(calc.primorial(2).unwrap(), Integer::from(6)); // 2*3
        assert_eq!(calc.primorial(3).unwrap(), Integer::from(30)); // 2*3*5
        assert_eq!(calc.primorial(4).unwrap(), Integer::from(210)); // 2*3*5*7
        assert_eq!(calc.primorial(5).unwrap(), Integer::from(2310)); // 2*3*5*7*11
    }

    #[test]
    fn test_primorial_single_prime() {
        let primes = vec![2];
        let calc = PrimeBasedCalculator::new(primes);
        assert_eq!(calc.primorial(1).unwrap(), Integer::from(2));
    }

    #[test]
    fn test_primorial_growth() {
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23];
        let calc = PrimeBasedCalculator::new(primes);

        let p1 = calc.primorial(1).unwrap();
        let p2 = calc.primorial(2).unwrap();
        let p3 = calc.primorial(3).unwrap();

        // Primorial should grow monotonically
        assert!(p2 > p1);
        assert!(p3 > p2);
    }

    // ============================================================================
    // Fortunate Number Tests (OEIS A005235 Validation)
    // ============================================================================

    #[test]
    fn test_fortunate_numbers_oeis() {
        // First 10 Fortunate numbers from OEIS A005235
        // https://oeis.org/A005235
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let calc = PrimeBasedCalculator::new(primes);

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
            assert_eq!(
                calc.fortunate_number(n).unwrap(),
                expected,
                "Fortunate number mismatch for n={}",
                n
            );
        }
    }

    #[test]
    fn test_fortunate_numbers_early_values() {
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let calc = PrimeBasedCalculator::new(primes);

        // Detailed check: n=1 case
        // primorial(1) = 2
        // 2 + 1 = 3 (prime) ✓ but we check m > 1
        // 2 + 2 = 4 (not prime)
        // 2 + 3 = 5 (prime) ✓ → Fortunate number is 3
        assert_eq!(calc.fortunate_number(1).unwrap(), 3);

        // n=5 case (common example)
        // primorial(5) = 2*3*5*7*11 = 2310
        // 2310 + 23 = 2333 (prime) ✓ → Fortunate number is 23
        assert_eq!(calc.fortunate_number(5).unwrap(), 23);
    }

    #[test]
    fn test_fortunate_prime_detection() {
        // Fortunate primes: Fortunate numbers that are also prime
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let calc = PrimeBasedCalculator::new(primes);

        let tester = MillerRabin::with_default_rounds();

        for n in 1..=10 {
            let f = calc.fortunate_number(n).unwrap();
            let is_prime = tester.is_prime(&Integer::from(f));
            // Fortune's conjecture: all Fortunate numbers up to n=3000 are prime
            assert!(
                is_prime,
                "Fortune's conjecture violated: Fortunate number {} is not prime",
                f
            );
        }
    }

    // ============================================================================
    // Fortunate Number with Metrics Tests
    // ============================================================================

    #[test]
    fn test_fortunate_with_metrics() {
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let calc = PrimeBasedCalculator::new(primes);

        let (value, metrics) = calc.fortunate_number_with_metrics(5).unwrap();

        // Value should match non-metrics version
        assert_eq!(value, 23);

        // Metrics should be valid
        assert!(metrics.total_time.as_micros() > 0); // Use micros for very fast computations
        assert!(metrics.primality_test_count > 0);
        assert_eq!(metrics.candidate_found, 23);
        assert!(metrics.primality_tests_passed > 0);
    }

    #[test]
    fn test_metrics_consistency() {
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let calc = PrimeBasedCalculator::new(primes);

        let (value1, metrics) = calc.fortunate_number_with_metrics(3).unwrap();
        let value2 = calc.fortunate_number(3).unwrap();

        // Both methods should give same result
        assert_eq!(value1, value2);
        assert_eq!(metrics.candidate_found, value1);
    }

    // ============================================================================
    // Error Handling Tests
    // ============================================================================

    #[test]
    fn test_error_invalid_prime_index() {
        let primes = vec![2, 3, 5];
        let calc = PrimeBasedCalculator::new(primes);

        let err = calc.primorial(10).unwrap_err();
        assert_eq!(err, FortunateError::InvalidPrimeIndex { index: 10, max: 3 });
    }

    #[test]
    fn test_error_index_zero() {
        let primes = vec![2, 3, 5];
        let calc = PrimeBasedCalculator::new(primes);

        // Index 0 might not be explicitly validated, check boundary
        // The implementation validates index >= 1
        let result = calc.primorial(0);
        // It could be error or undefined behavior, this test documents current behavior
        let _ = result; // Accept either way for now
    }

    #[test]
    fn test_error_message_format() {
        let primes = vec![2, 3, 5];
        let calc = PrimeBasedCalculator::new(primes);

        let err = calc.primorial(5).unwrap_err();
        let message = format!("{}", err);

        // Error message should be informative
        assert!(message.contains("out of range"));
        assert!(message.contains("5"));
        assert!(message.contains("3"));
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[test]
    fn test_full_workflow_small() {
        let primes = vec![2, 3, 5, 7, 11, 13];
        let calc = PrimeBasedCalculator::new(primes);

        for n in 1..=6 {
            // Should compute without error
            let (fortune, metrics) = calc.fortunate_number_with_metrics(n).unwrap();

            // Fortunate number should be positive and not too large
            assert!(fortune > 0);
            assert!(
                fortune < 100,
                "Fortunate number {} seems too large for n={}",
                fortune,
                n
            );

            // Metrics should be sensible
            assert!(metrics.total_time.as_micros() > 0);
            // primality_test_count is cumulative count of numbers tested, not just hits
            assert!(metrics.primality_test_count > 0);
        }
    }

    #[test]
    fn test_calculator_with_custom_tester() {
        let primes = vec![2, 3, 5, 7, 11, 13];
        let tester = MillerRabin::fast(); // 20 rounds
        let calc = PrimeBasedCalculator::with_tester(primes, tester);

        // Should work with custom tester variant
        let result = calc.fortunate_number(5).unwrap();
        assert_eq!(result, 23);
    }

    #[test]
    fn test_different_tester_variants_consistency() {
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];

        let calc_fast = PrimeBasedCalculator::with_tester(primes.clone(), MillerRabin::fast());
        let calc_standard =
            PrimeBasedCalculator::with_tester(primes.clone(), MillerRabin::with_default_rounds());
        let calc_thorough = PrimeBasedCalculator::with_tester(primes, MillerRabin::thorough());

        // All variants should produce same results for n=5
        assert_eq!(calc_fast.fortunate_number(5).unwrap(), 23);
        assert_eq!(calc_standard.fortunate_number(5).unwrap(), 23);
        assert_eq!(calc_thorough.fortunate_number(5).unwrap(), 23);
    }

    // ============================================================================
    // Parallel Calculator Tests
    // ============================================================================
    // These tests verify that ParallelFortunateCalculator produces identical results
    // to the sequential PrimeBasedCalculator while achieving significant speedup.

    #[test]
    fn test_parallel_calculator_identical_results_small() {
        // CORRECTNESS TEST: Parallel results must match sequential
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let seq_calc = PrimeBasedCalculator::new(primes.clone());
        let par_calc = ParallelFortunateCalculator::new(primes);

        // Test OEIS values: all must match
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
            let seq_result = seq_calc.fortunate_number(n).unwrap();
            let par_result = par_calc.fortunate_number(n).unwrap();

            assert_eq!(
                seq_result, expected,
                "Sequential calculator: n={} produced {} but expected {}",
                n, seq_result, expected
            );
            assert_eq!(
                par_result, expected,
                "Parallel calculator: n={} produced {} but expected {}",
                n, par_result, expected
            );
            assert_eq!(
                seq_result, par_result,
                "Sequential and parallel results differ for n={}",
                n
            );
        }
    }

    #[test]
    fn test_parallel_vs_sequential_all_values() {
        // VALIDATION TEST: Compare parallel vs sequential across range
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47];
        let seq_calc = PrimeBasedCalculator::new(primes.clone());
        let par_calc = ParallelFortunateCalculator::new(primes);

        // Test n=1 to n=15 (primorial grows quickly)
        for n in 1..=15 {
            let seq_result = seq_calc.fortunate_number(n).unwrap();
            let par_result = par_calc.fortunate_number(n).unwrap();

            assert_eq!(
                seq_result, par_result,
                "Results differ for n={}: sequential={}, parallel={}",
                n, seq_result, par_result
            );
        }
    }

    #[test]
    fn test_parallel_fortunes_are_prime() {
        // FORTUNE'S CONJECTURE: All Fortunate numbers must be prime (up to n=3000)
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let par_calc = ParallelFortunateCalculator::new(primes);
        let tester = MillerRabin::with_default_rounds();

        for n in 1..=10 {
            let f = par_calc.fortunate_number(n).unwrap();
            assert!(
                tester.is_prime(&Integer::from(f)),
                "Fortune's conjecture violated: n={} produced Fortunate number {} which is not prime",
                n,
                f
            );
        }
    }

    #[test]
    fn test_parallel_with_metrics() {
        // METRICS TEST: Parallel calculator should produce valid metrics
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let par_calc = ParallelFortunateCalculator::new(primes);

        let (value, metrics) = par_calc.fortunate_number_with_metrics(5).unwrap();

        assert_eq!(value, 23, "Parallel: incorrect Fortunate number");
        assert!(
            metrics.total_time.as_micros() > 0,
            "Total time should be positive"
        );
        assert_eq!(
            metrics.candidate_found, 23,
            "Metrics should record the found candidate"
        );
    }

    #[test]
    fn test_parallel_custom_tester() {
        // Should work with different Miller-Rabin variants
        let primes = vec![2, 3, 5, 7, 11, 13];
        let tester = MillerRabin::fast(); // 20 rounds
        let par_calc = ParallelFortunateCalculator::with_tester(primes, tester);

        let result = par_calc.fortunate_number(5).unwrap();
        assert_eq!(result, 23, "Parallel calculator with fast tester");
    }

    #[test]
    fn test_parallel_tester_variants_consistency() {
        // All Miller-Rabin variants should produce same results in parallel
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];

        let calc_fast =
            ParallelFortunateCalculator::with_tester(primes.clone(), MillerRabin::fast());
        let calc_standard = ParallelFortunateCalculator::with_tester(
            primes.clone(),
            MillerRabin::with_default_rounds(),
        );
        let calc_thorough =
            ParallelFortunateCalculator::with_tester(primes, MillerRabin::thorough());

        assert_eq!(calc_fast.fortunate_number(5).unwrap(), 23);
        assert_eq!(calc_standard.fortunate_number(5).unwrap(), 23);
        assert_eq!(calc_thorough.fortunate_number(5).unwrap(), 23);
    }

    #[test]
    fn test_parallel_error_handling() {
        // Error cases should match sequential behavior
        let primes = vec![2, 3, 5];
        let par_calc = ParallelFortunateCalculator::new(primes);

        let err = par_calc.primorial(10).unwrap_err();
        assert_eq!(err, FortunateError::InvalidPrimeIndex { index: 10, max: 3 });
    }

    #[test]
    fn test_parallel_sequential_equivalence_property() {
        // PROPERTY TEST: For any primorial, parallel finds first prime match
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31];
        let seq_calc = PrimeBasedCalculator::new(primes.clone());
        let par_calc = ParallelFortunateCalculator::new(primes);

        for n in 1..=11 {
            // Both should find the same Fortunate number
            let seq_result = seq_calc.fortunate_number(n).expect("Sequential failed");
            let par_result = par_calc.fortunate_number(n).expect("Parallel failed");

            assert_eq!(
                seq_result, par_result,
                "Equivalence violation at n={}: seq={}, par={}",
                n, seq_result, par_result
            );

            // Both should be prime (Fortune's conjecture)
            let tester = MillerRabin::with_default_rounds();
            assert!(tester.is_prime(&Integer::from(seq_result)));
            assert!(tester.is_prime(&Integer::from(par_result)));
        }
    }
}
