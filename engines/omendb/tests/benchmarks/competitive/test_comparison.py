#!/usr/bin/env python3
"""
Comprehensive competitive benchmarking: OmenDB vs ChromaDB, Faiss, and others.

This benchmark provides honest, rigorous performance comparison across multiple
vector database implementations using identical datasets and testing conditions.
"""

import sys
import time
import math
import numpy as np
import psutil
import traceback
from typing import List, Tuple, Dict, Any, Optional
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, "python")


@dataclass
class BenchmarkResult:
    """Results from a benchmark test."""

    database: str
    dataset_name: str
    vector_count: int
    dimension: int
    construction_time: float
    construction_rate: float  # vectors/second
    avg_query_time: float  # milliseconds
    memory_usage: float  # MB
    accuracy: float  # average similarity for exact matches
    success: bool
    notes: str = ""


class DatabaseInterface:
    """Abstract interface for testing different vector databases."""

    def __init__(self, name: str):
        self.name = name

    def setup(self) -> bool:
        """Setup the database. Return True if successful."""
        raise NotImplementedError

    def add_vectors(self, vectors: List[Tuple[str, List[float]]]) -> Tuple[float, bool]:
        """Add vectors. Return (time_taken, success)."""
        raise NotImplementedError

    def query(
        self, vector: List[float], k: int = 10
    ) -> Tuple[List[Tuple[str, float]], float]:
        """Query for similar vectors. Return (results, time_taken)."""
        raise NotImplementedError

    def get_memory_usage(self) -> float:
        """Get current memory usage in MB."""
        process = psutil.Process()
        return process.memory_info().rss / 1024 / 1024

    def cleanup(self):
        """Clean up resources."""
        pass


class OmenDBInterface(DatabaseInterface):
    """OmenDB implementation for benchmarking."""

    def __init__(self):
        super().__init__("OmenDB")
        self.db = None

    def setup(self) -> bool:
        try:
            from omendb import DB

            self.db = DB()
            return True
        except Exception as e:
            print(f"Failed to setup OmenDB: {e}")
            return False

    def add_vectors(self, vectors: List[Tuple[str, np.ndarray]]) -> Tuple[float, bool]:
        if not self.db:
            return 0.0, False

        start_time = time.time()
        try:
            # Use batch API for fair comparison with NumPy arrays
            if len(vectors) > 1:
                ids = [v[0] for v in vectors]
                vecs = np.array([v[1] for v in vectors], dtype=np.float32)
                result_ids = self.db.add_batch(vectors=vecs, ids=ids)
                success = len(result_ids) == len(vectors)
            else:
                # Single vector
                success = self.db.add(vectors[0][0], vectors[0][1])
            return time.time() - start_time, success
        except Exception as e:
            print(f"OmenDB add_vectors failed: {e}")
            return time.time() - start_time, False

    def query(
        self, vector: List[float], k: int = 10
    ) -> Tuple[List[Tuple[str, float]], float]:
        if not self.db:
            return [], 0.0

        start_time = time.time()
        try:
            results = self.db.search(vector, limit=k)
            query_time = time.time() - start_time

            # Convert to expected format
            formatted_results = [(r.id, r.score) for r in results]
            return formatted_results, query_time * 1000  # Convert to milliseconds
        except Exception as e:
            print(f"OmenDB query failed: {e}")
            return [], 0.0

    def cleanup(self):
        if self.db:
            try:
                self.db.close()
            except:
                pass


