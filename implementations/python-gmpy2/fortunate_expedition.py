"""
Fortunate Numbers Calculator - "Expedition" edition.

Designed for resilient long-running batch computations with:
- Self-adjusting batch size based on moving average of batch times
- Work overflow: when F(n) has all intervals assigned, workers start on F(n+1)
- Full state checkpointing: resume from where you left off after interruption
- Orchestrator as single source of truth for all state

Architecture:
    Orchestrator (main loop)
        │
        ├── State Manager (checkpoint save/load)
        │       └── expedition_checkpoint.json
        │
        ├── Batch Sizer (adaptive timing)
        │       └── Moving average of batch completion times
        │
        └── Worker Pool
                ├── Worker 1 ──► test_batch(n, start, size) ──► result
                ├── Worker 2 ──► ...
                └── Worker N ──► ...

Usage:
    python fortunate_expedition.py 4610 4650              # Compute F(4610)..F(4650)
    python fortunate_expedition.py 4610 4650 --resume     # Resume from checkpoint
    python fortunate_expedition.py --status               # Show checkpoint status

References:
- OEIS A005235: https://oeis.org/A005235
- Fortune's conjecture: All Fortunate numbers are prime
"""

import gmpy2
from multiprocessing import Process, Queue, cpu_count
from dataclasses import dataclass, asdict
from typing import Optional, Dict, List, Tuple, Deque, Any
from collections import deque
from pathlib import Path
import signal
import json
import sys
import time


# =============================================================================
# Configuration
# =============================================================================

TARGET_BATCH_TIME = 5.0      # Target seconds per batch
BATCH_TIME_WINDOW = 20       # Number of batches for moving average
MIN_BATCH_SIZE = 10          # Minimum batch size
MAX_BATCH_SIZE = 5000        # Maximum batch size
BATCH_GROW_FACTOR = 1.5      # Grow factor when too fast
BATCH_SHRINK_FACTOR = 0.7    # Shrink factor when too slow
CHECKPOINT_FILE = "expedition_checkpoint.json"


# =============================================================================
# Data Structures
# =============================================================================

@dataclass
class SearchState:
    """State for searching F(n)."""
    n: int
    p_n_plus_1: int           # Starting offset (Firoozbakht optimization)
    next_offset: int          # Next unassigned offset
    completed_up_to: int      # All offsets below this have been searched
    pending_ranges: List[Tuple[int, int]]  # Ranges in flight (start, end)
    best_candidate: Optional[int] = None  # Best prime found so far
    completed: bool = False
    
    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)
    
    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "SearchState":
        # JSON deserializes tuples as lists - convert back
        d["pending_ranges"] = [tuple(r) for r in d.get("pending_ranges", [])]
        return cls(**d)


@dataclass
class WorkerTask:
    """A task assigned to a worker."""
    worker_id: int
    n: int
    start_offset: int
    end_offset: int
    assigned_at: float


@dataclass
class ExpeditionState:
    """Complete expedition state for checkpointing."""
    start_n: int
    end_n: int
    results: Dict[int, int]                    # n -> F(n)
    result_times: Dict[int, float]             # n -> computation time
    searches: Dict[int, SearchState]           # n -> search state
    batch_times: List[float]                   # Recent batch times
    current_batch_size: int
    total_elapsed: float                       # Total time spent
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            "start_n": self.start_n,
            "end_n": self.end_n,
            "results": {str(k): v for k, v in self.results.items()},
            "result_times": {str(k): v for k, v in self.result_times.items()},
            "searches": {str(k): v.to_dict() for k, v in self.searches.items()},
            "batch_times": self.batch_times,
            "current_batch_size": self.current_batch_size,
            "total_elapsed": self.total_elapsed,
        }
    
    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> "ExpeditionState":
        return cls(
            start_n=d["start_n"],
            end_n=d["end_n"],
            results={int(k): v for k, v in d["results"].items()},
            result_times={int(k): v for k, v in d["result_times"].items()},
            searches={int(k): SearchState.from_dict(v) for k, v in d["searches"].items()},
            batch_times=d["batch_times"],
            current_batch_size=d["current_batch_size"],
            total_elapsed=d["total_elapsed"],
        )


