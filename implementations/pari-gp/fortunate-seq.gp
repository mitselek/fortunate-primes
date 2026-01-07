/*
 * Fortunate Numbers Calculator - Sequential (for debugging)
 * Pure PARI/GP implementation - single threaded
 */

fortunate_seq(n) = {
  my(pn, m, start_time, elapsed);
  
  start_time = gettime();
  
  /* Compute primorial(n) */
  print("Computing primorial(", n, ")...");
  pn = prod(i=1, n, prime(i));
  print("Primorial computed (", #digits(pn), " digits)");
  print("Searching sequentially...");
  
  /* Sequential search */
  m = 2;
  while(1,
    if(ispseudoprime(pn + m), break());
    m++
  );
  
  elapsed = gettime() - start_time;
  print("F(", n, ") = ", m, " (", elapsed/1000.0, "s)");
  return(m);
}

/* Quick test */
test() = {
  my(tests, i, n, expected, result);
  tests = [[5, 23], [10, 61]];
  
  for(i=1, #tests,
    n = tests[i][1]; 
    expected = tests[i][2];
    result = fortunate_seq(n);
    if(result == expected,
      print("✓"),
      print("✗ expected ", expected)
    );
  );
}

print("Loaded. Usage: fortunate_seq(n) or test()");
