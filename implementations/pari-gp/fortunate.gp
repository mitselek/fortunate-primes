/*
 * Fortunate Numbers - Fixed batch parallel search
 * KISS principle: simple fixed-size batches
 */

/* Worker: compute primorial and test batch [start, end) */
test_batch(n, start, batch_size) = {
  my(pn, m, end);
  pn = prod(i=1, n, prime(i));
  end = start + batch_size;
  for(m=start, end-1,
    if(ispseudoprime(pn + m), return(m))
  );
  return(0); \\ Not found
}
export(test_batch);

fortunate_batch(n, batch_size=100) = {
  my(start_time, num_threads, batch_round, pn_temp, pn_digits, batch_starts, results, i, fn, elapsed, tested);
  
  start_time = gettime();
  
  /* Show primorial size */
  print("Computing primorial(", n, ")...");
  pn_temp = prod(i=1, n, prime(i));
  pn_digits = #digits(pn_temp);
  pn_temp = 0; \\ Free memory
  print("Primorial: ", pn_digits, " digits");
  
  num_threads = default(nbthreads);
  print("Searching: ", num_threads, " workers, batch_size=", batch_size);
  
  /* Process batches until found */
  batch_round = 0;
  while(1,
    /* Generate batch start positions */
    batch_starts = vector(num_threads, i, 2 + (batch_round * num_threads + i - 1) * batch_size);
    
    /* Test batches in parallel */
    results = parapply(start_pos -> test_batch(n, start_pos, batch_size), batch_starts);
    
    /* Check results */
    for(i=1, #results,
      if(results[i] > 0,
        fn = results[i];
        elapsed = gettime() - start_time;
        print("F(", n, ") = ", fn, " (", elapsed/1000.0, "s, ", batch_round+1, " rounds)");
        return(fn)
      )
    );
    
    batch_round++;
    
    /* Progress every 10 rounds */
    if(batch_round % 10 == 0,
      tested = batch_round * num_threads * batch_size;
      elapsed = gettime() - start_time;
      print("Round ", batch_round, ": tested ~", tested, " (", elapsed/1000.0, "s)")
    )
  );
}

/* Test */
test() = {
  my(tests, i, n, expected, result);
  tests = [[5, 23], [10, 61]];
  
  for(i=1, #tests,
    n = tests[i][1]; 
    expected = tests[i][2];
    result = fortunate_batch(n, 20);
    if(result == expected,
      print("✓ F(", n, ") = ", result),
      print("✗ got ", result, " expected ", expected)
    )
  );
}

print("Loaded. Usage: fortunate_batch(n, batch_size) or test()");
