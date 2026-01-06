//! Parallel batch search coordinator
//!
//! Divides search space into batches and coordinates parallel workers.

use crate::pari::PariSearch;
use crate::progress::ProgressReporter;
use std::sync::mpsc;
use std::thread;

/// Batch size for parallel search
const BATCH_SIZE: u64 = 1_000;

/// Number of parallel workers
const NUM_WORKERS: usize = 32;

/// Find the Fortunate number F(n) using parallel batch search
///
/// Strategy:
/// 1. Launch batches [2..B], [B+1..2B], ... in parallel
/// 2. When batch K returns result R:
///    - Kill all batches > K (their candidates > R, can't be smaller)
///    - Wait for batches < K (might have smaller result)
/// 3. Return minimum result found
pub fn find_fortunate(n: usize) -> Result<u64, String> {
    let mut reporter = ProgressReporter::new(n, 2.0, 1.0);
    let (tx, rx) = mpsc::channel();

    // Launch all workers, keeping handles to kill later batches
    let mut handles: Vec<Option<thread::JoinHandle<()>>> = Vec::new();

    for batch_id in 0..NUM_WORKERS {
        let tx = tx.clone();
        let start = if batch_id == 0 { 2 } else { batch_id as u64 * BATCH_SIZE + 1 };
        let end = (batch_id as u64 + 1) * BATCH_SIZE;

        let handle = thread::spawn(move || {
            match PariSearch::start(n, start, end) {
                Ok(search) => {
                    let result = search.wait().ok().flatten();
                    let _ = tx.send((batch_id, result));
                }
                Err(_) => {
                    let _ = tx.send((batch_id, None));
                }
            }
        });

        handles.push(Some(handle));
    }

    drop(tx);

    // Track results and best candidate
    let mut completed: Vec<usize> = Vec::new();
    let mut best: Option<(usize, u64)> = None; // (batch_id, candidate)

    for (batch_id, result) in rx {
        completed.push(batch_id);

        if let Some(candidate) = result {
            let dominated = best.map(|(_, b)| candidate >= b).unwrap_or(false);

            if !dominated {
                best = Some((batch_id, candidate));
                reporter.report_progress(candidate);

                // Note: We can't actually kill threads in Rust, but the PARI processes
                // will be cleaned up when their threads finish
            }
        }

        // Check if we can terminate: all batches <= best_batch have completed
        if let Some((best_batch, _)) = best {
            let all_earlier_done = (0..=best_batch).all(|id| completed.contains(&id));
            if all_earlier_done {
                break;
            }
        }
    }

    match best {
        Some((_, result)) => {
            reporter.report_final(result);
            Ok(result)
        }
        None => Err("No Fortunate number found (try larger batch size)".to_string()),
    }
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
