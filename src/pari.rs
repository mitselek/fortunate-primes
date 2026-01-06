//! PARI/GP subprocess interface
//!
//! Provides functions to call PARI/GP for primality testing.

use std::io::Write;
use std::process::{Child, Command, Stdio};

/// A running PARI/GP search that can be killed
pub struct PariSearch {
    child: Child,
}

impl PariSearch {
    /// Start a PARI/GP search for first prime in range [start, end]
    pub fn start(n: usize, start: u64, end: u64) -> Result<Self, String> {
        // Ensure start >= 2 (Fortunate numbers must be > 1)
        let actual_start = start.max(2);

        // PARI script: search for first prime in range
        let script = format!(
            "pn=prod(i=1,{},prime(i)); for(m={},{}, if(ispseudoprime(pn+m), print(m); break))\n",
            n, actual_start, end
        );

        let mut child = Command::new("gp")
            .arg("-q")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn PARI/GP: {}", e))?;

        // Write script to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(script.as_bytes())
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
        }

        Ok(Self { child })
    }

    /// Wait for the search to complete and return result
    pub fn wait(self) -> Result<Option<u64>, String> {
        let output = self
            .child
            .wait_with_output()
            .map_err(|e| format!("Failed to wait for PARI/GP: {}", e))?;

        if !output.status.success() {
            // Process was killed or failed - return None, not error
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();

        if trimmed.is_empty() {
            Ok(None)
        } else {
            trimmed
                .parse::<u64>()
                .map(Some)
                .map_err(|e| format!("Failed to parse PARI output '{}': {}", trimmed, e))
        }
    }

    /// Kill the subprocess
    pub fn kill(&mut self) {
        let _ = self.child.kill();
    }
}

/// Search for first prime in range (convenience wrapper)
pub fn search_range(n: usize, start: u64, end: u64) -> Result<Option<u64>, String> {
    let search = PariSearch::start(n, start, end)?;
    search.wait()
}

/// Check if PARI/GP is installed
pub fn check_installation() -> Result<String, String> {
    let output = Command::new("gp")
        .arg("--version-short")
        .output()
        .map_err(|_| "PARI/GP not found. Install with: sudo apt install pari-gp".to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("PARI/GP not working correctly".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_range_finds_prime() {
        // F(5) = 23: primorial(5) = 2310, 2310 + 23 = 2333 is prime
        // Search range [2, 50] should find 23
        let result = search_range(5, 2, 50).unwrap();
        assert_eq!(result, Some(23));
    }

    #[test]
    fn test_search_range_no_prime_in_range() {
        // F(5) = 23, so range [2, 10] should find nothing
        let result = search_range(5, 2, 10).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_search_range_exact_match() {
        // Range [23, 23] should find 23
        let result = search_range(5, 23, 23).unwrap();
        assert_eq!(result, Some(23));
    }

    #[test]
    fn test_f10_equals_61() {
        // F(10) = 61: primorial(10) + 61 is prime
        let result = search_range(10, 2, 100).unwrap();
        assert_eq!(result, Some(61));
    }

    #[test]
    fn test_range_starting_at_1_skips_to_2() {
        // Range [1, 50] should skip 1 and still find 23 for F(5)
        let result = search_range(5, 1, 50).unwrap();
        assert_eq!(result, Some(23));
    }
}
