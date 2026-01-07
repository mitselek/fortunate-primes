/*
 * Fortunate Numbers Calculator - Interleaved Strategy (Simplified)
 * Pure PARI/GP implementation
 *
 * Strategy: Test all candidates sequentially (simple parallel search)
 * Best for: n < 1000
 *
 * Usage: gp -q fortunate-simple.gp
 */

/* Worker function: search from start with given stride */
search_strided(pn, start, stride) = {
  my(m);
  m = start;
  while(1,
    if(ispseudoprime(pn + m), return(m));
    m += stride
  );
}
export(search_strided);

/*
 * Find Fortunate number F(n) using parallel interleaved search
 */
fortunate(n) = {
  my(pn, num_threads, results, fn, elapsed, start_time);
  
  start_time = gettime();
  
  /* Compute primorial(n) once */
  print("Computing primorial(", n, ")...");
  pn = prod(i=1, n, prime(i));
  print("Primorial computed (", #digits(pn), " digits)");
  
  num_threads = default(nbthreads);
  print("Searching with ", num_threads, " threads...");
  
  /* Launch workers with different starting offsets */
  results = parapply(i -> search_strided(pn, i + 1, num_threads), vector(num_threads, j, j));
  
  /* Find minimum (first prime found) */
  fn = vecmin(results);
  elapsed = gettime() - start_time;
  
  print("F(", n, ") = ", fn, " (", elapsed/1000.0, "s)");
  return(fn);
}

/* Self-test with known values */
test() = {
  my(tests, i, n, expected, result);
  tests = [[5, 23], [10, 61], [20, 103]];
  
  print("Running self-tests...");
  for(i=1, #tests,
    n = tests[i][1]; 
    expected = tests[i][2];
    result = fortunate(n);
    if(result == expected,
      print("✓ F(", n, ") = ", result),
      print("✗ F(", n, ") = ", result, " (expected ", expected, ")")
    );
  );
}

print("Loaded fortunate-simple.gp");
print("Usage: fortunate(n) or test()");
