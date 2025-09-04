#!/usr/bin/env python3
"""
Save benchmark results with timestamp.
Runs regression tests and saves results to benchmarks/results/
"""

import subprocess
import json
import os
from datetime import datetime
import platform


def run_benchmark(dimension):
    """Run benchmark for a single dimension."""
    result = subprocess.run(
        [
            "pixi",
            "run",
            "python",
            "test/performance/test_single_dimension.py",
            str(dimension),
        ],
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        return None

    # Parse output
    lines = result.stdout.strip().split("\n")
    for line in lines:
        if "Performance:" in line:
            # Extract performance value
            parts = line.split()
            perf_value = float(parts[1])
            std_value = float(parts[3].strip("(±)"))
            return {"mean": perf_value, "std": std_value}

    return None


def main():
    """Run all benchmarks and save results."""
    dimensions = [32, 64, 128, 256, 384, 512, 768, 1024, 1536]

    # Create results directory if it doesn't exist
    os.makedirs("benchmarks/results", exist_ok=True)

    # Run benchmarks
    results = {
        "timestamp": datetime.now().isoformat(),
        "platform": {
            "system": platform.system(),
            "machine": platform.machine(),
            "processor": platform.processor(),
            "python": platform.python_version(),
        },
        "dimensions": {},
    }

    print("Running benchmarks...")
    for dim in dimensions:
        print(f"Testing {dim}D...", end="", flush=True)
        result = run_benchmark(dim)
        if result:
            results["dimensions"][dim] = result
            print(f" {result['mean']:.0f} vec/s (±{result['std']:.0f})")
        else:
            print(" ERROR")

    # Save results
    filename = (
        f"benchmarks/results/{datetime.now().strftime('%Y%m%d_%H%M%S')}_results.json"
    )
    with open(filename, "w") as f:
        json.dump(results, f, indent=2)

    print(f"\nResults saved to {filename}")

    # Also update latest.json
    with open("benchmarks/results/latest.json", "w") as f:
        json.dump(results, f, indent=2)

    print("Updated benchmarks/results/latest.json")


if __name__ == "__main__":
    main()
