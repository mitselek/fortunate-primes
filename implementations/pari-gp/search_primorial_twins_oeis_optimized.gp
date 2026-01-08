(PARI) lista(nn) = {
  for (n=1, nn, 
    pr = prod(i=1, n, prime(i));
    \\ Stage 1: Fast pseudoprime filter
    if (ispseudoprime(pr-1) && ispseudoprime(pr+1),
      \\ Stage 2: Deterministic confirmation
      if (isprime(pr-1) && isprime(pr+1),
        print1(pr, ", ")
      )
    )
  );
};
\\ Example usage: lista(5100)
