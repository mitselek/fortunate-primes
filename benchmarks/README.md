# Cross-Implementation Benchmarking

Infrastructure for comparing performance across all four Fortunate number implementations.

## Standard Test Suite

All implementations are benchmarked against these test cases:

| n    | F(n) | Source             |
| ---- | ---- | ------------------ |
| 5    | 23   | Unit test          |
| 10   | 61   | Unit test          |
| 20   | 103  | Unit test          |
| 500  | 5167 | Standard benchmark |
| 1000 | 8719 | Standard benchmark |

## Running Benchmarks

### Individual Implementation

```bash
cd implementations/rust && cargo build --release && time ./target/release/fortunate-primes 500
cd implementations/pari-gp && time gp -q fortunate.gp 500
cd implementations/python-gmpy2 && source venv/bin/activate && time python fortunate.py 500
cd implementations/node-ts && npm run build && time npm start -- 500
```

### All Implementations

```bash
# From project root
./benchmarks/compare-all.sh

# Output goes to benchmarks/results/*.log
```

### Standard Test Suite

```bash
# Run all test cases across all implementations
./benchmarks/test-suite.sh

# Generates comparison table
```

## Benchmark Scripts

- `compare-all.sh` - Run all implementations with standard parameters
- `test-suite.sh` - Run full test suite (n=5,10,20,500,1000)
- `results/` - Output logs for each run

## Acceptance Criteria

For an implementation to be considered viable:

1. **Correctness**: All test cases must produce correct F(n) values
2. **OEIS validation**: Results match OEIS A005235 through tested range
3. **Performance**: Documented runtime comparison with baseline
4. **Reliability**: Consistent results across multiple runs

## Performance Comparison (Current Results)

| Implementation | F(500)       | F(1000)      | vs Rust (F(500)) | Memory Peak |
| -------------- | ------------ | ------------ | ---------------- | ----------- |
| Python + gmpy2 | **1.25s** ⚡ | **3.86s** ⚡ | **900%**         | ~400 MB     |
| PARI/GP        | 6.8s         | 68.9s*       | 167%             | ~416 MB     |
| Rust           | 11.31s       | 85.8s        | 100% (baseline)  | ~197 MB     |
| Node.js + TS   | N/A          | N/A          | Not implemented  | N/A         |

*PARI/GP F(1000) benchmarked under heavy load (30-36).

**Extended Results:**

| Implementation | F(2500)         | F(4602)    | Notes                    |
| -------------- | --------------- | ---------- | ------------------------ |
| Python + gmpy2 | **2m 50s** ⚡   | -          | 9.6x faster than Rust    |
| Rust           | 27.4m           | 5h 52m     | First result beyond OEIS |

See [main README](../README.md#performance-comparison-clean-system) for detailed analysis and methodology.

## Measurement Methodology

**Timing:**

- Use `time` command or built-in timing
- Report median of 3 runs for n≥500
- Single run acceptable for n<500 (deterministic)

**Memory:**

- Use `/usr/bin/time -v` on Linux
- Use Activity Monitor or `top` on macOS
- Report peak memory usage during computation

**CPU:**

- Record core utilization (e.g., "15 workers @ 99%")
- Note if implementation successfully parallelizes

## Adding New Test Cases

To add new benchmark values:

1. Verify against OEIS A005235: <https://oeis.org/A005235>
2. Add to standard test suite table above
3. Update `test-suite.sh` with new n value
4. Update comparison templates in implementation READMEs

## References

- OEIS A005235: <https://oeis.org/A005235>
- Rust baseline: [../implementations/rust/](../implementations/rust/)
- Implementation comparison: [../implementations/README.md](../implementations/README.md)
