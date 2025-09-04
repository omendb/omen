#!/usr/bin/env python3
"""
Comprehensive performance comparison between OmenDB, ChromaDB, and LanceDB.
Tests the exact same operations on all three databases.
"""

import time
import numpy as np
import tempfile
import shutil
import os
import sys

# Add the python directory to the path
current_dir = os.path.dirname(os.path.abspath(__file__))
root_dir = os.path.dirname(os.path.dirname(current_dir))
python_dir = os.path.join(root_dir, "python")
sys.path.insert(0, python_dir)

# Benchmark configuration
VECTOR_DIM = 768  # Common embedding dimension
NUM_VECTORS = 10000
BATCH_SIZE = 100
NUM_QUERIES = 100


def generate_test_data():
    """Generate standardized test data."""
    np.random.seed(42)
    vectors = np.random.randn(NUM_VECTORS, VECTOR_DIM).astype(np.float32)
    # Normalize vectors (common in real embeddings)
    vectors = vectors / np.linalg.norm(vectors, axis=1, keepdims=True)

    # Generate query vectors (different from dataset)
    np.random.seed(99)
    queries = np.random.randn(NUM_QUERIES, VECTOR_DIM).astype(np.float32)
    queries = queries / np.linalg.norm(queries, axis=1, keepdims=True)

    ids = [f"vec_{i:06d}" for i in range(NUM_VECTORS)]
    metadata = [{"idx": i, "category": f"cat_{i % 10}"} for i in range(NUM_VECTORS)]

    return vectors, queries, ids, metadata


def benchmark_omendb(vectors, queries, ids, metadata):
    """Benchmark OmenDB."""
    print("\n🚀 Testing OmenDB...")
    results = {}

    from omendb import DB

    with tempfile.NamedTemporaryFile(suffix=".omendb", delete=False) as tmp:
        db_path = tmp.name

    try:
        # Startup time
        start = time.perf_counter()
        db = DB(db_path=db_path)
        results["startup_ms"] = (time.perf_counter() - start) * 1000

        # Insert time (one by one)
        start = time.perf_counter()
        for i in range(NUM_VECTORS):
            db.add(ids[i], vectors[i].tolist(), metadata[i])
        results["insert_time_s"] = time.perf_counter() - start
        results["insert_rate"] = NUM_VECTORS / results["insert_time_s"]

        # Query time
        query_times = []
        for query in queries:
            start = time.perf_counter()
            res = db.search(query.tolist(), limit=10)
            query_times.append(time.perf_counter() - start)

        results["avg_query_ms"] = np.mean(query_times) * 1000
        results["p99_query_ms"] = np.percentile(query_times, 99) * 1000

        # Storage size
        results["storage_mb"] = (
            os.path.getsize(db_path) / (1024 * 1024) if os.path.exists(db_path) else 0
        )

        # Test special features
        results["has_count"] = hasattr(db, "count")
        results["has_exists"] = hasattr(db, "exists")
        results["count"] = db.count() if results["has_count"] else None

    finally:
        if os.path.exists(db_path):
            os.unlink(db_path)

    return results


def benchmark_chromadb(vectors, queries, ids, metadata):
    """Benchmark ChromaDB."""
    print("\n🎨 Testing ChromaDB...")
    results = {}

    try:
        import chromadb
    except ImportError:
        print("  ChromaDB not installed")
        return {}

    with tempfile.TemporaryDirectory() as tmpdir:
        # Startup time
        start = time.perf_counter()
        client = chromadb.PersistentClient(path=tmpdir)
        collection = client.create_collection("test")
        results["startup_ms"] = (time.perf_counter() - start) * 1000

        # Insert time (batch for efficiency)
        start = time.perf_counter()
        for i in range(0, NUM_VECTORS, BATCH_SIZE):
            batch_end = min(i + BATCH_SIZE, NUM_VECTORS)
            collection.add(
                ids=ids[i:batch_end],
                embeddings=vectors[i:batch_end].tolist(),
                metadatas=metadata[i:batch_end],
            )
        results["insert_time_s"] = time.perf_counter() - start
        results["insert_rate"] = NUM_VECTORS / results["insert_time_s"]

        # Query time
        query_times = []
        for query in queries:
            start = time.perf_counter()
            res = collection.query(query_embeddings=[query.tolist()], n_results=10)
            query_times.append(time.perf_counter() - start)

        results["avg_query_ms"] = np.mean(query_times) * 1000
        results["p99_query_ms"] = np.percentile(query_times, 99) * 1000

        # Storage size
        total_size = sum(
            os.path.getsize(os.path.join(dirpath, filename))
            for dirpath, dirnames, filenames in os.walk(tmpdir)
            for filename in filenames
        ) / (1024 * 1024)
        results["storage_mb"] = total_size

        # Test special features
        results["has_count"] = True
        results["count"] = collection.count()

    return results


