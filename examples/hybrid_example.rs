// Example integration into main.rs

use hybrid::fortunate_hybrid;

// In your menu, add a new option:

println!("Select implementation:");
println!("  1. Rust only (current implementation)");
println!("  2. Hybrid (Rust for n<200, PARI/GP for n≥200) - recommended");
println!("  3. PARI/GP only (requires installation)");

match choice {
    1 => {
        // Your current implementation
        let result = calculate_fortunate_rust(n);
    }
    2 => {
        // Hybrid approach - best of both worlds
        match fortunate_hybrid(n) {
            Ok((fortunate, iterations)) => {
                println!("F({}) = {}", n, fortunate);
                println!("Iterations: {}", iterations);
                println!("(Used {} backend)", if n < 200 { "Rust" } else { "PARI/GP" });
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                eprintln!("Falling back to Rust implementation...");
                // Fallback to pure Rust
            }
        }
    }
    3 => {
        // Pure PARI/GP
        match fortunate_pari(n) {
            Ok((fortunate, iterations)) => {
                println!("F({}) = {}", n, fortunate);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
