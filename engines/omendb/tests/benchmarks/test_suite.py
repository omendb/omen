#!/usr/bin/env python3
"""
OmenDB Comprehensive Benchmark Suite

Standardized benchmarks for comparing vector database performance.
Tests insertion throughput, query latency, memory usage, and scaling.
"""

import time
import psutil
import numpy as np
from typing import Dict, List, Optional, Tuple
import json
from datetime import datetime
import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


class VectorDBBenchmark:
    """Base class for vector database benchmarks."""

    def __init__(self, name: str):
        self.name = name
        self.results = {}

    def generate_vectors(self, num_vectors: int, dimension: int) -> np.ndarray:
        """Generate random unit vectors for testing."""
        vectors = np.random.randn(num_vectors, dimension).astype(np.float32)
        # Normalize to unit vectors
        norms = np.linalg.norm(vectors, axis=1, keepdims=True)
        vectors = vectors / norms
        return vectors

    def generate_queries(self, num_queries: int, dimension: int) -> np.ndarray:
        """Generate query vectors."""
        return self.generate_vectors(num_queries, dimension)

    def measure_memory(self) -> float:
        """Measure current memory usage in MB."""
        process = psutil.Process()
        return process.memory_info().rss / 1024 / 1024

    def benchmark_insertion(
        self, num_vectors: int, dimension: int = 128, batch_size: int = 1000
    ) -> Dict:
        """Benchmark batch insertion performance."""
        raise NotImplementedError("Subclasses must implement benchmark_insertion")

    def benchmark_query(self, num_queries: int = 100, k: int = 10) -> Dict:
        """Benchmark query performance."""
        raise NotImplementedError("Subclasses must implement benchmark_query")

    def benchmark_memory(self) -> Dict:
        """Benchmark memory usage."""
        raise NotImplementedError("Subclasses must implement benchmark_memory")

    def run_all_benchmarks(self, sizes: List[int] = [1000, 5000, 10000, 25000]) -> Dict:
        """Run all benchmarks at different scales."""
        results = {
            "database": self.name,
            "timestamp": datetime.now().isoformat(),
            "benchmarks": {},
        }

        for size in sizes:
            print(f"\n📊 Benchmarking {self.name} with {size} vectors...")

            # Insertion benchmark
            insertion_results = self.benchmark_insertion(size)

            # Query benchmark
            query_results = self.benchmark_query()

            # Memory benchmark
            memory_results = self.benchmark_memory()

            results["benchmarks"][size] = {
                "insertion": insertion_results,
                "query": query_results,
                "memory": memory_results,
            }

        return results


class OmenDBBenchmark(VectorDBBenchmark):
    """OmenDB benchmark implementation."""

    def __init__(self):
        super().__init__("OmenDB")
        import omendb

        self.db = None
        self.vectors = None
        self.ids = None

    def benchmark_insertion(
        self, num_vectors: int, dimension: int = 128, batch_size: int = 1000
    ) -> Dict:
        """Benchmark OmenDB insertion."""
        import omendb

        # Generate test data
        self.vectors = self.generate_vectors(num_vectors, dimension)
        self.ids = [f"vec_{i}" for i in range(num_vectors)]

        # Create fresh database
        self.db = omendb.DB()
        if hasattr(self.db, "clear"):
            self.db.clear()

        # Measure insertion
        start_mem = self.measure_memory()
        start_time = time.time()

        # Batch insertion
        for i in range(0, num_vectors, batch_size):
            batch_end = min(i + batch_size, num_vectors)
            batch_vectors = self.vectors[i:batch_end]
            batch_ids = self.ids[i:batch_end]
            self.db.add_batch(batch_vectors, batch_ids)

        elapsed = time.time() - start_time
        end_mem = self.measure_memory()

        return {
            "throughput": num_vectors / elapsed,
            "total_time": elapsed,
            "memory_delta": end_mem - start_mem,
            "vectors_per_second": num_vectors / elapsed,
        }

    def benchmark_query(self, num_queries: int = 100, k: int = 10) -> Dict:
        """Benchmark OmenDB query performance."""
        if self.db is None or self.vectors is None:
            return {"error": "Must run insertion benchmark first"}

        queries = self.generate_queries(num_queries, self.vectors.shape[1])
        latencies = []

        for query in queries:
            start = time.time()
            results = self.db.search(query, limit=k)
            latencies.append((time.time() - start) * 1000)  # Convert to ms

        return {
            "avg_latency_ms": np.mean(latencies),
            "p50_latency_ms": np.percentile(latencies, 50),
            "p95_latency_ms": np.percentile(latencies, 95),
            "p99_latency_ms": np.percentile(latencies, 99),
            "queries_per_second": 1000 / np.mean(latencies),
        }

    def benchmark_memory(self) -> Dict:
        """Benchmark OmenDB memory usage."""
        if self.db is None or self.vectors is None:
            return {"error": "Must run insertion benchmark first"}

        current_mem = self.measure_memory()
        vector_count = len(self.ids) if self.ids else 0
        bytes_per_vector = (
            (current_mem * 1024 * 1024) / vector_count if vector_count > 0 else 0
        )

        return {
            "total_memory_mb": current_mem,
            "bytes_per_vector": bytes_per_vector,
            "vector_count": vector_count,
        }


