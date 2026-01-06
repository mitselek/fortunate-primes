/* PARI/GP script to calculate Fortunate numbers
 * Compare performance with Rust implementation
 */

/* Calculate primorial P#(n) */
primorial(n) = {
    local(result, p);
    result = 1;
    forprime(p = 2, prime(n),
        result *= p;
    );
    return(result);
}

/* Find Fortunate number F(n) */
fortunate(n) = {
    local(pn, candidate, rounds);
    pn = primorial(n);
    candidate = pn + 1;
    rounds = 0;
    
    /* Find first prime after P#(n) */
    while(!ispseudoprime(candidate),
        candidate++;
        rounds++;
    );
    
    return([candidate - pn, rounds]);  /* Return [F(n), iterations] */
}

/* Benchmark function */
benchmark(n) = {
    local(start, result, elapsed);
    print("Calculating F(", n, ")...");
    start = getabstime();
    result = fortunate(n);
    elapsed = getabstime() - start;
    
    print("F(", n, ") = ", result[1]);
    print("Iterations: ", result[2]);
    print("Time: ", elapsed, " ms");
    print("");
    return(result);
}

/* Run benchmarks */
print("=== PARI/GP Fortunate Number Benchmark ===");
print("");
benchmark(10);
benchmark(50);
benchmark(100);
benchmark(123);
benchmark(200);
benchmark(300);
benchmark(400);
