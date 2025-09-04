#!/usr/bin/env python3
"""
Deep profiling to find why we're 100x slower than Faiss HNSW.
Faiss: 28K-52K vec/s, Us: 434 vec/s = 65-120x slower
"""

import sys
import os
import time
import numpy as np
import cProfile
import pstats

# Add the parent directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "python"))

import omendb


def profile_hnsw_single_insertion():
    """Profile a single HNSW insertion to find bottlenecks."""
    print("🔬 Profiling Single HNSW Insertion")
    print("=" * 40)

    dimensions = 128

    # Create database and add initial vectors to trigger HNSW
    db = omendb.DB()

    # Add 5000 vectors to trigger HNSW
    print("📦 Adding 5000 vectors to trigger HNSW...")
    vectors = []
    for i in range(5000):
        vector = [float(np.random.randn()) for _ in range(dimensions)]
        vectors.append(vector)

    ids = [f"vec_{i}" for i in range(5000)]
    db.add_batch(list(zip(ids, vectors)))

    stats = db.stats()
    print(f"✅ Algorithm: {stats.get('algorithm')}")

    if stats.get("algorithm") != "hnsw":
        print("❌ HNSW not active, can't profile")
        return

    # Profile single insertions
    print("\n🔬 Profiling individual HNSW insertions...")

    test_vectors = []
    test_ids = []
    for i in range(100):  # Profile 100 insertions
        vector = [float(np.random.randn()) for _ in range(dimensions)]
        test_vectors.append(vector)
        test_ids.append(f"test_vec_{i}")

    # Time individual insertions
    insertion_times = []
    for i in range(100):
        start_time = time.time()
        success = db.add(test_ids[i], test_vectors[i])
        insertion_time = time.time() - start_time
        insertion_times.append(insertion_time)

        if not success:
            print(f"⚠️  Insertion {i} failed")

        if i % 20 == 0:
            avg_time = sum(insertion_times) / len(insertion_times)
            speed = 1.0 / avg_time if avg_time > 0 else 0
            print(f"  Insertion {i}: {avg_time * 1000:.2f}ms avg ({speed:.0f} vec/s)")

    avg_time = sum(insertion_times) / len(insertion_times)
    speed = 1.0 / avg_time if avg_time > 0 else 0

    print(f"\n📊 Individual Insertion Performance:")
    print(f"   Average time: {avg_time * 1000:.2f}ms")
    print(f"   Speed: {speed:.0f} vec/s")
    print(f"   vs Faiss 28K vec/s: {28000 / speed:.1f}x faster")


def analyze_distance_calculation_overhead():
    """Analyze if distance calculations are the bottleneck."""
    print("\n🧮 Analyzing Distance Calculation Overhead")
    print("=" * 45)

    dimensions = 128
    num_comparisons = 10000

    # Generate test data
    query = [float(np.random.randn()) for _ in range(dimensions)]
    vectors = []
    for i in range(num_comparisons):
        vector = [float(np.random.randn()) for _ in range(dimensions)]
        vectors.append(vector)

    # Time pure Python cosine distance
    start_time = time.time()
    distances = []
    for vector in vectors:
        # Simple cosine distance calculation
        dot_product = sum(a * b for a, b in zip(query, vector))
        norm_a = sum(a * a for a in query) ** 0.5
        norm_b = sum(b * b for b in vector) ** 0.5
        if norm_a > 0 and norm_b > 0:
            similarity = dot_product / (norm_a * norm_b)
            distance = 1.0 - similarity
        else:
            distance = 1.0
        distances.append(distance)

    python_time = time.time() - start_time
    python_speed = num_comparisons / python_time

    print(f"📊 Python Distance Calculations:")
    print(f"   {num_comparisons} calculations in {python_time:.3f}s")
    print(f"   Speed: {python_speed:.0f} calc/s")
    print(f"   Per calculation: {python_time / num_comparisons * 1000:.3f}ms")

    # If distance calculations take 0.003ms each and we do hundreds per insertion,
    # that could explain why we're slow
    if python_time / num_comparisons > 0.001:
        print("⚠️  Distance calculations are suspiciously slow!")
    else:
        print("✅ Distance calculations seem reasonable")


