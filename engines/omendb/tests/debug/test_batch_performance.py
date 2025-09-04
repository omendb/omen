#!/usr/bin/env python3
"""Test batch performance of OmenDB vs competitors."""

import sys
import time
import numpy as np

sys.path.insert(0, "python")
import omendb.native as native

print("=" * 70)
print("🏆 BATCH PERFORMANCE COMPARISON")
print("=" * 70)


def test_omendb_batch():
    """Test OmenDB batch performance."""
    sizes = [1000, 5000, 10000, 25000, 50000]
    dimension = 128

    print("\n📊 OmenDB Batch Performance (NumPy arrays):")
    print("-" * 50)

    results = []
    for size in sizes:
        # Generate test data
        vectors = np.random.rand(size, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(size)]
        metadata = [{}] * size

        # Clear database
        native.clear_database()

        # Measure batch insertion
        start = time.time()
        native.add_vector_batch(ids, vectors, metadata)
        elapsed = time.time() - start

        rate = size / elapsed
        results.append((size, rate, elapsed))

        print(f"  {size:6,} vectors: {elapsed:6.3f}s = {rate:8,.0f} vec/s")

    return results


def test_chromadb_batch():
    """Test ChromaDB batch performance if available."""
    try:
        import chromadb

        sizes = [1000, 5000, 10000, 25000, 50000]
        dimension = 128

        print("\n📊 ChromaDB Batch Performance:")
        print("-" * 50)

        results = []
        for size in sizes:
            # Generate test data
            vectors = np.random.rand(size, dimension).astype(np.float32)
            ids = [f"vec_{i}" for i in range(size)]
            metadatas = [{"index": i} for i in range(size)]

            # Initialize ChromaDB
            client = chromadb.Client()
            collection = client.create_collection(name="bench")

            # Measure batch insertion (ChromaDB has batch size limits)
            start = time.time()
            batch_size = 5000  # ChromaDB max batch size
            for i in range(0, size, batch_size):
                end_idx = min(i + batch_size, size)
                collection.add(
                    embeddings=vectors[i:end_idx].tolist(),
                    ids=ids[i:end_idx],
                    metadatas=metadatas[i:end_idx],
                )
            elapsed = time.time() - start

            rate = size / elapsed
            results.append((size, rate, elapsed))

            print(f"  {size:6,} vectors: {elapsed:6.3f}s = {rate:8,.0f} vec/s")

            # Cleanup
            client.delete_collection(name="bench")

        return results
    except ImportError:
        print("\n⚠️ ChromaDB not installed")
        return None


def test_faiss_batch():
    """Test Faiss batch performance if available."""
    try:
        import faiss

        sizes = [1000, 5000, 10000, 25000, 50000]
        dimension = 128

        print("\n📊 Faiss Batch Performance (gold standard):")
        print("-" * 50)

        results = []
        for size in sizes:
            # Generate test data
            vectors = np.random.rand(size, dimension).astype(np.float32)

            # Initialize Faiss
            index = faiss.IndexFlatL2(dimension)

            # Measure batch insertion
            start = time.time()
            index.add(vectors)
            elapsed = time.time() - start

            rate = size / elapsed
            results.append((size, rate, elapsed))

            print(f"  {size:6,} vectors: {elapsed:6.3f}s = {rate:8,.0f} vec/s")

        return results
    except ImportError:
        print("\n⚠️ Faiss not installed")
        return None


def analyze_results(omen_results, chroma_results=None, faiss_results=None):
    """Analyze and compare results."""
    print("\n" + "=" * 70)
    print("📈 PERFORMANCE ANALYSIS")
    print("=" * 70)

    # Scaling analysis
    print("\n🔍 Scaling Behavior:")
    print("-" * 50)

    if len(omen_results) > 1:
        # Check scaling efficiency
        small_size, small_rate, _ = omen_results[0]
        large_size, large_rate, _ = omen_results[-1]

        scaling_factor = large_size / small_size
        perf_ratio = large_rate / small_rate
        efficiency = perf_ratio

        print(f"  Size increase: {scaling_factor:.0f}x ({small_size} → {large_size})")
        print(f"  Performance ratio: {perf_ratio:.2f}")
        print(f"  Scaling efficiency: {efficiency:.2%}")

        if efficiency > 0.8:
            print(f"  ✅ Excellent scaling")
        elif efficiency > 0.5:
            print(f"  ⚠️ Moderate scaling degradation")
        else:
            print(f"  ❌ Poor scaling")

    # Competitive comparison
    if chroma_results or faiss_results:
        print("\n⚔️ Competitive Position (10K vectors):")
        print("-" * 50)

        # Find 10K results
        omen_10k = next((r for r in omen_results if r[0] == 10000), None)

        if omen_10k and chroma_results:
            chroma_10k = next((r for r in chroma_results if r[0] == 10000), None)
            if chroma_10k:
                ratio = omen_10k[1] / chroma_10k[1]
                print(
                    f"  vs ChromaDB: {ratio:.1f}x {'faster' if ratio > 1 else 'slower'}"
                )

        if omen_10k and faiss_results:
            faiss_10k = next((r for r in faiss_results if r[0] == 10000), None)
            if faiss_10k:
                ratio = omen_10k[1] / faiss_10k[1]
                print(f"  vs Faiss: {ratio:.2f}x {'faster' if ratio > 1 else 'slower'}")


def test_query_performance():
    """Test query performance comparison."""
    print("\n" + "=" * 70)
    print("⚡ QUERY PERFORMANCE")
    print("=" * 70)

    # Setup database with vectors
    size = 10000
    dimension = 128
    num_queries = 100

    vectors = np.random.rand(size, dimension).astype(np.float32)
    ids = [f"vec_{i}" for i in range(size)]
    metadata = [{}] * size

    native.clear_database()
    native.add_vector_batch(ids, vectors, metadata)

    # Generate query vectors
    query_vectors = np.random.rand(num_queries, dimension).astype(np.float32)

    # Test OmenDB queries
    print(f"\n📊 Testing {num_queries} queries on {size:,} vectors:")
    print("-" * 50)

    query_times = []
    for query in query_vectors:
        start = time.time()
        results = native.search_vectors(query.tolist(), 10)
        query_times.append((time.time() - start) * 1000)

    avg_time = np.mean(query_times)
    p95_time = np.percentile(query_times, 95)
    p99_time = np.percentile(query_times, 99)

    print(f"  OmenDB Query Latency:")
    print(f"    Average: {avg_time:.2f}ms")
    print(f"    P95: {p95_time:.2f}ms")
    print(f"    P99: {p99_time:.2f}ms")
    print(f"    QPS: {1000 / avg_time:.0f}")


# Run tests
print("\n🧪 Running batch performance tests...\n")

omen_results = test_omendb_batch()
chroma_results = test_chromadb_batch()
faiss_results = test_faiss_batch()

analyze_results(omen_results, chroma_results, faiss_results)
test_query_performance()

print("\n" + "=" * 70)
print("🏁 PERFORMANCE SUMMARY")
print("=" * 70)

if omen_results:
    # Get 10K performance
    result_10k = next((r for r in omen_results if r[0] == 10000), None)
    if result_10k:
        _, rate, _ = result_10k
        print(f"\n📊 OmenDB Performance @ 10K vectors: {rate:,.0f} vec/s")

        if rate > 80000:
            print("  ✅ Excellent performance (>80K vec/s)")
        elif rate > 40000:
            print("  ⚠️ Good performance (>40K vec/s)")
        else:
            print("  ❌ Needs optimization (<40K vec/s)")

print("\n✅ Benchmark complete!")