class ChromaDBInterface(DatabaseInterface):
    """ChromaDB implementation for benchmarking."""

    def __init__(self):
        super().__init__("ChromaDB")
        self.client = None
        self.collection = None

    def setup(self) -> bool:
        try:
            import chromadb
            from chromadb.config import Settings

            # Create in-memory ChromaDB for fair comparison
            self.client = chromadb.Client(
                Settings(
                    chroma_db_impl="duckdb+parquet",
                    persist_directory=None,  # In-memory
                )
            )

            # Create collection
            self.collection = self.client.create_collection(
                name="benchmark_collection", metadata={"hnsw:space": "cosine"}
            )
            return True
        except ImportError:
            print("ChromaDB not available - skipping ChromaDB benchmarks")
            return False
        except Exception as e:
            print(f"Failed to setup ChromaDB: {e}")
            return False

    def add_vectors(self, vectors: List[Tuple[str, List[float]]]) -> Tuple[float, bool]:
        if not self.collection:
            return 0.0, False

        start_time = time.time()
        try:
            ids = [vector_id for vector_id, _ in vectors]
            embeddings = [vector for _, vector in vectors]

            self.collection.add(ids=ids, embeddings=embeddings)
            return time.time() - start_time, True
        except Exception as e:
            print(f"ChromaDB add_vectors failed: {e}")
            return time.time() - start_time, False

    def query(
        self, vector: List[float], k: int = 10
    ) -> Tuple[List[Tuple[str, float]], float]:
        if not self.collection:
            return [], 0.0

        start_time = time.time()
        try:
            results = self.collection.query(query_embeddings=[vector], top_k=k)
            query_time = time.time() - start_time

            # Extract results
            if results["ids"] and len(results["ids"]) > 0:
                ids = results["ids"][0]
                distances = results["distances"][0]

                # Convert distances to similarities (ChromaDB returns cosine distances)
                similarities = [1.0 - d for d in distances]
                formatted_results = list(zip(ids, similarities))
                return formatted_results, query_time * 1000
            else:
                return [], query_time * 1000
        except Exception as e:
            print(f"ChromaDB query failed: {e}")
            return [], 0.0

    def cleanup(self):
        if self.client:
            try:
                self.client.reset()
            except:
                pass


class FaissInterface(DatabaseInterface):
    """Faiss implementation for benchmarking."""

    def __init__(self):
        super().__init__("Faiss")
        self.index = None
        self.id_map = {}  # Map from Faiss indices to our IDs
        self.vectors = []

    def setup(self) -> bool:
        try:
            import faiss

            self.faiss = faiss
            return True
        except ImportError:
            print("Faiss not available - skipping Faiss benchmarks")
            return False
        except Exception as e:
            print(f"Failed to setup Faiss: {e}")
            return False

    def add_vectors(self, vectors: List[Tuple[str, List[float]]]) -> Tuple[float, bool]:
        if not self.faiss:
            return 0.0, False

        start_time = time.time()
        try:
            if not vectors:
                return 0.0, True

            # Get dimension from first vector
            dimension = len(vectors[0][1])

            # Create index if not exists
            if self.index is None:
                self.index = self.faiss.IndexFlatIP(
                    dimension
                )  # Inner product (cosine similarity)

            # Prepare vectors for Faiss
            vector_array = np.array([vector for _, vector in vectors], dtype=np.float32)

            # Normalize vectors for cosine similarity
            self.faiss.normalize_L2(vector_array)

            # Add to index
            start_idx = len(self.vectors)
            self.index.add(vector_array)

            # Update mappings
            for i, (vector_id, vector) in enumerate(vectors):
                self.id_map[start_idx + i] = vector_id
                self.vectors.append(vector)

            return time.time() - start_time, True
        except Exception as e:
            print(f"Faiss add_vectors failed: {e}")
            return time.time() - start_time, False

    def query(
        self, vector: List[float], k: int = 10
    ) -> Tuple[List[Tuple[str, float]], float]:
        if not self.index or not self.faiss:
            return [], 0.0

        start_time = time.time()
        try:
            # Prepare query vector
            query_array = np.array([vector], dtype=np.float32)
            self.faiss.normalize_L2(query_array)

            # Search
            similarities, indices = self.index.search(query_array, k)
            query_time = time.time() - start_time

            # Format results
            results = []
            for i, idx in enumerate(indices[0]):
                if idx in self.id_map:
                    vector_id = self.id_map[idx]
                    similarity = float(similarities[0][i])
                    results.append((vector_id, similarity))

            return results, query_time * 1000
        except Exception as e:
            print(f"Faiss query failed: {e}")
            return [], 0.0

    def cleanup(self):
        self.index = None
        self.id_map = {}
        self.vectors = []


