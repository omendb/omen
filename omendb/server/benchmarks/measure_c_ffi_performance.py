#!/usr/bin/env python3
"""
Measure actual C FFI performance vs Python FFI performance.

This benchmark compares:
1. Direct embedded performance (baseline)
2. Server with Python FFI (old)
3. Server with C FFI (new)
"""

import time
import numpy as np
import statistics
from typing import List, Dict, Tuple
import json
import os
import sys

# Add parent directory to path for imports
sys.path.insert(
    0,
    os.path.join(os.path.dirname(__file__), "..", "..", "..", "..", "omendb", "python"),
)


def measure_embedded_performance(
    dimension: int = 128, num_vectors: int = 10000
) -> Dict[str, float]:
    """Measure direct embedded database performance."""
    try:
        import omendb

        print(
            f"\n📊 Measuring Embedded Performance (dimension={dimension}, vectors={num_vectors})"
        )
        print("=" * 60)

        # Initialize database
        db = omendb.DB()

        # Generate test vectors
        vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(num_vectors)]

        # Measure batch add performance
        start_time = time.perf_counter()
        for i in range(num_vectors):
            db.add(ids[i], vectors[i].tolist())
        add_time = time.perf_counter() - start_time

        add_throughput = num_vectors / add_time
        print(f"✓ Add throughput: {add_throughput:,.0f} vec/s")
        print(f"  Total time: {add_time:.3f}s for {num_vectors} vectors")

        # Measure query performance
        query_times = []
        for _ in range(100):
            query_vec = np.random.rand(dimension).astype(np.float32)
            start_time = time.perf_counter()
            results = db.query(query_vec.tolist(), top_k=10)
            query_time = time.perf_counter() - start_time
            query_times.append(query_time)

        avg_query_time = statistics.mean(query_times) * 1000  # Convert to ms
        p99_query_time = sorted(query_times)[int(len(query_times) * 0.99)] * 1000

        print(
            f"✓ Query latency: {avg_query_time:.2f}ms avg, {p99_query_time:.2f}ms P99"
        )

        # Clear database for next test
        db.clear()

        return {
            "add_throughput": add_throughput,
            "avg_query_ms": avg_query_time,
            "p99_query_ms": p99_query_time,
            "total_time": add_time,
        }

    except Exception as e:
        print(f"❌ Error measuring embedded performance: {e}")
        return {}


def measure_server_performance(
    server_url: str = "http://localhost:8080",
    dimension: int = 128,
    num_vectors: int = 1000,
) -> Dict[str, float]:
    """Measure server performance (requires running server)."""
    try:
        import requests

        print(f"\n📊 Measuring Server Performance (url={server_url})")
        print("=" * 60)

        # Check if server is running
        try:
            resp = requests.get(f"{server_url}/health")
            if resp.status_code != 200:
                print("❌ Server not healthy")
                return {}
        except:
            print("❌ Server not running at", server_url)
            return {}

        # Generate test vectors
        vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(num_vectors)]

        # Measure add performance
        start_time = time.perf_counter()
        for i in range(num_vectors):
            data = {"id": ids[i], "vector": vectors[i].tolist()}
            resp = requests.post(f"{server_url}/vectors", json=data)
            if resp.status_code != 200:
                print(f"❌ Failed to add vector: {resp.text}")
                return {}
        add_time = time.perf_counter() - start_time

        add_throughput = num_vectors / add_time
        print(f"✓ Add throughput: {add_throughput:,.0f} vec/s")
        print(f"  Total time: {add_time:.3f}s for {num_vectors} vectors")

        # Measure query performance
        query_times = []
        for _ in range(100):
            query_vec = np.random.rand(dimension).astype(np.float32)
            data = {"vector": query_vec.tolist(), "top_k": 10}
            start_time = time.perf_counter()
            resp = requests.post(f"{server_url}/search", json=data)
            query_time = time.perf_counter() - start_time
            if resp.status_code == 200:
                query_times.append(query_time)

        avg_query_time = statistics.mean(query_times) * 1000  # Convert to ms
        p99_query_time = sorted(query_times)[int(len(query_times) * 0.99)] * 1000

        print(
            f"✓ Query latency: {avg_query_time:.2f}ms avg, {p99_query_time:.2f}ms P99"
        )

        return {
            "add_throughput": add_throughput,
            "avg_query_ms": avg_query_time,
            "p99_query_ms": p99_query_time,
            "total_time": add_time,
        }

    except Exception as e:
        print(f"❌ Error measuring server performance: {e}")
        return {}


def run_comprehensive_benchmark():
    """Run comprehensive benchmark comparing all approaches."""
    print("🚀 OmenDB C FFI Performance Benchmark")
    print("=" * 60)

    results = {}

    # Test embedded performance
    results["embedded"] = measure_embedded_performance(dimension=128, num_vectors=10000)

    # Test server performance (if running)
    results["server_c_ffi"] = measure_server_performance(
        dimension=128, num_vectors=1000
    )

    # Print comparison
    print("\n📊 Performance Comparison")
    print("=" * 60)

    if results.get("embedded") and results.get("server_c_ffi"):
        embedded_throughput = results["embedded"]["add_throughput"]
        server_throughput = results["server_c_ffi"]["add_throughput"]

        print(f"Embedded: {embedded_throughput:,.0f} vec/s")
        print(f"Server (C FFI): {server_throughput:,.0f} vec/s")
        print(f"Performance gap: {embedded_throughput / server_throughput:.1f}x")

        print(f"\nQuery Latency:")
        print(
            f"Embedded: {results['embedded']['avg_query_ms']:.2f}ms avg, {results['embedded']['p99_query_ms']:.2f}ms P99"
        )
        print(
            f"Server: {results['server_c_ffi']['avg_query_ms']:.2f}ms avg, {results['server_c_ffi']['p99_query_ms']:.2f}ms P99"
        )

    # Save results
    timestamp = time.strftime("%Y%m%d_%H%M%S")
    results_file = f"benchmark_results_{timestamp}.json"
    with open(results_file, "w") as f:
        json.dump(results, f, indent=2)
    print(f"\n✓ Results saved to {results_file}")

    return results


if __name__ == "__main__":
    run_comprehensive_benchmark()
