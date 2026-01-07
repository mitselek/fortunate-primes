//! Progress reporting
//!
//! Handles terminal output with overwriting for progress updates.

use std::io::{self, Write};
use std::time::{Duration, Instant};

/// Progress reporter that starts reporting after a delay
pub struct ProgressReporter {
    n: usize,
    start_time: Instant,
    last_report: Option<Instant>,
    initial_delay: Duration,
    min_interval: Duration,
    reported: bool,
    last_batch_size: u64,
    lower_bound: u64,  // Highest value tested without finding result
}

impl ProgressReporter {
    /// Create a new reporter
    /// - initial_delay: seconds before first output (e.g., 2.0)
    /// - min_interval: seconds between outputs (e.g., 1.0)
    pub fn new(n: usize, initial_delay_secs: f64, min_interval_secs: f64) -> Self {
        Self {
            n,
            start_time: Instant::now(),
            last_report: None,
            initial_delay: Duration::from_secs_f64(initial_delay_secs),
            min_interval: Duration::from_secs_f64(min_interval_secs),
            reported: false,
            last_batch_size: 0,
            lower_bound: 0,
        }
    }

    /// Check if we should report now
    pub fn should_report(&self) -> bool {
        let elapsed = self.start_time.elapsed();

        // Must be past initial delay
        if elapsed < self.initial_delay {
            return false;
        }

        // Must be past min_interval since last report
        if let Some(last) = self.last_report {
            if last.elapsed() < self.min_interval {
                return false;
            }
        }

        true
    }

    /// Report intermediate progress with bounds in interval notation
    /// Format: "F(n) : [lower; upper] [start+size] (elapsed)" or "F(n) : [lower; ?]" if no candidate yet
    /// Only outputs if timing conditions are met
    pub fn report_progress(&mut self, candidate: u64, batch_start: u64, batch_size: u64, lower_bound: u64) {
        if !self.should_report() {
            return;
        }

        let elapsed = self.start_time.elapsed();
        let time_str = format_duration(elapsed);

        let bounds_str = if candidate > 0 {
            format!("[{}; {}]", lower_bound, candidate)
        } else {
            format!("[{}; ?]", lower_bound)
        };

        eprintln!("F({}) : {} [{}+{}] ({})", self.n, bounds_str, batch_start, batch_size, time_str);
        self.last_report = Some(Instant::now());
        self.reported = true;
        self.last_batch_size = batch_size;
        self.lower_bound = lower_bound;
    }

    /// Report final result: "F(n) = result (elapsed)"
    /// Always outputs
    pub fn report_final(&self, result: u64) {
        let elapsed = self.start_time.elapsed();
        let time_str = format_duration(elapsed);

        println!("F({}) = {} ({})", self.n, result, time_str);
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Format duration as human-readable string
pub fn format_duration(d: Duration) -> String {
    let secs = d.as_secs_f64();
    if secs < 1.0 {
        format!("{:.0}ms", secs * 1000.0)
    } else if secs < 60.0 {
        format!("{:.2}s", secs)
    } else {
        let mins = secs / 60.0;
        format!("{:.2}m", mins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(Duration::from_millis(50)), "50ms");
    }

    #[test]
    fn test_format_duration_secs() {
        assert_eq!(format_duration(Duration::from_secs_f64(1.5)), "1.50s");
        assert_eq!(format_duration(Duration::from_secs_f64(59.99)), "59.99s");
    }

    #[test]
    fn test_format_duration_mins() {
        assert_eq!(format_duration(Duration::from_secs(60)), "1.00m");
        assert_eq!(format_duration(Duration::from_secs(90)), "1.50m");
    }

    #[test]
    fn test_reporter_respects_initial_delay() {
        let mut reporter = ProgressReporter::new(100, 10.0, 1.0); // 10s delay
        reporter.report_progress(500, 2, 100, 1000); // Should not output (delay not passed)
        assert!(!reporter.reported);
    }
}
