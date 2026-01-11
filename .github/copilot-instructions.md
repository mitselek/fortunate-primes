# Copilot Instructions - Fortunate Primes

Mathematical research project comparing architectures for computing Fortunate numbers F(n) = smallest m > 1 where primorial(n) + m is prime.

## Architecture Overview

Four parallel implementations for cross-architecture comparison:

| Implementation | Language | Status | Best For |
|---------------|----------|--------|----------|
| `python-gmpy2/` | Python 3.12 | **Winner** (9-22x faster) | Production, n ≤ 4650 |
| `pari-gp/` | PARI/GP 2.15 | Runner-up | Prototyping, simplicity |
| `rust/` | Rust 1.92 | Baseline | Very large n, type safety |
| `node-ts/` | TypeScript | Planned | TBD |

**Key insight**: Python+gmpy2 wins because GMP bindings + process isolation (`multiprocessing.Pool`) beats Rust's subprocess orchestration of PARI/GP workers.

## Quick Start Commands

```bash
# Python (fastest - use this)
cd implementations/python-gmpy2
source .venv/bin/activate
python fortunate_marathon.py 500 510 --md output.md   # Range computation
python fortunate_expedition.py 4610 4650 --resume     # Long-running with checkpointing

# Testing & validation
pytest test_fortunate.py -v                           # Validates against OEIS A005235

# Rust (baseline reference)
cd implementations/rust
cargo build --release
./target/release/fortunate-primes 500
cargo test && cargo clippy && cargo fmt -- --check

# PARI/GP (simple scripting)
cd implementations/pari-gp
gp -q fortunate.gp
```

## Python Implementation Variants

| Script | Use Case |
|--------|----------|
| `fortunate_marathon.py` | **Production**: Real-time markdown table, queue-based workers |
| `fortunate_expedition.py` | **Long-running**: Checkpointing, adaptive batch sizing, resume support |
| `fortunate_v3.py` / `v2.py` | Legacy: kept for comparison benchmarks |

## OEIS Validation

Ground truth in `OEIS/b005235.txt` (n=1-4600). Tests in `test_fortunate.py` validate against this:

```python
# Pattern: test functions import from fortunate.py, use OEIS reference
@pytest.mark.parametrize("n,expected", [(5, 23), (10, 61), (500, 5167)])
def test_oeis_validation(n: int, expected: int) -> None:
    result = fortunate_batch(n, batch_size=100, verbose=False)
    assert result == expected
```

Results beyond n=4600 are computed but **unverified** - document as such.

## Key Patterns

### Firoozbakht Optimization
All implementations skip offsets below `p_(n+1)` (the (n+1)th prime) since those can't be Fortunate:
```python
p_n_plus_1 = compute_nth_prime(n + 1)
for offset in range(p_n_plus_1, 1000000):  # Start from p_(n+1), not 2
```

### Process Isolation (Python)
Uses `multiprocessing.Pool` to avoid GIL - each worker is a separate process with its own GMP state:
```python
def worker(worker_id, task_queue, result_queue):
    while True:
        n = task_queue.get()  # Block until task
        if n is None: break   # Poison pill shutdown
        f_n = compute_fortunate(n)
        result_queue.put((worker_id, n, f_n, elapsed))
```

### Checkpoint/Resume (Expedition)
For multi-hour computations, `expedition_checkpoint.json` tracks:
- `next_offset`: Next unassigned search offset
- `pending_ranges`: Ranges currently being computed by workers
- `completed_up_to`: Watermark for continuous completion

Resume with `python fortunate_expedition.py START END --resume`

## Performance Notes

- **Batch sizing**: Python uses 50-candidate batches; PARI/GP uses 100-150
- **Load sensitivity**: Python degrades 2.5x under heavy CPU load but remains fastest
- **F(n) variance**: Within a batch, some F(n) take 150x longer due to "hard" composites (see `ANOMALY_F4601_HARD_COMPOSITE.md`)

## Documentation Files

- `BENCHMARKS.md`: Detailed timing data across implementations
- `CASE_STUDY_*.md`: Anomaly investigations for specific F(n)
- `expedition_*.md`: Session logs for long computations
