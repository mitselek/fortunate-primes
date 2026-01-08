"""
Fortunate Numbers Calculator - Python + gmpy2 implementation v3.

Worker-based sequential primorial assignment:
- Each worker gets assigned a primorial index n (dynamic queue-based)
- Worker computes F(primorial(n)) to completion
- Returns result and immediately gets next available index
- Natural load balancing: whoever finishes first gets next work
- No redundant primorial recomputation across workers

References:
- OEIS A005235: https://oeis.org/A005235
- Fortune's conjecture: All Fortunate numbers are prime
- Firoozbakht (2003): F(n) >= p_{n+1}
- gmpy2 docs: https://gmpy2.readthedocs.io/
"""

import gmpy2  # type: ignore
from multiprocessing import Process, Queue, cpu_count
from typing import Tuple, Optional, Dict, List
import sys
import time


# =============================================================================
# Helper Functions (Pure)
# =============================================================================

def format_duration(seconds: float) -> str:
    """Format duration as human-readable string."""
    if seconds < 1.0:
        return f"{seconds * 1000:.0f}ms"
    elif seconds < 60.0:
        return f"{seconds:.2f}s"
    else:
        return f"{seconds / 60.0:.2f}m"


def compute_primorial_oeis(n: int) -> int:
    """
    Compute primorial(n) = product of first n primes (OEIS A005235 definition).
    
    Note: gmpy2.primorial(p) computes product of primes <= p, which is different.
    """
    if n == 0:
        return 1
    result = 1
    p = 2
    for _ in range(n):
        result *= p
        p = int(gmpy2.next_prime(p))  # type: ignore[attr-defined]
    return result


def compute_nth_prime(n: int) -> int:
    """Get the nth prime (1-indexed)."""
    p = 2
    for _ in range(n - 1):
        p = int(gmpy2.next_prime(p))  # type: ignore[attr-defined]
    return p


def compute_fortunate(n: int) -> int:
    """
    Find Fortunate number F(n) = smallest m > 1 where primorial(n) + m is prime.
    
    Uses OEIS A005235 definition: primorial(n) = product of first n primes.
    Starts search at p_{n+1} (Firoozbakht optimization).
    
    Args:
        n: Primorial index
    
    Returns:
        F(n): The Fortunate number
    """
    # Compute primorial(n) using OEIS definition
    pn = compute_primorial_oeis(n)
    
    # Firoozbakht: F(n) >= p_{n+1}, so start at p_{n+1}
    p_n_plus_1 = compute_nth_prime(n + 1)
    
    # Search for first prime of form primorial(n) + m where m >= p_{n+1}
    for offset in range(p_n_plus_1, 1000000):
        if gmpy2.is_prime(pn + offset, 25):  # type: ignore[attr-defined]
            return offset
    
    raise RuntimeError(f"No Fortunate number found for F({n}) within 1M search range")


# =============================================================================
# Worker Process
# =============================================================================

def worker(
    worker_id: int,
    work_queue: "Queue[Optional[int]]",
    result_queue: "Queue[Tuple[int, int, float]]"
) -> None:
    """
    Worker process: pull primorial indices from queue, compute F(n), return results.
    
    Each work item is a primorial index n.
    Worker computes F(primorial(n)) and returns (n, result, elapsed_time).
    """
    while True:
        try:
            n = work_queue.get(timeout=0.1)
            if n is None:  # Poison pill
                break
            
            start = time.time()
            f_n = compute_fortunate(n)
            elapsed = time.time() - start
            
            result_queue.put((n, f_n, elapsed))
        except:
            continue


# =============================================================================
# Main Orchestrator
# =============================================================================

def compute_fortunates(
    start_n: int,
    end_n: int,
    verbose: bool = True
) -> Dict[int, Tuple[int, float]]:
    """
    Compute Fortunate numbers F(start_n) through F(end_n) using parallel workers.
    
    Args:
        start_n: Starting primorial index (inclusive)
        end_n: Ending primorial index (inclusive)
        verbose: Print progress on every result
    
    Returns:
        Dictionary mapping n → (F(n), elapsed_time)
    """
    start_time = time.time()
    num_workers = cpu_count()
    
    if verbose:
        print(f"Computing F({start_n})..F({end_n}) with {num_workers} workers",
              file=sys.stderr, flush=True)
    
    # Create queues
    work_queue: "Queue[Optional[int]]" = Queue(maxsize=num_workers * 2)
    result_queue: "Queue[Tuple[int, int, float]]" = Queue()
    
    # Start workers
    workers: List[Process] = []
    for worker_id in range(num_workers):
        p = Process(target=worker, args=(worker_id, work_queue, result_queue))
        p.start()
        workers.append(p)
    
    # Dispatch initial work
    for n in range(start_n, end_n + 1):
        work_queue.put(n)
    
    # Collect results
    results: Dict[int, Tuple[int, float]] = {}
    remaining = end_n - start_n + 1
    
    while remaining > 0:
        try:
            n, f_n, elapsed = result_queue.get(timeout=0.5)
            results[n] = (f_n, elapsed)
            remaining -= 1
            
            if verbose:
                elapsed_total = time.time() - start_time
                print(f"F({n:4}) = {f_n:6} ({format_duration(elapsed):>6}) [{remaining:2} remaining, {format_duration(elapsed_total):>6} elapsed]",
                      file=sys.stderr, flush=True)
        except:
            continue
    
    # Cleanup workers
    for _ in workers:
        try:
            work_queue.put(None, timeout=0.1)
        except:
            pass
    for p in workers:
        p.terminate()
        p.join(timeout=1.0)
    
    return results


# =============================================================================
# CLI
# =============================================================================

def main() -> None:
    """Command-line interface."""
    if len(sys.argv) < 2:
        print("Usage: python fortunate_v3.py <start_n> [end_n]")
        print("Example: python fortunate_v3.py 500")
        print("         python fortunate_v3.py 500 510")
        sys.exit(1)
    
    start_n = int(sys.argv[1])
    end_n = int(sys.argv[2]) if len(sys.argv) > 2 else start_n
    
    results = compute_fortunates(start_n, end_n, verbose=True)
    
    print(f"\nSummary:", file=sys.stderr, flush=True)
    for n in sorted(results.keys()):
        f_n, elapsed = results[n]
        print(f"F({n:4}) = {f_n:6} ({format_duration(elapsed):>6})")


if __name__ == "__main__":
    main()