def estimate_theoretical_performance():
    """Estimate what our theoretical max performance should be."""
    print("\n📐 Theoretical Performance Analysis")
    print("=" * 40)

    # HNSW typically does:
    # - M connections per layer (M=16)
    # - ef_construction candidates evaluated (ef=200)
    # - log(N) layers on average

    N = 5000  # Database size
    M = 16
    ef_construction = 200
    avg_layers = int(np.log2(N))

    # Estimate distance calculations per insertion
    distance_calcs_per_insert = avg_layers * ef_construction

    print(f"📊 HNSW Parameters:")
    print(f"   Database size: {N}")
    print(f"   M (connections): {M}")
    print(f"   ef_construction: {ef_construction}")
    print(f"   Average layers: {avg_layers}")
    print(f"   Distance calculations per insert: ~{distance_calcs_per_insert}")

    # If each distance calc takes 0.001ms (reasonable for SIMD)
    distance_time_per_insert = distance_calcs_per_insert * 0.000001  # 1μs per calc
    theoretical_speed = 1.0 / distance_time_per_insert

    print(f"\n📐 Theoretical Performance (with 1μs distance calcs):")
    print(f"   Time per insert: {distance_time_per_insert * 1000:.2f}ms")
    print(f"   Theoretical speed: {theoretical_speed:.0f} vec/s")

    if theoretical_speed > 10000:
        print("✅ Theory suggests we should be much faster!")
    else:
        print("⚠️  Even theoretical performance seems limited")


def compare_with_brute_force_efficiency():
    """Compare HNSW construction with brute force to find inefficiencies."""
    print("\n⚖️  HNSW vs Brute Force Efficiency Analysis")
    print("=" * 50)

    dimensions = 128

    # Test brute force performance
    print("🚀 Testing brute force performance...")
    db_brute = omendb.DB()

    vectors = []
    for i in range(1000):
        vector = [float(np.random.randn()) for _ in range(dimensions)]
        vectors.append(vector)

    ids = [f"brute_{i}" for i in range(1000)]

    start_time = time.time()
    db_brute.add_batch(list(zip(ids, vectors)))
    brute_time = time.time() - start_time
    brute_speed = 1000 / brute_time

    print(f"   Brute force: {brute_speed:.0f} vec/s")

    # Compare with expected HNSW overhead
    # HNSW should be maybe 10-50x slower due to graph construction
    # But we're seeing 10-15x slower, which suggests there are inefficiencies

    hnsw_speed = 434  # Our measured speed
    overhead_ratio = brute_speed / hnsw_speed

    print(f"   HNSW: {hnsw_speed} vec/s")
    print(f"   Overhead ratio: {overhead_ratio:.1f}x")

    if overhead_ratio > 50:
        print("❌ HNSW has excessive overhead!")
    elif overhead_ratio > 20:
        print("⚠️  HNSW overhead is high but possibly normal")
    else:
        print("✅ HNSW overhead seems reasonable")


if __name__ == "__main__":
    print("🎯 Deep HNSW Performance Analysis")
    print("Why are we 100x slower than Faiss?")
    print("=" * 50)

    try:
        profile_hnsw_single_insertion()
        analyze_distance_calculation_overhead()
        estimate_theoretical_performance()
        compare_with_brute_force_efficiency()

        print(f"\n🔍 Analysis Summary:")
        print("This analysis should reveal where our 100x performance gap comes from.")
        print("Common culprits:")
        print("1. Excessive memory allocations")
        print("2. Inefficient distance calculations")
        print("3. Poor graph construction algorithm")
        print("4. Missing vectorization/SIMD")
        print("5. Python/Mojo interface overhead")

    except Exception as e:
        print(f"❌ Error during analysis: {e}")
        import traceback

        traceback.print_exc()
