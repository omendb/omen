#!/usr/bin/env python3
"""
OmenDB vs ChromaDB Competitive Benchmark

Head-to-head performance comparison using identical datasets and operations.
This provides real market positioning data vs established competitors.

Metrics Compared:
- Insert latency and throughput
- Query latency and accuracy
- Memory usage
- Storage efficiency
- Setup/teardown time
"""

import os
import sys
import time
import random
import statistics
import tempfile
import json
import tracemalloc
from typing import List, Dict, Any, Tuple
from dataclasses import dataclass

# Add the python directory to the path
current_dir = os.path.dirname(os.path.abspath(__file__))
root_dir = os.path.dirname(os.path.dirname(current_dir))
python_dir = os.path.join(root_dir, "python")
sys.path.insert(0, python_dir)

from omendb import DB


@dataclass
class BenchmarkResult:
    """Results from a competitive benchmark run."""

    database: str
    scale: int
    dimension: int
    insert_latency_ms: float
    insert_throughput_ops_sec: float
    query_latency_ms: float
    query_throughput_ops_sec: float
    memory_usage_mb: float
    setup_time_ms: float
    file_size_mb: float
    accuracy_score: float  # Similarity ranking correlation

    def to_dict(self) -> Dict[str, Any]:
        return {
            "database": self.database,
            "scale": self.scale,
            "dimension": self.dimension,
            "insert_latency_ms": self.insert_latency_ms,
            "insert_throughput_ops_sec": self.insert_throughput_ops_sec,
            "query_latency_ms": self.query_latency_ms,
            "query_throughput_ops_sec": self.query_throughput_ops_sec,
            "memory_usage_mb": self.memory_usage_mb,
            "setup_time_ms": self.setup_time_ms,
            "file_size_mb": self.file_size_mb,
            "accuracy_score": self.accuracy_score,
        }


