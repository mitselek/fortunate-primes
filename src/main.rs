use fortunate_primes::{
    primes, hybrid, FortunateCalculator, MillerRabin, PrimeBasedCalculator, WheelFortunateCalculator,
};
use std::io::{self, Write};

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     Fortunate Primes Calculator - Performance Testing      ║");
    println!("║                                                            ║");
    println!("║   Phase 1 Optimizations: Parallel & Wheel Factorization    ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Check PARI/GP installation
    match hybrid::check_pari_installation() {
        Ok(version) => println!("✓ PARI/GP {} detected\n", version),
        Err(e) => {
            eprintln!("✗ Error: {}\n", e);
            eprintln!("This program requires PARI/GP for optimal performance.");
            eprintln!("You can still use the Rust implementation (slower).");
            eprintln!("\nContinuing without PARI/GP...\n");
        }
    }

    let prime_list = primes::get_primes();
    println!("Available primes: {}\n", prime_list.len());

    loop {
        println!("\n┌─ Menu ─────────────────────────────────────────────────────┐");
        println!("│ 1. Find Fortunate number (with metrics)                    │");
        println!("│ 2. Find Fortunate number (PARI/GP - faster)                │");
        println!("│ 3. Benchmark different algorithms                         │");
        println!("│ 4. Exit                                                    │");
        println!("└────────────────────────────────────────────────────────────┘");
        print!("\nChoice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read input");

        match choice.trim() {
            "1" => find_fortunate(prime_list),
            "2" => find_fortunate_pari(),
            "3" => benchmark_algorithms(prime_list),
            "4" => {
                println!("\nGoodbye!");
                break;
            }
            _ => println!("Invalid choice"),
        }
    }
}

fn find_fortunate(primes: &[u32]) {
    print!("\nEnter n (1-{}): ", primes.len());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    match input.trim().parse::<usize>() {
        Ok(n) if n > 0 && n <= primes.len() => {
            println!("\nSelect algorithm:");
            println!("  1. Fast (20 rounds)");
            println!("  2. Standard (40 rounds) - default");
            println!("  3. Thorough (64 rounds)");
            print!("Choice (default 2): ");
            io::stdout().flush().unwrap();

            let mut algo_choice = String::new();
            io::stdin()
                .read_line(&mut algo_choice)
                .expect("Failed to read input");

            let tester = match algo_choice.trim() {
                "1" => MillerRabin::fast(),
                "3" => MillerRabin::thorough(),
                _ => MillerRabin::with_default_rounds(),
            };

            let mut calc = PrimeBasedCalculator::with_tester(primes.to_vec(), tester);
            calc.set_max_candidate(1000000);

            match calc.fortunate_number_with_metrics(n) {
                Ok((f, metrics)) => {
                    println!("\n┌─ Results ────────────────────────────────────────────────────┐");
                    println!("│ Fortunate number for n={}: {}", n, f);
                    println!("├──────────────────────────────────────────────────────────────┤");
                    println!("│ Primorial calculation:     {:?}", metrics.primorial_time);
                    println!(
                        "│ Primality tests run:       {}",
                        metrics.primality_test_count
                    );
                    println!(
                        "│ Primality tests passed:    {}",
                        metrics.primality_tests_passed
                    );
                    println!("│ Total time:                {:?}", metrics.total_time);
                    println!("└──────────────────────────────────────────────────────────────┘");
                }
                Err(e) => eprintln!("\n✗ Error: {}", e),
            }
        }
        Ok(_) => {
            eprintln!("\n✗ n must be between 1 and {}", primes.len());
        }
        Err(_) => {
            eprintln!("\n✗ Invalid input");
        }
    }
}

fn find_fortunate_pari() {
    let primes = primes::get_primes();
    print!("\nEnter n (1-{}): ", primes.len());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    match input.trim().parse::<usize>() {
        Ok(n) if n > 0 && n <= primes.len() => {
            println!("\nCalculating F({}) using PARI/GP...", n);

            let start = std::time::Instant::now();
            match hybrid::fortunate_pari_calculate(n) {
                Ok((f, iterations)) => {
                    let elapsed = start.elapsed();
                    println!("\n┌─ Results (PARI/GP) ──────────────────────────────────────────┐");
                    println!("│ Fortunate number for n={}: {}", n, f);
                    println!("├──────────────────────────────────────────────────────────────┤");
                    println!("│ Iterations:                {}", iterations);
                    println!("│ Total time:                {:?}", elapsed);
                    println!("└──────────────────────────────────────────────────────────────┘");
                }
                Err(e) => {
                    eprintln!("\n✗ Error: {}", e);
                    eprintln!("Make sure PARI/GP is installed: sudo apt install pari-gp");
                }
            }
        }
        Ok(_) => {
            eprintln!("\n✗ n must be between 1 and {}", primes.len());
        }
        Err(_) => {
            eprintln!("\n✗ Invalid input");
        }
    }
}

fn benchmark_algorithms(primes: &[u32]) {
    print!("\nEnter n (1-{}): ", primes.len());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    match input.trim().parse::<usize>() {
        Ok(n) if n > 0 && n <= primes.len() => {
            let algorithms = vec![
                ("Standard (40 rounds)", MillerRabin::with_default_rounds()),
                ("Fast (20 rounds)", MillerRabin::fast()),
                ("Thorough (64 rounds)", MillerRabin::thorough()),
            ];

            println!(
                "\n┌─ Benchmarking n={} ─────────────────────────────────┐",
                n
            );

            println!("│ STANDARD IMPLEMENTATION                              │");
            for (name, tester) in &algorithms {
                let mut calc = PrimeBasedCalculator::with_tester(primes.to_vec(), tester.clone());
                calc.set_max_candidate(1000000);

                match calc.fortunate_number_with_metrics(n) {
                    Ok((f, metrics)) => {
                        println!("│ {} ─────────────────────────────────────────", name);
                        println!("│   Result: {}                  ", f);
                        println!("│   Time: {:?}          ", metrics.total_time);
                        println!(
                            "│   Tests: {}/{}               ",
                            metrics.primality_tests_passed, metrics.primality_test_count
                        );
                    }
                    Err(e) => {
                        println!("│ {} ERROR: {}", name, e);
                    }
                }
            }

            println!("│                                                    │");
            println!("│ WHEEL FACTORIZATION OPTIMIZED                     │");
            for (name, tester) in &algorithms {
                let mut calc =
                    WheelFortunateCalculator::with_tester(primes.to_vec(), tester.clone());
                calc.set_max_candidate(1000000);

                match calc.fortunate_number_with_metrics(n) {
                    Ok((f, metrics)) => {
                        println!("│ {} (wheel)  ─────────────────────────────", name);
                        println!("│   Result: {}                  ", f);
                        println!("│   Time: {:?}          ", metrics.total_time);
                        println!(
                            "│   Tests: {}/{}               ",
                            metrics.primality_tests_passed, metrics.primality_test_count
                        );
                    }
                    Err(e) => {
                        println!("│ {} ERROR: {}", name, e);
                    }
                }
            }
            println!("└────────────────────────────────────────────────────────┘");
        }
        Ok(_) => {
            eprintln!("\n✗ n must be between 1 and {}", primes.len());
        }
        Err(_) => {
            eprintln!("\n✗ Invalid input");
        }
    }
}
