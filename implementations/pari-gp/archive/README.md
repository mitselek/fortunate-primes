# Archived PARI/GP Prototypes

This directory contains abandoned prototypes from Issue #11 development. These files are kept for documentation purposes to show what **doesn't work** in PARI/GP parallelism.

## Files

- **fortunate-simple.gp** - Interleaved strategy with closure over primorial → **Hangs indefinitely**
- **fortunate-interleaved.gp** - Multi-line lambda attempt → **Syntax error**
- **fortunate-batch.gp** - Inline `my()` declarations → **Parser failure**
- **fortunate-par.gp** - Complex closure patterns → **Syntax error**

## Why These Failed

### Common PARI/GP Parallelism Issues

1. **Closures over large integers**:

   - Problem: Passing primorial via closure in `parapply()`
   - Result: Workers hang indefinitely (no error, just freeze)
   - Example: `fortunate-simple.gp`

2. **Multi-line lambda expressions**:

   - Problem: PARI/GP parser doesn't support complex lambdas in `parapply()`
   - Result: Syntax errors
   - Example: `fortunate-interleaved.gp`

3. **Inline variable declarations**:

   - Problem: `my()` declarations inside `parapply()` lambda
   - Result: Parser failure
   - Example: `fortunate-batch.gp`

4. **Complex function composition**:
   - Problem: Nested functions, shared state patterns
   - Result: Various syntax errors
   - Example: `fortunate-par.gp`

## What Works (See ../fortunate.gp)

**Pattern that succeeds:**

```gp
/* Define worker function separately */
test_batch(n, start, batch_size) = {
  my(pn, m, end);
  pn = prod(i=1, n, prime(i));  /* Each worker recomputes */
  end = start + batch_size;
  for(m=start, end-1, if(ispseudoprime(pn + m), return(m)));
  return(0);
}
export(test_batch);  /* REQUIRED for parallel execution */

/* Coordinator with simple lambda */
fortunate_batch(n, batch_size=100) = {
  /* ... setup ... */
  results = parapply(i -> test_batch(n, i*batch_size, batch_size),
                     vector(num_workers));
  /* ... process results ... */
}
```

**Key principles:**

1. ✅ Export worker functions explicitly
2. ✅ Pass all data as parameters (no closures over large data)
3. ✅ Keep lambdas simple (single function call)
4. ✅ Accept recomputation overhead (primorial per worker)

## Lessons Learned

- PARI/GP parallelism is **powerful but restrictive**
- Keep worker functions **simple and independent**
- **Recomputing** expensive values often cheaper than **coordination**
- When in doubt, **test sequentially first** (fortunate-seq.gp)

## References

- Working implementation: [../fortunate.gp](../fortunate.gp)
- Benchmarks: [../BENCHMARKS.md](../BENCHMARKS.md)
- Issue: [#11](https://github.com/mitselek/fortunate-primes/issues/11)
