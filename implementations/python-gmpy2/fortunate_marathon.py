"""
Fortunate Numbers Calculator - "Marathon" edition.

Designed for long-running, stable computations with:
- Clean worker/orchestrator separation
- Per-worker task queues (no shared state)
- Authoritative main-loop state tracking
- Real-time markdown visualization of worker assignments

Built for endurance: hours/days of computation with observable progress.

References:
- OEIS A005235: https://oeis.org/A005235
- Fortune's conjecture: All Fortunate numbers are prime
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
    """
    pn = compute_primorial_oeis(n)
    p_n_plus_1 = compute_nth_prime(n + 1)
    
    for offset in range(p_n_plus_1, 1000000):
        if gmpy2.is_prime(pn + offset, 25):  # type: ignore[attr-defined]
            return offset
    
    raise RuntimeError(f"No Fortunate number found for F({n}) within 1M search range")


# =============================================================================
# Worker Process (Simple - just computes)
# =============================================================================

def worker(
    worker_id: int,
    task_queue: "Queue[Optional[int]]",
    result_queue: "Queue[Tuple[int, int, int, float]]"
) -> None:
    """
    Worker process: receives tasks on dedicated queue, computes F(n), returns results.
    
    Simple design - no shared state, just compute and report.
    """
    while True:
        n = task_queue.get()  # Block until task received
        if n is None:  # Poison pill
            break
        
        start = time.time()
        f_n = compute_fortunate(n)
        elapsed = time.time() - start
        
        result_queue.put((worker_id, n, f_n, elapsed))


# =============================================================================
# Markdown Table Output
# =============================================================================

def write_md_header(md_file, start_n: int, end_n: int, num_workers: int) -> None:
    """Write markdown file header with table structure."""
    md_file.write(f"# F({start_n}-{end_n}) Worker Assignment Log\n\n")
    md_file.write(f"Queue-based load balancing with {num_workers} workers.\n\n")
    md_file.write("**Reading the table**: Each cell shows which index that worker is currently computing.\n")
    md_file.write("**Bold** marks the worker that just finished (cell shows their NEW assignment or ✓ if done).\n\n")
    
    # Table header
    header = "| Time | Result |"
    separator = "|------|--------|"
    for w in range(num_workers):
        header += f" W{w+1:02d} |"
        separator += "------|"
    md_file.write(header + "\n")
    md_file.write(separator + "\n")
    md_file.flush()


def write_md_row(
    md_file,
    elapsed_str: str,
    assignments: List[int],
    num_workers: int,
    finished_worker: Optional[int] = None,
    result_str: str = ""
) -> None:
    """
    Write a table row showing current worker assignments.
    
    Args:
        assignments: List of current assignments (main loop's authoritative state)
        finished_worker: Worker ID that just finished (will be bolded)
        result_str: The F(n)=value string for what was computed
    """
    row = f"| {elapsed_str:>6} | {result_str:<6} |"
    for w in range(num_workers):
        idx = assignments[w]
        if idx == 0:
            cell = "—"  # Not yet assigned
        elif idx == -1:
            cell = "✓"  # Done/idle
        else:
            cell = str(idx)
        
        # Bold the worker that just finished
        if w == finished_worker:
            cell = f"**{cell}**"
        
        row += f" {cell:>4} |"
    
    md_file.write(row + "\n")
    md_file.flush()


def write_md_summary(
    md_file,
    results: Dict[int, Tuple[int, float]],
    total_time: float
) -> None:
    """Write summary section to markdown file."""
    md_file.write("\n## Results Summary\n\n")
    
    # Calculate average time
    total_compute_time = sum(elapsed for _, elapsed in results.values())
    avg_time = total_compute_time / len(results) if results else 0
    
    md_file.write(f"**Total time**: {format_duration(total_time)} | ")
    md_file.write(f"**Average**: {format_duration(avg_time)} | ")
    md_file.write(f"**Count**: {len(results)}\n\n")
    
    md_file.write("| n | F(n) | Time |\n")
    md_file.write("|---|------|------|\n")
    
    for n in sorted(results.keys()):
        f_n, elapsed = results[n]
        md_file.write(f"| {n} | {f_n} | {format_duration(elapsed)} |\n")
    
    md_file.flush()


# =============================================================================
# Main Orchestrator (Single source of truth for assignments)
# =============================================================================

