#!/usr/bin/env python3
"""
OmenDB Performance Testing Example
=================================

Comprehensive performance testing and benchmarking for OmenDB.
This example demonstrates how to measure and monitor OmenDB performance
in production environments.

This example shows:
- Performance benchmark setup using batch operations
- Regression detection
- Memory usage monitoring
- Realistic throughput testing (80K+ vec/s)
- CI/CD integration patterns

Requirements:
- Python 3.8+
- OmenDB Python SDK
- psutil (optional, for system metrics)
"""

import os
import sys
import time
import json
import statistics
from typing import List, Dict, Any, Optional
from dataclasses import dataclass, asdict
import math
import random

# Optional system monitoring
try:
    import psutil

    PSUTIL_AVAILABLE = True
except ImportError:
    PSUTIL_AVAILABLE = False
    print("📝 psutil not available - install for detailed system metrics")

# OmenDB Python SDK
try:
    import omendb

    NATIVE_AVAILABLE = True
except ImportError:
    NATIVE_AVAILABLE = False

    # Use stub for testing framework
    class BenchmarkOmenDBStub:
        def __init__(self, path: str):
            self.path = path
            self.vectors = []
            self.metadata = []
            self.operation_times = []

        def insert(self, vector: List[float], metadata: Dict[str, Any] = None):
            start = time.perf_counter()
            self.vectors.append(vector)
            self.metadata.append(metadata or {})
            end = time.perf_counter()
            self.operation_times.append(("insert", end - start))
            return len(self.vectors) - 1

        def search(self, query: List[float], k: int = 10) -> List[Dict[str, Any]]:
            start = time.perf_counter()

            results = []
            for i, vec in enumerate(self.vectors):
                dot_product = sum(a * b for a, b in zip(query, vec))
                norm_q = math.sqrt(sum(x * x for x in query))
                norm_v = math.sqrt(sum(x * x for x in vec))

                if norm_q > 0 and norm_v > 0:
                    similarity = dot_product / (norm_q * norm_v)
                else:
                    similarity = 0.0

                results.append(
                    {"id": i, "similarity": similarity, "metadata": self.metadata[i]}
                )

            results.sort(key=lambda x: x["similarity"], reverse=True)
            end = time.perf_counter()
            self.operation_times.append(("search", end - start))
            return results[:k]

        def get_memory_usage(self) -> int:
            # Estimate memory usage
            vector_size = (
                len(self.vectors[0]) * 4 if self.vectors else 0
            )  # 4 bytes per float
            metadata_size = len(str(self.metadata)) if self.metadata else 0
            return len(self.vectors) * (vector_size + 100) + metadata_size

        def close(self):
            pass

    class omendb:
        DB = BenchmarkOmenDBStub


@dataclass
class BenchmarkResult:
    """Store benchmark results for analysis."""

    operation: str
    dataset_size: int
    dimension: int
    k_value: Optional[int]
    latency_ms: float
    throughput: float
    memory_mb: float
    success_rate: float
    timestamp: float


@dataclass
class SystemMetrics:
    """System-level performance metrics."""

    cpu_percent: float
    memory_percent: float
    available_memory_mb: float
    process_memory_mb: float


