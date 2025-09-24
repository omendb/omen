#!/usr/bin/env python3
"""
Detailed HNSW Insertion Profiling Script

Profiles the current 867 vec/s implementation to identify exact bottlenecks.
This script will help us understand where the performance limitation comes from.

Based on Week 1 Day 1 Action Plan from WEEK_1_ACTION_PLAN.md
"""

import sys
import time
import numpy as np
from typing import Dict, List
sys.path.append('python/omendb')
import native

def generate_profiling_vectors(n_vectors: int = 100, dimensions: int = 128) -> np.ndarray:
    """Generate small dataset for detailed profiling."""
    print(f"🎲 Generating profiling dataset: {n_vectors} vectors ({dimensions}D)")

    # Use deterministic seed for reproducible profiling
    np.random.seed(42)
    vectors = np.random.randn(n_vectors, dimensions).astype(np.float32)

    # Normalize vectors (realistic)
    norms = np.linalg.norm(vectors, axis=1, keepdims=True)
    vectors = vectors / (norms + 1e-8)

    return vectors

def profile_current_implementation():
    """Profile the current 867 vec/s implementation in detail."""
    print("🔍 HNSW Insertion Profiling - Week 1 Day 1")
    print("=" * 60)

    # Generate test vectors (need 1000+ to trigger bulk insertion profiling)
    vectors = generate_profiling_vectors(1200, 128)  # Large enough to trigger bulk insertion
    n_vectors = len(vectors)

    # Configure database for profiling
    native.clear_database()  # Clear any existing data
    config = {"buffer_size": 10000}  # Configure for profiling
    native.configure_database(config)

    print("✅ Database configured for profiling")
    print(f"📊 Testing with {n_vectors} vectors...")

    # Create IDs and metadata
    ids = [f"prof_{i:04d}" for i in range(n_vectors)]
    metadata = [{"type": "profiling", "index": i} for i in range(n_vectors)]

    # Profile individual insertions (first 5 vectors for detailed analysis)
    print("\n🔬 Individual Insertion Profiling:")
    print("-" * 40)

    individual_times = []
    for i in range(min(5, n_vectors)):  # Profile first 5 insertions in detail
        vector = vectors[i:i+1]  # Single vector
        metadata_single = [metadata[i]]
        id_single = [ids[i]]

        print(f"\n📍 Profiling vector {i+1}/5:")

        start_time = time.perf_counter()
        results = native.add_vector_batch(id_single, vector, metadata_single)
        end_time = time.perf_counter()

        insertion_time = (end_time - start_time) * 1000  # Convert to ms
        individual_times.append(insertion_time)

        if results and results[0]:
            print(f"  ✅ Inserted in {insertion_time:.3f}ms")
        else:
            print(f"  ❌ Failed to insert vector {i}")

        # Small delay to separate timing measurements
        time.sleep(0.001)

    # Calculate individual insertion statistics
    if individual_times:
        avg_individual = np.mean(individual_times)
        median_individual = np.median(individual_times)
        min_individual = np.min(individual_times)
        max_individual = np.max(individual_times)

        print(f"\n📈 Individual Insertion Statistics:")
        print(f"  Average: {avg_individual:.3f}ms per vector")
        print(f"  Median:  {median_individual:.3f}ms per vector")
        print(f"  Min:     {min_individual:.3f}ms per vector")
        print(f"  Max:     {max_individual:.3f}ms per vector")
        print(f"  Rate:    {1000/avg_individual:.1f} vectors/second")

    # Profile batch insertion for comparison
    print("\n🚀 Batch Insertion Profiling:")
    print("-" * 40)

    # Clear database for batch test
    native.clear_database()
    config = {"buffer_size": 10000}
    native.configure_database(config)

    # Insert remaining vectors in batch (will trigger bulk HNSW profiling)
    remaining_vectors = vectors[5:]
    remaining_ids = ids[5:]
    remaining_metadata = metadata[5:]

    if len(remaining_vectors) > 0:
        print(f"📊 Batch inserting {len(remaining_vectors)} vectors...")

        start_time = time.perf_counter()
        batch_results = native.add_vector_batch(remaining_ids, remaining_vectors, remaining_metadata)
        end_time = time.perf_counter()

        batch_time = end_time - start_time
        successful_inserts = sum(1 for r in batch_results if r) if batch_results else 0

        print(f"  ✅ Batch inserted {successful_inserts} vectors in {batch_time:.3f}s")

        if batch_time > 0:
            batch_rate = successful_inserts / batch_time
            print(f"  🚀 Batch insertion rate: {batch_rate:.1f} vectors/second")

            # Compare to current baseline
            baseline_rate = 867  # Current known baseline
            comparison = batch_rate / baseline_rate
            print(f"  📊 vs Current baseline: {comparison:.2f}x")

    # Analyze expected bottlenecks (from Week 1 Action Plan)
    print("\n🎯 Expected Bottleneck Analysis:")
    print("-" * 40)
    print("Based on Week 1 Action Plan, expected breakdown:")
    print("  - Distance calculations: ~40% (SIMD broken)")
    print("  - Graph traversal: ~30% (cache misses)")
    print("  - Memory allocation: ~15% (hot path allocation)")
    print("  - Connection management: ~10% (lock overhead)")
    print("  - FFI overhead: ~5% (Python ↔ Mojo)")

    print("\n🔍 To get detailed component timing:")
    print("  1. The Mojo code now includes _insert_node_with_profiling()")
    print("  2. This function provides detailed timing breakdown")
    print("  3. Run this script and check the console for detailed timing")

    return {
        'individual_times': individual_times,
        'individual_avg_ms': avg_individual if individual_times else 0,
        'individual_rate_vec_per_sec': 1000/avg_individual if individual_times and avg_individual > 0 else 0,
        'batch_rate_vec_per_sec': batch_rate if 'batch_rate' in locals() else 0,
    }

def main():
    """Run the profiling analysis."""
    print("🚀 OmenDB HNSW Insertion Profiling")
    print("Week 1 Day 1: Identify Current Bottlenecks")
    print("=" * 60)

    try:
        results = profile_current_implementation()

        print("\n✅ Profiling Complete!")
        print("=" * 60)
        print("📊 Summary:")
        if results['individual_times']:
            print(f"  Individual avg: {results['individual_avg_ms']:.3f}ms per vector")
            print(f"  Individual rate: {results['individual_rate_vec_per_sec']:.1f} vec/s")
        if results['batch_rate_vec_per_sec'] > 0:
            print(f"  Batch rate: {results['batch_rate_vec_per_sec']:.1f} vec/s")

        print("\n🎯 Next Steps (Week 1 Day 2):")
        print("  1. Analyze the detailed component timings from Mojo output")
        print("  2. Identify the slowest component")
        print("  3. Fix bulk construction memory issues")
        print("  4. Target 2,000-5,000 vec/s improvement")

    except Exception as e:
        print(f"❌ Profiling failed: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()