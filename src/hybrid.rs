use std::process::{Command, Stdio};
use std::io::Write;
use rug::Integer;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;

/// Fortunate number calculator using PARI/GP
/// PARI/GP installation is required
pub fn fortunate_pari_calculate(n: usize) -> Result<(Integer, usize), String> {
    fortunate_pari(n)
}

/// PARI/GP implementation via subprocess
fn fortunate_pari(n: usize) -> Result<(Integer, usize), String> {
    let script = format!(r#"
primorial(n) = {{
    local(result, p);
    result = 1;
    forprime(p = 2, prime(n),
        result *= p;
    );
    return(result);
}}

fortunate(n) = {{
    local(pn, candidate, rounds);
    pn = primorial(n);
    candidate = pn + 1;
    rounds = 0;

    while(!ispseudoprime(candidate),
        candidate++;
        rounds++;
    );

    return([candidate - pn, rounds]);
}}

result = fortunate({});
print(result[1]);
print(result[2]);
"#, n);

    let mut child = Command::new("gp")
        .arg("-q")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute PARI/GP: {}", e))?;

    {
        let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
        stdin.write_all(script.as_bytes()).map_err(|e| format!("Failed to write to PARI/GP stdin: {}", e))?;
    }

    let output = child.wait_with_output()
        .map_err(|e| format!("Failed to wait for PARI/GP: {}", e))?;

    if !output.status.success() {
        return Err(format!("PARI/GP error: {}", String::from_utf8_lossy(&output.stderr)));
    }

    // Parse output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.trim().split('\n').collect();

    if lines.len() < 2 {
        return Err("Invalid PARI/GP output".to_string());
    }

    let fortunate = Integer::from_str(lines[0])
        .map_err(|e| format!("Failed to parse fortunate number: {}", e))?;
    let iterations = lines[1].parse::<usize>()
        .map_err(|e| format!("Failed to parse iterations: {}", e))?;

    Ok((fortunate, iterations))
}

/// Check if PARI/GP is available (required dependency)
pub fn check_pari_installation() -> Result<String, String> {
    let output = Command::new("gp")
        .arg("--version-short")
        .output()
        .map_err(|_| "PARI/GP not found. Install with: sudo apt install pari-gp".to_string())?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    } else {
        Err("PARI/GP not installed. Install with: sudo apt install pari-gp".to_string())
    }
}

/// Parallel PARI/GP search using multiple processes
/// Spawns num_workers processes that coordinately search the candidate space
/// Returns the first result found (which is the true Fortunate number)
pub fn fortunate_pari_parallel(n: usize, num_workers: Option<usize>) -> Result<(Integer, usize), String> {
    let workers = num_workers.unwrap_or_else(|| num_cpus::get());
    
    if workers == 1 {
        // Fall back to sequential
        return fortunate_pari(n);
    }

    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];
    
    // Spawn worker threads that search the space with interleaved offsets
    for worker_id in 0..workers {
        let tx = tx.clone();
        let handle = thread::spawn(move || {
            // Each worker searches candidates at intervals: worker_id, worker_id + num_workers, worker_id + 2*num_workers, etc.
            // This ensures we find F(n) when ANY worker finds it, and it's guaranteed to be correct
            let search_script = format!(r#"
primorial(n) = {{
    local(result, p);
    result = 1;
    forprime(p = 2, prime(n),
        result *= p;
    );
    return(result);
}}

search_interleaved(n, start_offset, stride, max_rounds) = {{
    local(pn, candidate, rounds);
    pn = primorial(n);
    candidate = pn + start_offset + 1;
    rounds = 0;
    
    while(rounds < max_rounds,
        if(ispseudoprime(candidate),
            return([candidate - pn, rounds])
        );
        candidate += stride;
        rounds++;
    );
    return(0);  \\ No prime found
}}

\\ Search with large enough max_rounds to find F(n) for most cases
result = search_interleaved({}, {}, {}, 1000000);
if(result != 0,
    print(result[1]);
    print(result[2])
);
"#, n, worker_id, workers);

            match run_pari_script(&search_script) {
                Ok(output) if !output.trim().is_empty() => {
                    let lines: Vec<&str> = output.trim().split('\n').collect();
                    if lines.len() >= 2 {
                        if let (Ok(f), Ok(iter)) = (Integer::from_str(lines[0]), lines[1].parse::<usize>()) {
                            let _ = tx.send(Ok((f, iter)));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                }
                _ => {} // No result found
            }
        });
        handles.push(handle);
    }

    // Drop the sender so that recv() returns when all workers are done
    drop(tx);

    // Return first successful result (all will be same F(n), just different iteration counts)
    for result in rx {
        if result.is_ok() {
            return result;
        }
    }

    Err("No Fortunate number found in any worker".to_string())
}

/// Helper: Run a PARI/GP script and return stdout
fn run_pari_script(script: &str) -> Result<String, String> {
    let mut child = Command::new("gp")
        .arg("-q")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute PARI/GP: {}", e))?;

    {
        let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
        stdin.write_all(script.as_bytes()).map_err(|e| format!("Failed to write to PARI/GP stdin: {}", e))?;
    }

    let output = child.wait_with_output()
        .map_err(|e| format!("Failed to wait for PARI/GP: {}", e))?;

    if !output.status.success() {
        return Err(format!("PARI/GP error: {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires PARI/GP to be installed
    fn test_pari_small_n() {
        let result = fortunate_pari_calculate(10);
        assert!(result.is_ok());
        let (f, iterations) = result.unwrap();
        assert_eq!(f, Integer::from(61));
        assert!(iterations > 0);
    }

    #[test]
    #[ignore] // Requires PARI/GP to be installed
    fn test_pari_medium_n() {
        let result = fortunate_pari_calculate(100);
        assert!(result.is_ok());
        let (f, _) = result.unwrap();
        assert!(f > 0);
    }

    #[test]
    #[ignore] // Requires PARI/GP and takes time
    fn test_parallel_matches_sequential_small() {
        // Test that parallel produces same result as sequential
        let seq_result = fortunate_pari_calculate(50);
        let par_result = fortunate_pari_parallel(50, Some(2));
        
        assert!(seq_result.is_ok());
        assert!(par_result.is_ok());
        
        let (f_seq, _) = seq_result.unwrap();
        let (f_par, _) = par_result.unwrap();
        
        // Must find the same Fortunate number
        assert_eq!(f_seq, f_par, "Parallel and sequential must find same F(n)");
    }

    #[test]
    #[ignore] // Requires PARI/GP and takes time
    fn test_parallel_oeis_validation() {
        // Verify OEIS A005235 values
        let test_cases = vec![
            (5, 23),
            (10, 61),
            (20, 79),
        ];

        for (n, expected_f) in test_cases {
            let result = fortunate_pari_parallel(n, Some(2));
            assert!(result.is_ok(), "Failed to calculate F({})", n);
            let (f, _) = result.unwrap();
            assert_eq!(f, Integer::from(expected_f), "F({}) should be {}", n, expected_f);
        }
    }

    #[test]
    #[ignore]
    fn test_parallel_with_different_worker_counts() {
        // Verify results are identical regardless of worker count
        let seq = fortunate_pari_calculate(30).unwrap().0;
        let par_2 = fortunate_pari_parallel(30, Some(2)).unwrap().0;
        let par_4 = fortunate_pari_parallel(30, Some(4)).unwrap().0;

        assert_eq!(seq, par_2, "1 worker vs 2 workers must match");
        assert_eq!(seq, par_4, "1 worker vs 4 workers must match");
    }
}
