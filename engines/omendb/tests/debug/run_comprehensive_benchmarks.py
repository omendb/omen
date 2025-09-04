#!/usr/bin/env python3
"""Comprehensive benchmarks comparing OmenDB to all major competitors."""

import sys
import os
import time
import numpy as np
import json
from datetime import datetime

# Add parent directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "omendb"))
sys.path.insert(0, "python")

# Import OmenDB
import omendb

print("=" * 80)
print("🏆 COMPREHENSIVE COMPETITIVE BENCHMARKS")
print("=" * 80)
print(f"Date: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
print(f"OmenDB Version: 0.2.0")
print()

# Test configurations
DIMENSIONS = [128, 768, 1536]
DATASET_SIZES = [1000, 5000, 10000, 25000, 50000]
NUM_QUERIES = 100
TOP_K = 10


def format_number(n):
    """Format large numbers with commas."""
    return f"{n:,}"


def benchmark_omendb(dimension, num_vectors):
    """Benchmark OmenDB performance."""
    # Generate test data
    vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
    query_vectors = np.random.rand(NUM_QUERIES, dimension).astype(np.float32)

    # Initialize database
    start = time.time()
    db = omendb.DB()
    startup_time = (time.time() - start) * 1000  # ms

    # Insertion benchmark
    ids = [f"vec_{i}" for i in range(num_vectors)]
    metadata = [{"index": i} for i in range(num_vectors)]

    start = time.time()
    db.add_batch(ids, vectors, metadata)
    insert_time = time.time() - start
    insert_rate = num_vectors / insert_time

    # Query benchmark
    query_times = []
    for query in query_vectors:
        start = time.time()
        results = db.search(query, limit=TOP_K)
        query_times.append((time.time() - start) * 1000)  # ms

    avg_query_time = np.mean(query_times)
    p95_query_time = np.percentile(query_times, 95)
    p99_query_time = np.percentile(query_times, 99)

    # Memory usage (approximate from stats)
    stats = db.get_stats()
    memory_mb = (num_vectors * dimension * 4) / (1024 * 1024)  # Approximate

    db.clear()

    return {
        "startup_ms": startup_time,
        "insert_rate": insert_rate,
        "insert_time": insert_time,
        "query_avg_ms": avg_query_time,
        "query_p95_ms": p95_query_time,
        "query_p99_ms": p99_query_time,
        "memory_mb": memory_mb,
        "buffer_size": stats.get("buffer_size", 0),
        "main_index_size": stats.get("main_index_size", 0),
    }


def benchmark_chromadb(dimension, num_vectors):
    """Benchmark ChromaDB if available."""
    try:
        import chromadb

        # Initialize
        start = time.time()
        client = chromadb.Client()
        collection = client.create_collection(name="bench")
        startup_time = (time.time() - start) * 1000

        # Generate test data
        vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
        query_vectors = np.random.rand(NUM_QUERIES, dimension).astype(np.float32)

        # Insertion
        ids = [f"vec_{i}" for i in range(num_vectors)]
        metadatas = [{"index": i} for i in range(num_vectors)]

        start = time.time()
        collection.add(embeddings=vectors.tolist(), ids=ids, metadatas=metadatas)
        insert_time = time.time() - start
        insert_rate = num_vectors / insert_time

        # Query
        query_times = []
        for query in query_vectors:
            start = time.time()
            results = collection.query(
                query_embeddings=[query.tolist()], n_results=TOP_K
            )
            query_times.append((time.time() - start) * 1000)

        avg_query_time = np.mean(query_times)
        p95_query_time = np.percentile(query_times, 95)
        p99_query_time = np.percentile(query_times, 99)

        client.delete_collection(name="bench")

        return {
            "startup_ms": startup_time,
            "insert_rate": insert_rate,
            "insert_time": insert_time,
            "query_avg_ms": avg_query_time,
            "query_p95_ms": p95_query_time,
            "query_p99_ms": p99_query_time,
            "memory_mb": 0,  # ChromaDB doesn't easily report memory
        }
    except ImportError:
        return None


def benchmark_lancedb(dimension, num_vectors):
    """Benchmark LanceDB if available."""
    try:
        import lancedb
        import pyarrow as pa
        import tempfile
        import shutil

        tmpdir = tempfile.mkdtemp()

        try:
            # Initialize
            start = time.time()
            db = lancedb.connect(tmpdir)
            startup_time = (time.time() - start) * 1000

            # Generate test data
            vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
            query_vectors = np.random.rand(NUM_QUERIES, dimension).astype(np.float32)

            # Create table
            ids = [f"vec_{i}" for i in range(num_vectors)]
            data = []
            for i, vec in enumerate(vectors):
                data.append({"id": ids[i], "vector": vec, "index": i})

            # Insertion
            start = time.time()
            table = db.create_table("bench", data=data)
            insert_time = time.time() - start
            insert_rate = num_vectors / insert_time

            # Create index for better query performance
            table.create_index(num_partitions=256, num_sub_vectors=96)

            # Query
            query_times = []
            for query in query_vectors:
                start = time.time()
                results = table.search(query).limit(TOP_K).to_list()
                query_times.append((time.time() - start) * 1000)

            avg_query_time = np.mean(query_times)
            p95_query_time = np.percentile(query_times, 95)
            p99_query_time = np.percentile(query_times, 99)

            return {
                "startup_ms": startup_time,
                "insert_rate": insert_rate,
                "insert_time": insert_time,
                "query_avg_ms": avg_query_time,
                "query_p95_ms": p95_query_time,
                "query_p99_ms": p99_query_time,
                "memory_mb": 0,  # LanceDB is disk-based
            }
        finally:
            shutil.rmtree(tmpdir, ignore_errors=True)
    except ImportError:
        return None


def benchmark_faiss(dimension, num_vectors):
    """Benchmark Faiss if available."""
    try:
        import faiss

        # Generate test data
        vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
        query_vectors = np.random.rand(NUM_QUERIES, dimension).astype(np.float32)

        # Initialize
        start = time.time()
        index = faiss.IndexFlatL2(dimension)
        startup_time = (time.time() - start) * 1000

        # Insertion
        start = time.time()
        index.add(vectors)
        insert_time = time.time() - start
        insert_rate = num_vectors / insert_time

        # Query
        query_times = []
        for query in query_vectors:
            start = time.time()
            distances, indices = index.search(query.reshape(1, -1), TOP_K)
            query_times.append((time.time() - start) * 1000)

        avg_query_time = np.mean(query_times)
        p95_query_time = np.percentile(query_times, 95)
        p99_query_time = np.percentile(query_times, 99)

        memory_mb = (num_vectors * dimension * 4) / (1024 * 1024)

        return {
            "startup_ms": startup_time,
            "insert_rate": insert_rate,
            "insert_time": insert_time,
            "query_avg_ms": avg_query_time,
            "query_p95_ms": p95_query_time,
            "query_p99_ms": p99_query_time,
            "memory_mb": memory_mb,
        }
    except ImportError:
        return None


def run_benchmark_suite():
    """Run complete benchmark suite."""
    results = {}

    # Test each dimension
    for dim in DIMENSIONS:
        print(f"\n📐 Testing Dimension: {dim}")
        print("-" * 60)

        results[dim] = {}

        # Test each dataset size
        for size in DATASET_SIZES:
            if size > 25000 and dim > 768:
                continue  # Skip very large high-dim tests

            print(f"\n  Dataset Size: {format_number(size)} vectors")

            # Benchmark OmenDB
            print(f"    🔷 OmenDB...", end=" ")
            omen_results = benchmark_omendb(dim, size)
            results[dim][size] = {"omendb": omen_results}
            print(f"✓ {omen_results['insert_rate']:.0f} vec/s")

            # Benchmark ChromaDB
            chroma_results = benchmark_chromadb(dim, size)
            if chroma_results:
                print(f"    🟠 ChromaDB...", end=" ")
                results[dim][size]["chromadb"] = chroma_results
                print(f"✓ {chroma_results['insert_rate']:.0f} vec/s")

            # Benchmark LanceDB
            lance_results = benchmark_lancedb(dim, size)
            if lance_results:
                print(f"    🟣 LanceDB...", end=" ")
                results[dim][size]["lancedb"] = lance_results
                print(f"✓ {lance_results['insert_rate']:.0f} vec/s")

            # Benchmark Faiss
            faiss_results = benchmark_faiss(dim, size)
            if faiss_results:
                print(f"    🔵 Faiss...", end=" ")
                results[dim][size]["faiss"] = faiss_results
                print(f"✓ {faiss_results['insert_rate']:.0f} vec/s")

    return results


def print_comparison_table(results):
    """Print comparison table of results."""
    print("\n" + "=" * 80)
    print("📊 PERFORMANCE COMPARISON SUMMARY")
    print("=" * 80)

    # Standard embedding dimension (768)
    if 768 in results and 10000 in results[768]:
        print("\n🎯 Standard Configuration (768D, 10K vectors):")
        print("-" * 60)

        data = results[768][10000]
        databases = list(data.keys())

        # Insertion Performance
        print("\n📥 Insertion Performance (vec/s):")
        for db in databases:
            if db in data:
                rate = data[db]["insert_rate"]
                print(f"  {db.capitalize():12} {rate:>10,.0f} vec/s")

        # Query Latency
        print("\n⚡ Query Latency (ms):")
        print(f"  {'Database':12} {'Avg':>8} {'P95':>8} {'P99':>8}")
        for db in databases:
            if db in data:
                avg = data[db]["query_avg_ms"]
                p95 = data[db]["query_p95_ms"]
                p99 = data[db]["query_p99_ms"]
                print(f"  {db.capitalize():12} {avg:>8.2f} {p95:>8.2f} {p99:>8.2f}")

        # Startup Time
        print("\n🚀 Startup Time (ms):")
        for db in databases:
            if db in data:
                startup = data[db]["startup_ms"]
                print(f"  {db.capitalize():12} {startup:>10.2f} ms")

    # Scaling Analysis
    print("\n📈 Scaling Analysis (128D):")
    print("-" * 60)

    if 128 in results:
        sizes = sorted([s for s in DATASET_SIZES if s in results[128]])
        databases = set()
        for size in sizes:
            databases.update(results[128][size].keys())
        databases = sorted(list(databases))

        print(f"\n  {'Size':>10} ", end="")
        for db in databases:
            print(f"{db.capitalize():>12} ", end="")
        print()

        for size in sizes:
            print(f"  {size:>10,} ", end="")
            for db in databases:
                if db in results[128][size]:
                    rate = results[128][size][db]["insert_rate"]
                    print(f"{rate:>12,.0f} ", end="")
                else:
                    print(f"{'N/A':>12} ", end="")
            print()


def analyze_architecture_differences(results):
    """Analyze architectural differences between databases."""
    print("\n" + "=" * 80)
    print("🏗️ ARCHITECTURAL ANALYSIS")
    print("=" * 80)

    architectures = {
        "omendb": {
            "Language": "Mojo",
            "Index": "HNSW + Buffer",
            "Storage": "In-memory arrays",
            "SIMD": "Yes (native)",
            "Startup": "Instant (<1ms)",
            "Persistence": "Snapshot-based",
            "Best For": "Embedded, real-time",
        },
        "chromadb": {
            "Language": "Python/C++",
            "Index": "HNSW",
            "Storage": "SQLite + Parquet",
            "SIMD": "Limited",
            "Startup": "Slow (200ms+)",
            "Persistence": "SQLite DB",
            "Best For": "Small-medium datasets",
        },
        "lancedb": {
            "Language": "Rust/Python",
            "Index": "IVF-PQ, DiskANN",
            "Storage": "Lance (columnar)",
            "SIMD": "Yes (Arrow)",
            "Startup": "Medium",
            "Persistence": "Lance format",
            "Best For": "Large datasets, analytics",
        },
        "faiss": {
            "Language": "C++",
            "Index": "Many options",
            "Storage": "In-memory",
            "SIMD": "Yes (optimized)",
            "Startup": "Instant",
            "Persistence": "Manual save/load",
            "Best For": "Research, benchmarks",
        },
    }

    for db, arch in architectures.items():
        print(f"\n📦 {db.upper()}:")
        for key, value in arch.items():
            print(f"  {key:12} {value}")


def suggest_optimizations(results):
    """Suggest optimizations based on benchmark results."""
    print("\n" + "=" * 80)
    print("🔧 OPTIMIZATION RECOMMENDATIONS")
    print("=" * 80)

    # Analyze OmenDB performance vs competitors
    if 768 in results and 10000 in results[768]:
        data = results[768][10000]
        omen_rate = data.get("omendb", {}).get("insert_rate", 0)

        # Compare to best performer
        best_rate = max([data[db]["insert_rate"] for db in data])

        if omen_rate < best_rate * 0.5:
            print("\n⚠️ Performance Gap Detected:")
            print(f"  OmenDB: {omen_rate:.0f} vec/s")
            print(f"  Best: {best_rate:.0f} vec/s")
            print(f"  Gap: {(1 - omen_rate / best_rate) * 100:.1f}%")

    print("\n📋 Priority Optimizations:")
    print("\n1. **SIMD Optimization** (High Impact)")
    print("   - Current: Basic SIMD in distance calculations")
    print("   - Opportunity: AVX-512 for modern CPUs")
    print("   - Expected gain: 2-3x on distance calculations")

    print("\n2. **True Batch HNSW Insertion** (High Impact)")
    print("   - Current: Individual insertions (2.4K vec/s)")
    print("   - Opportunity: Batch graph construction")
    print("   - Expected gain: 5-10x on HNSW operations")

    print("\n3. **Memory-Mapped Storage** (Medium Impact)")
    print("   - Current: All in-memory")
    print("   - Opportunity: mmap for larger-than-RAM")
    print("   - Expected gain: Handle 10x larger datasets")

    print("\n4. **Write-Ahead Log** (Server Mode)")
    print("   - Current: No durability")
    print("   - Opportunity: WAL for crash recovery")
    print("   - Expected gain: Production-ready persistence")

    print("\n5. **IVF-PQ Index Option** (Memory Optimization)")
    print("   - Current: HNSW only (high memory)")
    print("   - Opportunity: IVF-PQ for compression")
    print("   - Expected gain: 10x memory reduction")


# Run benchmarks
print("🧪 Starting comprehensive benchmarks...")
print("This will take several minutes...\n")

results = run_benchmark_suite()
print_comparison_table(results)
analyze_architecture_differences(results)
suggest_optimizations(results)

# Save results
timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
filename = f"benchmark_results_{timestamp}.json"
with open(filename, "w") as f:
    json.dump(results, f, indent=2)

print(f"\n💾 Results saved to: {filename}")
print("\n✅ Benchmark suite complete!")