# =============================================================================
# Helper Functions
# =============================================================================

def format_duration(seconds: float) -> str:
    """Format duration as human-readable string."""
    if seconds < 60:
        return f"{seconds:.1f}s"
    elif seconds < 3600:
        mins = int(seconds // 60)
        secs = int(seconds % 60)
        return f"{mins}m {secs}s"
    else:
        hours = int(seconds // 3600)
        mins = int((seconds % 3600) // 60)
        return f"{hours}h {mins}m"


def compute_nth_prime(n: int) -> int:
    """Get the nth prime (1-indexed)."""
    if n < 1:
        return 2
    p = 2
    for _ in range(n - 1):
        p = int(gmpy2.next_prime(p))
    return p


def compute_primorial(n: int) -> int:
    """Compute primorial(n) = product of first n primes."""
    if n == 0:
        return 1
    result = 1
    p = 2
    for _ in range(n):
        result *= p
        p = int(gmpy2.next_prime(p))
    return result


# =============================================================================
# Markdown Table Logging
# =============================================================================

class TableLogger:
    """Logs worker assignments to markdown table for real-time visualization."""
    
    def __init__(
        self,
        filepath: Path,
        start_n: int,
        end_n: int,
        num_workers: int,
        resume: bool = False,
    ):
        self.filepath = filepath
        self.num_workers = num_workers
        
        if resume and filepath.exists():
            # Resume: truncate at summary section and append
            self._truncate_at_summary()
            self.file = open(filepath, 'a')
        else:
            # Fresh start: write header
            self.file = open(filepath, 'w')
            self._write_header(start_n, end_n)
    
    def _truncate_at_summary(self) -> None:
        """Remove results summary section from file for clean resume."""
        content = self.filepath.read_text()
        
        # Find the summary section
        summary_marker = "\n## Results Summary"
        summary_pos = content.find(summary_marker)
        
        if summary_pos != -1:
            # Truncate at the summary section
            truncated = content[:summary_pos]
            self.filepath.write_text(truncated)
            print(f"Truncated log at summary section for resume")
    
    def _write_header(self, start_n: int, end_n: int) -> None:
        """Write markdown file header with table structure."""
        self.file.write(f"# Expedition F({start_n}-{end_n}) Worker Log\n\n")
        self.file.write(f"Batch-based with checkpoint/resume, {self.num_workers} workers.\n\n")
        self.file.write("**Worker cells**: `n:offset` = F(n) batch starting at offset.\n")
        self.file.write("**Bold** = worker that just confirmed this result.\n\n")
        
        # Table header
        header = "| Time | n | F(n) |"
        separator = "|-----:|---:|-----:|"
        for w in range(self.num_workers):
            header += f" W{w+1:02d} |"
            separator += "--------:|"
        self.file.write(header + "\n")
        self.file.write(separator + "\n")
        self.file.flush()
    
    def log_row(
        self,
        elapsed_str: str,
        n: int,
        f_n: int,
        worker_tasks: Dict[int, Optional["WorkerTask"]],
        finished_worker: Optional[int] = None,
    ) -> None:
        """Write a table row showing current worker assignments."""
        row = f"| {elapsed_str:>5} | {n} | {f_n} |"
        
        for w in range(self.num_workers):
            task = worker_tasks.get(w)
            if task is None:
                cell = "—"
            else:
                # Show n:offset (which F value and where in the search)
                cell = f"{task.n}:{task.start_offset}"
            
            # Bold the n value for the worker that just confirmed this result
            if w == finished_worker:
                cell = f"**{n}**"
            
            row += f" {cell:>5} |"
        
        self.file.write(row + "\n")
        self.file.flush()
    
    def log_summary(self, results: Dict[int, int], total_time: float) -> None:
        """Write summary section."""
        self.file.write("\n## Results Summary\n\n")
        self.file.write(f"**Total time**: {format_duration(total_time)} | ")
        self.file.write(f"**Count**: {len(results)}\n\n")
        
        self.file.write("| n | F(n) |\n")
        self.file.write("|---:|-----:|\n")
        for n in sorted(results.keys()):
            self.file.write(f"| {n} | {results[n]} |\n")
        self.file.flush()
    
    def close(self) -> None:
        """Close the log file."""
        self.file.close()


# =============================================================================
# Worker Process
# =============================================================================

def worker_process(
    worker_id: int,
    task_queue: "Queue[Optional[Tuple[int, int, int, int]]]",
    result_queue: "Queue[Tuple[int, int, int, int, Optional[int], float]]",
) -> None:
    """
    Worker: receives (worker_id, n, start, end), tests batch, returns result.
    
    Result tuple: (worker_id, n, start, end, found_m, elapsed)
    
    Workers just do the primality tests - the orchestrator handles
    early termination by not assigning more work for completed n values.
    """
    # Ignore signals in workers - only main process handles shutdown
    signal.signal(signal.SIGINT, signal.SIG_IGN)
    signal.signal(signal.SIGTERM, signal.SIG_IGN)
    
    # Cache primorials to avoid recomputation
    primorial_cache: Dict[int, int] = {}
    
    while True:
        try:
            task = task_queue.get(timeout=2.0)
        except Exception:
            continue  # Queue may be closing or timeout
        
        if task is None:  # Poison pill
            break
        
        wid, n, start, end = task
        
        # Get or compute primorial
        if n not in primorial_cache:
            primorial_cache[n] = compute_primorial(n)
        pn = primorial_cache[n]
        
        # Test batch - do the actual work!
        start_time = time.time()
        found_m: Optional[int] = None
        
        for m in range(start, end):
            if gmpy2.is_prime(pn + m, 25):
                found_m = m
                break
        
        elapsed = time.time() - start_time
        
        try:
            result_queue.put((wid, n, start, end, found_m, elapsed))
        except (BrokenPipeError, ConnectionResetError, EOFError):
            break  # Shutdown in progress


# =============================================================================
# Adaptive Batch Sizer
# =============================================================================

class BatchSizer:
    """Manages adaptive batch sizing based on timing feedback."""
    
    def __init__(
        self,
        initial_size: int = 100,
        target_time: float = TARGET_BATCH_TIME,
        window_size: int = BATCH_TIME_WINDOW,
    ):
        self.current_size = initial_size
        self.target_time = target_time
        self.times: Deque[float] = deque(maxlen=window_size)
    
    def record_batch(self, elapsed: float, batch_size: int) -> None:
        """Record a batch completion and adjust size."""
        # Normalize to per-candidate time
        time_per_candidate = elapsed / max(batch_size, 1)
        self.times.append(time_per_candidate)
        
        if len(self.times) >= 3:  # Need some data
            avg_time_per_candidate = sum(self.times) / len(self.times)
            expected_batch_time = avg_time_per_candidate * self.current_size
            
            if expected_batch_time < self.target_time * 0.5:
                # Too fast - grow
                self.current_size = min(
                    int(self.current_size * BATCH_GROW_FACTOR),
                    MAX_BATCH_SIZE
                )
            elif expected_batch_time > self.target_time * 1.5:
                # Too slow - shrink
                self.current_size = max(
                    int(self.current_size * BATCH_SHRINK_FACTOR),
                    MIN_BATCH_SIZE
                )
    
    def get_batch_size(self) -> int:
        return self.current_size
    
    def get_times_list(self) -> List[float]:
        return list(self.times)
    
    def load_times(self, times: List[float]) -> None:
        self.times = deque(times, maxlen=BATCH_TIME_WINDOW)


# =============================================================================
# Checkpoint Manager
# =============================================================================

class CheckpointManager:
    """Manages state persistence for resume capability."""
    
    def __init__(self, filepath: Path):
        self.filepath = filepath
    
    def save(self, state: ExpeditionState) -> None:
        """Save state to checkpoint file."""
        temp_path = self.filepath.with_suffix(".tmp")
        with open(temp_path, "w") as f:
            json.dump(state.to_dict(), f, indent=2)
        temp_path.rename(self.filepath)  # Atomic rename
    
    def load(self) -> Optional[ExpeditionState]:
        """Load state from checkpoint file."""
        if not self.filepath.exists():
            return None
        try:
            with open(self.filepath, "r") as f:
                data = json.load(f)
            return ExpeditionState.from_dict(data)
        except (json.JSONDecodeError, KeyError) as e:
            print(f"Warning: Could not load checkpoint: {e}")
            return None
    
    def exists(self) -> bool:
        return self.filepath.exists()
    
    def remove(self) -> None:
        if self.filepath.exists():
            self.filepath.unlink()


# =============================================================================
# Expedition Orchestrator
# =============================================================================

class Expedition:
    """
    Main orchestrator for Fortunate number computation.
    
    Manages workers, distributes batches, handles checkpointing.
    """
    
    def __init__(
        self,
        start_n: int,
        end_n: int,
        checkpoint_path: Optional[Path] = None,
        resume: bool = False,
        log_path: Optional[Path] = None,
    ):
        self.start_n = start_n
        self.end_n = end_n
        self.num_workers = cpu_count()
        
        # Checkpoint management
        self.checkpoint_mgr = CheckpointManager(
            checkpoint_path or Path(CHECKPOINT_FILE)
        )
        
        # State initialization (determine if we're actually resuming)
        actually_resuming = False
        if resume and self.checkpoint_mgr.exists():
            loaded = self.checkpoint_mgr.load()
            if loaded and loaded.start_n == start_n and loaded.end_n == end_n:
                self.state = loaded
                self.batch_sizer = BatchSizer(initial_size=loaded.current_batch_size)
                self.batch_sizer.load_times(loaded.batch_times)
                print(f"Resumed from checkpoint: {len(loaded.results)} results completed")
                actually_resuming = True
            else:
                print("Checkpoint range mismatch, starting fresh")
                self.state = self._create_initial_state()
                self.batch_sizer = BatchSizer()
        else:
            self.state = self._create_initial_state()
            self.batch_sizer = BatchSizer()
        
        # Table logger (optional, created AFTER resume decision)
        self.logger: Optional[TableLogger] = None
        if log_path:
            self.logger = TableLogger(
                log_path, start_n, end_n, self.num_workers,
                resume=actually_resuming
            )
        
        # Runtime state (not checkpointed)
        self.worker_tasks: Dict[int, Optional[WorkerTask]] = {
            i: None for i in range(self.num_workers)
        }
        self.start_time = time.time()
        self.baseline_elapsed = self.state.total_elapsed  # Time from previous sessions
        self.shutdown_requested = False
        
        # Queues
        self.task_queue: "Queue[Optional[Tuple[int, int, int, int]]]" = Queue()
        self.result_queue: "Queue[Tuple[int, int, int, int, Optional[int], float]]" = Queue()
        
        # Workers
        self.workers: List[Process] = []
    
    def _create_initial_state(self) -> ExpeditionState:
        """Create fresh expedition state."""
        searches: Dict[int, SearchState] = {}
        for n in range(self.start_n, self.end_n + 1):
            p_n_plus_1 = compute_nth_prime(n + 1)
            searches[n] = SearchState(
                n=n,
                p_n_plus_1=p_n_plus_1,
                next_offset=p_n_plus_1,  # Start at p_{n+1} (Firoozbakht)
                completed_up_to=p_n_plus_1,  # Nothing completed yet
                pending_ranges=[],  # No ranges in flight
            )
        
        return ExpeditionState(
            start_n=self.start_n,
            end_n=self.end_n,
            results={},
            result_times={},
            searches=searches,
            batch_times=[],
            current_batch_size=100,
            total_elapsed=0.0,
        )
    
    def _setup_signal_handlers(self) -> None:
        """Install signal handlers for graceful shutdown."""
        def handle_signal(signum: int, frame: Any) -> None:
            if not self.shutdown_requested:  # Only print once
                print(f"\n[Signal {signum}] Saving checkpoint and shutting down...")
            self.shutdown_requested = True
        
        signal.signal(signal.SIGINT, handle_signal)
        signal.signal(signal.SIGTERM, handle_signal)
    
    def _start_workers(self) -> None:
        """Start worker processes."""
        for i in range(self.num_workers):
            p = Process(
                target=worker_process,
                args=(i, self.task_queue, self.result_queue)
            )
            p.start()
            self.workers.append(p)
    
    def _stop_workers(self) -> None:
        """Stop all worker processes."""
        for _ in range(self.num_workers):
            self.task_queue.put(None)
        for p in self.workers:
            p.join(timeout=2.0)
            if p.is_alive():
                p.terminate()
    
    def _get_next_task(self) -> Optional[Tuple[int, int, int]]:
        """
        Get next (n, start, end) task to assign.
        
        Priority:
        1. Re-dispatch orphaned ranges (from resume) - first in pending_ranges
        2. Assign new work from next_offset
        """
        batch_size = self.batch_sizer.get_batch_size()
        
        for n in range(self.start_n, self.end_n + 1):
            if n in self.state.results:
                continue  # Already found
            
            search = self.state.searches[n]
            if search.completed:
                continue
            
            # Check if we have a candidate and all prior offsets assigned
            if search.best_candidate is not None:
                if search.next_offset >= search.best_candidate:
                    # All new ranges assigned - but check for orphaned ranges
                    if not search.pending_ranges:
                        continue  # Nothing to dispatch
            
            # Priority 1: Re-dispatch orphaned ranges from previous run
            # These are ranges that were in-flight when process died
            # We pop from front (oldest first) since they're closer to completed_up_to
            if search.pending_ranges:
                # Check if this is an orphaned range (no worker assigned to it)
                orphan = self._find_orphaned_range(n, search.pending_ranges)
                if orphan:
                    return (n, orphan[0], orphan[1])
            
            # Priority 2: Assign new batch from next_offset
            if search.best_candidate is not None:
                if search.next_offset >= search.best_candidate:
                    continue  # All necessary ranges assigned
            
            start = search.next_offset
            end = start + batch_size
            search.next_offset = end
            
            # Track this range as in-flight
            search.pending_ranges.append((start, end))
            
            return (n, start, end)
        
        return None
    
    def _find_orphaned_range(
        self, n: int, pending_ranges: List[Tuple[int, int]]
    ) -> Optional[Tuple[int, int]]:
        """
        Find a pending range that has no worker assigned to it.
        These are orphaned from a previous run and need re-dispatch.
        """
        # Get all ranges currently assigned to workers
        worker_ranges: set[Tuple[int, int]] = set()
        for task in self.worker_tasks.values():
            if task is not None and task.n == n:
                worker_ranges.add((task.start_offset, task.end_offset))
        
        # Find first pending range not in worker_ranges
        for r in pending_ranges:
            if r not in worker_ranges:
                return r
        
        return None
    
    def _dispatch_tasks(self) -> int:
        """Dispatch tasks to idle workers. Returns number dispatched."""
        dispatched = 0
        
        for worker_id in range(self.num_workers):
            if self.worker_tasks[worker_id] is not None:
                continue  # Worker busy
            
            task = self._get_next_task()
            if task is None:
                break  # No more work
            
            n, start, end = task
            self.task_queue.put((worker_id, n, start, end))
            self.worker_tasks[worker_id] = WorkerTask(
                worker_id=worker_id,
                n=n,
                start_offset=start,
                end_offset=end,
                assigned_at=time.time(),
            )
            dispatched += 1
        
        return dispatched
    
    def _process_result(
        self,
        worker_id: int,
        n: int,
        start: int,
        end: int,
        found_m: Optional[int],
        elapsed: float,
    ) -> None:
        """Process a result from a worker."""
        # Record batch timing
        self.batch_sizer.record_batch(elapsed, end - start)
        
        # Free worker
        self.worker_tasks[worker_id] = None
        
        # Skip if already finalized
        if n in self.state.results:
            return
        
        search = self.state.searches[n]
        
        # Remove this range from pending
        try:
            search.pending_ranges.remove((start, end))
        except ValueError:
            pass  # Range not found (shouldn't happen)
        
        # Update completed_up_to: advance it while there are no gaps
        # Sort pending ranges and find the lowest start
        if not search.pending_ranges:
            # No pending ranges, completed_up_to = next_offset
            search.completed_up_to = search.next_offset
        else:
            # completed_up_to = min start of pending ranges
            min_pending_start = min(r[0] for r in search.pending_ranges)
            search.completed_up_to = min_pending_start
        
        if found_m is not None:
            # Found a prime! Update best candidate if better
            if search.best_candidate is None or found_m < search.best_candidate:
                search.best_candidate = found_m
        
        # Check if search is complete:
        # Have a candidate AND all offsets up to that candidate have COMPLETED
        if search.best_candidate is not None:
            if search.completed_up_to >= search.best_candidate:
                self._finalize_result(n, search.best_candidate, worker_id)
                return  # Already saved in _finalize_result
        
        # Live bookkeeping: save after every batch for power-loss recovery
        self._save_checkpoint()
    
    def _finalize_result(self, n: int, f_n: int, worker_id: int) -> None:
        """Finalize and record a result."""
        search = self.state.searches[n]
        search.completed = True
        search.pending_ranges = []  # Clean up stale metadata
        search.completed_up_to = search.next_offset  # Mark all as done
        self.state.results[n] = f_n
        
        # Total elapsed: baseline from checkpoint + current session time
        elapsed = self.baseline_elapsed + (time.time() - self.start_time)
        self.state.result_times[n] = elapsed
        
        remaining = (self.end_n - self.start_n + 1) - len(self.state.results)
        
        # Log to markdown table
        if self.logger:
            self.logger.log_row(
                elapsed_str=format_duration(elapsed),
                n=n,
                f_n=f_n,
                worker_tasks=self.worker_tasks,
                finished_worker=worker_id,
            )
        
        print(
            f"F({n}) = {f_n:6d} | "
            f"batch={self.batch_sizer.get_batch_size():4d} | "
            f"elapsed={format_duration(elapsed)} | "
            f"remaining={remaining}"
        )
        
        # Save checkpoint
        self._save_checkpoint()
    
    def _save_checkpoint(self) -> None:
        """Save current state to checkpoint."""
        self.state.batch_times = self.batch_sizer.get_times_list()
        self.state.current_batch_size = self.batch_sizer.get_batch_size()
        # Compute total elapsed: baseline from checkpoint + current session time
        self.state.total_elapsed = self.baseline_elapsed + (time.time() - self.start_time)
        self.checkpoint_mgr.save(self.state)
    
    def _all_complete(self) -> bool:
        """Check if all computations are complete."""
        return len(self.state.results) >= (self.end_n - self.start_n + 1)
    
    def _any_workers_busy(self) -> bool:
        """Check if any workers are still working."""
        return any(t is not None for t in self.worker_tasks.values())
    
    def run(self) -> Dict[int, int]:
        """Run the expedition. Returns results dict."""
        self._setup_signal_handlers()
        self._start_workers()
        
        print(f"Expedition F({self.start_n})..F({self.end_n}) with {self.num_workers} workers")
        print(f"Checkpoint: {self.checkpoint_mgr.filepath}")
        print()
        
        try:
            # Initial dispatch
            self._dispatch_tasks()
            
            # Main loop
            while not self._all_complete() and not self.shutdown_requested:
                try:
                    result = self.result_queue.get(timeout=0.5)
                    self._process_result(*result)
                    self._dispatch_tasks()
                except Exception:
                    continue
            
            # No drain on shutdown - pending_ranges will be re-dispatched on resume
        
        finally:
            self._save_checkpoint()
            self._stop_workers()
            
            # Finalize markdown log
            if self.logger:
                # Use time of last result found if complete, else current elapsed time
                if self._all_complete() and self.state.result_times:
                    total_elapsed = max(self.state.result_times.values())
                else:
                    total_elapsed = self.baseline_elapsed + (time.time() - self.start_time)
                self.logger.log_summary(self.state.results, total_elapsed)
                self.logger.close()
            
            if self._all_complete():
                print(f"\nExpedition complete! {len(self.state.results)} results.")
                self.checkpoint_mgr.remove()
            else:
                print(f"\nExpedition paused. {len(self.state.results)} results saved.")
                print(f"Resume with: python {sys.argv[0]} {self.start_n} {self.end_n} --resume")
        
        return self.state.results


# =============================================================================
# CLI
# =============================================================================

def show_checkpoint_status(checkpoint_path: Path) -> None:
    """Display checkpoint file status."""
    mgr = CheckpointManager(checkpoint_path)
    
    if not mgr.exists():
        print(f"No checkpoint found at: {checkpoint_path}")
        return
    
    state = mgr.load()
    if state is None:
        print("Checkpoint file corrupted or invalid")
        return
    
    completed = len(state.results)
    total = state.end_n - state.start_n + 1
    
    print(f"Checkpoint: {checkpoint_path}")
    print(f"Range: F({state.start_n})..F({state.end_n})")
    print(f"Progress: {completed}/{total} ({100*completed/total:.1f}%)")
    print(f"Total elapsed: {format_duration(state.total_elapsed)}")
    print(f"Current batch size: {state.current_batch_size}")
    
    if state.results:
        print(f"\nCompleted results:")
        for n in sorted(state.results.keys())[:10]:
            print(f"  F({n}) = {state.results[n]}")
        if len(state.results) > 10:
            print(f"  ... and {len(state.results) - 10} more")


def main() -> None:
    """Command-line interface."""
    if len(sys.argv) < 2:
        print("Usage: python fortunate_expedition.py <start_n> <end_n> [--resume] [--log FILE]")
        print("       python fortunate_expedition.py --status")
        print()
        print("Options:")
        print("  --resume    Resume from checkpoint")
        print("  --log FILE  Write worker log to markdown file")
        print()
        print("Examples:")
        print("  python fortunate_expedition.py 4610 4650")
        print("  python fortunate_expedition.py 4610 4650 --resume")
        print("  python fortunate_expedition.py 4610 4650 --log workers.md")
        print("  python fortunate_expedition.py --status")
        sys.exit(1)
    
    # Handle --status
    if sys.argv[1] == "--status":
        show_checkpoint_status(Path(CHECKPOINT_FILE))
        return
    
    # Parse arguments
    start_n = int(sys.argv[1])
    end_n = int(sys.argv[2]) if len(sys.argv) > 2 and sys.argv[2].isdigit() else start_n
    resume = "--resume" in sys.argv
    
    # Parse --log option
    log_path: Optional[Path] = None
    if "--log" in sys.argv:
        log_idx = sys.argv.index("--log")
        if log_idx + 1 < len(sys.argv):
            log_path = Path(sys.argv[log_idx + 1])
    
    # Run expedition
    expedition = Expedition(start_n, end_n, resume=resume, log_path=log_path)
    results = expedition.run()
    
    # Print summary
    if results:
        print("\nResults:")
        for n in sorted(results.keys()):
            print(f"  F({n}) = {results[n]}")


if __name__ == "__main__":
    main()
