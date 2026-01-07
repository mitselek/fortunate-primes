"""
Fortunate Numbers Calculator - Python + gmpy2 implementation v2.

Enhanced with Rust-style early termination and progress reporting:
- Channel-based work distribution (like Rust's crossbeam channels)
- Stop dispatching when first candidate found
- Only wait for batches needed to close [lower_bound; candidate] gap
- Interval notation progress: F(n) : [lower; upper] [batch_start+size] (time)

Uses gmpy2 (GMP bindings) for efficient big integer arithmetic and
primality testing, with multiprocessing for parallel worker distribution.

References:
- OEIS A005235: https://oeis.org/A005235
- Fortune's conjecture: All Fortunate numbers are prime
- gmpy2 docs: https://gmpy2.readthedocs.io/
"""

import gmpy2  # type: ignore
from multiprocessing import Process, Queue, cpu_count
from typing import Tuple, Optional, Dict, List
import sys
import time


def test_batch(args: Tuple[int, int, int]) -> Tuple[int, int, Optional[int]]:
    """
    Worker function: Test batch [start, start+batch_size) for primality.
    
    Each worker recomputes the primorial independently (acceptable overhead
    vs IPC serialization cost for large integers).
    
    Args:
        args: Tuple of (n, start, batch_size)
    
    Returns:
        Tuple of (batch_start, batch_end, result) where result is the first m 
        where primorial(n) + m is prime, or None if batch exhausted
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
            return (start, end, m)
    
    return (start, end, None)


def worker(work_queue: "Queue[Optional[Tuple[int, int, int]]]", result_queue: "Queue[Tuple[int, int, Optional[int]]]") -> None:
    """Worker process that pulls batches from work queue and reports results."""
    while True:
        try:
            batch_args = work_queue.get(timeout=0.1)
            if batch_args is None:  # Poison pill
                break
            result = test_batch(batch_args)
            result_queue.put(result)
        except:
            continue


def format_duration(seconds: float) -> str:
    """Format duration as human-readable string matching Rust output."""
    if seconds < 1.0:
        return f"{seconds * 1000:.0f}ms"
    elif seconds < 60.0:
        return f"{seconds:.2f}s"
    else:
        mins = seconds / 60.0
        return f"{mins:.2f}m"


def fortunate_streaming(n: int, batch_size: int = 1, verbose: bool = True, adaptive: bool = True) -> int:
    """
    Find Fortunate number F(n) using channel-based parallel search with adaptive batch sizing.
    
    Mimics Rust's architecture: main thread dispatches work, workers report results,
    stops dispatching when candidate found, waits only for gap-closing batches.
    
    Adaptive batch sizing dynamically adjusts batch size based on completion times:
    - Grows when batches complete too quickly (< target_time / tk)
    - Shrinks when batches complete too slowly (> target_time * tk)
    - Targets optimal batch duration for maximum throughput
    
    Args:
        n: Index of the Fortunate number (F(n))
        batch_size: Initial batch size (default 1 for adaptive mode)
        verbose: Print progress updates
        adaptive: Enable adaptive batch sizing (default True)
    
    Returns:
        F(n): The smallest prime of form primorial(n) + m where m > 1
    """
    start_time = time.time()
    num_workers = cpu_count() - 1  # Reserve one core for main thread
    
    # Adaptive batch sizing parameters
    target_time = 60.0 / num_workers  # Target batch completion time in seconds
    tk = 2.0  # Threshold constant (acceptable range: [target/tk, target*tk])
    
    if verbose:
        print(f"Computing F({n})...")
        if adaptive:
            print(f"Using {num_workers} workers, adaptive batch sizing (initial={batch_size}, target={target_time:.2f}s)")
        else:
            print(f"Using {num_workers} workers, batch_size={batch_size}")
    
    # Create work and result queues
    work_queue: "Queue[Optional[Tuple[int, int, int]]]" = Queue(maxsize=num_workers * 2)
    result_queue: "Queue[Tuple[int, int, Optional[int]]]" = Queue()
    
    # Start worker processes
    workers: List[Process] = []
    for _ in range(num_workers):
        p = Process(target=worker, args=(work_queue, result_queue))
        p.start()
        workers.append(p)
    
    try:
        best_candidate: Optional[int] = None
        completed_no_result: Dict[int, int] = {}  # {batch_start: batch_end}
        dispatched_batches: Dict[int, Tuple[int, float]] = {}  # {batch_start: (batch_size, dispatch_time)}
        last_report_time: float = start_time
        report_interval = 1.0
        initial_delay = 2.0
        
        next_offset = 2  # Start from m=2
        batches_in_flight = 0
        
        # Dispatch initial batches
        for _ in range(num_workers):
            dispatch_time = time.time()
            work_queue.put((n, next_offset, batch_size))
            dispatched_batches[next_offset] = (batch_size, dispatch_time)
            batches_in_flight += 1
            next_offset += batch_size
        
        # Main loop: collect results and dispatch new work
        while batches_in_flight > 0:
            try:
                batch_start, batch_end, result = result_queue.get(timeout=0.1)
                batches_in_flight -= 1
                
                current_time = time.time()
                elapsed = current_time - start_time
                recent_batch_size = batch_end - batch_start
                
                # Calculate completion time for adaptive sizing
                completion_time = 0.0
                if batch_start in dispatched_batches:
                    original_size, dispatch_time = dispatched_batches.pop(batch_start)
                    completion_time = current_time - dispatch_time
                    
                    # Adaptive batch size adjustment
                    if adaptive and result is None:  # Only adjust when no result found
                        if completion_time < target_time / tk:
                            # Too fast - grow batch size
                            if recent_batch_size > batch_size:
                                batch_size = int(recent_batch_size * tk)
                        elif completion_time > target_time * tk:
                            # Too slow - shrink batch size
                            if recent_batch_size < batch_size:
                                batch_size = max(1, int(recent_batch_size / tk))
                
                if result is not None:
                    # Found a candidate
                    is_better = best_candidate is None or result < best_candidate
                    if is_better:
                        best_candidate = result
                else:
                    # No result in this batch
                    completed_no_result[batch_start] = batch_end
                
                # Compute contiguous lower bound
                lower_bound = 2
                for start in sorted(completed_no_result.keys()):
                    if start <= lower_bound:
                        lower_bound = max(lower_bound, completed_no_result[start])
                    else:
                        break
                
                # Check if gap is closed
                if best_candidate is not None and lower_bound >= best_candidate:
                    time_str = format_duration(elapsed)
                    if verbose:
                        print(f"F({n}) = {best_candidate} ({time_str})")
                    return best_candidate
                
                # Dispatch more work ONLY if no candidate OR batch is before candidate
                if best_candidate is None or next_offset < best_candidate:
                    try:
                        dispatch_time = time.time()
                        work_queue.put((n, next_offset, batch_size), timeout=0.01)
                        dispatched_batches[next_offset] = (batch_size, dispatch_time)
                        batches_in_flight += 1
                        next_offset += batch_size
                    except:
                        pass  # Queue full, will retry next iteration
                
                # Enhanced progress reporting with batch completion time
                if verbose and elapsed >= initial_delay and (current_time - last_report_time) >= report_interval:
                    elapsed_str = format_duration(elapsed)
                    completion_str = format_duration(completion_time) if completion_time > 0 else "?"
                    
                    if best_candidate is not None:
                        bounds_str = f"[{lower_bound}; {best_candidate}]"
                    else:
                        bounds_str = f"[{lower_bound}; ?]"
                    
                    # Show batch size in progress if adaptive mode
                    if adaptive:
                        print(f"F({n}) : {bounds_str} [{batch_start}+{recent_batch_size}] ({completion_str}) ({elapsed_str}) batch_size={batch_size}", 
                              file=sys.stderr, flush=True)
                    else:
                        print(f"F({n}) : {bounds_str} [{batch_start}+{recent_batch_size}] ({completion_str}) ({elapsed_str})", 
                              file=sys.stderr, flush=True)
                    last_report_time = current_time
                    
            except:
                continue  # Timeout or other error, keep waiting
        
        # Should not reach here
        if best_candidate is not None:
            elapsed = time.time() - start_time
            time_str = format_duration(elapsed)
            if verbose:
                print(f"F({n}) = {best_candidate} ({time_str})")
            return best_candidate
        
        raise RuntimeError(f"No Fortunate number found for F({n})")
        
    finally:
        # Cleanup: send poison pills and join workers
        for _ in workers:
            try:
                work_queue.put(None, timeout=0.1)
            except:
                pass
        for p in workers:
            p.terminate()
            p.join(timeout=1.0)



def main():
    """Command-line interface"""
    if len(sys.argv) < 2:
        print("Usage: python fortunate_v2.py <n> [batch_size] [--no-adaptive]")
        print("Example: python fortunate_v2.py 500          # Adaptive mode, initial batch_size=1")
        print("         python fortunate_v2.py 500 50       # Adaptive mode, initial batch_size=50")
        print("         python fortunate_v2.py 500 50 --no-adaptive  # Fixed batch_size=50")
        sys.exit(1)
    
    n = int(sys.argv[1])
    batch_size = int(sys.argv[2]) if len(sys.argv) > 2 and sys.argv[2] != '--no-adaptive' else 1
    adaptive = '--no-adaptive' not in sys.argv
    
    result = fortunate_streaming(n, batch_size=batch_size, adaptive=adaptive)
    print(f"\n{'='*50}")
    print(f"Result: F({n}) = {result}")
    print(f"{'='*50}")


if __name__ == "__main__":
    main()
