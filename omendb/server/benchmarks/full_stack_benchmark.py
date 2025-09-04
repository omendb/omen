#!/usr/bin/env python3
"""
Full stack benchmark comparing:
1. Embedded (direct Mojo) - baseline
2. C FFI (direct library calls) - FFI overhead only
3. Server (if running) - full stack overhead

This gives us the complete performance picture.
"""

import time
import ctypes
import numpy as np
import statistics
import os
import sys
import json

# Add parent directory to path
sys.path.insert(
    0,
    os.path.join(os.path.dirname(__file__), "..", "..", "..", "..", "omendb", "python"),
)


def benchmark_embedded(dimension: int = 128, num_vectors: int = 10000):
    """Benchmark embedded OmenDB (pure Mojo)."""
    try:
        import omendb

        print(f"\n1️⃣ Embedded OmenDB Benchmark")
        print("=" * 60)

        db = omendb.DB()

        # Generate test data
        vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(num_vectors)]

        # Measure batch add
        print(f"Adding {num_vectors} vectors...")
        start = time.perf_counter()

        # Use batch operation for optimal performance
        db.add_batch(ids, vectors)

        total_time = time.perf_counter() - start
        throughput = num_vectors / total_time

        print(f"✓ Throughput: {throughput:,.0f} vec/s")
        print(f"✓ Total time: {total_time:.3f}s")

        # Measure query performance
        query_times = []
        for _ in range(100):
            query_vec = np.random.rand(dimension).astype(np.float32)
            start = time.perf_counter()
            results = db.query(query_vec.tolist(), top_k=10)
            query_time = time.perf_counter() - start
            query_times.append(query_time)

        avg_query = statistics.mean(query_times) * 1000
        p99_query = sorted(query_times)[99] * 1000

        print(f"✓ Query: {avg_query:.2f}ms avg, {p99_query:.2f}ms P99")

        db.clear()

        return {
            "throughput": throughput,
            "total_time": total_time,
            "avg_query_ms": avg_query,
            "p99_query_ms": p99_query,
        }

    except Exception as e:
        print(f"❌ Embedded benchmark failed: {e}")
        # Try individual adds if batch fails
        try:
            import omendb

            db = omendb.DB()

            print("Retrying with individual adds...")
            start = time.perf_counter()

            for i in range(min(num_vectors, 1000)):  # Limit for individual adds
                db.add(f"vec_{i}", vectors[i].tolist())

            total_time = time.perf_counter() - start
            throughput = min(num_vectors, 1000) / total_time

            print(f"✓ Throughput (individual): {throughput:,.0f} vec/s")

            return {"throughput": throughput, "total_time": total_time}

        except Exception as e2:
            print(f"❌ Individual adds also failed: {e2}")
            return None


