#!/usr/bin/env python3
"""
Test the specific offset that took 22.59s: F(4601) at offset 44207
"""

import sys
sys.set_int_max_str_digits(10000)  # Allow large integer string conversion

import gmpy2  # type: ignore
import time

n = 4601
offset = 44207

print(f"Testing F({n}) at offset {offset}...")

# Compute primorial(n) = product of first n primes
start = time.time()
pn = 1
p = 2
for _ in range(n):
    pn *= p
    p = int(gmpy2.next_prime(p))  # type: ignore[attr-defined]
prim_time = time.time() - start
print(f"Primorial computed in {prim_time:.2f}s")

# Test the candidate
candidate = pn + offset
print(f"Testing if primorial({n}) + {offset} is prime...")

start = time.time()
result = gmpy2.is_prime(candidate, 25)  # type: ignore[attr-defined]
test_time = time.time() - start

print(f"Result: {'PRIME' if result else 'COMPOSITE'}")
print(f"Test time: {test_time:.2f}s")
print(f"Total time: {test_time + prim_time:.2f}s")

if result:
    print(f"\n>>> F({n}) = {offset} <<<")
else:
    print(f"(Not prime, which is expected for most offsets)")
