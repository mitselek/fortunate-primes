---
name: Adaptive Batch Sizing with Bidirectional Adjustment
about: Implement dynamic batch size adjustment based on completion times
title: "Python: Implement Adaptive Batch Sizing with Performance-Based Adjustment"
labels: enhancement, python, performance
assignees: ""
---

## Summary

Implement adaptive batch sizing for Python implementation that can both **increase** and **decrease** batch sizes based on completion time feedback, optimizing for a target batch duration throughout execution.

## Current Behavior

- Fixed batch size (default 50) throughout entire run
- Rust has adaptive sizing but only grows (doubles when batches complete quickly)
- No mechanism to shrink batch size if performance degrades during long runs

## Proposed Behavior

### Dynamic Batch Size Adjustment

**Algorithm:**

1. **Initialize**:

   - `batch_size = 1` (start small)
   - `target_time = 60.0 / num_workers` (desired completion time in seconds)
   - `tk = 2.0` (threshold constant for acceptable range)
   - Track `(batch_start, batch_size, dispatch_time)` for in-flight batches

2. **On Batch Completion**:

   - Calculate `completion_time = end_time - start_time`
   - Compare to acceptable range `[target_time/tk, target_time*tk]`

3. **Adjustment Rules**:

   **Case A: Too Fast** (`completion_time < target_time / tk`)

   - If `recent_batch_size <= batch_size`: Do nothing (already growing)
   - If `recent_batch_size > batch_size`: Set `batch_size = recent_batch_size * tk`

   **Case B: Too Slow** (`completion_time > target_time * tk`)

   - If `recent_batch_size >= batch_size`: Do nothing (already shrinking)
   - If `recent_batch_size < batch_size`: Set `batch_size = recent_batch_size / tk`

   **Case C: Within Range**: Do nothing (optimal)

### Enhanced Progress Reporting

**Current Format:**

```text
F(4602) : [56452; ?] [57002+50] (35.35m)
```

**Proposed Format:**

```text
F(4602) : [56452; ?] [57002+50] (2.34s) (35.35m)
                                 ^^^^^^   ^^^^^^^
                          batch completion  total elapsed
```

**Components:**

- `F(n)` - Fortunate number being computed
- `[lower; upper]` - Search interval (lower bound; candidate or ?)
- `[start+size]` - Batch range just completed
- `(completion)` - **NEW**: Time taken for this specific batch
- `(elapsed)` - Total wall time since start

## Benefits

1. **Self-tuning**: Automatically finds optimal batch size for current n
2. **Adaptive to system load**: Shrinks batches if system becomes busy
3. **Performance visibility**: Completion times reveal batch efficiency
4. **Handles scaling**: Adjusts as primorial size grows (smaller n vs larger n)
5. **Better coordination**: Target time keeps batches synchronized across workers

## Implementation Details

### Data Structures

```python
# Track in-flight batches
dispatched_batches: Dict[int, Tuple[int, float]] = {}
# Key: batch_start, Value: (batch_size, dispatch_time)

# Current batch size
batch_size: int = 1

# Adaptive parameters
target_time: float = 60.0 / num_workers  # e.g., 3.75s for 16 workers
tk: float = 2.0  # Threshold constant
```

### Modified Dispatch Logic

```python
# When dispatching new batch
dispatch_time = time.time()
work_queue.put((n, next_offset, batch_size))
dispatched_batches[next_offset] = (batch_size, dispatch_time)
```

### Modified Result Processing

```python
# When receiving batch result
batch_start, batch_end, result = result_queue.get(timeout=0.1)
recent_batch_size = batch_end - batch_start

# Calculate completion time
if batch_start in dispatched_batches:
    original_size, dispatch_time = dispatched_batches.pop(batch_start)
    completion_time = time.time() - dispatch_time

    # Adaptive adjustment
    if completion_time < target_time / tk:
        if recent_batch_size > batch_size:
            batch_size = int(recent_batch_size * tk)
    elif completion_time > target_time * tk:
        if recent_batch_size < batch_size:
            batch_size = max(1, int(recent_batch_size / tk))

    # Enhanced progress reporting
    completion_str = format_duration(completion_time)
    elapsed_str = format_duration(time.time() - start_time)
    print(f"F({n}) : {bounds} [{batch_start}+{recent_batch_size}] ({completion_str}) ({elapsed_str})")
```

## Testing Strategy

1. **F(500)**: Should grow quickly from 1 → ~50-100
2. **F(3000)**: Should adapt to larger primorial (may shrink as n increases)
3. **Under load**: Should shrink if system becomes busy
4. **Long runs**: Verify batch size adjusts as primorial grows

## Success Criteria

- [ ] Batch size starts at 1 and grows to optimal size within first 10-20 batches
- [ ] Progress reports show batch completion times
- [ ] Performance matches or exceeds current fixed batch_size=50
- [ ] System adapts to changing load conditions
- [ ] Works correctly for small (n=500) and large (n=3000+) cases

## Related

- Rust implementation: `implementations/rust/src/search.rs` (lines 140-149) - adaptive doubling
- Current Python: `implementations/python-gmpy2/fortunate_v2.py` - fixed batch size

## Parameters to Tune

- `target_time`: Currently `60.0 / num_workers` (~3.75s for 16 cores)
- `tk`: Threshold constant (2.0 gives range [1.875s, 7.5s] for 16 cores)
- Initial `batch_size`: Start with 1 or allow configuration

**Suggested defaults:**

- `target_time = 60.0 / num_workers`
- `tk = 2.0` (accept 2x variation)
- `initial_batch_size = 1`
