#!/bin/bash
# Benchmark PARI/GP implementations against Rust baseline
# Usage: ./benchmark.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== PARI/GP Implementation Benchmarks ==="
echo ""

# Check if gp is available
if ! command -v gp &> /dev/null; then
    echo "Error: PARI/GP (gp) not found. Install with:"
    echo "  sudo apt-get install pari-gp  # Debian/Ubuntu"
    echo "  brew install pari             # macOS"
    exit 1
fi

echo "PARI/GP version:"
gp --version-short
echo ""

# Test cases
declare -a TEST_CASES=(
    "5:23"
    "10:61"
    "20:103"
)

BENCH_CASES=(
    "500:5167",
    "1000:8719",
    "2000:51137"
)

echo "=== Unit Tests (Interleaved Strategy) ==="
for test in "${TEST_CASES[@]}"; do
    n="${test%%:*}"
    expected="${test##*:}"
    echo -n "F($n) = $expected ... "

    result=$(gp -q << EOF
\r fortunate-interleaved.gp
fortunate_interleaved($n);
quit
EOF
)

    # Extract F(n) = value from output
    actual=$(echo "$result" | grep "F($n) =" | sed 's/.*= \([0-9]*\).*/\1/')

    if [ "$actual" = "$expected" ]; then
        echo "✓"
    else
        echo "✗ (got $actual)"
    fi
done

echo ""
echo "=== Unit Tests (Batch Strategy, batch_size=50) ==="
for test in "${TEST_CASES[@]}"; do
    n="${test%%:*}"
    expected="${test##*:}"
    echo -n "F($n) = $expected ... "

    result=$(gp -q << EOF
\r fortunate-batch.gp
fortunate_batch($n, 50);
quit
EOF
)

    # Extract F(n) = value from output
    actual=$(echo "$result" | grep "F($n) =" | sed 's/.*= \([0-9]*\).*/\1/')

    if [ "$actual" = "$expected" ]; then
        echo "✓"
    else
        echo "✗ (got $actual)"
    fi
done

echo ""
echo "=== Performance Benchmarks ==="
echo ""

# Benchmark F(500) - Interleaved
echo "--- F(500) with Interleaved Strategy ---"
time gp -q << 'EOF'
\r fortunate-interleaved.gp
fortunate_interleaved(500);
quit
EOF

echo ""

# Benchmark F(500) - Batch with different sizes
for batch_size in 50 100 200; do
    echo "--- F(500) with Batch Strategy (size=$batch_size) ---"
    time gp -q << EOF
\r fortunate-batch.gp
fortunate_batch(500, $batch_size);
quit
EOF
    echo ""
done

echo ""
echo "=== Comparison with Rust Baseline ==="
echo "Rust F(500) = 5167 in 11.31s (worker-count-aware adaptive batching)"
echo ""
echo "Results saved. Compare PARI/GP times above with Rust baseline."
