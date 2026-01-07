"""
Unit tests for Fortunate number calculator.

Validates against OEIS A005235: https://oeis.org/A005235
Fortune's conjecture: All Fortunate numbers are prime.
"""

import pytest  # type: ignore
from pathlib import Path
from typing import Dict


# Load OEIS reference data
OEIS_DIR = Path(__file__).parent.parent.parent / "OEIS"
FORTUNATE_NUMBERS: Dict[int, int] = {}  # n -> F(n)

if (OEIS_DIR / "b005235.txt").exists():
    with open(OEIS_DIR / "b005235.txt") as f:
        for line in f:
            if line.strip() and not line.startswith("#"):
                n, fn = map(int, line.split())
                FORTUNATE_NUMBERS[n] = fn


def test_fortunate_5() -> None:
    """F(5) = 23 (OEIS A005235)"""
    from fortunate import fortunate_batch

    result = fortunate_batch(5, batch_size=100, verbose=False)
    assert result == 23, f"Expected F(5)=23, got {result}"


def test_fortunate_10() -> None:
    """F(10) = 61 (OEIS A005235)"""
    from fortunate import fortunate_batch

    result = fortunate_batch(10, batch_size=100, verbose=False)
    assert result == 61, f"Expected F(10)=61, got {result}"


def test_fortunate_20() -> None:
    """F(20) = 103 (OEIS A005235)"""
    from fortunate import fortunate_batch

    result = fortunate_batch(20, batch_size=100, verbose=False)
    assert result == 103, f"Expected F(20)=103, got {result}"


def test_fortunate_result_is_prime() -> None:
    """Fortune's conjecture: All Fortunate numbers must be prime"""
    import gmpy2  # type: ignore
    from fortunate import fortunate_batch

    for n in [5, 10, 20]:
        fn = fortunate_batch(n, batch_size=100, verbose=False)
        assert gmpy2.is_prime(fn), f"F({n})={fn} is not prime!"  # type: ignore[attr-defined]


def test_fortunate_sequential() -> None:
    """Test sequential consistency: F(n) values are deterministic"""
    from fortunate import fortunate_batch

    # Run twice, should get same results
    result1 = fortunate_batch(5, batch_size=100, verbose=False)
    result2 = fortunate_batch(5, batch_size=100, verbose=False)
    assert result1 == result2, "Results should be deterministic"


@pytest.mark.parametrize("n,expected", [  # type: ignore[misc]
    (1, 3), (2, 5), (3, 7), (4, 13), (5, 23),
    (10, 61), (15, 107), (20, 103), (25, 103),
    (30, 191), (40, 191), (50, 293), (100, 641),
])
def test_oeis_validation(n: int, expected: int) -> None:
    """Validate against OEIS A005235 reference values"""
    from fortunate import fortunate_batch

    result = fortunate_batch(n, batch_size=100, verbose=False)
    assert result == expected, f"F({n}): expected {expected}, got {result}"


@pytest.mark.skipif(not FORTUNATE_NUMBERS, reason="OEIS reference data not available")  # type: ignore[misc]
def test_oeis_comprehensive() -> None:
    """Comprehensive validation against all OEIS reference values (up to n=100)"""
    from fortunate import fortunate_batch

    # Test first 100 values (reasonable test duration)
    for n in range(1, min(101, len(FORTUNATE_NUMBERS) + 1)):
        if n in FORTUNATE_NUMBERS:
            expected = FORTUNATE_NUMBERS[n]
            result = fortunate_batch(n, batch_size=100, verbose=False)
            assert result == expected, f"F({n}): expected {expected}, got {result}"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])  # type: ignore[attr-defined]
