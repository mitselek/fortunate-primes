"""
Tests for fortunate_v3.py - Worker-based sequential primorial assignment.

Validates against OEIS A005235 benchmark data.
"""

import pytest
from pathlib import Path
import sys

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent))

from fortunate_v3 import compute_fortunate, compute_fortunates


# Load OEIS A005235 benchmark data
OEIS_DATA = {}
oeis_file = Path(__file__).parent.parent.parent / "OEIS" / "b005235.txt"
if oeis_file.exists():
    with open(oeis_file) as f:
        for line in f:
            parts = line.strip().split()
            if len(parts) >= 2:
                n = int(parts[0])
                f_n = int(parts[1])
                OEIS_DATA[n] = f_n


class TestComputeFortunate:
    """Test compute_fortunate() against OEIS A005235."""
    
    @pytest.mark.parametrize("n,expected_f_n", [
        (1, 3),
        (2, 5),
        (3, 7),
        (4, 13),
        (5, 23),
        (6, 17),
        (7, 19),
        (8, 23),
        (9, 37),
        (10, 61),
    ])
    def test_compute_fortunate_oeis_values(self, n, expected_f_n):
        """Test against known OEIS A005235 values."""
        result = compute_fortunate(n)
        assert result == expected_f_n, f"F({n}) = {result}, expected {expected_f_n}"
    
    @pytest.mark.parametrize("n", range(1, 21))
    def test_compute_fortunate_against_file(self, n):
        """Test first 20 values against OEIS file data."""
        if n not in OEIS_DATA:
            pytest.skip(f"OEIS data for n={n} not available")
        
        expected = OEIS_DATA[n]
        result = compute_fortunate(n)
        assert result == expected, f"F({n}) = {result}, expected {expected}"
    
    def test_compute_fortunate_result_is_prime(self):
        """Fortune's conjecture: all Fortunate numbers should be prime."""
        import gmpy2
        
        for n in range(1, 21):
            f_n = compute_fortunate(n)
            assert gmpy2.is_prime(f_n), f"F({n}) = {f_n} is not prime (Fortune's conjecture violated)"


class TestComputeFortunates:
    """Test compute_fortunates() with multiple workers."""
    
    def test_compute_single_value(self):
        """Test computing single Fortunate number."""
        results = compute_fortunates(1, 1, verbose=False)
        assert len(results) == 1
        assert 1 in results
        assert results[1][0] == 3  # F(1) = 3
    
    def test_compute_range(self):
        """Test computing range of Fortunate numbers."""
        results = compute_fortunates(1, 5, verbose=False)
        assert len(results) == 5
        
        expected = {1: 3, 2: 5, 3: 7, 4: 13, 5: 23}
        for n, (f_n, _) in results.items():
            assert f_n == expected[n], f"F({n}) = {f_n}, expected {expected[n]}"
    
    def test_compute_range_returns_times(self):
        """Test that compute_fortunates returns elapsed times."""
        results = compute_fortunates(1, 3, verbose=False)
        
        for n, (f_n, elapsed) in results.items():
            assert isinstance(f_n, int)
            assert isinstance(elapsed, float)
            assert elapsed > 0
    
    def test_compute_medium_range(self):
        """Test on moderate range (n=100-105)."""
        results = compute_fortunates(100, 105, verbose=False)
        assert len(results) == 6
        
        # Verify all results are positive integers (without loading external data)
        for n, (f_n, elapsed) in results.items():
            assert isinstance(f_n, int)
            assert f_n > 0
            assert elapsed > 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