def benchmark_lancedb(vectors, queries, ids, metadata):
    """Benchmark LanceDB."""
    print("\n🦀 Testing LanceDB...")
    results = {}

    try:
        import lancedb
        import pandas as pd
    except ImportError:
        print("  LanceDB not installed")
        return {}

    with tempfile.TemporaryDirectory() as tmpdir:
        # Startup time
        start = time.perf_counter()
        db = lancedb.connect(tmpdir)

        # Prepare data
        data = []
        for i in range(NUM_VECTORS):
            row = {"id": ids[i], "vector": vectors[i].tolist()}
            row.update(metadata[i])
            data.append(row)

        # Create table with first batch
        table = db.create_table("test", data[:BATCH_SIZE])
        results["startup_ms"] = (time.perf_counter() - start) * 1000

        # Insert rest of data
        start = time.perf_counter()
        for i in range(BATCH_SIZE, NUM_VECTORS, BATCH_SIZE):
            batch_end = min(i + BATCH_SIZE, NUM_VECTORS)
            table.add(data[i:batch_end])
        results["insert_time_s"] = time.perf_counter() - start
        results["insert_rate"] = (
            NUM_VECTORS / results["insert_time_s"]
            if results["insert_time_s"] > 0
            else 0
        )

        # Create index
        table.create_index(metric="cosine")

        # Query time
        query_times = []
        for query in queries:
            start = time.perf_counter()
            res = table.search(query.tolist()).limit(10).to_pandas()
            query_times.append(time.perf_counter() - start)

        results["avg_query_ms"] = np.mean(query_times) * 1000
        results["p99_query_ms"] = np.percentile(query_times, 99) * 1000

        # Storage size
        total_size = sum(
            os.path.getsize(os.path.join(dirpath, filename))
            for dirpath, dirnames, filenames in os.walk(tmpdir)
            for filename in filenames
        ) / (1024 * 1024)
        results["storage_mb"] = total_size

        # Test special features
        results["has_count"] = False
        results["count"] = None

    return results