class ChromaDBBenchmark(VectorDBBenchmark):
    """ChromaDB benchmark implementation."""

    def __init__(self):
        super().__init__("ChromaDB")
        self.client = None
        self.collection = None
        self.vectors = None
        self.ids = None

    def benchmark_insertion(
        self, num_vectors: int, dimension: int = 128, batch_size: int = 1000
    ) -> Dict:
        """Benchmark ChromaDB insertion."""
        try:
            import chromadb

            # Generate test data
            self.vectors = self.generate_vectors(num_vectors, dimension)
            self.ids = [f"vec_{i}" for i in range(num_vectors)]

            # Create fresh database
            self.client = chromadb.Client()

            # Delete collection if exists
            try:
                self.client.delete_collection("benchmark")
            except:
                pass

            self.collection = self.client.create_collection("benchmark")

            # Measure insertion
            start_mem = self.measure_memory()
            start_time = time.time()

            # Batch insertion
            for i in range(0, num_vectors, batch_size):
                batch_end = min(i + batch_size, num_vectors)
                batch_vectors = self.vectors[i:batch_end].tolist()
                batch_ids = self.ids[i:batch_end]

                self.collection.add(embeddings=batch_vectors, ids=batch_ids)

            elapsed = time.time() - start_time
            end_mem = self.measure_memory()

            return {
                "throughput": num_vectors / elapsed,
                "total_time": elapsed,
                "memory_delta": end_mem - start_mem,
                "vectors_per_second": num_vectors / elapsed,
            }

        except ImportError:
            return {"error": "ChromaDB not installed"}

    def benchmark_query(self, num_queries: int = 100, k: int = 10) -> Dict:
        """Benchmark ChromaDB query performance."""
        if self.collection is None or self.vectors is None:
            return {"error": "Must run insertion benchmark first"}

        queries = self.generate_queries(num_queries, self.vectors.shape[1])
        latencies = []

        for query in queries:
            start = time.time()
            results = self.collection.query(
                query_embeddings=[query.tolist()], n_results=k
            )
            latencies.append((time.time() - start) * 1000)

        return {
            "avg_latency_ms": np.mean(latencies),
            "p50_latency_ms": np.percentile(latencies, 50),
            "p95_latency_ms": np.percentile(latencies, 95),
            "p99_latency_ms": np.percentile(latencies, 99),
            "queries_per_second": 1000 / np.mean(latencies),
        }

    def benchmark_memory(self) -> Dict:
        """Benchmark ChromaDB memory usage."""
        if self.collection is None:
            return {"error": "Must run insertion benchmark first"}

        current_mem = self.measure_memory()
        vector_count = self.collection.count()
        bytes_per_vector = (
            (current_mem * 1024 * 1024) / vector_count if vector_count > 0 else 0
        )

        return {
            "total_memory_mb": current_mem,
            "bytes_per_vector": bytes_per_vector,
            "vector_count": vector_count,
        }


class FaissBenchmark(VectorDBBenchmark):
    """Faiss benchmark implementation."""

    def __init__(self):
        super().__init__("Faiss")
        self.index = None
        self.vectors = None
        self.dimension = None

    def benchmark_insertion(
        self, num_vectors: int, dimension: int = 128, batch_size: int = 1000
    ) -> Dict:
        """Benchmark Faiss insertion."""
        try:
            import faiss

            # Generate test data
            self.vectors = self.generate_vectors(num_vectors, dimension)
            self.dimension = dimension

            # Create index
            self.index = faiss.IndexFlatL2(dimension)

            # Measure insertion
            start_mem = self.measure_memory()
            start_time = time.time()

            # Batch insertion
            for i in range(0, num_vectors, batch_size):
                batch_end = min(i + batch_size, num_vectors)
                batch_vectors = self.vectors[i:batch_end]
                self.index.add(batch_vectors)

            elapsed = time.time() - start_time
            end_mem = self.measure_memory()

            return {
                "throughput": num_vectors / elapsed,
                "total_time": elapsed,
                "memory_delta": end_mem - start_mem,
                "vectors_per_second": num_vectors / elapsed,
            }

        except ImportError:
            return {"error": "Faiss not installed"}

    def benchmark_query(self, num_queries: int = 100, k: int = 10) -> Dict:
        """Benchmark Faiss query performance."""
        if self.index is None or self.vectors is None:
            return {"error": "Must run insertion benchmark first"}

        queries = self.generate_queries(num_queries, self.dimension)
        latencies = []

        for query in queries:
            start = time.time()
            distances, indices = self.index.search(query.reshape(1, -1), k)
            latencies.append((time.time() - start) * 1000)

        return {
            "avg_latency_ms": np.mean(latencies),
            "p50_latency_ms": np.percentile(latencies, 50),
            "p95_latency_ms": np.percentile(latencies, 95),
            "p99_latency_ms": np.percentile(latencies, 99),
            "queries_per_second": 1000 / np.mean(latencies),
        }

    def benchmark_memory(self) -> Dict:
        """Benchmark Faiss memory usage."""
        if self.index is None:
            return {"error": "Must run insertion benchmark first"}

        current_mem = self.measure_memory()
        vector_count = self.index.ntotal
        bytes_per_vector = (
            (current_mem * 1024 * 1024) / vector_count if vector_count > 0 else 0
        )

        return {
            "total_memory_mb": current_mem,
            "bytes_per_vector": bytes_per_vector,
            "vector_count": vector_count,
        }


