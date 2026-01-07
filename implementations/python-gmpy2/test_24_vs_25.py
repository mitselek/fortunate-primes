#!/usr/bin/env python3
"""
Compare Miller-Rabin rounds: test the same composite with 24 vs 25 rounds
"""

import sys
sys.set_int_max_str_digits(10000)

import gmpy2  # type: ignore
import time

n = 4601
offset = 44207

print(f"Testing F({n}) at offset {offset}...")

# Compute primorial(n)
start = time.time()
pn = 1
p = 2
for _ in range(n):
    pn *= p
    p = int(gmpy2.next_prime(p))  # type: ignore[attr-defined]
prim_time = time.time() - start

candidate = pn + offset

# Test with 24 rounds
print("\n=== Testing with 24 rounds ===")
start = time.time()
result_24 = gmpy2.is_prime(candidate, 24)  # type: ignore[attr-defined]
time_24 = time.time() - start
print(f"Result: {'PRIME' if result_24 else 'COMPOSITE'}")
print(f"Time: {time_24:.2f}s")

# Test with 25 rounds
print("\n=== Testing with 25 rounds ===")
start = time.time()
result_25 = gmpy2.is_prime(candidate, 25)  # type: ignore[attr-defined]
time_25 = time.time() - start
print(f"Result: {'PRIME' if result_25 else 'COMPOSITE'}")
print(f"Time: {time_25:.2f}s")

# Summary
print("\n=== Summary ===")
if result_24 == result_25:
    print(f"✓ Both rounds agree: {result_24}")
else:
    print(f"✗ DISAGREEMENT:")
    print(f"  24 rounds: {'PRIME' if result_24 else 'COMPOSITE'}")
    print(f"  25 rounds: {'PRIME' if result_25 else 'COMPOSITE'}")
    print(f"  >>> Round 25 caught a FALSE POSITIVE! <<<")

print(f"\nTime difference: {time_25 - time_24:.2f}s ({(time_25/time_24 - 1)*100:.1f}% slower with 25 rounds)")