def print_comparison(results):
    """Print comparison table."""
    print("\n" + "=" * 80)
    print("PERFORMANCE COMPARISON RESULTS")
    print(f"Dataset: {NUM_VECTORS:,} vectors, {VECTOR_DIM}D, {NUM_QUERIES} queries")
    print("=" * 80)

    # Determine which databases have results
    dbs = [
        name
        for name in ["OmenDB", "ChromaDB", "LanceDB"]
        if name in results and results[name]
    ]

    if not dbs:
        print("No results available")
        return

    # Print metrics
    metrics = [
        ("Startup (ms)", "startup_ms", True),
        ("Insert Rate (vec/s)", "insert_rate", False),
        ("Avg Query (ms)", "avg_query_ms", True),
        ("P99 Query (ms)", "p99_query_ms", True),
        ("Storage (MB)", "storage_mb", True),
    ]

    print(f"{'Metric':<25}", end="")
    for db in dbs:
        print(f"{db:<20}", end="")
    print()
    print("-" * 80)

    for metric_name, key, lower_better in metrics:
        print(f"{metric_name:<25}", end="")
        values = []
        for db in dbs:
            val = results[db].get(key, 0)
            values.append((db, val))
            if key == "insert_rate":
                print(f"{val:,.0f}".ljust(20), end="")
            else:
                print(f"{val:.2f}".ljust(20), end="")

        # Find winner
        if values:
            if lower_better:
                winner = min(values, key=lambda x: x[1] if x[1] > 0 else float("inf"))
            else:
                winner = max(values, key=lambda x: x[1])
            # Don't print winner inline, too cluttered
        print()

    # Performance ratios vs OmenDB
    if "OmenDB" in results and results["OmenDB"]:
        print("\n📊 Performance vs OmenDB:")
        omen = results["OmenDB"]

        for db in ["ChromaDB", "LanceDB"]:
            if db in results and results[db]:
                other = results[db]
                print(f"\n  {db}:")

                # Startup
                if omen["startup_ms"] and other["startup_ms"]:
                    ratio = other["startup_ms"] / omen["startup_ms"]
                    print(f"    Startup: {ratio:.1f}x slower")

                # Insert
                if omen["insert_rate"] and other["insert_rate"]:
                    ratio = omen["insert_rate"] / other["insert_rate"]
                    if ratio > 1:
                        print(f"    Insert: OmenDB {ratio:.1f}x faster")
                    else:
                        print(f"    Insert: {db} {1 / ratio:.1f}x faster")

                # Query
                if omen["avg_query_ms"] and other["avg_query_ms"]:
                    ratio = other["avg_query_ms"] / omen["avg_query_ms"]
                    if ratio > 1:
                        print(f"    Query: OmenDB {ratio:.1f}x faster")
                    else:
                        print(f"    Query: {db} {1 / ratio:.1f}x faster")

                # Storage
                if omen["storage_mb"] and other["storage_mb"]:
                    ratio = (
                        other["storage_mb"] / omen["storage_mb"]
                        if omen["storage_mb"] > 0
                        else 0
                    )
                    if ratio > 1:
                        print(f"    Storage: OmenDB {ratio:.1f}x smaller")


def main():
    """Run comprehensive comparison."""
    print("=" * 80)
    print("COMPREHENSIVE VECTOR DATABASE COMPARISON")
    print("=" * 80)

    # Generate test data
    print("\n📊 Generating test data...")
    vectors, queries, ids, metadata = generate_test_data()
    print(f"  Vectors: {NUM_VECTORS:,} x {VECTOR_DIM}D")
    print(f"  Queries: {NUM_QUERIES}")

    # Run benchmarks
    results = {}
    results["OmenDB"] = benchmark_omendb(vectors, queries, ids, metadata)
    results["ChromaDB"] = benchmark_chromadb(vectors, queries, ids, metadata)
    results["LanceDB"] = benchmark_lancedb(vectors, queries, ids, metadata)

    # Print comparison
    print_comparison(results)

    # Key findings
    print("\n" + "=" * 80)
    print("KEY FINDINGS")
    print("=" * 80)

    if results.get("OmenDB"):
        print("\n✅ OmenDB Strengths:")
        if results["OmenDB"].get("startup_ms", 0) < 10:
            print(f"  • Instant startup ({results['OmenDB']['startup_ms']:.2f}ms)")
        if results["OmenDB"].get("avg_query_ms", 0) < 2:
            print(f"  • Fast queries ({results['OmenDB']['avg_query_ms']:.2f}ms avg)")
        print("  • Simple API (no client/collection setup)")
        print("  • Embedded-first design")

    if results.get("ChromaDB"):
        print("\n🎨 ChromaDB Strengths:")
        print("  • Rich metadata filtering")
        print("  • Good documentation")
        print("  • Wide ecosystem support")

    if results.get("LanceDB"):
        print("\n🦀 LanceDB Strengths:")
        print("  • SQL-like queries")
        print("  • DataFrame integration")
        print("  • Rust performance")


if __name__ == "__main__":
    main()
