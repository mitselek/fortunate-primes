/*
 * Fortunate Numbers Calculator - Parallel with primorial closure
 * Pure PARI/GP - each worker computes primorial independently
 */

/* Worker: compute primorial and search from offset with stride */
worker_search(n, offset, stride) = {
  my(pn, m);
  pn = prod(i=1, n, prime(i));
  m = offset;
  while(1,
    if(ispseudoprime(pn + m), return(m));
    m += stride
  );
}
export(worker_search);

fortunate_par(n, num_workers=0) = {
  my(start_time, num_threads, pn_digits, tasks, results, fn, elapsed);
  
  start_time = gettime();
  
  /* Get thread count */
  if(num_workers == 0, num_workers = default(nbthreads));
  
  /* Compute primorial once just to show progress */
  print("Computing primorial(", n, ")...");
  my(pn_temp = prod(i=1, n, prime(i)));
  pn_digits = #digits(pn_temp);
  print("Primorial computed (", pn_digits, " digits)");
  print("Launching ", num_workers, " parallel workers...");
  
  /* Create worker tasks: each gets (n, starting_offset, stride) */
  tasks = vector(num_workers, i, [n, i + 1, num_workers]);
  
  /* Run in parallel - each worker recomputes primorial */
  results = parapply(task -> worker_search(task[1], task[2], task[3]), tasks);
  
  /* Find minimum */
  fn = vecmin(results);
  elapsed = gettime() - start_time;
  
  print("F(", n, ") = ", fn, " (", elapsed/1000.0, "s, ", num_workers, " workers)");
  return(fn);
}

/* Test */
test() = {
  my(tests, i, n, expected, result);
  tests = [[5, 23], [10, 61], [20, 103]];
  
  print("Running tests...");
  for(i=1, #tests,
    n = tests[i][1]; 
    expected = tests[i][2];
    result = fortunate_par(n, 4); \\ Use 4 workers for tests
    if(result == expected,
      print("✓ F(", n, ") = ", result),
      print("✗ F(", n, ") = ", result, " (expected ", expected, ")")
    );
  );
}

print("Loaded. Usage: fortunate_par(n) or test()");
