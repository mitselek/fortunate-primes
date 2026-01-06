//! Parallel batch search coordinator with adaptive batching
//!
//! Uses work queue pattern: workers request batch ranges from main thread,
//! with batch size adapting based on completion time.

use crate::pari::PariSearch;
use crate::progress::ProgressReporter;
use crossbeam_channel::bounded;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

/// Initial batch size
const INITIAL_BATCH_SIZE: u64 = 100;

/// Compute contiguous lower bound from start=2 by finding all touching intervals with no result
fn compute_contiguous_lower_bound(completed_no_result: &BTreeMap<u64, u64>) -> u64 {
    let mut bound = 2u64;
    for (&batch_start, &batch_end) in completed_no_result.iter() {
        if batch_start <= bound {
            bound = bound.max(batch_end);
        } else {
            break;  // Gap found, stop extending
        }
    }
    bound
}

/// Find the Fortunate number F(n) using adaptive parallel batch search
///
/// Strategy:
/// 1. Spawn (CPU cores - 1) worker threads with shared stop flag
/// 2. Main thread distributes batch ranges with adaptive sizing:
///    - Start with batch_size = 100
///    - If batch completes in < 30s with no result, double batch_size
/// 3. When any batch returns candidate:
///    - STOP dispatching new work
///    - Workers beyond candidate exit early (cooperative cancellation)
/// 4. Track only batches needed to close [lower_bound; candidate] gap
/// 5. Exit when gap is closed
pub fn find_fortunate(n: usize) -> Result<u64, String> {
    let num_workers = num_cpus::get().saturating_sub(1).max(1);
    let mut reporter = ProgressReporter::new(n, 2.0, 1.0);

    let (work_tx, work_rx) = bounded::<(u64, u64)>(num_workers * 2);
    let (result_tx, result_rx) = bounded::<(u64, u64, Instant, Option<u64>)>(num_workers * 2);

    // Shared state for cooperative cancellation
    let candidate_found = Arc::new(AtomicU64::new(0)); // 0 = no candidate, >0 = candidate value
    let stop_flag = Arc::new(AtomicBool::new(false));

    // Spawn workers
    for _ in 0..num_workers {
        let work_rx = work_rx.clone();
        let result_tx = result_tx.clone();
        let candidate_found = Arc::clone(&candidate_found);
        let stop_flag = Arc::clone(&stop_flag);

        thread::spawn(move || {
            while let Ok((start, end)) = work_rx.recv() {
                // Check if we should skip this batch entirely
                let current_candidate = candidate_found.load(Ordering::Relaxed);
                if stop_flag.load(Ordering::Relaxed) && current_candidate > 0 && start >= current_candidate {
                    // Batch is beyond candidate, skip it
                    let batch_start_time = Instant::now();
                    let batch_size = end - start;
                    let _ = result_tx.send((batch_size, end, batch_start_time, None));
                    continue;
                }

                let batch_start_time = Instant::now();
                let batch_size = end - start;
                let result = PariSearch::start(n, start, end)
                    .ok()
                    .and_then(|search| search.wait().ok().flatten());
                let _ = result_tx.send((batch_size, end, batch_start_time, result));
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
    let mut completed_no_result: BTreeMap<u64, u64> = BTreeMap::new();  // Track batches with no result

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
        if let Ok((completed_batch_size, end, batch_start_time, result)) = result_rx.recv() {
            batches_in_flight -= 1;
            let batch_start = end - completed_batch_size;
            let batch_duration = batch_start_time.elapsed();

            if let Some(candidate) = result {
                let is_better = best.map(|b| candidate < b).unwrap_or(true);
                if is_better {
                    best = Some(candidate);
                    candidate_found.store(candidate, Ordering::Relaxed);
                    stop_flag.store(true, Ordering::Relaxed);
                }
            } else {
                // Track batch with no result
                completed_no_result.insert(batch_start, end);
            }

            // Compute contiguous lower bound from all completed no-result batches
            let lower_bound = compute_contiguous_lower_bound(&completed_no_result);

            // Check if we've closed the gap
            if let Some(candidate) = best {
                if lower_bound >= candidate {
                    // Gap closed! We have definitive answer
                    reporter.report_final(candidate);
                    return Ok(candidate);
                }
            }

            // Report progress if timing allows (even if no candidate found yet)
            if reporter.should_report() {
                let candidate_to_report = best.unwrap_or(0);
                reporter.report_progress(candidate_to_report, batch_start, completed_batch_size, lower_bound);
            }

            // Adaptive sizing: double batch size only if:
            // 1. No result found
            // 2. Batch completed in under 30 seconds
            // 3. Current batch_size <= completed batch size (only grow if we're behind)
            if result.is_none()
                && batch_duration.as_secs_f64() < 30.0
                && batch_size <= completed_batch_size
            {
                batch_size = batch_size.saturating_mul(2);
            }

            // ONLY dispatch new work if no candidate found yet
            if best.is_none() {
                // No result yet, continue searching
                let batch_end = next_start + batch_size;
                if work_tx.send((next_start, batch_end)).is_ok() {
                    batches_in_flight += 1;
                    next_start = batch_end;
                }
            }
            // If candidate found, don't dispatch new work - just wait for critical batches
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