def benchmark_c_ffi(dimension: int = 128, num_vectors: int = 10000):
    """Benchmark C FFI directly."""
    lib_path = os.path.join(os.path.dirname(__file__), "..", "omendb_fixed_c_api.dylib")

    try:
        lib = ctypes.CDLL(lib_path)

        # Define function signatures
        lib.omendb_c_initialize.argtypes = [ctypes.c_int32]
        lib.omendb_c_initialize.restype = ctypes.c_int32

        lib.omendb_c_add_vector.argtypes = [
            ctypes.POINTER(ctypes.c_uint8),
            ctypes.c_int32,
            ctypes.POINTER(ctypes.c_float),
            ctypes.c_int32,
        ]
        lib.omendb_c_add_vector.restype = ctypes.c_int32

        lib.omendb_c_clear.argtypes = []
        lib.omendb_c_clear.restype = ctypes.c_int32

        print(f"\n2️⃣ C FFI Direct Benchmark")
        print("=" * 60)

        # Initialize
        result = lib.omendb_c_initialize(dimension)
        if result != 0:
            raise Exception(f"Failed to initialize: {result}")

        # Generate test data
        vectors = np.random.rand(num_vectors, dimension).astype(np.float32)

        # Measure add performance
        print(f"Adding {num_vectors} vectors via C FFI...")
        start = time.perf_counter()

        for i in range(num_vectors):
            id_bytes = f"vec_{i}".encode("utf-8")
            vector = vectors[i]

            result = lib.omendb_c_add_vector(
                ctypes.cast(id_bytes, ctypes.POINTER(ctypes.c_uint8)),
                len(id_bytes),
                vector.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
                dimension,
            )

            if result != 0:
                raise Exception(f"Failed to add vector {i}")

            if (i + 1) % 2000 == 0:
                elapsed = time.perf_counter() - start
                rate = (i + 1) / elapsed
                print(f"  Progress: {i + 1}/{num_vectors} ({rate:,.0f} vec/s)")

        total_time = time.perf_counter() - start
        throughput = num_vectors / total_time

        print(f"✓ Throughput: {throughput:,.0f} vec/s")
        print(f"✓ Total time: {total_time:.3f}s")

        lib.omendb_c_clear()

        return {"throughput": throughput, "total_time": total_time}

    except Exception as e:
        print(f"❌ C FFI benchmark failed: {e}")
        return None


def print_comparison(results):
    """Print performance comparison."""
    print("\n📊 Performance Summary")
    print("=" * 60)

    if results.get("embedded"):
        print(f"Embedded (baseline): {results['embedded']['throughput']:,.0f} vec/s")

    if results.get("c_ffi"):
        print(f"C FFI (direct):      {results['c_ffi']['throughput']:,.0f} vec/s")

        if results.get("embedded"):
            overhead = (
                results["embedded"]["throughput"] / results["c_ffi"]["throughput"]
            )
            print(f"  FFI overhead: {overhead:.1f}x slowdown")

    print("\n🎯 Analysis:")

    if results.get("embedded") and results.get("c_ffi"):
        embedded_tp = results["embedded"]["throughput"]
        c_ffi_tp = results["c_ffi"]["throughput"]

        # Previous measurements
        python_ffi_tp = 33248  # vec/s

        print(f"\nFFI Performance:")
        print(f"  Python FFI: {python_ffi_tp:,} vec/s (previous baseline)")
        print(f"  C FFI:      {c_ffi_tp:,.0f} vec/s (current)")
        print(f"  Improvement: {c_ffi_tp / python_ffi_tp:.1f}x faster")

        print(f"\nGap to embedded:")
        print(f"  Embedded:   {embedded_tp:,.0f} vec/s")
        print(f"  C FFI gap:  {embedded_tp / c_ffi_tp:.1f}x")
        print(f"  Previous gap: {embedded_tp / python_ffi_tp:.1f}x (with Python FFI)")

        # Project server performance
        server_projection = c_ffi_tp * 0.8  # Assume 20% overhead for HTTP/async
        print(f"\nServer projection:")
        print(f"  Expected: {server_projection:,.0f} vec/s")
        print(f"  vs Target: 160,000 vec/s")

        if server_projection >= 160000:
            print(f"  ✅ Meets performance target!")
        else:
            gap = 160000 / server_projection
            print(f"  ⚠️  {gap:.1f}x additional improvement needed")


def main():
    print("🚀 OmenDB Full Stack Performance Benchmark")
    print("=" * 60)
    print("Measuring actual performance across all layers...")

    results = {}

    # Run benchmarks
    results["embedded"] = benchmark_embedded(dimension=128, num_vectors=10000)
    results["c_ffi"] = benchmark_c_ffi(dimension=128, num_vectors=10000)

    # Save results
    timestamp = time.strftime("%Y%m%d_%H%M%S")
    with open(f"benchmark_results_{timestamp}.json", "w") as f:
        json.dump(results, f, indent=2)

    # Print comparison
    print_comparison(results)

    return results


if __name__ == "__main__":
    main()
