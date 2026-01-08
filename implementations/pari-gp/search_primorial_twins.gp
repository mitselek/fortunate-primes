/* Search for primorial twin primes up to n=10000
 * OEIS A088256: Primorial numbers k such that both k-1 and k+1 are prime
 * 
 * Known terms:
 *   n=2: primorial(2)=6, twins (5,7)
 *   n=3: primorial(3)=30, twins (29,31)
 *   n=5: primorial(5)=2310, twins (2309,2311)
 *
 * Two-stage approach:
 * 1. Fast pseudoprime filter (milliseconds)
 * 2. Deterministic isprime() only for candidates passing stage 1
 */

default(realprecision, 100000);

{
  local(n, p, tm, found_count, digit_count, t0);
  
  found_count = 0;
  print("Searching for primorial twin primes (n=5000 to 5100)");
  print("Using two-stage filter: pseudoprime → isprime");
  print();
  
  for(n = 5000, 5100,
    t0 = gettime();
    p = vecprod(primes(n));
    digit_count = #Str(p);
    
    /* Stage 1: Fast pseudoprime filter */
    if(ispseudoprime(p-1) && ispseudoprime(p+1),
      /* Stage 2: Slow deterministic test only for candidates */
      if(isprime(p-1) && isprime(p+1),
        found_count++;
        printf("*** FOUND TWIN: n=%d, primorial(%d) has %d digits ***\n", n, n, digit_count);
      ,
        tm = gettime() - t0;
        printf("[n=%5d] %6d digits | pseudoprime only | %6d ms\n", n, digit_count, tm);
      );
    ,
      tm = gettime() - t0;
      printf("[n=%5d] %6d digits | composite | %6d ms\n", n, digit_count, tm);
    );
  );
  
  print();
  print("========================================");
  printf("Search complete: %d primorial twins found\n", found_count);
  print("========================================");
}
