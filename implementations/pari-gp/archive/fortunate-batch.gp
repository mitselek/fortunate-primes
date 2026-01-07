/*
 * Fortunate Numbers Calculator - Batch Strategy with Fixed Size
 * Pure PARI/GP implementation using native parallelism
 *
 * Strategy: Workers pull consecutive-candidate batches from range
 * Best for: n ≥ 1000 (cache locality, early termination)
 *
 * Usage: gp -q fortunate-batch.gp
 */

/* Set number of parallel threads (get system default) */
\\ Note: PARI/GP auto-detects threads, use default(nbthreads) to check

/*
 * Search batch [start, start+batch_size) for prime
 * Returns [found_m, is_prime_found]
 */
search_batch(pn, start, batch_size) = {
  my(end, m);
  end = start + batch_size - 1;
  for(m = start, end,
    if(ispseudoprime(pn + m), return([m, 1]))
  );
  return([0, 0]); \\ Not found in this batch
}

/*
 * Find Fortunate number F(n) using batch-based parallel search
 * batch_size: From Rust experiments, 64-128 optimal for n~1000
 */
fortunate_batch(n, batch_size=100) = {
  my(pn, num_threads, start_time, current_batch, max_batches);
  
  start_time = gettime();
  
  /* Compute primorial(n) once */
  print("Computing primorial(", n, ")...");
  pn = prod(i=1, n, prime(i));
  print("Primorial computed (", #digits(pn), " digits)");
  print("Searching with ", default(nbthreads), " threads (batch size ", batch_size, ")...");
  
  num_threads = default(nbthreads);
  current_batch = 0;
  max_batches = 100000; \\ Safety limit
  
  /* Process batches until prime found */
  for(batch_round = 1, max_batches,
    /* Generate batch starting positions for this round */
    batch_starts = vector(num_threads, i, 2 + (current_batch + i - 1) * batch_size);
    
    /* Search batches in parallel */
    results = parapply(start -> search_batch(pn, start, batch_size), batch_starts);
    
    /* Check if any worker found a prime */
    for(i = 1, #results,
      if(results[i][2] == 1,  \\ Prime found
        fn = results[i][1];
        elapsed = gettime() - start_time;
        print("F(", n, ") = ", fn, " (", elapsed/1000.0, "s, ", batch_round, " rounds)");
        return(fn)
      )
    );
    
    /* Progress report every 10 rounds */
    if(batch_round % 10 == 0,
      tested = current_batch * batch_size;
      elapsed = gettime() - start_time;
      print("Tested up to m=", tested, " (", elapsed/1000.0, "s, ", batch_round, " rounds)")
    );
    
    current_batch += num_threads
  );
  
  error("No prime found within search limit");
}

/* Self-test with known values */
test_fortunate() = {
  my(tests, i, n, expected, result);
  tests = [[5, 23], [10, 61], [20, 103]];
  
  print("Running self-tests...");
  for(i=1, #tests,
    n = tests[i][1]; 
    expected = tests[i][2];
    result = fortunate_batch(n, 50); \\ Small batch for tests
    if(result == expected,
      print("✓ F(", n, ") = ", result),
      print("✗ F(", n, ") = ", result, " (expected ", expected, ")")
    );
  );
}

/* Example: Run test suite */
/* test_fortunate(); */

/* Example: Calculate F(500) with different batch sizes */
/* fortunate_batch(500, 50);  \\ Small batches */
/* fortunate_batch(500, 100); \\ Medium batches (default) */
/* fortunate_batch(500, 200); \\ Large batches */

print("Loaded fortunate-batch.gp");
print("Usage: fortunate_batch(n, batch_size)");
print("Tests: test_fortunate()");
print("Suggested batch sizes: 50-100 for n~500, 100-200 for n~1000");
