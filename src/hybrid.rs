use std::process::Command;
use rug::Integer;
use std::str::FromStr;

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

    let output = Command::new("gp")
        .arg("-q")
        .arg("-c")
        .arg(&script)
        .output()
        .map_err(|e| format!("Failed to execute PARI/GP: {}", e))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_small_n() {
        // Should use Rust implementation
        let result = fortunate_hybrid(10);
        assert!(result.is_ok());
        let (f, _) = result.unwrap();
        assert_eq!(f, Integer::from(61));
    }

    #[test]
    #[ignore] // Only run if PARI/GP is installed
    fn test_hybrid_large_n() {
        // Should use PARI/GP implementation
        let result = fortunate_hybrid(300);
        assert!(result.is_ok());
        let (f, _) = result.unwrap();
        assert_eq!(f, Integer::from(5641));
    }

    #[test]
    #[ignore]
    fn test_pari_performance() {
        use std::time::Instant;

        let start = Instant::now();
        let result = fortunate_pari(400);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        println!("F(400) via PARI/GP: {:?} in {:?}", result.unwrap().0, elapsed);
        // Should be ~10s instead of ~30s with pure Rust
    }
}
