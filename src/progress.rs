//! Progress reporting
//!
//! Handles terminal output with overwriting for progress updates.

use std::io::{self, Write};
use std::time::{Duration, Instant};

/// Progress reporter that starts reporting after a delay
pub struct ProgressReporter {
    n: usize,
    start_time: Instant,
    delay: Duration,
    started: bool,
}

impl ProgressReporter {
    /// Create a new reporter that starts after `delay_secs` seconds
    pub fn new(n: usize, delay_secs: f64) -> Self {
        Self {
            n,
            start_time: Instant::now(),
            delay: Duration::from_secs_f64(delay_secs),
            started: false,
        }
    }

    /// Report progress with current best candidate
    /// Format: "F(n) > candidate (elapsed)"
    /// Only outputs if delay has passed
    pub fn report(&mut self, candidate: u64) {
        if self.start_time.elapsed() < self.delay {
            return;
        }

        let elapsed = self.start_time.elapsed();
        let time_str = format_duration(elapsed);

        // Overwrite line with \r
        eprint!("\rF({}) > {} ({})   ", self.n, candidate, time_str);
        let _ = io::stderr().flush();
        self.started = true;
    }

    /// Clear the progress line if we started reporting
    pub fn clear(&self) {
        if self.started {
            eprint!("\r{}\r", " ".repeat(60));
            let _ = io::stderr().flush();
        }
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
    fn test_reporter_respects_delay() {
        let mut reporter = ProgressReporter::new(100, 10.0); // 10s delay
        reporter.report(500); // Should not output (delay not passed)
        assert!(!reporter.started);
    }
}
