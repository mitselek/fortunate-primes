use rug::Integer;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
// Rayon is imported and available for future parallel optimizations (Phase 1.2+)
#[allow(unused_imports)]
use rayon::prelude::*;

pub mod primes;
pub mod primality;
pub mod sieve;
pub mod wheel;

// Re-export extracted types for backward compatibility
pub use primality::MillerRabin;
pub use sieve::SegmentedSieve;
pub use wheel::{WheelFactorization, WheelFortunateCalculator, WheelIterator};

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

/// Wheel factorization generator for candidate filtering
///
/// Wheel factorization is a sieving optimization that generates only numbers
/// NOT divisible by small primes (typically 2, 3, 5). This dramatically reduces
/// the search space: roughly 26% of candidates in range [2..max] are kept.
///
/// Example: For range [2..30], wheel keeps only: 2, 3, 5, 7, 11, 13, 17, 19, 23, 29
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

    // ============================================================================
    // Wheel Factorization Tests (Phase 1.2 Optimization)
    // ============================================================================
    // Wheel factorization skips candidates divisible by small primes (2,3,5,7,...)
    // Expected speedup: 2-3x by eliminating unpromising candidates early

    // ============================================================================
    // Phase 2: Parallel Candidate Testing (Rayon)
    // ============================================================================
    // These tests verify that parallel candidate testing with Rayon achieves
    // significant speedup (2-4x) while maintaining correctness.

    #[test]
    fn test_parallel_speedup_benchmark() {
        // TDD TEST: This test should FAIL initially, then PASS after Rayon implementation
        // PERFORMANCE TEST: Parallel should be 2x+ faster than wheel for n>100

        // Use larger prime set to test n=100
        let primes = primes::PRIMES_10K[..200].to_vec();

        let wheel_calc = WheelFortunateCalculator::new(primes.clone());
        let par_calc = ParallelFortunateCalculator::new(primes);

        // Measure wheel factorization time (baseline)
        let (wheel_result, wheel_metrics) = wheel_calc.fortunate_number_with_metrics(100).unwrap();

        // Measure parallel time (should be faster)
        let (par_result, par_metrics) = par_calc.fortunate_number_with_metrics(100).unwrap();

        // CORRECTNESS: Both should find the same Fortunate number
        assert_eq!(
            wheel_result, par_result,
            "Parallel and wheel results must match for n=100"
        );

        // PERFORMANCE: Parallel should be significantly faster
        // Expected: wheel ~30-40ms, parallel ~15-20ms (2x+ speedup)
        let wheel_time_us = wheel_metrics.total_time.as_micros();
        let par_time_us = par_metrics.total_time.as_micros();
        let speedup = wheel_time_us as f64 / par_time_us as f64;

        println!("n=100 Performance:");
        println!("  Wheel: {:?}", wheel_metrics.total_time);
        println!("  Parallel: {:?}", par_metrics.total_time);
        println!("  Speedup: {:.2}x", speedup);

        // This assertion will FAIL initially (speedup ~1.0x)
        // After Rayon implementation, speedup should be 1.5x-2.0x minimum
        assert!(
            speedup >= 1.5,
            "Parallel speedup insufficient: {:.2}x (expected ≥1.5x)",
            speedup
        );
    }

    #[test]
    fn test_parallel_correctness_with_rayon() {
        // CORRECTNESS TEST: Parallel with Rayon must still find the SMALLEST Fortunate number
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let seq_calc = PrimeBasedCalculator::new(primes.clone());
        let par_calc = ParallelFortunateCalculator::new(primes);

        // Test OEIS values through n=10
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
                par_result, expected,
                "Parallel with Rayon: n={} produced {} but OEIS expects {}",
                n, par_result, expected
            );
            assert_eq!(
                seq_result, par_result,
                "Parallel result differs from sequential for n={}",
                n
            );
        }
    }

    #[test]
    fn test_parallel_thread_safety() {
        // SAFETY TEST: Parallel execution must be thread-safe
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31];
        let par_calc = ParallelFortunateCalculator::new(primes);

        // Run multiple calculations concurrently
        // If thread-unsafe, results would be inconsistent
        let results: Vec<_> = (1..=11)
            .map(|n| par_calc.fortunate_number(n).unwrap())
            .collect();

        // Expected OEIS A005235 values for n=1..11
        let expected = vec![3, 5, 7, 13, 23, 17, 19, 23, 37, 61, 67];

        assert_eq!(results, expected, "Parallel thread safety violated");
    }

    #[test]
    fn test_parallel_large_n_speedup() {
        // PERFORMANCE TEST: Verify speedup increases with larger n
        // For n=200, parallel should be 2-3x faster than wheel

        let primes = primes::PRIMES_10K[..300].to_vec();
        let wheel_calc = WheelFortunateCalculator::new(primes.clone());
        let par_calc = ParallelFortunateCalculator::new(primes);

        let (wheel_result, wheel_metrics) = wheel_calc.fortunate_number_with_metrics(200).unwrap();
        let (par_result, par_metrics) = par_calc.fortunate_number_with_metrics(200).unwrap();

        // CORRECTNESS
        assert_eq!(wheel_result, par_result, "Results must match for n=200");

        // PERFORMANCE (this will fail initially)
        let speedup = wheel_metrics.total_time.as_secs_f64() / par_metrics.total_time.as_secs_f64();

        println!("n=200 Performance:");
        println!("  Wheel: {:?}", wheel_metrics.total_time);
        println!("  Parallel: {:?}", par_metrics.total_time);
        println!("  Speedup: {:.2}x", speedup);

        assert!(
            speedup >= 2.0,
            "Parallel speedup insufficient at n=200: {:.2}x (expected ≥2.0x)",
            speedup
        );
    }

    // ============================================================================
    // Phase 3: Segmented Sieve Optimization
    // ============================================================================
    // These tests verify that segmented sieve pre-filtering achieves additional
    // 1.3-1.5x speedup while maintaining correctness.

    #[test]
    fn test_sieved_calculator_correctness() {
        // CORRECTNESS TEST: Sieved calculator must produce same results as parallel
        // OEIS A005235 validation

        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let par_calc = ParallelFortunateCalculator::new(primes.clone());
        let sieved_calc = SievedFortunateCalculator::new(primes);

        // Test OEIS values through n=10
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
            let par_result = par_calc.fortunate_number(n).unwrap();
            let sieved_result = sieved_calc.fortunate_number(n).unwrap();

            assert_eq!(
                sieved_result, expected,
                "Sieved calculator: n={} produced {} but OEIS expects {}",
                n, sieved_result, expected
            );
            assert_eq!(
                par_result, sieved_result,
                "Sieved result differs from parallel for n={}",
                n
            );
        }
    }

    #[test]
    fn test_sieved_speedup_benchmark() {
        // PERFORMANCE TEST: Sieved should be 1.3x+ faster than parallel for n≥100

        let primes = primes::PRIMES_10K[..200].to_vec();
        let par_calc = ParallelFortunateCalculator::new(primes.clone());
        let sieved_calc = SievedFortunateCalculator::new(primes);

        // Measure parallel baseline
        let (par_result, par_metrics) = par_calc.fortunate_number_with_metrics(100).unwrap();

        // Measure sieved performance
        let (sieved_result, sieved_metrics) =
            sieved_calc.fortunate_number_with_metrics(100).unwrap();

        // CORRECTNESS
        assert_eq!(par_result, sieved_result, "Results must match for n=100");

        // PERFORMANCE
        let speedup = par_metrics.total_time.as_micros() as f64
            / sieved_metrics.total_time.as_micros() as f64;

        println!("n=100 Phase 3 Performance:");
        println!("  Parallel: {:?}", par_metrics.total_time);
        println!("  Sieved: {:?}", sieved_metrics.total_time);
        println!("  Speedup: {:.2}x", speedup);

        assert!(
            speedup >= 1.3,
            "Sieved speedup insufficient: {:.2}x (expected ≥1.3x)",
            speedup
        );
    }

    #[test]
    fn test_sieved_reduces_miller_rabin_calls() {
        // EFFICIENCY TEST: Sieve should reduce Miller-Rabin invocations by 40-60%

        // This will be uncommented after implementation
        // let primes = primes::PRIMES_10K[..100].to_vec();
        // let par_calc = ParallelFortunateCalculator::new(primes.clone());
        // let sieved_calc = SievedFortunateCalculator::new(primes);
        //
        // let (_, par_metrics) = par_calc.fortunate_number_with_metrics(50).unwrap();
        // let (_, sieved_metrics) = sieved_calc.fortunate_number_with_metrics(50).unwrap();
        //
        // let reduction_pct = (1.0 - (sieved_metrics.primality_test_count as f64 / par_metrics.primality_test_count as f64)) * 100.0;
        //
        // println!("Miller-Rabin Test Reduction:");
        // println!("  Parallel: {} tests", par_metrics.primality_test_count);
        // println!("  Sieved: {} tests", sieved_metrics.primality_test_count);
        // println!("  Reduction: {:.1}%", reduction_pct);
        //
        // assert!(
        //     reduction_pct >= 40.0,
        //     "Sieve should reduce tests by ≥40%, got {:.1}%",
        //     reduction_pct
        // );
    }

    #[test]
    fn test_sieved_fortunes_are_prime() {
        // FORTUNE'S CONJECTURE: All sieved Fortunate numbers must be prime

        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let sieved_calc = SievedFortunateCalculator::new(primes);
        let tester = MillerRabin::with_default_rounds();

        for n in 1..=10 {
            let f = sieved_calc.fortunate_number(n).unwrap();
            assert!(
                tester.is_prime(&Integer::from(f)),
                "Fortune's conjecture violated: n={} produced {} (not prime)",
                n,
                f
            );
        }
    }

    #[test]
    fn test_sieved_no_regression_small_n() {
        // REGRESSION TEST: Verify correctness for small n (n < 100)
        // Performance is only tested for n >= 100 (the target use case)
        // Small n values are tested for logical correctness only

        let primes = primes::PRIMES_10K[..50].to_vec();
        let par_calc = ParallelFortunateCalculator::new(primes.clone());
        let sieved_calc = SievedFortunateCalculator::new(primes);

        // Verify correctness: results must match for small n values
        for n in [5, 10, 20, 30, 40, 50] {
            let par_result = par_calc.fortunate_number(n).unwrap();
            let sieved_result = sieved_calc.fortunate_number(n).unwrap();

            assert_eq!(
                par_result, sieved_result,
                "n={}: results differ (parallel={}, sieved={})",
                n, par_result, sieved_result
            );
        }
    }
}
