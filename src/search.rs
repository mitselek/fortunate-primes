//! Parallel batch search coordinator with adaptive batching
//!
//! Uses work queue pattern: workers request batch ranges from main thread,
//! with batch size adapting based on completion time.

use crate::pari::PariSearch;
use crate::progress::ProgressReporter;
use crossbeam_channel::bounded;
use std::thread;
use std::time::Instant;

/// Initial batch size
const INITIAL_BATCH_SIZE: u64 = 100;

/// Threshold (seconds) below which batch size doubles
const GROWTH_THRESHOLD_SECS: u64 = 30;

/// Find the Fortunate number F(n) using adaptive parallel batch search
///
/// Strategy:
/// 1. Spawn (CPU cores - 1) worker threads
/// 2. Main thread distributes batch ranges with adaptive sizing:
///    - Start with batch_size = 100
///    - If batch completes in < 30s with no result, double batch_size
/// 3. When any batch returns result R, remaining batches only search [2..R)
/// 4. Return minimum result found
pub fn find_fortunate(n: usize) -> Result<u64, String> {
    let num_workers = num_cpus::get().saturating_sub(1).max(1);
    let mut reporter = ProgressReporter::new(n, 2.0, 1.0);

    let (work_tx, work_rx) = bounded::<(u64, u64)>(num_workers * 2);
    let (result_tx, result_rx) = bounded::<(u64, Option<u64>)>(num_workers * 2);

    // Spawn workers
    for _ in 0..num_workers {
        let work_rx = work_rx.clone();
        let result_tx = result_tx.clone();

        thread::spawn(move || {
            while let Ok((start, end)) = work_rx.recv() {
                let result = PariSearch::start(n, start, end)
                    .ok()
                    .and_then(|search| search.wait().ok().flatten());
                let _ = result_tx.send((end, result));
            }
        });
    }

    drop(work_rx);
    drop(result_tx);

    // Main thread: distribute work with adaptive sizing
    let mut batch_size = INITIAL_BATCH_SIZE;
    let mut next_start: u64 = 2;
    let mut best: Option<u64> = None;
    let mut batches_in_flight = 0;

    // Launch initial batches
    for _ in 0..num_workers {
        let end = next_start + batch_size;
        if work_tx.send((next_start, end)).is_ok() {
            batches_in_flight += 1;
            next_start = end;
        }
    }

    // Receive results and spawn new batches
    while batches_in_flight > 0 {
        if let Ok((_end, result)) = result_rx.recv() {
            batches_in_flight -= 1;

            if let Some(candidate) = result {
                let is_better = best.map(|b| candidate < b).unwrap_or(true);
                if is_better {
                    best = Some(candidate);
                    reporter.report_progress(candidate);
                }
            }

            // Adaptive sizing: if batch was fast and no result yet, grow batch size
            let batch_time = Instant::now();
            if result.is_none() && batch_time.elapsed().as_secs() < GROWTH_THRESHOLD_SECS {
                batch_size = batch_size.saturating_mul(2);
            }

            // Assign next batch if we haven't found result yet or batch is before result
            if let Some(best_candidate) = best {
                // Only search up to best_candidate
                if next_start < best_candidate {
                    let batch_end = (next_start + batch_size).min(best_candidate);
                    if work_tx.send((next_start, batch_end)).is_ok() {
                        batches_in_flight += 1;
                        next_start = batch_end;
                    }
                }
            } else {
                // No result yet, continue searching
                let batch_end = next_start + batch_size;
                if work_tx.send((next_start, batch_end)).is_ok() {
                    batches_in_flight += 1;
                    next_start = batch_end;
                }
            }
        } else {
            break;
        }
    }

    match best {
        Some(result) => {
            reporter.report_final(result);
            Ok(result)
        }
        None => Err("No Fortunate number found".to_string()),
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
