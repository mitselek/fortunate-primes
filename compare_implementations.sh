#!/bin/bash
# Compare PARI/GP vs Rust implementation

echo "=== Performance Comparison: PARI/GP vs Rust ==="
echo ""

# Test values
values=(10 50 100 123 200)

for n in "${values[@]}"; do
    echo "Testing n=$n:"

    # PARI/GP
    pari_result=$(gp -q <<EOF
primorial(n) = { local(result, p); result = 1; forprime(p = 2, prime(n), result *= p;); return(result); }
fortunate(n) = { local(pn, candidate, rounds); pn = primorial(n); candidate = pn + 1; rounds = 0; while(!ispseudoprime(candidate), candidate++; rounds++;); return([candidate - pn, rounds]); }
getabstime(); r = fortunate($n); t = getabstime(); print(r[1], " ", t, " ", r[2]);
EOF
)

    pari_f=$(echo "$pari_result" | awk '{print $1}')
    pari_time=$(echo "$pari_result" | awk '{print $2}')
    pari_iterations=$(echo "$pari_result" | awk '{print $3}')

    echo "  PARI/GP:  F($n) = $pari_f  (${pari_time}ms, $pari_iterations iterations)"

    # Note: We can't easily test Rust in non-interactive mode without modifying the code
    # This is a placeholder for comparison

    echo ""
done

echo ""
echo "Key findings:"
echo "- PARI/GP uses Baillie-PSW test (Miller-Rabin + Lucas)"
echo "- Your Rust uses pure Miller-Rabin with 64 rounds"
echo "- PARI/GP is highly optimized for this specific use case"
