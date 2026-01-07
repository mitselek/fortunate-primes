"""
Fortunate Numbers Calculator - Python + gmpy2 implementation.

Uses gmpy2 (GMP bindings) for efficient big integer arithmetic and
primality testing, with multiprocessing for parallel worker distribution.

References:
- OEIS A005235: https://oeis.org/A005235
- Fortune's conjecture: All Fortunate numbers are prime
- gmpy2 docs: https://gmpy2.readthedocs.io/
"""

import gmpy2  # type: ignore
from multiprocessing import Pool, cpu_count
from typing import Tuple, Optional
import sys
import time


def test_batch(args: Tuple[int, int, int]) -> Optional[int]:
    """
    Worker function: Test batch [start, start+batch_size) for primality.
    
    Each worker recomputes the primorial independently (acceptable overhead
    vs IPC serialization cost for large integers).
    
    Args:
        args: Tuple of (n, start, batch_size)
    
    Returns:
        The first m where primorial(n) + m is prime, or None if batch exhausted
    """
    n, start, batch_size = args
    
    # Compute primorial(n) = product of first n primes
    # Generate first n primes using gmpy2.next_prime()
    pn: int = 1
    p: int = 2  # First prime
    for _ in range(n):
        pn *= p
        p = int(gmpy2.next_prime(p))  # type: ignore[attr-defined]
    
    # Test candidates in this batch
    end = start + batch_size
    for m in range(start, end):
        candidate = pn + m
        # Use probabilistic primality test (25 rounds Miller-Rabin)
        if gmpy2.is_prime(candidate, 25):  # type: ignore[attr-defined]
            return m
    
    return None


def fortunate_batch(n: int, batch_size: int = 100, verbose: bool = True) -> int:
    """
    Find Fortunate number F(n) using parallel batch search.
    
    Distributes batches of candidates across worker processes, where each
    batch tests consecutive values m where primorial(n) + m might be prime.
    
    Args:
        n: Index of the Fortunate number (F(n))
        batch_size: Size of each batch distributed to workers
        verbose: Print progress updates
    
    Returns:
        F(n): The smallest prime of form primorial(n) + m where m > 1
    
    Examples:
        >>> fortunate_batch(5, batch_size=100)
        23
        >>> fortunate_batch(10, batch_size=100)
        61
    """
    start_time = time.time()
    num_workers = cpu_count()
    
    if verbose:
        print(f"Computing F({n})...")
        print(f"Using {num_workers} workers, batch_size={batch_size}")
    
    with Pool(num_workers) as pool:
        round_num = 0
        
        while True:
            # Generate batch arguments for this round
            # Start from m=2 (Fortune's definition: smallest prime > primorial(n) + 1)
            # Each worker gets: (n, start_offset, batch_size)
            batch_args = [
                (n, 2 + (round_num * num_workers + i) * batch_size, batch_size)
                for i in range(num_workers)
            ]
            
            round_num += 1
            
            if verbose and round_num % 10 == 0:
                start_m = 2 + (round_num - 1) * num_workers * batch_size
                print(f"Round {round_num}: testing m={start_m}...")
            
            # Execute batches in parallel
            results = pool.map(test_batch, batch_args)
            
            # Check if any worker found a prime
            for result in results:
                if result is not None:
                    elapsed = time.time() - start_time
                    if verbose:
                        print(f"\nF({n}) = {result} (found in round {round_num})")
                        print(f"Elapsed time: {elapsed:.2f}s")
                    return result


def main():
    """Command-line interface"""
    if len(sys.argv) < 2:
        print("Usage: python fortunate.py <n> [batch_size]")
        print("Example: python fortunate.py 500 50")
        sys.exit(1)
    
    n = int(sys.argv[1])
    batch_size = int(sys.argv[2]) if len(sys.argv) > 2 else 50
    
    result = fortunate_batch(n, batch_size=batch_size)
    print(f"\n{'='*50}")
    print(f"Result: F({n}) = {result}")
    print(f"{'='*50}")


if __name__ == "__main__":
    main()
