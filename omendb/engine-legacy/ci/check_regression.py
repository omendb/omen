#!/usr/bin/env python3
"""
Check for performance regressions in benchmark results.
"""

import json
import sys
import glob
from typing import Dict, List, Tuple

# Performance thresholds (% degradation that triggers alert)
THRESHOLDS = {
    "insertion_throughput": 0.10,  # 10% degradation
    "query_latency": 0.20,  # 20% increase
    "memory_usage": 0.15,  # 15% increase
}

# Baseline performance targets
BASELINES = {
    "1000": {
        "insertion_throughput": 100000,  # vec/s
        "query_latency_p50": 0.5,  # ms
        "bytes_per_vector": 150000,  # bytes
    },
    "10000": {
        "insertion_throughput": 48000,  # vec/s (our target)
        "query_latency_p50": 3.0,  # ms
        "bytes_per_vector": 20000,  # bytes
    },
}


def load_latest_results() -> Dict:
    """Load the most recent benchmark results."""
    result_files = glob.glob("benchmark_results_*.json")
    if not result_files:
        print("❌ No benchmark results found")
        return None

    latest_file = sorted(result_files)[-1]
    print(f"📊 Loading results from {latest_file}")

    with open(latest_file, "r") as f:
        return json.load(f)


def check_regression(results: Dict) -> List[Tuple[str, float, float]]:
    """Check for performance regressions against baselines."""
    regressions = []

    # Check OmenDB results
    if "OmenDB" not in results:
        print("❌ OmenDB results not found")
        return regressions

    omen_results = results["OmenDB"]["benchmarks"]

    for size_str, baselines in BASELINES.items():
        size = int(size_str)

        if size not in omen_results:
            continue

        bench = omen_results[size]

        # Check insertion throughput
        if "insertion" in bench:
            throughput = bench["insertion"].get("throughput", 0)
            baseline = baselines["insertion_throughput"]

            if throughput < baseline * (1 - THRESHOLDS["insertion_throughput"]):
                degradation = (baseline - throughput) / baseline * 100
                regressions.append(
                    (f"Insertion throughput @{size}", throughput, baseline, degradation)
                )

        # Check query latency
        if "query" in bench:
            latency = bench["query"].get("p50_latency_ms", 999)
            baseline = baselines["query_latency_p50"]

            if latency > baseline * (1 + THRESHOLDS["query_latency"]):
                increase = (latency - baseline) / baseline * 100
                regressions.append(
                    (f"Query latency @{size}", latency, baseline, increase)
                )

        # Check memory usage
        if "memory" in bench:
            bytes_per_vec = bench["memory"].get("bytes_per_vector", 0)
            baseline = baselines["bytes_per_vector"]

            if bytes_per_vec > baseline * (1 + THRESHOLDS["memory_usage"]):
                increase = (bytes_per_vec - baseline) / baseline * 100
                regressions.append(
                    (f"Memory usage @{size}", bytes_per_vec, baseline, increase)
                )

    return regressions


def print_results(regressions: List[Tuple[str, float, float, float]]):
    """Print regression check results."""
    print("\n" + "=" * 60)
    print("🔍 PERFORMANCE REGRESSION CHECK")
    print("=" * 60)

    if not regressions:
        print("\n✅ No performance regressions detected!")
        print("\nCurrent performance:")

        results = load_latest_results()
        if results and "OmenDB" in results:
            omen = results["OmenDB"]["benchmarks"]

            if 10000 in omen:
                bench = omen[10000]
                if "insertion" in bench:
                    print(
                        f"  10K insertion: {bench['insertion'].get('throughput', 0):,.0f} vec/s"
                    )
                if "query" in bench:
                    print(
                        f"  10K query p50: {bench['query'].get('p50_latency_ms', 0):.2f}ms"
                    )
    else:
        print("\n⚠️  Performance regressions detected:")
        print("-" * 60)

        for metric, actual, baseline, change in regressions:
            print(f"\n❌ {metric}:")
            print(f"   Actual: {actual:,.1f}")
            print(f"   Baseline: {baseline:,.1f}")
            print(f"   Degradation: {change:.1f}%")

        return 1  # Exit with error code

    return 0


def main():
    """Main regression check."""
    results = load_latest_results()

    if not results:
        print("⚠️  No results to check")
        return 0

    regressions = check_regression(results)
    return print_results(regressions)


if __name__ == "__main__":
    sys.exit(main())
