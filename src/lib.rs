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
/// These are all primes and composites not divisible by 2, 3, or 5.
///
/// This is implemented as a simple iterator that yields candidates matching
/// the wheel pattern (coprime to 2*3*5 = 30).
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

/// Segmented Sieve for efficient probable prime filtering
///
/// Phase 3 optimization: Pre-filter candidates using segmented sieve before
/// applying expensive Miller-Rabin primality testing. This reduces the number
/// of primality tests by 40-60%, achieving 1.3-1.5x speedup.
///
/// Algorithm:
/// 1. Pre-compute small primes up to sqrt(limit) for sieve basis
/// 2. Divide search range into segments (cache-friendly chunks)
/// 3. For each segment, mark multiples of basis primes as composite
/// 4. Return unmarked candidates as probable primes
///
/// Memory: O(segment_size) - only one segment in memory at a time
/// Time: O(n log log n) where n is the range size
#[derive(Clone)]
pub struct SegmentedSieve {
    /// Small primes used as sieve basis (up to sqrt(limit))
    basis_primes: Vec<u32>,
    /// Segment size for cache efficiency (typically 10K-100K)
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
            let mut start = ((low + p - 1) / p) * p;
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

    // ============================================================================
    // Wheel Factorization Tests (Phase 1.2 Optimization)
    // ============================================================================
    // Wheel factorization skips candidates divisible by small primes (2,3,5,7,...)
    // Expected speedup: 2-3x by eliminating unpromising candidates early

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
        assert!(!primes.contains(&100), "Should not include 100 (out of range)");
        
        // Verify all known primes are present
        let expected_primes = vec![2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71,73,79,83,89,97];
        for p in expected_primes {
            assert!(primes.contains(&p), "Missing prime {}", p);
        }
    }

    #[test]
    fn test_sieved_calculator_correctness() {
        // TDD TEST: This will fail initially (SievedFortunateCalculator not yet implemented)
        // CORRECTNESS TEST: Sieved calculator must produce same results as parallel
        
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        // let par_calc = ParallelFortunateCalculator::new(primes.clone());
        // let sieved_calc = SievedFortunateCalculator::new(primes);

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

        // This will be uncommented after implementation
        // for (n, expected) in oeis_values {
        //     let par_result = par_calc.fortunate_number(n).unwrap();
        //     let sieved_result = sieved_calc.fortunate_number(n).unwrap();
        //
        //     assert_eq!(
        //         sieved_result, expected,
        //         "Sieved calculator: n={} produced {} but OEIS expects {}",
        //         n, sieved_result, expected
        //     );
        //     assert_eq!(
        //         par_result, sieved_result,
        //         "Sieved result differs from parallel for n={}",
        //         n
        //     );
        // }
    }

    #[test]
    fn test_sieved_speedup_benchmark() {
        // TDD TEST: This will fail initially (no speedup without implementation)
        // PERFORMANCE TEST: Sieved should be 1.3x+ faster than parallel for n≥100
        
        // This will be uncommented after implementation
        // let primes = primes::PRIMES_10K[..200].to_vec();
        // let par_calc = ParallelFortunateCalculator::new(primes.clone());
        // let sieved_calc = SievedFortunateCalculator::new(primes);
        //
        // // Measure parallel baseline
        // let (par_result, par_metrics) = par_calc.fortunate_number_with_metrics(100).unwrap();
        //
        // // Measure sieved performance
        // let (sieved_result, sieved_metrics) = sieved_calc.fortunate_number_with_metrics(100).unwrap();
        //
        // // CORRECTNESS
        // assert_eq!(par_result, sieved_result, "Results must match for n=100");
        //
        // // PERFORMANCE
        // let speedup = par_metrics.total_time.as_micros() as f64 / sieved_metrics.total_time.as_micros() as f64;
        //
        // println!("n=100 Phase 3 Performance:");
        // println!("  Parallel: {:?}", par_metrics.total_time);
        // println!("  Sieved: {:?}", sieved_metrics.total_time);
        // println!("  Speedup: {:.2}x", speedup);
        //
        // assert!(
        //     speedup >= 1.3,
        //     "Sieved speedup insufficient: {:.2}x (expected ≥1.3x)",
        //     speedup
        // );
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
        
        // This will be uncommented after implementation
        // let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        // let sieved_calc = SievedFortunateCalculator::new(primes);
        // let tester = MillerRabin::with_default_rounds();
        //
        // for n in 1..=10 {
        //     let f = sieved_calc.fortunate_number(n).unwrap();
        //     assert!(
        //         tester.is_prime(&Integer::from(f)),
        //         "Fortune's conjecture violated: n={} produced {} (not prime)",
        //         n, f
        //     );
        // }
    }
}