class PerformanceBenchmark:
    """Comprehensive performance benchmarking framework."""

    def __init__(self, db_path: str):
        self.db_path = db_path
        self.db = None
        self.baseline_file = "omendb_performance_baseline.json"
        self.results = []

    def setup_database(self, clean: bool = True):
        """Setup database for benchmarking."""
        if clean and os.path.exists(self.db_path):
            os.remove(self.db_path)

        self.db = omendb.DB(self.db_path)
        self.db.clear()  # Ensure database is completely empty
        print(f"📊 Benchmark database ready: {self.db_path}")

    def cleanup_database(self):
        """Cleanup database after benchmarking."""
        if self.db and hasattr(self.db, "close"):
            self.db.close()
        self.db = None
        if os.path.exists(self.db_path):
            os.remove(self.db_path)

    def get_system_metrics(self) -> SystemMetrics:
        """Get current system metrics."""
        if PSUTIL_AVAILABLE:
            process = psutil.Process()
            return SystemMetrics(
                cpu_percent=psutil.cpu_percent(interval=0.1),
                memory_percent=psutil.virtual_memory().percent,
                available_memory_mb=psutil.virtual_memory().available / (1024 * 1024),
                process_memory_mb=process.memory_info().rss / (1024 * 1024),
            )
        else:
            return SystemMetrics(0.0, 0.0, 0.0, 0.0)

    def generate_test_vectors(self, count: int, dimension: int) -> List[List[float]]:
        """Generate test vectors with realistic patterns."""
        vectors = []
        num_clusters = max(
            1, min(20, count // 50)
        )  # Realistic clustering, ensure at least 1

        for i in range(count):
            cluster_id = i % num_clusters
            cluster_center = cluster_id * 0.4 - 0.5

            vector = []
            for j in range(dimension):
                base_value = cluster_center + 0.1 * math.sin(j * 0.1)
                noise = random.uniform(-0.1, 0.1)
                vector.append(base_value + noise)

            # Normalize
            norm = math.sqrt(sum(x * x for x in vector))
            if norm > 0:
                vector = [x / norm for x in vector]

            vectors.append(vector)

        return vectors

    def benchmark_insertions(
        self, dataset_size: int, dimension: int
    ) -> BenchmarkResult:
        """Benchmark vector insertion performance using batch operations."""
        print(f"   Benchmarking insertions: {dataset_size} vectors, {dimension}D")

        vectors = self.generate_test_vectors(dataset_size, dimension)
        ids = [f"vec_{i}" for i in range(dataset_size)]
        metadata = [
            {"id": f"vec_{i}", "benchmark": "insertion_test", "timestamp": time.time()}
            for i in range(dataset_size)
        ]

        start_time = time.perf_counter()
        start_memory = self.get_system_metrics()

        try:
            # Use batch API for realistic performance (91K+ vec/s)
            result_ids = self.db.add_batch(vectors=vectors, ids=ids, metadata=metadata)
            successful_inserts = len(result_ids)
        except Exception as e:
            print(f"      Batch insert failed: {e}")
            successful_inserts = 0

        end_time = time.perf_counter()
        end_memory = self.get_system_metrics()

        total_time = end_time - start_time
        throughput = successful_inserts / total_time
        avg_latency_ms = (
            (total_time * 1000) / successful_inserts if successful_inserts > 0 else 0
        )
        success_rate = successful_inserts / dataset_size

        # Memory usage (try database method first, fall back to system metrics)
        memory_mb = 0
        if hasattr(self.db, "get_memory_usage"):
            memory_mb = self.db.get_memory_usage() / (1024 * 1024)
        else:
            memory_mb = end_memory.process_memory_mb - start_memory.process_memory_mb

        result = BenchmarkResult(
            operation="insert",
            dataset_size=dataset_size,
            dimension=dimension,
            k_value=None,
            latency_ms=avg_latency_ms,
            throughput=throughput,
            memory_mb=memory_mb,
            success_rate=success_rate,
            timestamp=time.time(),
        )

        self.results.append(result)
        return result

    def benchmark_searches(
        self, query_count: int, dimension: int, k_values: List[int]
    ) -> Dict[int, BenchmarkResult]:
        """Benchmark search performance with different k values."""
        print(f"   Benchmarking searches: {query_count} queries, k={k_values}")

        # Generate query vectors
        query_vectors = self.generate_test_vectors(query_count, dimension)
        results = {}

        for k in k_values:
            print(f"      Testing k={k}...")

            successful_searches = 0
            search_times = []

            start_memory = self.get_system_metrics()

            for query in query_vectors:
                try:
                    search_start = time.perf_counter()
                    search_results = self.db.search(query, limit=k)
                    search_end = time.perf_counter()

                    search_times.append(search_end - search_start)
                    successful_searches += 1

                except Exception as e:
                    print(f"         Search failed: {e}")

            end_memory = self.get_system_metrics()

            if search_times:
                avg_latency_ms = statistics.mean(search_times) * 1000
                throughput = successful_searches / sum(search_times)
                success_rate = successful_searches / query_count

                # Memory usage
                memory_mb = 0
                if hasattr(self.db, "get_memory_usage"):
                    memory_mb = self.db.get_memory_usage() / (1024 * 1024)
                else:
                    memory_mb = end_memory.process_memory_mb

                result = BenchmarkResult(
                    operation="search",
                    dataset_size=len(self.db.vectors)
                    if hasattr(self.db, "vectors")
                    else 0,
                    dimension=dimension,
                    k_value=k,
                    latency_ms=avg_latency_ms,
                    throughput=throughput,
                    memory_mb=memory_mb,
                    success_rate=success_rate,
                    timestamp=time.time(),
                )

                self.results.append(result)
                results[k] = result

        return results

    def save_baseline(self):
        """Save current results as performance baseline."""
        baseline_data = {
            "created_at": time.time(),
            "native_implementation": NATIVE_AVAILABLE,
            "results": [asdict(result) for result in self.results],
        }

        with open(self.baseline_file, "w") as f:
            json.dump(baseline_data, f, indent=2)

        print(f"💾 Baseline saved: {self.baseline_file}")

    def load_baseline(self) -> Optional[Dict[str, Any]]:
        """Load performance baseline for comparison."""
        if not os.path.exists(self.baseline_file):
            print("📝 No baseline found - will establish new baseline")
            return None

        with open(self.baseline_file, "r") as f:
            baseline_data = json.load(f)

        print(f"📊 Loaded baseline from {time.ctime(baseline_data['created_at'])}")
        return baseline_data

    def detect_regressions(
        self, baseline_data: Dict[str, Any], threshold_percent: float = 25.0
    ) -> List[str]:
        """Detect performance regressions compared to baseline."""
        if not baseline_data:
            return []

        baseline_results = {
            (r["operation"], r["dataset_size"], r.get("k_value")): r
            for r in baseline_data["results"]
        }

        regressions = []

        for current in self.results:
            key = (current.operation, current.dataset_size, current.k_value)

            if key in baseline_results:
                baseline = baseline_results[key]

                # Check latency regression
                if baseline["latency_ms"] > 0:
                    latency_change = (
                        (current.latency_ms - baseline["latency_ms"])
                        / baseline["latency_ms"]
                        * 100
                    )
                    if latency_change > threshold_percent:
                        regressions.append(
                            f"{current.operation} latency regression: +{latency_change:.1f}% "
                            f"({current.latency_ms:.2f}ms vs {baseline['latency_ms']:.2f}ms)"
                        )

                # Check throughput regression
                if baseline["throughput"] > 0:
                    throughput_change = (
                        (baseline["throughput"] - current.throughput)
                        / baseline["throughput"]
                        * 100
                    )
                    if throughput_change > threshold_percent:
                        regressions.append(
                            f"{current.operation} throughput regression: -{throughput_change:.1f}% "
                            f"({current.throughput:.0f} vs {baseline['throughput']:.0f} ops/sec)"
                        )

        return regressions

    def print_results(self):
        """Print formatted benchmark results."""
        print()
        print("📊 BENCHMARK RESULTS")
        print("=" * 50)

        # Group results by operation
        insert_results = [r for r in self.results if r.operation == "insert"]
        search_results = [r for r in self.results if r.operation == "search"]

        if insert_results:
            print()
            print("📥 INSERT PERFORMANCE:")
            for result in insert_results:
                print(f"   {result.dataset_size:,} vectors ({result.dimension}D):")
                print(f"      Latency: {result.latency_ms:.2f}ms avg")
                print(f"      Throughput: {result.throughput:.0f} vectors/sec")
                print(f"      Memory: {result.memory_mb:.1f}MB")
                print(f"      Success rate: {result.success_rate * 100:.1f}%")

        if search_results:
            print()
            print("🔍 SEARCH PERFORMANCE:")
            for result in search_results:
                print(f"   {result.dataset_size:,} vectors (k={result.k_value}):")
                print(f"      Latency: {result.latency_ms:.2f}ms avg")
                print(f"      Throughput: {result.throughput:.0f} queries/sec")
                print(f"      Success rate: {result.success_rate * 100:.1f}%")


def run_comprehensive_benchmark(quick_mode=False):
    """Run comprehensive performance benchmark suite."""

    print("🔬 OmenDB Performance Testing Suite")
    if quick_mode:
        print("⚡ Running in QUICK MODE for CI/testing")
    print("=" * 60)
    print()

    benchmark = PerformanceBenchmark("performance_test.omen")

    try:
        # Setup
        benchmark.setup_database()

        # Load baseline for regression detection
        baseline_data = benchmark.load_baseline()

        # Test configurations
        if quick_mode:
            # Reduced configurations for quick testing
            test_configs = [
                {"size": 100, "dimension": 128},
                {"size": 500, "dimension": 128},
            ]
            search_k_values = [5, 10]
            query_count = 10
        else:
            # Full configurations for comprehensive testing
            test_configs = [
                {"size": 1000, "dimension": 256},
                {"size": 5000, "dimension": 384},
                {"size": 10000, "dimension": 512},
            ]
            search_k_values = [1, 5, 10, 20]
            query_count = 50

        print("🧪 Running Performance Tests...")
        print()

        for config in test_configs:
            dataset_size = config["size"]
            dimension = config["dimension"]

            print(f"📊 Testing scale: {dataset_size:,} vectors, {dimension}D")

            # Recreate database for each configuration to avoid dimension conflicts
            benchmark.cleanup_database()
            benchmark.setup_database()

            # Benchmark insertions
            insert_result = benchmark.benchmark_insertions(dataset_size, dimension)
            print(
                f"   ✅ Insertions: {insert_result.throughput:.0f} vectors/sec, "
                f"{insert_result.latency_ms:.2f}ms avg"
            )

            # Benchmark searches
            search_results = benchmark.benchmark_searches(
                query_count, dimension, search_k_values
            )
            for k, result in search_results.items():
                print(
                    f"   ✅ Search k={k}: {result.latency_ms:.2f}ms avg, "
                    f"{result.throughput:.0f} queries/sec"
                )

            print()

        # Print comprehensive results
        benchmark.print_results()

        # Regression detection
        print()
        print("🔍 REGRESSION DETECTION")
        print("=" * 30)

        if baseline_data:
            regressions = benchmark.detect_regressions(baseline_data)

            if regressions:
                print("❌ PERFORMANCE REGRESSIONS DETECTED:")
                for regression in regressions:
                    print(f"   - {regression}")
            else:
                print("✅ No performance regressions detected")
        else:
            print("📝 No baseline available - establishing new baseline")
            benchmark.save_baseline()

        # Production validation
        print()
        print("🎯 PRODUCTION VALIDATION")
        print("=" * 30)

        # Check against production targets
        latest_search = next(
            (
                r
                for r in reversed(benchmark.results)
                if r.operation == "search" and r.k_value == 10
            ),
            None,
        )
        latest_insert = next(
            (r for r in reversed(benchmark.results) if r.operation == "insert"), None
        )

        if latest_search and latest_insert:
            targets_met = []

            # Search latency target: <2ms
            search_target_met = latest_search.latency_ms <= 2.0
            targets_met.append(
                (
                    "Search latency",
                    latest_search.latency_ms,
                    2.0,
                    "ms",
                    search_target_met,
                )
            )

            # Insert throughput target: >80,000 vectors/sec (realistic for batch ops)
            throughput_target_met = latest_insert.throughput >= 80000.0
            targets_met.append(
                (
                    "Insert throughput",
                    latest_insert.throughput,
                    80000.0,
                    "vectors/sec",
                    throughput_target_met,
                )
            )

            # Memory efficiency target: <5MB per 1K vectors
            memory_per_1k = latest_insert.memory_mb / (
                latest_insert.dataset_size / 1000
            )
            memory_target_met = memory_per_1k <= 5.0
            targets_met.append(
                (
                    "Memory efficiency",
                    memory_per_1k,
                    5.0,
                    "MB/1K vectors",
                    memory_target_met,
                )
            )

            all_targets_met = all(met for _, _, _, _, met in targets_met)

            for name, actual, target, unit, met in targets_met:
                status = "✅ PASS" if met else "❌ FAIL"
                target_op = (
                    "≤"
                    if "latency" in name.lower() or "memory" in name.lower()
                    else "≥"
                )
                print(
                    f"   {name}: {actual:.2f} {unit} (target: {target_op}{target}) {status}"
                )

            print()
            if all_targets_met:
                print("🎉 PRODUCTION VALIDATION: PASSED")
                print("   All performance targets met for production deployment")
            else:
                print("⚠️  PRODUCTION VALIDATION: NEEDS IMPROVEMENT")
                print("   Some performance targets not met")

        # Save updated baseline if no regressions
        if not baseline_data or not regressions:
            benchmark.save_baseline()

        print()
        print("🔗 Integration Options:")
        print("   CI/CD: Run with ./scripts/run-ci-benchmarks.sh")
        print("   Production: Use ./scripts/run-production-benchmarks.sh")
        print("   Monitoring: Set up alerts on regression detection")

    finally:
        benchmark.cleanup_database()


def main():
    """Main performance testing entry point."""
    # Check if running in CI/test mode
    quick_mode = (
        os.environ.get("OMENDB_TEST_MODE") == "quick"
        or os.environ.get("CI") == "true"
        or "--quick" in sys.argv
    )

    run_comprehensive_benchmark(quick_mode=quick_mode)


if __name__ == "__main__":
    main()
