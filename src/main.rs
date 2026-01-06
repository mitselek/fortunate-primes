use fortunate_primes::progress::format_duration;
use fortunate_primes::search;
use std::env;
use std::process;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <n>", args[0]);
        eprintln!("\nCalculates Fortunate number F(n).");
        eprintln!("Requires PARI/GP: sudo apt install pari-gp");
        process::exit(1);
    }

    let n: usize = match args[1].parse() {
        Ok(num) if num > 0 => num,
        _ => {
            eprintln!("Error: n must be a positive integer");
            process::exit(1);
        }
    };

    let start = Instant::now();

    match search::find_fortunate(n) {
        Ok(result) => {
            let elapsed = start.elapsed();
            println!("F({}) = {}", n, result);
            println!("time: {}", format_duration(elapsed));
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
