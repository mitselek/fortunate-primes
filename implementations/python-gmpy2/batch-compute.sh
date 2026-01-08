#!/bin/bash
#
# Batch Fortunate number calculator
# Sequentially computes F(n), F(n+1), F(n+2), ... F(m)
# Output is logged to a timestamped file
#
# Usage:
#   ./batch-compute.sh 4600          # F(4600), F(4601), ... F(4610) (default: +10)
#   ./batch-compute.sh 4600 4605     # F(4600) through F(4605)
#   ./batch-compute.sh 4600 4605 20  # Each with 20 workers

set -e

if [ $# -lt 1 ]; then
    echo "Usage: $0 <starting_n> [ending_n] [num_workers]"
    echo ""
    echo "Examples:"
    echo "  $0 4600              # Compute F(4600) through F(4610) with default workers"
    echo "  $0 4600 4605         # Compute F(4600) through F(4605)"
    echo "  $0 4600 4605 12      # Same, but use 12 workers instead of default (15)"
    exit 1
fi

start=$1
end=${2:-$((start + 10))}
workers=${3:-15}

if [ $start -gt $end ]; then
    echo "Error: starting_n ($start) must be <= ending_n ($end)"
    exit 1
fi

count=$((end - start + 1))

# Create logs directory if it doesn't exist
mkdir -p logs

# Generate timestamped log filename
timestamp=$(date +%Y%m%d_%H%M%S)
logfile="logs/batch_F${start}-F${end}_${timestamp}.log"

echo "=================================="
echo "Batch Fortunate Number Calculator"
echo "=================================="
echo "Range: F($start) to F($end)"
echo "Count: $count numbers"
echo "Workers: $workers"
echo "Logging to: $logfile"
echo ""

{
    echo "=================================="
    echo "Batch Fortunate Number Calculator"
    echo "=================================="
    echo "Start time: $(date)"
    echo "Range: F($start) to F($end)"
    echo "Count: $count numbers"
    echo "Workers: $workers"
    echo ""

    total_start=$(date +%s)

    for n in $(seq $start $end); do
        echo ""
        echo "────────────────────────────────────────"
        echo "Computing F($n) [$((n - start + 1))/$count]"
        echo "────────────────────────────────────────"

        batch_start=$(date +%s)

        if .venv/bin/python fortunate_v2.py "$n"; then
            batch_end=$(date +%s)
            batch_elapsed=$((batch_end - batch_start))
            echo "✓ F($n) completed in ${batch_elapsed}s"
        else
            echo "✗ F($n) failed - skipping"
        fi
    done

    total_end=$(date +%s)
    total_elapsed=$((total_end - total_start))

    echo ""
    echo "=================================="
    echo "Batch Complete!"
    echo "=================================="
    echo "End time: $(date)"
    echo "Total time: ${total_elapsed}s (~$((total_elapsed / 60))m)"
    echo ""

} | tee "$logfile"

echo "Log saved to: $logfile"

