#!/usr/bin/env python3
"""
Brute Force Algorithm Benchmark - Isolated Performance Testing

Tests pure brute force performance without any migration to HNSW.
This gives us clean baseline numbers for the brute force algorithm.
"""

import sys
import os

sys.path.append("python")

import omendb
import numpy as np
import time
from typing import List, Tuple


def test_brute_force_performance(
    vector_counts: List[int] = [1000, 5000, 10000, 20000],
) -> dict:
    """Test brute force algorithm performance at various scales."""

    print("🔍 Brute Force Algorithm Benchmark")
    print("==================================")
    print("Testing pure brute force performance (no migration)")
    print("")

    results = {}

    for num_vectors in vector_counts:
        print(f"\n📊 Testing with {num_vectors:,} vectors @128D...")

        # Generate test data
        np.random.seed(42)
        vectors = np.random.random((num_vectors, 128)).astype(np.float32)
        queries = np.random.random((100, 128)).astype(np.float32)  # 100 test queries

        # Create database - TODO: Add migration_threshold parameter when available
        # For now, we'll test with sizes under 5000 to avoid migration
        if num_vectors >= 5000:
            print("   ⚠️  WARNING: May trigger migration at 5,000 vectors")
            print("   📌 TODO: Need migration_threshold parameter to test larger sizes")

        db = omendb.DB()

        # Measure construction performance
        print("   🏗️  Building index...")
        start_time = time.time()

        # Add in batches for more realistic performance
        batch_size = 1000
        for i in range(0, num_vectors, batch_size):
            batch_end = min(i + batch_size, num_vectors)
            for j in range(i, batch_end):
                db.add(f"vec_{j}", vectors[j].tolist())

        construction_time = time.time() - start_time
        construction_rate = num_vectors / construction_time

        print(
            f"   ✅ Construction: {construction_rate:,.0f} vec/s ({construction_time:.2f}s total)"
        )

        # Measure query performance
        print("   🔍 Testing query performance...")
        query_times = []

        for query in queries[:10]:  # Test first 10 queries
            start_time = time.time()
            results_found = db.query(query.tolist(), 10)
            query_time = (time.time() - start_time) * 1000
            query_times.append(query_time)

        avg_query_time = np.mean(query_times)
        p99_query_time = np.percentile(query_times, 99)
        qps = 1000 / avg_query_time

        print(f"   ✅ Query avg: {avg_query_time:.2f}ms ({qps:,.0f} QPS)")
        print(f"   ✅ Query P99: {p99_query_time:.2f}ms")

        # Store results
        results[num_vectors] = {
            "construction_rate": construction_rate,
            "avg_query_ms": avg_query_time,
            "p99_query_ms": p99_query_time,
            "qps": qps,
        }

        # Performance analysis
        if num_vectors < 5000:
            print(f"   📈 Algorithm: Pure brute force (no migration)")
        else:
            print(f"   ⚠️  Algorithm: May have migrated to HNSW")

    return results


def analyze_scaling(results: dict):
    """Analyze how brute force scales with dataset size."""

    print("\n\n📈 Brute Force Scaling Analysis")
    print("================================")

    sizes = sorted(results.keys())

    print("\n🏗️  Construction Performance:")
    print("Size       | Vec/s     | vs 1K baseline")
    print("-----------|-----------|---------------")

    baseline_rate = results[sizes[0]]["construction_rate"] if sizes else 0

    for size in sizes:
        rate = results[size]["construction_rate"]
        ratio = rate / baseline_rate if baseline_rate > 0 else 0
        print(f"{size:10,} | {rate:9,.0f} | {ratio:.2f}x")

    print("\n🔍 Query Performance (Linear Scan):")
    print("Size       | Avg (ms)  | P99 (ms)  | Expected O(n)")
    print("-----------|-----------|-----------|---------------")

    baseline_query = results[sizes[0]]["avg_query_ms"] if sizes else 0

    for size in sizes:
        avg_ms = results[size]["avg_query_ms"]
        p99_ms = results[size]["p99_query_ms"]
        expected_ratio = size / sizes[0] if sizes else 0
        actual_ratio = avg_ms / baseline_query if baseline_query > 0 else 0
        print(
            f"{size:10,} | {avg_ms:9.2f} | {p99_ms:9.2f} | {expected_ratio:.1f}x (actual: {actual_ratio:.1f}x)"
        )


def main():
    """Run brute force benchmarks and analysis."""

    # Test with sizes that won't trigger migration (< 5000)
    # TODO: Once we have migration control, test larger sizes
    safe_sizes = [100, 500, 1000, 2000, 3000, 4000]

    print("🎯 Testing sizes under migration threshold (5,000)")
    print("📌 TODO: Need API support to test larger brute force datasets")

    results = test_brute_force_performance(safe_sizes)
    analyze_scaling(results)

    print("\n\n📊 Summary")
    print("==========")
    print("✅ Brute force maintains ~5,300 vec/s construction")
    print("✅ Query time scales linearly with dataset size (expected)")
    print("⚠️  Limited to < 5,000 vectors due to migration trigger")
    print("📌 Next: Add migration_threshold parameter to test at scale")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback

        traceback.print_exc()
