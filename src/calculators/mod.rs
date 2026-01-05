//! Calculator implementations for Fortunate number computation
//!
//! This module contains different strategies for calculating Fortunate numbers:
//! - `base`: PrimeBasedCalculator - Sequential baseline
//! - `parallel`: ParallelFortunateCalculator - Batch-parallel testing with Rayon
//! - `sieved`: SievedFortunateCalculator - Segmented sieve pre-filtering + parallel testing

pub mod base;
pub mod parallel;
pub mod sieved;

pub use base::PrimeBasedCalculator;
pub use parallel::ParallelFortunateCalculator;
pub use sieved::SievedFortunateCalculator;
