#!/usr/bin/env python3
"""
Fabricate an expedition checkpoint from v2 log output.

Parses a running v2 log file and creates a checkpoint that the
expedition version can use to take over the computation.

Usage:
    python fabricate_checkpoint.py path/to/v2_F4610.log
    python fabricate_checkpoint.py path/to/v2_F4610.log --apply  # Also copy to expedition_checkpoint.json
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

import gmpy2  # type: ignore[import-untyped]


def compute_nth_prime(n: int) -> int:
    """Get the nth prime (1-indexed)."""
    if n < 1:
        return 2
    p = 2
    for _ in range(n - 1):
        p = int(gmpy2.next_prime(p))  # type: ignore[no-untyped-call]
    return p


def parse_log(log_path: Path) -> dict[str, Any]:
    """Parse v2 log to extract current state."""
    with open(log_path) as f:
        lines = f.readlines()
    
    # Extract n from header or first log line
    # Format: "Computing F(4610) with..."
    n = None
    for line in lines[:5]:
        match = re.search(r'F\((\d+)\)', line)
        if match:
            n = int(match.group(1))
            break
    
    if n is None:
        raise ValueError("Could not determine n from log file")
    
    # Find the latest confirmed offset
    # Format: "F(4610) W15 : [73951; ?]" - the number in brackets is confirmed_up_to
    confirmed_up_to = None
    elapsed_time = None
    
    for line in reversed(lines):
        # Match pattern like [73951; ?]
        match = re.search(r'\[(\d+); \?\]', line)
        if match and confirmed_up_to is None:
            confirmed_up_to = int(match.group(1))
        
        # Match elapsed time like (85.39m) or (5.24s)
        time_match = re.search(r'\((\d+\.?\d*)([ms])\)\s*$', line.strip())
        if time_match and elapsed_time is None:
            value = float(time_match.group(1))
            unit = time_match.group(2)
            if unit == 'm':
                elapsed_time = value * 60
            else:
                elapsed_time = value
        
        if confirmed_up_to is not None and elapsed_time is not None:
            break
    
    if confirmed_up_to is None:
        raise ValueError("Could not determine confirmed_up_to from log file")
    
    return {
        "n": n,
        "confirmed_up_to": confirmed_up_to,
        "elapsed_seconds": elapsed_time or 0.0
    }


def create_checkpoint(n: int, confirmed_up_to: int, elapsed_seconds: float) -> dict[str, Any]:
    """Create expedition checkpoint dict."""
    p_n_plus_1 = compute_nth_prime(n + 1)
    
    return {
        "start_n": n,
        "end_n": n,
        "results": {},
        "result_times": {},
        "searches": {
            str(n): {
                "n": n,
                "p_n_plus_1": p_n_plus_1,
                "next_offset": confirmed_up_to + 1,
                "completed_up_to": confirmed_up_to,
                "pending_ranges": [],
                "best_candidate": None,
                "completed": False
            }
        },
        "batch_times": [30.0] * 20,  # Seed with reasonable batch times
        "current_batch_size": 16,
        "total_elapsed": elapsed_seconds
    }


def main():
    parser = argparse.ArgumentParser(
        description="Fabricate expedition checkpoint from v2 log"
    )
    parser.add_argument("log_file", type=Path, help="Path to v2 log file")
    parser.add_argument(
        "--apply", "-a", action="store_true",
        help="Copy to expedition_checkpoint.json (ready to resume)"
    )
    parser.add_argument(
        "--output", "-o", type=Path,
        help="Output checkpoint file (default: expedition_checkpoint_{n}.json)"
    )
    args = parser.parse_args()
    
    if not args.log_file.exists():
        print(f"Error: Log file not found: {args.log_file}", file=sys.stderr)
        sys.exit(1)
    
    # Parse the log
    print(f"Parsing {args.log_file}...")
    state = parse_log(args.log_file)
    
    n = state["n"]
    confirmed = state["confirmed_up_to"]
    elapsed = state["elapsed_seconds"]
    
    print(f"  n = {n}")
    print(f"  Confirmed up to offset: {confirmed}")
    print(f"  Elapsed time: {elapsed/60:.1f} minutes")
    
    # Create checkpoint
    checkpoint = create_checkpoint(n, confirmed, elapsed)
    p_start = checkpoint["searches"][str(n)]["p_n_plus_1"]
    
    print(f"\nCheckpoint state:")
    print(f"  Starting offset p_{{n+1}}: {p_start}")
    print(f"  Offsets already searched: {confirmed - p_start + 1}")
    print(f"  Next offset to search: {confirmed + 1}")
    
    # Write output
    output_file = args.output or Path(f"expedition_checkpoint_{n}.json")
    with open(output_file, 'w') as f:
        json.dump(checkpoint, f, indent=2)
    print(f"\nWrote: {output_file}")
    
    if args.apply:
        target = Path("expedition_checkpoint.json")
        with open(target, 'w') as f:
            json.dump(checkpoint, f, indent=2)
        print(f"Applied to: {target}")
        print(f"\nReady to resume:")
        print(f"  python fortunate_expedition.py {n} {n} --resume")
    else:
        print(f"\nTo use this checkpoint:")
        print(f"  mv {output_file} expedition_checkpoint.json")
        print(f"  python fortunate_expedition.py {n} {n} --resume")


if __name__ == "__main__":
    main()