class CompetitiveBenchmark:
    """Framework for competitive benchmarking against other vector databases."""

    def __init__(self):
        self.results = []
        self.test_vectors = None
        self.query_vectors = None

    def generate_standard_dataset(
        self, count: int, dimension: int
    ) -> List[Tuple[str, List[float]]]:
        """Generate standardized dataset for consistent comparison."""
        vectors = []
        random.seed(42)  # Consistent seed for reproducible results

        for i in range(count):
            # Generate normalized vectors for better similarity comparisons
            vector = [random.gauss(0, 1) for _ in range(dimension)]
            norm = sum(x * x for x in vector) ** 0.5
            if norm > 0:
                vector = [x / norm for x in vector]
            vectors.append((f"doc_{i:06d}", vector))

        return vectors

    def generate_query_vectors(self, count: int, dimension: int) -> List[List[float]]:
        """Generate query vectors for consistent testing."""
        queries = []
        random.seed(123)  # Different seed for queries

        for _ in range(count):
            vector = [random.gauss(0, 1) for _ in range(dimension)]
            norm = sum(x * x for x in vector) ** 0.5
            if norm > 0:
                vector = [x / norm for x in vector]
            queries.append(vector)

        return queries

    def benchmark_omendb(
        self,
        vectors: List[Tuple[str, List[float]]],
        queries: List[List[float]],
        k: int = 10,
    ) -> BenchmarkResult:
        """Benchmark OmenDB with the test dataset."""
        tracemalloc.start()

        with tempfile.NamedTemporaryFile(suffix=".omen", delete=False) as tmp:
            db_path = tmp.name

        try:
            # Measure setup time
            setup_start = time.time()
            db = DB(db_path)
            setup_time_ms = (time.time() - setup_start) * 1000

            # Benchmark insertions
            insert_start = time.time()
            for vec_id, vector in vectors:
                success = db.add(vec_id, vector)
                if not success:
                    raise Exception(f"Failed to insert {vec_id}")
            insert_duration = time.time() - insert_start

            insert_latency_ms = (insert_duration * 1000) / len(vectors)
            insert_throughput_ops_sec = len(vectors) / insert_duration

            # Benchmark queries and collect results for accuracy
            query_times = []
            omendb_results = []

            for query in queries:
                query_start = time.time()
                results = db.search(query, limit=k)
                query_time = (time.time() - query_start) * 1000
                query_times.append(query_time)

                # Store results for accuracy comparison
                result_ids = [r.id for r in results]
                omendb_results.append(result_ids)

            query_latency_ms = statistics.mean(query_times)
            query_throughput_ops_sec = len(queries) / (sum(query_times) / 1000)

            # Measure memory usage
            current, peak = tracemalloc.get_traced_memory()
            memory_usage_mb = current / 1024 / 1024

            # Measure file size
            file_size_mb = (
                os.path.getsize(db_path) / 1024 / 1024 if os.path.exists(db_path) else 0
            )

            # Store results for accuracy comparison
            self.omendb_query_results = omendb_results

            return BenchmarkResult(
                database="OmenDB",
                scale=len(vectors),
                dimension=len(vectors[0][1]),
                insert_latency_ms=insert_latency_ms,
                insert_throughput_ops_sec=insert_throughput_ops_sec,
                query_latency_ms=query_latency_ms,
                query_throughput_ops_sec=query_throughput_ops_sec,
                memory_usage_mb=memory_usage_mb,
                setup_time_ms=setup_time_ms,
                file_size_mb=file_size_mb,
                accuracy_score=1.0,  # Reference accuracy
            )

        finally:
            tracemalloc.stop()
            if os.path.exists(db_path):
                os.unlink(db_path)

    def benchmark_chromadb(
        self,
        vectors: List[Tuple[str, List[float]]],
        queries: List[List[float]],
        k: int = 10,
    ) -> BenchmarkResult:
        """Benchmark ChromaDB with the same dataset."""
        try:
            import chromadb
            from chromadb.config import Settings
        except ImportError:
            print("❌ ChromaDB not installed. Install with: pip install chromadb")
            return self._create_mock_result(
                "ChromaDB", len(vectors), len(vectors[0][1])
            )

        tracemalloc.start()

        with tempfile.TemporaryDirectory() as tmp_dir:
            try:
                # Measure setup time
                setup_start = time.time()
                client = chromadb.PersistentClient(
                    path=tmp_dir, settings=Settings(anonymized_telemetry=False)
                )
                collection = client.create_collection("test_collection")
                setup_time_ms = (time.time() - setup_start) * 1000

                # Benchmark insertions (ChromaDB batch size limit)
                ids = [vec_id for vec_id, _ in vectors]
                embeddings = [vector for _, vector in vectors]

                insert_start = time.time()
                batch_size = 5000  # ChromaDB's max batch size
                for i in range(0, len(ids), batch_size):
                    batch_ids = ids[i : i + batch_size]
                    batch_embeddings = embeddings[i : i + batch_size]
                    collection.add(ids=batch_ids, embeddings=batch_embeddings)
                insert_duration = time.time() - insert_start

                insert_latency_ms = (insert_duration * 1000) / len(vectors)
                insert_throughput_ops_sec = len(vectors) / insert_duration

                # Benchmark queries
                query_times = []
                chromadb_results = []

                for query in queries:
                    query_start = time.time()
                    results = collection.query(query_embeddings=[query], n_results=k)
                    query_time = (time.time() - query_start) * 1000
                    query_times.append(query_time)

                    # Store results for accuracy comparison
                    result_ids = results["ids"][0] if results["ids"] else []
                    chromadb_results.append(result_ids)

                query_latency_ms = statistics.mean(query_times)
                query_throughput_ops_sec = len(queries) / (sum(query_times) / 1000)

                # Measure memory usage
                current, peak = tracemalloc.get_traced_memory()
                memory_usage_mb = current / 1024 / 1024

                # Calculate accuracy vs OmenDB
                accuracy_score = self._calculate_accuracy(
                    chromadb_results, self.omendb_query_results
                )

                # Estimate storage size (ChromaDB uses SQLite + other files)
                file_size_mb = (
                    sum(
                        os.path.getsize(os.path.join(root, file))
                        for root, dirs, files in os.walk(tmp_dir)
                        for file in files
                    )
                    / 1024
                    / 1024
                )

                return BenchmarkResult(
                    database="ChromaDB",
                    scale=len(vectors),
                    dimension=len(vectors[0][1]),
                    insert_latency_ms=insert_latency_ms,
                    insert_throughput_ops_sec=insert_throughput_ops_sec,
                    query_latency_ms=query_latency_ms,
                    query_throughput_ops_sec=query_throughput_ops_sec,
                    memory_usage_mb=memory_usage_mb,
                    setup_time_ms=setup_time_ms,
                    file_size_mb=file_size_mb,
                    accuracy_score=accuracy_score,
                )

            finally:
                tracemalloc.stop()

    def _calculate_accuracy(
        self, results1: List[List[str]], results2: List[List[str]]
    ) -> float:
        """Calculate similarity ranking correlation between two result sets."""
        if not results1 or not results2 or len(results1) != len(results2):
            return 0.0

        # Calculate average overlap in top-k results
        overlaps = []
        for r1, r2 in zip(results1, results2):
            if not r1 or not r2:
                overlaps.append(0.0)
                continue

            set1, set2 = set(r1), set(r2)
            overlap = len(set1.intersection(set2)) / max(len(set1), len(set2))
            overlaps.append(overlap)

        return statistics.mean(overlaps) if overlaps else 0.0

    def _create_mock_result(
        self, db_name: str, scale: int, dimension: int
    ) -> BenchmarkResult:
        """Create mock result when database is not available."""
        return BenchmarkResult(
            database=f"{db_name} (Not Available)",
            scale=scale,
            dimension=dimension,
            insert_latency_ms=0.0,
            insert_throughput_ops_sec=0.0,
            query_latency_ms=0.0,
            query_throughput_ops_sec=0.0,
            memory_usage_mb=0.0,
            setup_time_ms=0.0,
            file_size_mb=0.0,
            accuracy_score=0.0,
        )

    def run_competitive_benchmark(
        self, scale: int = 10000, dimension: int = 128, num_queries: int = 100
    ) -> List[BenchmarkResult]:
        """Run head-to-head benchmark between OmenDB and competitors."""
        print(f"🏆 Competitive Benchmark: {scale:,} vectors, {dimension}D")
        print("=" * 60)

        # Generate standard dataset
        print(f"📊 Generating standardized dataset...")
        vectors = self.generate_standard_dataset(scale, dimension)
        queries = self.generate_query_vectors(num_queries, dimension)

        results = []

        # Benchmark OmenDB (reference)
        print(f"\n🧪 Benchmarking OmenDB...")
        omendb_result = self.benchmark_omendb(vectors, queries)
        results.append(omendb_result)
        self._print_result(omendb_result)

        # Benchmark ChromaDB
        print(f"\n🧪 Benchmarking ChromaDB...")
        chromadb_result = self.benchmark_chromadb(vectors, queries)
        results.append(chromadb_result)
        self._print_result(chromadb_result)

        # Store results
        self.results.extend(results)

        return results

    def _print_result(self, result: BenchmarkResult):
        """Print formatted benchmark result."""
        print(f"   Database: {result.database}")
        print(
            f"   Insert: {result.insert_latency_ms:.2f}ms avg, {result.insert_throughput_ops_sec:.0f} ops/sec"
        )
        print(
            f"   Query: {result.query_latency_ms:.2f}ms avg, {result.query_throughput_ops_sec:.0f} ops/sec"
        )
        print(f"   Memory: {result.memory_usage_mb:.1f}MB")
        print(f"   Storage: {result.file_size_mb:.1f}MB")
        print(f"   Setup: {result.setup_time_ms:.1f}ms")
        print(f"   Accuracy: {result.accuracy_score:.3f}")

    def generate_comparison_report(self) -> str:
        """Generate detailed comparison report."""
        if len(self.results) < 2:
            return "❌ Need at least 2 database results for comparison"

        omendb = next(r for r in self.results if r.database == "OmenDB")
        competitors = [r for r in self.results if r.database != "OmenDB"]

        report = []
        report.append("🏆 COMPETITIVE BENCHMARK REPORT")
        report.append("=" * 50)
        report.append(f"Scale: {omendb.scale:,} vectors, {omendb.dimension}D")
        report.append("")

        for competitor in competitors:
            report.append(f"📊 OmenDB vs {competitor.database}")
            report.append("-" * 40)

            # Performance comparisons
            metrics = [
                (
                    "Insert Latency",
                    omendb.insert_latency_ms,
                    competitor.insert_latency_ms,
                    "ms",
                    True,
                ),
                (
                    "Insert Throughput",
                    omendb.insert_throughput_ops_sec,
                    competitor.insert_throughput_ops_sec,
                    "ops/sec",
                    False,
                ),
                (
                    "Query Latency",
                    omendb.query_latency_ms,
                    competitor.query_latency_ms,
                    "ms",
                    True,
                ),
                (
                    "Query Throughput",
                    omendb.query_throughput_ops_sec,
                    competitor.query_throughput_ops_sec,
                    "ops/sec",
                    False,
                ),
                (
                    "Memory Usage",
                    omendb.memory_usage_mb,
                    competitor.memory_usage_mb,
                    "MB",
                    True,
                ),
                (
                    "Storage Size",
                    omendb.file_size_mb,
                    competitor.file_size_mb,
                    "MB",
                    True,
                ),
                (
                    "Setup Time",
                    omendb.setup_time_ms,
                    competitor.setup_time_ms,
                    "ms",
                    True,
                ),
            ]

            for metric, omen_val, comp_val, unit, lower_is_better in metrics:
                if comp_val > 0:  # Skip if competitor not available
                    if lower_is_better:
                        improvement = ((comp_val - omen_val) / comp_val) * 100
                        winner = "✅ OmenDB" if omen_val < comp_val else "❌ Competitor"
                    else:
                        improvement = ((omen_val - comp_val) / comp_val) * 100
                        winner = "✅ OmenDB" if omen_val > comp_val else "❌ Competitor"

                    report.append(
                        f"{metric:15}: {omen_val:8.1f} vs {comp_val:8.1f} {unit:8} ({improvement:+5.1f}%) {winner}"
                    )

            report.append("")

        return "\n".join(report)

    def save_results(self, filename: str = None):
        """Save benchmark results to JSON file."""
        if filename is None:
            filename = os.path.join(current_dir, "competitive_benchmark_results.json")

        data = {
            "timestamp": time.time(),
            "results": [r.to_dict() for r in self.results],
        }

        with open(filename, "w") as f:
            json.dump(data, f, indent=2)

        print(f"💾 Results saved to {filename}")


def main():
    """Main benchmark runner."""
    benchmark = CompetitiveBenchmark()

    # Run competitive benchmark
    results = benchmark.run_competitive_benchmark(scale=10000, dimension=128)

    if len(results) < 2:
        print("❌ Need multiple databases for meaningful comparison")
        return False

    # Generate and print comparison report
    print("\n" + benchmark.generate_comparison_report())

    # Save results
    benchmark.save_results()

    return True


if __name__ == "__main__":
    success = main()
    exit(0 if success else 1)
