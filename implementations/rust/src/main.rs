use fortunate_primes::search;
use std::env;
use std::process;

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

    match search::find_fortunate(n) {
        Ok(_) => {
            // Output already printed by reporter
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