def run_comparison(sizes: List[int] = [1000, 5000, 10000]) -> Dict:
    """Run comparison benchmarks across all databases."""

    benchmarks = [
        OmenDBBenchmark(),
        ChromaDBBenchmark(),
        FaissBenchmark(),
    ]

    all_results = {}

    for benchmark in benchmarks:
        print(f"\n{'=' * 60}")
        print(f"🚀 Running {benchmark.name} benchmarks")
        print("=" * 60)

        results = benchmark.run_all_benchmarks(sizes)
        all_results[benchmark.name] = results

    return all_results


def print_comparison_table(results: Dict):
    """Print a comparison table of results."""
    print("\n" + "=" * 80)
    print("📊 PERFORMANCE COMPARISON")
    print("=" * 80)

    # Extract metrics for comparison
    databases = list(results.keys())
    sizes = list(results[databases[0]]["benchmarks"].keys())

    for size in sizes:
        print(f"\n📈 {size} vectors:")
        print("-" * 60)

        # Insertion throughput
        print("\nInsertion Throughput (vectors/second):")
        for db in databases:
            if size in results[db]["benchmarks"]:
                throughput = results[db]["benchmarks"][size]["insertion"].get(
                    "throughput", "N/A"
                )
                if throughput != "N/A":
                    print(f"  {db:15} {throughput:12,.0f} vec/s")

        # Query latency
        print("\nQuery Latency (p50/p95/p99 ms):")
        for db in databases:
            if size in results[db]["benchmarks"]:
                query = results[db]["benchmarks"][size]["query"]
                if "p50_latency_ms" in query:
                    p50 = query["p50_latency_ms"]
                    p95 = query["p95_latency_ms"]
                    p99 = query["p99_latency_ms"]
                    print(f"  {db:15} {p50:5.2f} / {p95:5.2f} / {p99:5.2f} ms")

        # Memory usage
        print("\nMemory Usage (bytes/vector):")
        for db in databases:
            if size in results[db]["benchmarks"]:
                memory = results[db]["benchmarks"][size]["memory"].get(
                    "bytes_per_vector", "N/A"
                )
                if memory != "N/A":
                    print(f"  {db:15} {memory:8.0f} bytes")


def save_results(results: Dict, filename: str = None):
    """Save benchmark results to JSON file."""
    if filename is None:
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"benchmark_results_{timestamp}.json"

    with open(filename, "w") as f:
        json.dump(results, f, indent=2, default=str)

    print(f"\n💾 Results saved to {filename}")


def main():
    """Main benchmark runner."""
    print("🎯 OmenDB Competitive Benchmark Suite")
    print("=" * 60)

    # Run benchmarks
    results = run_comparison(sizes=[1000, 5000, 10000])

    # Print comparison table
    print_comparison_table(results)

    # Save results
    save_results(results)

    # Print winner summary
    print("\n" + "=" * 60)
    print("🏆 PERFORMANCE SUMMARY")
    print("=" * 60)

    # Find best performer for each metric
    if "OmenDB" in results and "ChromaDB" in results:
        omen_10k = results["OmenDB"]["benchmarks"].get(10000, {})
        chroma_10k = results["ChromaDB"]["benchmarks"].get(10000, {})

        if omen_10k and chroma_10k:
            omen_throughput = omen_10k["insertion"].get("throughput", 0)
            chroma_throughput = chroma_10k["insertion"].get("throughput", 0)

            if omen_throughput > 0 and chroma_throughput > 0:
                speedup = omen_throughput / chroma_throughput
                print(
                    f"\n✅ OmenDB is {speedup:.1f}x faster than ChromaDB at 10K vectors"
                )


if __name__ == "__main__":
    main()
