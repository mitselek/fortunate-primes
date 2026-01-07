"""
Fortunate Numbers Calculator - Python + gmpy2 implementation v2.

Enhanced with Rust-style early termination and adaptive batch sizing:
- Channel-based work distribution (like Rust's crossbeam channels)
- Stop dispatching when first candidate found
- Only wait for batches needed to close [lower_bound; candidate] gap
- Adaptive batch sizing: grow when fast, shrink when slow
- Logs EVERY batch completion
- **Firoozbakht optimization**: Start at p_{n+1} since F(n) ≥ p_{n+1}
  (all offsets 2..p_n are composite - they share a prime factor with primorial(n))

References:
- OEIS A005235: https://oeis.org/A005235
- Fortune's conjecture: All Fortunate numbers are prime
- Firoozbakht (2003): F(n) > p_{n+1} - 1 (OEIS A005235 comment)
- gmpy2 docs: https://gmpy2.readthedocs.io/
"""

import gmpy2  # type: ignore
from multiprocessing import Process, Queue, cpu_count
from typing import Tuple, Optional, Dict, List
import sys
import time


# =============================================================================
# Worker Functions (run in separate processes)
# =============================================================================

def test_batch(n: int, start: int, batch_size: int) -> Tuple[int, int, Optional[int]]:
    """
    Test batch [start, start+batch_size) for primality.
    
    Returns:
        (batch_start, batch_end, result) where result is the first m 
        where primorial(n) + m is prime, or None if batch exhausted
    """
    # Compute primorial(n) = product of first n primes
    pn: int = 1
    p: int = 2
    for _ in range(n):
        pn *= p
        p = int(gmpy2.next_prime(p))  # type: ignore[attr-defined]
    
    # Test candidates in this batch
    end = start + batch_size
    for m in range(start, end):
        if gmpy2.is_prime(pn + m, 25):  # type: ignore[attr-defined]
            return (start, end, m)
    
    return (start, end, None)


def worker(work_queue: "Queue[Optional[Tuple[int, int, int]]]", 
           result_queue: "Queue[Tuple[int, int, Optional[int]]]") -> None:
    """Worker process: pull batches from queue, push results back."""
    while True:
        try:
            args = work_queue.get(timeout=0.1)
            if args is None:  # Poison pill
                break
            n, start, size = args
            result_queue.put(test_batch(n, start, size))
        except:
            continue


# =============================================================================
# Pure Helper Functions
# =============================================================================

def format_duration(seconds: float) -> str:
    """Format duration as human-readable string."""
    if seconds < 1.0:
        return f"{seconds * 1000:.0f}ms"
    elif seconds < 60.0:
        return f"{seconds:.2f}s"
    else:
        return f"{seconds / 60.0:.2f}m"


def compute_min_offset(n: int) -> int:
    """
    Compute the minimum possible offset for F(n).
    
    By Firoozbakht (2003): F(n) >= p_{n+1} because all offsets m where
    2 <= m <= p_n are composite (they share a prime factor with primorial(n)).
    
    Returns:
        p_{n+1}: The (n+1)th prime, which is the first possible Fortunate offset.
    """
    p = 2
    for _ in range(n):
        p = int(gmpy2.next_prime(p))  # type: ignore[attr-defined]
    return p  # This is p_{n+1}


def compute_lower_bound(completed: Dict[int, int], min_offset: int = 2) -> int:
    """
    Find the contiguous lower bound from completed batches.
    
    The lower bound is the highest M where all [min_offset, M) have been tested.
    """
    if not completed:
        return min_offset
    lower = min_offset
    for start in sorted(completed.keys()):
        if start <= lower:
            lower = max(lower, completed[start])
        else:
            break  # Gap in coverage
    return lower


def adjust_batch_size(
    completion_time: float,
    recent_batch_size: int,
    current_batch_size: int,
    target_time: float,
    tk: float
) -> int:
    """
    Adjust batch size based on completion time.
    
    Algorithm (bidirectional):
    - Too fast (< target/tk) AND recent >= current: GROW
    - Too slow (> target*tk) AND recent <= current: SHRINK
    - Otherwise: keep current
    
    The recent_batch_size check filters out stale batches:
    - Stale small batches (from before growth) shouldn't trigger shrinking
    - Stale large batches (from before shrinking) shouldn't trigger growth
    """
    if completion_time < target_time / tk:
        # Too fast - grow if this batch reflects current state
        if recent_batch_size >= current_batch_size:
            return int(recent_batch_size * tk)
    elif completion_time > target_time * tk:
        # Too slow - shrink if this batch reflects current state  
        if recent_batch_size <= current_batch_size:
            return max(1, int(recent_batch_size / tk))
    return current_batch_size