def compute_fortunates(
    start_n: int,
    end_n: int,
    md_path: Optional[str] = None
) -> Dict[int, Tuple[int, float]]:
    """
    Compute Fortunate numbers F(start_n) through F(end_n) using parallel workers.
    
    Main loop is the single source of truth:
    - Tracks what each worker is computing
    - Assigns next task when worker finishes
    - Reports events with complete, accurate state
    """
    start_time = time.time()
    num_workers = cpu_count()
    
    # Main loop's authoritative state
    # 0 = not yet assigned, -1 = done/idle, >0 = computing that index
    assignments: List[int] = [0] * num_workers
    
    # Each worker gets its own task queue (main loop controls assignment)
    task_queues: List["Queue[Optional[int]]"] = [Queue() for _ in range(num_workers)]
    result_queue: "Queue[Tuple[int, int, int, float]]" = Queue()
    
    # Pending work
    pending: List[int] = list(range(start_n, end_n + 1))
    
    # Open markdown file if specified
    md_file = None
    if md_path:
        md_file = open(md_path, 'w')
        write_md_header(md_file, start_n, end_n, num_workers)
    
    # Start workers
    workers: List[Process] = []
    for worker_id in range(num_workers):
        p = Process(target=worker, args=(worker_id, task_queues[worker_id], result_queue))
        p.start()
        workers.append(p)
    
    # Initial dispatch - give each worker their first task
    for worker_id in range(num_workers):
        if pending:
            n = pending.pop(0)
            task_queues[worker_id].put(n)
            assignments[worker_id] = n
        else:
            assignments[worker_id] = -1  # No work for this worker
    
    # Write initial state row
    if md_file:
        write_md_row(md_file, "0", assignments, num_workers)
    
    # Collect results
    results: Dict[int, Tuple[int, float]] = {}
    total_tasks = end_n - start_n + 1
    
    while len(results) < total_tasks:
        try:
            worker_id, n, f_n, elapsed = result_queue.get(timeout=0.5)
            results[n] = (f_n, elapsed)
            
            # Assign next task to this worker (BEFORE updating state for print)
            if pending:
                next_n = pending.pop(0)
                task_queues[worker_id].put(next_n)
                assignments[worker_id] = next_n
            else:
                assignments[worker_id] = -1  # No more work
            
            # Now report the event with accurate state
            elapsed_total = time.time() - start_time
            elapsed_str = format_duration(elapsed_total)
            result_str = f"F({n})={f_n}"
            
            # Stdout: minimal logging
            next_n = assignments[worker_id]
            if next_n > 0:
                print(f"{elapsed_str:>7} W{worker_id+1:02d} {result_str} →{next_n}")
            else:
                print(f"{elapsed_str:>7} W{worker_id+1:02d} {result_str}")
            sys.stdout.flush()
            
            # Markdown: full state table
            if md_file:
                write_md_row(md_file, elapsed_str, assignments, num_workers, worker_id, result_str)
            
        except:
            continue
    
    total_time = time.time() - start_time
    
    # Write summary to markdown
    if md_file:
        write_md_summary(md_file, results, total_time)
        md_file.close()
        print(f"\nMarkdown table written to: {md_path}")
    
    # Cleanup workers - send poison pills
    for worker_id in range(num_workers):
        try:
            task_queues[worker_id].put(None, timeout=0.1)
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
        print("Usage: python fortunate_marathon.py <start_n> [end_n] [--md output.md]")
        print("Example: python fortunate_marathon.py 500")
        print("         python fortunate_marathon.py 500 510")
        print("         python fortunate_marathon.py 500 510 --md benchmark.md")
        sys.exit(1)
    
    # Parse arguments
    start_n = int(sys.argv[1])
    end_n = start_n
    md_path = None
    
    i = 2
    while i < len(sys.argv):
        if sys.argv[i] == "--md":
            md_path = sys.argv[i + 1]
            i += 2
        else:
            end_n = int(sys.argv[i])
            i += 1
    
    # Default md_path if not specified
    if md_path is None:
        md_path = f"marathon_{start_n}-{end_n}.md"
    
    print(f"Computing F({start_n})..F({end_n}) with {cpu_count()} workers")
    print(f"Markdown output: {md_path}\n")
    
    results = compute_fortunates(start_n, end_n, md_path)
    
    print(f"\nCompleted {len(results)} computations")


if __name__ == "__main__":
    main()