def generate_test_dataset(
    count: int, dimension: int, pattern: str = "gaussian"
) -> List[Tuple[str, List[float]]]:
    """Generate test dataset with specified pattern."""
    np.random.seed(42)  # Reproducible results
    vectors = []

    if pattern == "gaussian":
        # Gaussian distributed vectors
        for i in range(count):
            vector = np.random.normal(0, 1, dimension)
            vector = vector / np.linalg.norm(vector)  # Normalize
            vectors.append((f"vec_{i:05d}", vector))

    elif pattern == "clustered":
        # Clustered vectors (like real embeddings)
        num_clusters = max(5, count // 20)
        cluster_centers = []

        # Generate cluster centers
        for i in range(num_clusters):
            center = np.random.normal(0, 0.5, dimension)
            center = center / np.linalg.norm(center)
            cluster_centers.append(center)

        # Generate vectors around clusters
        for i in range(count):
            if np.random.random() < 0.8:  # 80% clustered
                cluster_idx = np.random.randint(0, len(cluster_centers))
                base_vector = cluster_centers[cluster_idx]
                noise = np.random.normal(0, 0.1, dimension)
                vector = base_vector + noise
            else:  # 20% random
                vector = np.random.normal(0, 0.5, dimension)

            vector = vector / np.linalg.norm(vector)
            vectors.append((f"clustered_vec_{i:05d}", vector))

    elif pattern == "uniform":
        # Uniform random vectors
        for i in range(count):
            vector = np.random.uniform(-1, 1, dimension)
            vector = vector / np.linalg.norm(vector)
            vectors.append((f"uniform_vec_{i:05d}", vector))

    return vectors


def benchmark_database(
    db_interface: DatabaseInterface,
    test_dataset: List[Tuple[str, List[float]]],
    dataset_name: str,
    num_queries: int = 10,
) -> BenchmarkResult:
    """Benchmark a single database with a test dataset."""
    print(f"  🧪 Testing {db_interface.name}...")

    if not db_interface.setup():
        return BenchmarkResult(
            database=db_interface.name,
            dataset_name=dataset_name,
            vector_count=len(test_dataset),
            dimension=len(test_dataset[0][1]) if test_dataset else 0,
            construction_time=0.0,
            construction_rate=0.0,
            avg_query_time=0.0,
            memory_usage=0.0,
            accuracy=0.0,
            success=False,
            notes="Setup failed",
        )

    # Measure initial memory
    initial_memory = db_interface.get_memory_usage()

    # Add vectors and measure construction time
    construction_time, add_success = db_interface.add_vectors(test_dataset)

    if not add_success:
        db_interface.cleanup()
        return BenchmarkResult(
            database=db_interface.name,
            dataset_name=dataset_name,
            vector_count=len(test_dataset),
            dimension=len(test_dataset[0][1]) if test_dataset else 0,
            construction_time=construction_time,
            construction_rate=0.0,
            avg_query_time=0.0,
            memory_usage=0.0,
            accuracy=0.0,
            success=False,
            notes="Vector addition failed",
        )

    construction_rate = (
        len(test_dataset) / construction_time if construction_time > 0 else 0
    )

    # Measure memory after construction
    final_memory = db_interface.get_memory_usage()
    memory_usage = final_memory - initial_memory

    # Test query performance and accuracy
    query_times = []
    total_accuracy = 0.0
    successful_queries = 0

    query_count = min(num_queries, len(test_dataset))
    for i in range(query_count):
        # Use existing vectors as queries for exact match testing
        query_idx = i * (len(test_dataset) // query_count)
        expected_id, query_vector = test_dataset[query_idx]

        results, query_time = db_interface.query(query_vector, k=5)

        if query_time > 0:
            query_times.append(query_time)

            # Check for exact match
            exact_match = next((r for r in results if r[0] == expected_id), None)
            if exact_match:
                total_accuracy += exact_match[1]  # Add similarity score
                successful_queries += 1

    avg_query_time = sum(query_times) / len(query_times) if query_times else 0.0
    avg_accuracy = (
        total_accuracy / successful_queries if successful_queries > 0 else 0.0
    )

    # Cleanup
    db_interface.cleanup()

    print(
        f"    📊 Construction: {construction_time:.3f}s ({construction_rate:.0f} vec/s)"
    )
    print(f"    ⚡ Query: {avg_query_time:.2f}ms average")
    print(f"    🎯 Accuracy: {avg_accuracy:.4f} (exact match similarity)")
    print(f"    💾 Memory: {memory_usage:.1f} MB")

    return BenchmarkResult(
        database=db_interface.name,
        dataset_name=dataset_name,
        vector_count=len(test_dataset),
        dimension=len(test_dataset[0][1]) if test_dataset else 0,
        construction_time=construction_time,
        construction_rate=construction_rate,
        avg_query_time=avg_query_time,
        memory_usage=memory_usage,
        accuracy=avg_accuracy,
        success=True,
        notes="",
    )


def run_competitive_benchmark():
    """Run comprehensive competitive benchmarking."""
    print("🏁 Competitive Vector Database Benchmark")
    print("=" * 60)

    # Test configurations
    test_configs = [
        {
            "count": 100,
            "dimension": 128,
            "pattern": "gaussian",
            "name": "Small Gaussian (100 x 128D)",
        },
        {
            "count": 100,
            "dimension": 384,
            "pattern": "clustered",
            "name": "Small Clustered (100 x 384D)",
        },
        {
            "count": 500,
            "dimension": 300,
            "pattern": "gaussian",
            "name": "Medium Gaussian (500 x 300D)",
        },
        {
            "count": 1000,
            "dimension": 256,
            "pattern": "clustered",
            "name": "Large Clustered (1000 x 256D)",
        },
    ]

    # Database interfaces to test
    databases = [
        OmenDBInterface(),
        ChromaDBInterface(),
        FaissInterface(),
    ]

    all_results = []

    for config in test_configs:
        print(f"\n📊 Testing: {config['name']}")
        print("-" * 40)

        # Generate test dataset
        test_dataset = generate_test_dataset(
            config["count"], config["dimension"], config["pattern"]
        )

        config_results = []

        # Test each database
        for db_interface in databases:
            try:
                result = benchmark_database(db_interface, test_dataset, config["name"])
                config_results.append(result)
                all_results.append(result)
            except Exception as e:
                print(f"    ❌ {db_interface.name} failed: {e}")
                traceback.print_exc()

        # Compare results for this configuration
        successful_results = [r for r in config_results if r.success]
        if len(successful_results) > 1:
            print(f"\n  📈 Comparison for {config['name']}:")

            # Sort by construction rate
            by_construction = sorted(
                successful_results, key=lambda x: x.construction_rate, reverse=True
            )
            print(
                f"    🏗️  Construction Rate: {by_construction[0].database} wins ({by_construction[0].construction_rate:.0f} vec/s)"
            )

            # Sort by query time
            by_query = sorted(successful_results, key=lambda x: x.avg_query_time)
            print(
                f"    ⚡ Query Speed: {by_query[0].database} wins ({by_query[0].avg_query_time:.2f}ms)"
            )

            # Sort by accuracy
            by_accuracy = sorted(
                successful_results, key=lambda x: x.accuracy, reverse=True
            )
            print(
                f"    🎯 Accuracy: {by_accuracy[0].database} wins ({by_accuracy[0].accuracy:.4f})"
            )

            # Sort by memory usage
            by_memory = sorted(successful_results, key=lambda x: x.memory_usage)
            print(
                f"    💾 Memory: {by_memory[0].database} wins ({by_memory[0].memory_usage:.1f} MB)"
            )

    # Overall analysis
    print("\n" + "=" * 60)
    print("🏆 OVERALL COMPETITIVE ANALYSIS")
    print("=" * 60)

    successful_results = [r for r in all_results if r.success]

    if successful_results:
        # Group by database
        by_database = {}
        for result in successful_results:
            if result.database not in by_database:
                by_database[result.database] = []
            by_database[result.database].append(result)

        print("\n📊 Performance Summary:")
        for db_name, results in by_database.items():
            avg_construction = sum(r.construction_rate for r in results) / len(results)
            avg_query = sum(r.avg_query_time for r in results) / len(results)
            avg_accuracy = sum(r.accuracy for r in results) / len(results)
            avg_memory = sum(r.memory_usage for r in results) / len(results)

            print(f"\n  🔹 {db_name}:")
            print(f"     Construction: {avg_construction:.0f} vec/s average")
            print(f"     Query: {avg_query:.2f}ms average")
            print(f"     Accuracy: {avg_accuracy:.4f} average")
            print(f"     Memory: {avg_memory:.1f} MB average")

        # Determine winners
        print(f"\n🏅 Category Winners:")

        # Overall construction rate
        best_construction = max(successful_results, key=lambda x: x.construction_rate)
        print(
            f"  🏗️  Best Construction: {best_construction.database} ({best_construction.construction_rate:.0f} vec/s)"
        )

        # Overall query speed
        best_query = min(successful_results, key=lambda x: x.avg_query_time)
        print(
            f"  ⚡ Best Query: {best_query.database} ({best_query.avg_query_time:.2f}ms)"
        )

        # Overall accuracy
        best_accuracy = max(successful_results, key=lambda x: x.accuracy)
        print(
            f"  🎯 Best Accuracy: {best_accuracy.database} ({best_accuracy.accuracy:.4f})"
        )

        # Overall memory efficiency
        best_memory = min(successful_results, key=lambda x: x.memory_usage)
        print(
            f"  💾 Most Memory Efficient: {best_memory.database} ({best_memory.memory_usage:.1f} MB)"
        )

        # Check if OmenDB participated and how it performed
        omendb_results = [r for r in successful_results if r.database == "OmenDB"]
        if omendb_results:
            print(f"\n🎯 OmenDB Performance Analysis:")
            print(f"  ✅ Successfully tested in {len(omendb_results)} configurations")

            omendb_wins = 0
            total_categories = 0

            for category in [
                "construction_rate",
                "avg_query_time",
                "accuracy",
                "memory_usage",
            ]:
                if category == "avg_query_time" or category == "memory_usage":
                    best_in_category = min(
                        successful_results, key=lambda x: getattr(x, category)
                    )
                else:
                    best_in_category = max(
                        successful_results, key=lambda x: getattr(x, category)
                    )

                if best_in_category.database == "OmenDB":
                    omendb_wins += 1
                total_categories += 1

            win_rate = omendb_wins / total_categories * 100
            print(
                f"  📈 Category Win Rate: {omendb_wins}/{total_categories} ({win_rate:.1f}%)"
            )

            if win_rate >= 50:
                print(f"  🏆 OmenDB shows competitive performance!")
            else:
                print(f"  📝 OmenDB shows room for improvement in some areas")
        else:
            print(f"\n❌ OmenDB failed to complete benchmarks - investigation needed")

    print(
        f"\n🔬 Benchmark completed with {len(successful_results)} successful test runs"
    )


if __name__ == "__main__":
    run_competitive_benchmark()