# =============================================================================
# Search State Class
# =============================================================================

class SearchState:
    """Encapsulates mutable state for parallel search."""
    
    def __init__(self, batch_size: int = 1, min_offset: int = 2):
        self.next_batch_size = batch_size  # Size for future dispatches
        self.prev_batch_size = batch_size  # For detecting changes
        self.next_offset = min_offset  # Start at p_{n+1} (Firoozbakht optimization)
        self.min_offset = min_offset  # Store for lower_bound calculation
        self.best_candidate: Optional[int] = None
        self.completed: Dict[int, int] = {}  # {batch_start: batch_end}
        self.dispatch_times: Dict[int, float] = {}  # {batch_start: time}
        self.in_flight = 0
    
    @property
    def lower_bound(self) -> int:
        return compute_lower_bound(self.completed, self.min_offset)
    
    def is_done(self) -> bool:
        """True when gap is closed: lower_bound >= best_candidate."""
        return self.best_candidate is not None and self.lower_bound >= self.best_candidate
    
    def should_dispatch(self) -> bool:
        """True if we should send more work."""
        return self.best_candidate is None or self.next_offset < self.best_candidate
    
    def dispatch(self, queue: "Queue[Optional[Tuple[int, int, int]]]", n: int) -> bool:
        """Send one batch to work queue. Returns True if dispatched."""
        try:
            queue.put((n, self.next_offset, self.next_batch_size), timeout=0.01)
            self.dispatch_times[self.next_offset] = time.time()
            self.next_offset += self.next_batch_size
            self.in_flight += 1
            return True
        except:
            return False
    
    def record_result(self, batch_start: int, batch_end: int, 
                      result: Optional[int]) -> float:
        """
        Record a batch result, return completion time.
        Updates best_candidate or completed map.
        """
        self.in_flight -= 1
        
        # Calculate completion time
        completion_time = 0.0
        if batch_start in self.dispatch_times:
            completion_time = time.time() - self.dispatch_times.pop(batch_start)
        
        # Update state
        if result is not None:
            if self.best_candidate is None or result < self.best_candidate:
                self.best_candidate = result
        else:
            self.completed[batch_start] = batch_end
        
        return completion_time


# =============================================================================
# Progress Reporting
# =============================================================================

def log_batch(
    n: int,
    state: SearchState,
    batch_start: int,
    batch_size: int,
    completion_time: float,
    elapsed: float,
    batch_size_changed: bool
) -> None:
    """Log a single batch completion to stderr."""
    bounds = f"[{state.lower_bound}; {state.best_candidate or '?'}]"
    comp_str = format_duration(completion_time) if completion_time > 0 else "?"
    elapsed_str = format_duration(elapsed)
    
    msg = f"F({n}) : {bounds} [{batch_start}+{batch_size}] ({comp_str}) ({elapsed_str})"
    if batch_size_changed:
        msg += f" next_batch_size={state.next_batch_size}"
    
    print(msg, file=sys.stderr, flush=True)


# =============================================================================
# Main Search Loop (extracted for low complexity)
# =============================================================================

