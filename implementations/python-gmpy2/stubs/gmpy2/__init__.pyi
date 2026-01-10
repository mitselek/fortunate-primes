"""
Type stubs for gmpy2 - GMP/MPIR, MPFR, and MPC for Python.

This file provides type hints for the gmpy2 functions used in this project.
It is not a complete type stub for gmpy2, only the functions we use.

Reference: https://gmpy2.readthedocs.io/en/latest/
"""

from typing import Union

# Type alias for integers (int or gmpy2.mpz)
IntLike = Union[int, "mpz"]


class mpz:
    """GMP arbitrary precision integer type."""
    
    def __init__(self, value: Union[int, str, "mpz"] = 0, base: int = 0) -> None: ...
    def __int__(self) -> int: ...
    def __add__(self, other: IntLike) -> "mpz": ...
    def __radd__(self, other: IntLike) -> "mpz": ...
    def __sub__(self, other: IntLike) -> "mpz": ...
    def __rsub__(self, other: IntLike) -> "mpz": ...
    def __mul__(self, other: IntLike) -> "mpz": ...
    def __rmul__(self, other: IntLike) -> "mpz": ...
    def __floordiv__(self, other: IntLike) -> "mpz": ...
    def __mod__(self, other: IntLike) -> "mpz": ...
    def __pow__(self, other: IntLike, mod: IntLike = ...) -> "mpz": ...
    def __neg__(self) -> "mpz": ...
    def __abs__(self) -> "mpz": ...
    def __eq__(self, other: object) -> bool: ...
    def __ne__(self, other: object) -> bool: ...
    def __lt__(self, other: IntLike) -> bool: ...
    def __le__(self, other: IntLike) -> bool: ...
    def __gt__(self, other: IntLike) -> bool: ...
    def __ge__(self, other: IntLike) -> bool: ...
    def __hash__(self) -> int: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def bit_length(self) -> int: ...


def is_prime(n: IntLike, k: int = 25) -> bool:
    """
    Test if n is prime using Miller-Rabin primality test.
    
    Args:
        n: The number to test for primality.
        k: Number of Miller-Rabin rounds (default 25).
           Higher k = more accuracy, slower.
    
    Returns:
        True if n is probably prime, False if definitely composite.
        With k=25, false positive probability is < 2^(-50).
    """
    ...


def next_prime(n: IntLike) -> mpz:
    """
    Return the smallest prime greater than n.
    
    Args:
        n: Starting number.
    
    Returns:
        The next prime after n as an mpz.
    """
    ...


def primorial(n: IntLike) -> mpz:
    """
    Return the product of all primes <= n.
    
    Note: This computes p# (primorial of p), not primorial(n) as the product
    of the first n primes. For the latter, use compute_primorial() helper.
    
    Args:
        n: Upper bound for primes to multiply.
    
    Returns:
        Product of all primes <= n.
    """
    ...


def gcd(a: IntLike, b: IntLike) -> mpz:
    """Return the greatest common divisor of a and b."""
    ...


def lcm(a: IntLike, b: IntLike) -> mpz:
    """Return the least common multiple of a and b."""
    ...


def isqrt(n: IntLike) -> mpz:
    """Return the integer square root of n."""
    ...


def iroot(n: IntLike, k: int) -> tuple[mpz, bool]:
    """
    Return (root, exact) where root is the integer k-th root of n.
    exact is True if root**k == n.
    """
    ...


def is_square(n: IntLike) -> bool:
    """Return True if n is a perfect square."""
    ...


def factorial(n: IntLike) -> mpz:
    """Return n!."""
    ...


def fib(n: IntLike) -> mpz:
    """Return the n-th Fibonacci number."""
    ...


def fib2(n: IntLike) -> tuple[mpz, mpz]:
    """Return (F(n), F(n-1))."""
    ...


# Version info
version: str
mp_version: str
mpfr_version: str
mpc_version: str
