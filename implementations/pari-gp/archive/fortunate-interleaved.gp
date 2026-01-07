/*
 * Fortunate Numbers Calculator - Interleaved Strategy
 * Pure PARI/GP implementation using native parallelism
 *
 * Strategy: Each worker tests every Nth candidate (strided access)
 * Best for: n < 1000 (simple, zero coordination overhead)
 *
 * Usage: gp -q fortunate-interleaved.gp
 */

/* Set number of parallel threads (get system default) */
\\ Note: PARI/GP auto-detects threads, use default(nbthreads) to check

/*
 * Find Fortunate number F(n) using interleaved parallel search
 * Each of N workers tests candidates at stride N
 */
fortunate_interleaved(n) = {
  my(pn, num_threads, results, start_time);
  
  start_time = gettime();
  
  /* Compute primorial(n) once */
  print("Computing primorial(", n, ")...");
  pn = prod(i=1, n, prime(i));
  print("Primorial computed (", #digits(pn), " digits)");
  print("Searching with ", default(nbthreads), " threads (interleaved)...");
  
  num_threads = default(nbthreads);
  
  /* Each worker searches with stride = num_threads
   * Worker 0: tests m=2, 2+N, 2+2N, 2+3N...
   * Worker 1: tests m=3, 3+N, 3+2N, 3+3N...
   * Worker k: tests m=(k+2), (k+2)+kN, (k+2)+2kN...
   */
  results = parapply(worker_id -> my(m = worker_id + 2, stride = num_threads); while(1, if(ispseudoprime(pn + m), return(m)); m += stride), vector(num_threads, i, i));
  
  /* Find minimum (first prime found) */
  fn = vecmin(results);
  elapsed = gettime() - start_time;
  
  print("F(", n, ") = ", fn, " (", elapsed/1000.0, "s)");
  return(fn);
}

/* Self-test with known values */
test_fortunate() = {
  my(tests, i, n, expected, result);
  tests = [[5, 23], [10, 61], [20, 103]];
  
  print("Running self-tests...");
  for(i=1, #tests,
    n = tests[i][1]; 
    expected = tests[i][2];
    result = fortunate_interleaved(n);
    if(result == expected,
      print("✓ F(", n, ") = ", result),
      print("✗ F(", n, ") = ", result, " (expected ", expected, ")")
    );
  );
}

/* Example: Run test suite */
/* test_fortunate(); */

/* Example: Calculate F(500) */
/* fortunate_interleaved(500); */

print("Loaded fortunate-interleaved.gp");
print("Usage: fortunate_interleaved(n)");
print("Tests: test_fortunate()");
