#!/usr/bin/env python3
"""Comprehensive SIMD performance benchmark for cross-platform testing."""

import numpy as np
import time
import platform
import subprocess
import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../python"))


def get_cpu_info():
    """Get CPU information for the benchmark report."""
    info = {
        "platform": platform.platform(),
        "processor": platform.processor(),
        "python": platform.python_version(),
    }

    # Try to get more detailed CPU info
    try:
        if platform.system() == "Linux":
            # Get CPU model
            with open("/proc/cpuinfo", "r") as f:
                for line in f:
                    if "model name" in line:
                        info["cpu_model"] = line.split(":")[1].strip()
                        break

            # Get CPU frequency
            result = subprocess.run(["lscpu"], capture_output=True, text=True)
            if result.returncode == 0:
                for line in result.stdout.split("\n"):
                    if "CPU MHz" in line:
                        info["cpu_mhz"] = line.split(":")[1].strip()
                    elif "CPU max MHz" in line:
                        info["cpu_max_mhz"] = line.split(":")[1].strip()

        elif platform.system() == "Darwin":
            result = subprocess.run(
                ["sysctl", "-n", "machdep.cpu.brand_string"],
                capture_output=True,
                text=True,
            )
            if result.returncode == 0:
                info["cpu_model"] = result.stdout.strip()
    except:
        pass

    return info


def benchmark_dimension(dimension, num_vectors=10000, num_queries=1000):
    """Benchmark performance at a specific dimension."""
    import omendb

    # Generate test data
    np.random.seed(42)
    vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
    queries = np.random.rand(num_queries, dimension).astype(np.float32)

    # Create database and add vectors
    db = omendb.DB()

    # Measure insertion performance
    start_time = time.time()
    for i in range(num_vectors):
        db.add(f"vec_{i}", vectors[i].tolist())
    insert_time = time.time() - start_time
    insert_rate = num_vectors / insert_time

    # Measure query performance
    start_time = time.time()
    for query in queries:
        results = db.search(query.tolist(), limit=10)
    query_time = time.time() - start_time
    query_rate = num_queries / query_time
    query_latency_ms = (query_time / num_queries) * 1000

    return {
        "dimension": dimension,
        "insert_rate": insert_rate,
        "query_rate": query_rate,
        "query_latency_ms": query_latency_ms,
        "num_vectors": num_vectors,
        "num_queries": num_queries,
    }


def main():
    """Run comprehensive benchmarks."""
    print("=" * 80)
    print("OmenDB SIMD Performance Benchmark")
    print("=" * 80)

    # Print system information
    cpu_info = get_cpu_info()
    print("\nSystem Information:")
    for key, value in cpu_info.items():
        print(f"  {key}: {value}")

    # Test dimensions
    dimensions = [32, 64, 128, 256, 512, 768, 1024, 1536]

    print("\n" + "=" * 80)
    print("Running benchmarks...")
    print("=" * 80)

    results = []

    for dim in dimensions:
        print(f"\nTesting {dim}D vectors...")

        # Adjust number of vectors for larger dimensions to keep runtime reasonable
        if dim <= 256:
            num_vectors = 10000
            num_queries = 1000
        elif dim <= 768:
            num_vectors = 5000
            num_queries = 500
        else:
            num_vectors = 2000
            num_queries = 200

        result = benchmark_dimension(dim, num_vectors, num_queries)
        results.append(result)

        print(f"  Insert rate: {result['insert_rate']:,.0f} vec/s")
        print(f"  Query rate: {result['query_rate']:,.0f} queries/s")
        print(f"  Query latency: {result['query_latency_ms']:.2f} ms")

    # Print summary table
    print("\n" + "=" * 80)
    print("SUMMARY TABLE")
    print("=" * 80)
    print(
        f"{'Dimension':>10} | {'Insert (vec/s)':>15} | {'Query (q/s)':>12} | {'Latency (ms)':>12}"
    )
    print("-" * 80)

    for r in results:
        print(
            f"{r['dimension']:>10}D | {r['insert_rate']:>15,.0f} | "
            f"{r['query_rate']:>12,.0f} | {r['query_latency_ms']:>12.2f}"
        )

    # Save detailed results
    timestamp = time.strftime("%Y%m%d_%H%M%S")
    filename = f"simd_benchmark_{platform.system().lower()}_{timestamp}.txt"

    with open(filename, "w") as f:
        f.write("OmenDB SIMD Performance Benchmark Results\n")
        f.write("=" * 80 + "\n\n")

        f.write("System Information:\n")
        for key, value in cpu_info.items():
            f.write(f"  {key}: {value}\n")

        f.write("\n" + "=" * 80 + "\n")
        f.write("Detailed Results:\n")
        f.write("=" * 80 + "\n\n")

        for r in results:
            f.write(f"Dimension: {r['dimension']}D\n")
            f.write(f"  Vectors: {r['num_vectors']}\n")
            f.write(f"  Queries: {r['num_queries']}\n")
            f.write(f"  Insert rate: {r['insert_rate']:,.0f} vec/s\n")
            f.write(f"  Query rate: {r['query_rate']:,.0f} queries/s\n")
            f.write(f"  Query latency: {r['query_latency_ms']:.2f} ms\n\n")

    print(f"\nResults saved to: {filename}")
    print("\nBenchmark complete!")


if __name__ == "__main__":
    main()