def run_search(
    n: int,
    state: SearchState,
    work_queue: "Queue[Optional[Tuple[int, int, int]]]",
    result_queue: "Queue[Tuple[int, int, Optional[int]]]",
    start_time: float,
    adaptive: bool,
    target_time: float,
    tk: float,
    verbose: bool
) -> int:
    """
    Run the main search loop until gap is closed.
    
    Returns:
        The Fortunate number F(n)
    """
    while state.in_flight > 0:
        # Wait for next result
        try:
            batch_start, batch_end, result = result_queue.get(timeout=0.1)
        except:
            continue
        
        batch_size = batch_end - batch_start
        completion_time = state.record_result(batch_start, batch_end, result)
        elapsed = time.time() - start_time
        
        # Adjust batch size on no-result batches (still actively searching)
        batch_size_changed = False
        if result is None and adaptive:
            state.next_batch_size = adjust_batch_size(
                completion_time, batch_size, state.next_batch_size, target_time, tk
            )
            if state.next_batch_size != state.prev_batch_size:
                batch_size_changed = True
                state.prev_batch_size = state.next_batch_size
        
        # Log EVERY batch
        if verbose:
            log_batch(n, state, batch_start, batch_size, 
                      completion_time, elapsed, batch_size_changed)
        
        # Check if done
        if state.is_done():
            return state.best_candidate  # type: ignore
        
        # Dispatch more work if needed
        if state.should_dispatch():
            state.dispatch(work_queue, n)
    
    # Should not reach here
    if state.best_candidate is not None:
        return state.best_candidate
    raise RuntimeError(f"No Fortunate number found for F({n})")


# =============================================================================
# Orchestrator
# =============================================================================

def fortunate_streaming(
    n: int, 
    batch_size: int = 1, 
    verbose: bool = True, 
    adaptive: bool = True
) -> int:
    """
    Find Fortunate number F(n) using channel-based parallel search.
    
    Args:
        n: Index of the Fortunate number (F(n))
        batch_size: Initial batch size (default 1 for adaptive mode)
        verbose: Print progress on every batch
        adaptive: Enable adaptive batch sizing
    
    Returns:
        F(n): The smallest prime of form primorial(n) + m where m > 1
    """
    start_time = time.time()
    num_workers = cpu_count() - 1
    
    # Compute minimum offset (Firoozbakht: F(n) >= p_{n+1})
    min_offset = compute_min_offset(n)
    
    # Adaptive parameters
    target_time = 60.0 / num_workers
    tk = 2.0
    
    if verbose:
        mode = f"adaptive (target={target_time:.2f}s, tk={tk})" if adaptive else f"fixed"
        print(f"Computing F({n}) with {num_workers} workers, batch_size={batch_size}, mode={mode}",
              file=sys.stderr, flush=True)
        print(f"Starting at offset {min_offset} (p_{{{n+1}}}), skipping {min_offset - 2} trivial offsets",
              file=sys.stderr, flush=True)
    
    # Create queues
    work_queue: "Queue[Optional[Tuple[int, int, int]]]" = Queue(maxsize=num_workers * 2)
    result_queue: "Queue[Tuple[int, int, Optional[int]]]" = Queue()
    
    # Start workers
    workers: List[Process] = []
    for _ in range(num_workers):
        p = Process(target=worker, args=(work_queue, result_queue))
        p.start()
        workers.append(p)
    
    # Initialize state and dispatch first batches
    state = SearchState(batch_size, min_offset)
    for _ in range(num_workers):
        state.dispatch(work_queue, n)
    
    try:
        result = run_search(
            n, state, work_queue, result_queue,
            start_time, adaptive, target_time, tk, verbose
        )
        
        if verbose:
            elapsed = time.time() - start_time
            print(f"F({n}) = {result} ({format_duration(elapsed)})",
                  file=sys.stderr, flush=True)
        
        return result
        
    finally:
        # Cleanup workers
        for _ in workers:
            try:
                work_queue.put(None, timeout=0.1)
            except:
                pass
        for p in workers:
            p.terminate()
            p.join(timeout=1.0)


# =============================================================================
# CLI
# =============================================================================

def main() -> None:
    """Command-line interface."""
    if len(sys.argv) < 2:
        print("Usage: python fortunate_v2.py <n> [batch_size] [--no-adaptive]")
        print("Example: python fortunate_v2.py 500")
        print("         python fortunate_v2.py 500 50 --no-adaptive")
        sys.exit(1)
    
    n = int(sys.argv[1])
    batch_size = 1
    if len(sys.argv) > 2 and sys.argv[2] != '--no-adaptive':
        batch_size = int(sys.argv[2])
    adaptive = '--no-adaptive' not in sys.argv
    
    result = fortunate_streaming(n, batch_size=batch_size, adaptive=adaptive)
    print(f"\nResult: F({n}) = {result}")


if __name__ == "__main__":
    main()
