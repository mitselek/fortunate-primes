//! Parallel batch search coordinator
//!
//! Divides search space into batches and coordinates parallel workers.

use crate::pari;
use crate::progress::ProgressReporter;
use std::sync::mpsc;
use std::thread;

/// Batch size for parallel search
const BATCH_SIZE: u64 = 10_000;

/// Number of parallel workers
const NUM_WORKERS: usize = 8;

/// Find the Fortunate number F(n) using parallel batch search
///
/// Strategy:
/// 1. Launch batches [1..B], [B+1..2B], ... in parallel
/// 2. Collect results
/// 3. When batch K finds result, we know F(n) <= result
/// 4. Only need to await batches that started before K
/// 5. Return minimum of all results
pub fn find_fortunate(n: usize) -> Result<u64, String> {
    let mut reporter = ProgressReporter::new(n, 2.0);
    let (tx, rx) = mpsc::channel();

    // Launch all workers at once
    for batch_id in 0..NUM_WORKERS {
        let tx = tx.clone();
        // Batch 0 starts at 2 (Fortunate numbers > 1), others follow
        let start = if batch_id == 0 { 2 } else { batch_id as u64 * BATCH_SIZE + 1 };
        let end = (batch_id as u64 + 1) * BATCH_SIZE;

        thread::spawn(move || {
            let result = pari::search_range(n, start, end).ok().flatten();
            tx.send((batch_id, result)).unwrap();
        });
    }

    drop(tx); // Important: close sender so rx.iter() terminates

    // Collect results
    let mut results: Vec<(usize, Option<u64>)> = Vec::new();
    let mut best: Option<u64> = None;

    for (batch_id, result) in rx {
        results.push((batch_id, result));

        if let Some(candidate) = result {
            match best {
                None => {
                    best = Some(candidate);
                    reporter.report(candidate);
                }
                Some(current_best) if candidate < current_best => {
                    best = Some(candidate);
                    reporter.report(candidate);
                }
                _ => {}
            }
        }

        // Check if we can terminate early
        if let Some(best_val) = best {
            // Find which batch has the best result
            let best_batch = results
                .iter()
                .filter(|(_, r)| *r == Some(best_val))
                .map(|(id, _)| *id)
                .min()
                .unwrap();

            // Check if all earlier batches have reported
            let earlier_batches_done = (0..best_batch).all(|id| {
                results.iter().any(|(bid, _)| *bid == id)
            });

            if earlier_batches_done {
                // All earlier batches done, we have the true minimum
                break;
            }
        }
    }

    reporter.clear();

    best.ok_or_else(|| "No Fortunate number found (try larger batch size)".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_f5() {
        let result = find_fortunate(5).unwrap();
        assert_eq!(result, 23);
    }

    #[test]
    fn test_find_f10() {
        let result = find_fortunate(10).unwrap();
        assert_eq!(result, 61);
    }

    #[test]
    fn test_find_f20() {
        let result = find_fortunate(20).unwrap();
        assert_eq!(result, 103);
    }
}
