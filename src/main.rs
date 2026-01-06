use fortunate_primes::hybrid;
use std::env;
use std::process;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} [OPTIONS] <n>", args[0]);
        eprintln!("\nCalculates the Fortunate number F(n) using PARI/GP.");
        eprintln!("  n: positive integer (1-10000, requires PARI/GP installed)");
        eprintln!("\nOptions:");
        eprintln!("  --parallel         Use parallel search (multi-core)");
        eprintln!("  --workers N        Number of parallel workers (default: auto-detect)");
        eprintln!("\nExamples:");
        eprintln!("  {} 123               # Sequential search", args[0]);
        eprintln!("  {} --parallel 123    # Parallel search (16 workers)", args[0]);
        eprintln!("  {} --parallel --workers 4 123  # Parallel with 4 workers", args[0]);
        eprintln!("\nFor more details, see README.md");
        process::exit(1);
    }

    let mut n: usize = 0;
    let mut use_parallel = false;
    let mut num_workers: Option<usize> = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--parallel" => use_parallel = true,
            "--workers" => {
                i += 1;
                if i < args.len() {
                    match args[i].parse() {
                        Ok(workers) => num_workers = Some(workers),
                        Err(_) => {
                            eprintln!("Error: '--workers' requires a positive integer");
                            process::exit(1);
                        }
                    }
                } else {
                    eprintln!("Error: '--workers' requires an argument");
                    process::exit(1);
                }
            }
            arg if !arg.starts_with("--") => {
                match arg.parse() {
                    Ok(num) => n = num,
                    Err(_) => {
                        eprintln!("Error: '{}' is not a valid positive integer", arg);
                        process::exit(1);
                    }
                }
            }
            arg => {
                eprintln!("Error: Unknown option '{}'", arg);
                process::exit(1);
            }
        }
        i += 1;
    }

    if n == 0 {
        eprintln!("Error: n must be a positive integer (> 0)");
        process::exit(1);
    }

    let start = Instant::now();
    let result = if use_parallel {
        hybrid::fortunate_pari_parallel(n, num_workers)
    } else {
        hybrid::fortunate_pari_calculate(n)
    };

    match result {
        Ok((f, iterations)) => {
            let elapsed = start.elapsed();
            let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
            let per_iteration_ms = elapsed_ms / (iterations as f64 + 1.0);

            println!("F({}) = {}", n, f);
            println!("time: {:.1}ms", elapsed_ms);
            println!("per_iteration: {:.3}ms", per_iteration_ms);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
