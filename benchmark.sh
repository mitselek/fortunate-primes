#!/bin/bash
# Quick benchmark script for Fortunate numbers

BINARY="./target/release/fortunate"

if [ ! -f "$BINARY" ]; then
    echo "Binary not found. Building..."
    cargo build --release
fi

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║        Fortunate Numbers - Scaling Performance           ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

for n in 100 200 300 400 500 750 1000; do
    echo "Benchmarking n=$n..."
    # Send: menu choice 2 (benchmark), n value, then exit (3)
    timeout 120 bash -c "printf '2\n$n\n3\n' | $BINARY" 2>&1 | grep -A 20 "Benchmarking n="
    echo ""
done
