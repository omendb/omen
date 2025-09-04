#!/usr/bin/env python3
"""
Test a single dimension - designed for AI agent use.
Run with: pixi run python test/performance/test_single_dimension.py <dimension>
"""

import sys
import time
import numpy as np
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../python"))
import omendb


def test_dimension(dim: int, num_vectors: int = 3000):
    """Test performance at a specific dimension."""
    db = omendb.DB()
    vectors = np.random.rand(num_vectors, dim).astype(np.float32)

    # Warmup
    for i in range(100):
        db.add(f"w{i}", vectors[i].tolist())

    # Run 3 times for stability
    rates = []
    for run in range(3):
        start = time.perf_counter()
        start_idx = 100 + run * 900
        end_idx = start_idx + 900

        for i in range(start_idx, min(end_idx, num_vectors)):
            db.add(f"v{i}", vectors[i].tolist())

        elapsed = time.perf_counter() - start
        count = min(end_idx, num_vectors) - start_idx
        rate = count / elapsed
        rates.append(rate)

    avg_rate = np.mean(rates)
    std_rate = np.std(rates)

    # Updated baselines (August 3, 2025)
    baselines = {
        32: 17012,  # Measured: 17012 vec/s
        64: 9563,  # Measured: 9563 vec/s
        128: 5170,  # Measured: 5170 vec/s
        256: 2529,  # Measured: 2529 vec/s
        384: 1854,  # Measured: 1854 vec/s
        512: 1373,  # Measured: 1373 vec/s
    }

    print(f"Dimension: {dim}D")
    print(f"Performance: {avg_rate:.0f} vec/s (±{std_rate:.0f})")

    if dim in baselines:
        baseline = baselines[dim]
        diff = (avg_rate - baseline) / baseline * 100
        print(f"Expected: {baseline} vec/s")
        print(f"Difference: {diff:+.1f}%")

        if diff < -5:
            print("Status: ❌ REGRESSION")
        elif diff > 5:
            print("Status: ✅ IMPROVED")
        else:
            print("Status: ➖ STABLE")
    else:
        print("Status: 📊 NEW DATA")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python test_single_dimension.py <dimension>")
        print("Example: python test_single_dimension.py 128")
        sys.exit(1)

    dim = int(sys.argv[1])
    test_dimension(dim)
