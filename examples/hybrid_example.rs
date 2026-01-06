// Example: Using the hybrid implementation
// The hybrid approach automatically selects the best backend:
// - Rust: Fast for small n (<200)
// - PARI/GP: Essential for large n (≥200)

use fortunate_primes::hybrid::fortunate_hybrid;

fn main() {
    let n = 100;

    match fortunate_hybrid(n) {
        Ok((fortunate, iterations)) => {
            println!("F({}) = {}", n, fortunate);
            println!("Iterations: {}", iterations);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
