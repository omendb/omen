#!/usr/bin/env python3
"""
OmenDB Production Scale Example
==============================

Demonstrates OmenDB at production scale with 10K+ vectors,
realistic patterns, and enterprise-grade functionality.

This example shows:
- Large-scale vector operations (10K+ vectors)
- Realistic embedding patterns and metadata
- Performance monitoring and metrics
- Error handling and resilience
- Memory usage optimization
- Batch operations for efficiency

Requirements:
- Python 3.8+
- OmenDB Python SDK
- At least 100MB RAM for 10K vectors
"""

import os
import time
import random
import json
from typing import List, Dict, Any, Optional
from dataclasses import dataclass
import math

# OmenDB Python SDK
try:
    import omendb

    NATIVE_AVAILABLE = True
    print("✅ Using native OmenDB implementation")
except ImportError:
    print("📝 Using development stub - compile native library for production")
    NATIVE_AVAILABLE = False

    # Production-oriented stub implementation
    class ProductionOmenDBStub:
        def __init__(self, path: str):
            self.path = path
            self.vectors = []
            self.metadata = []
            self.start_time = time.time()
            print(f"🗄️  Created production database: {path}")

        def insert(self, vector: List[float], metadata: Dict[str, Any] = None):
            self.vectors.append(vector)
            self.metadata.append(metadata or {})
            return len(self.vectors) - 1

        def insert_batch(
            self, vectors: List[List[float]], metadata_list: List[Dict[str, Any]]
        ):
            """Batch insert for better performance."""
            start_idx = len(self.vectors)
            self.vectors.extend(vectors)
            self.metadata.extend(metadata_list)
            return list(range(start_idx, start_idx + len(vectors)))

        def search(self, query: List[float], k: int = 10) -> List[Dict[str, Any]]:
            results = []
            for i, vec in enumerate(self.vectors):
                # Improved similarity calculation
                dot_product = sum(a * b for a, b in zip(query, vec))
                norm_q = math.sqrt(sum(x * x for x in query))
                norm_v = math.sqrt(sum(x * x for x in vec))

                if norm_q > 0 and norm_v > 0:
                    similarity = dot_product / (norm_q * norm_v)
                else:
                    similarity = 0.0

                results.append(
                    {
                        "id": i,
                        "similarity": similarity,
                        "metadata": self.metadata[i],
                        "vector": vec,
                    }
                )

            results.sort(key=lambda x: x["similarity"], reverse=True)
            return results[:k]

        def get_stats(self) -> Dict[str, Any]:
            """Get database statistics."""
            return {
                "total_vectors": len(self.vectors),
                "dimension": len(self.vectors[0]) if self.vectors else 0,
                "uptime_seconds": time.time() - self.start_time,
                "estimated_memory_mb": len(self.vectors)
                * (len(self.vectors[0]) * 4 + 200)
                / (1024 * 1024)
                if self.vectors
                else 0,
            }

        def close(self):
            print(f"💾 Closed production database: {self.path}")

    class omendb:
        DB = ProductionOmenDBStub


@dataclass
class PerformanceMetrics:
    """Track performance metrics during production operations."""

    operation: str
    count: int
    total_time: float
    memory_mb: float

    @property
    def throughput(self) -> float:
        return self.count / self.total_time if self.total_time > 0 else 0

    @property
    def avg_latency_ms(self) -> float:
        return (self.total_time * 1000) / self.count if self.count > 0 else 0


def generate_realistic_embedding(
    dimension: int, cluster_id: int, noise_level: float = 0.1
) -> List[float]:
    """Generate realistic embedding vector with clustering patterns."""
    # Create base pattern for the cluster
    base_pattern = []
    cluster_center = cluster_id * 0.3 - 0.5  # Spread clusters across [-0.5, 0.5]

    for i in range(dimension):
        # Frequency-based pattern (common in real embeddings)
        frequency = i / dimension * math.pi
        base_value = cluster_center + 0.2 * math.sin(frequency * 4)

        # Add controlled noise
        noise = random.uniform(-noise_level, noise_level)
        value = base_value + noise

        # Normalize to reasonable range
        value = max(-1.0, min(1.0, value))
        base_pattern.append(value)

    # Normalize to unit vector (common in embeddings)
    norm = math.sqrt(sum(x * x for x in base_pattern))
    if norm > 0:
        return [x / norm for x in base_pattern]
    else:
        return base_pattern


def generate_production_metadata(
    vector_id: int, model_version: str, batch_id: int
) -> Dict[str, Any]:
    """Generate realistic production metadata."""
    categories = ["technology", "science", "business", "health", "education"]
    sources = ["documents", "articles", "papers", "reports", "guides"]

    return {
        "vector_id": vector_id,
        "model_version": model_version,
        "batch_id": batch_id,
        "category": random.choice(categories),
        "source_type": random.choice(sources),
        "confidence_score": 0.7 + random.random() * 0.3,  # 0.7-1.0 range
        "created_at": time.time(),
        "processed_by": f"pipeline_{batch_id % 5}",
        "language": "en",
        "content_length": random.randint(100, 5000),
        "priority": random.choice(["low", "medium", "high"]),
    }


def benchmark_batch_insertion(
    db,
    vectors: List[List[float]],
    metadata_list: List[Dict[str, Any]],
    start_idx: int = 0,
) -> PerformanceMetrics:
    """Benchmark batch insertion performance."""
    print(f"   Inserting {len(vectors)} vectors...")

    start_time = time.time()

    if hasattr(db, "add_batch"):
        # Use batch insertion if available
        batch_ids = [f"vec_{start_idx + i}" for i in range(len(vectors))]
        ids = db.add_batch(vectors=vectors, ids=batch_ids, metadata=metadata_list)
    else:
        # Fall back to individual insertions
        ids = []
        for i, (vec, meta) in enumerate(zip(vectors, metadata_list)):
            success = db.add(f"vec_{start_idx + i}", vec, meta)
            ids.append(f"vec_{start_idx + i}" if success else None)

    end_time = time.time()
    total_time = end_time - start_time

    # Estimate memory usage
    stats = db.info() if hasattr(db, "get_stats") else {"estimated_memory_mb": 0}
    memory_mb = stats.get("estimated_memory_mb", 0)

    return PerformanceMetrics(
        operation="batch_insert",
        count=len(vectors),
        total_time=total_time,
        memory_mb=memory_mb,
    )


def benchmark_search_performance(
    db, query_vectors: List[List[float]], k: int = 10
) -> PerformanceMetrics:
    """Benchmark search performance with multiple queries."""
    print(f"   Running {len(query_vectors)} search queries (k={k})...")

    start_time = time.time()
    total_results = 0

    for query in query_vectors:
        results = db.search(query, k)
        total_results += len(results)

    end_time = time.time()
    total_time = end_time - start_time

    stats = db.info() if hasattr(db, "get_stats") else {"estimated_memory_mb": 0}
    memory_mb = stats.get("estimated_memory_mb", 0)

    return PerformanceMetrics(
        operation="search",
        count=len(query_vectors),
        total_time=total_time,
        memory_mb=memory_mb,
    )


def main():
    """Production scale demonstration with 10K+ vectors."""

    print("🏭 OmenDB Production Scale Example")
    print("=" * 60)
    print("Testing: 10K+ vectors with realistic patterns")
    print("Focus: Performance, memory usage, enterprise patterns")
    print()

    # Configuration
    PRODUCTION_SCALE = 10000  # Target scale
    EMBEDDING_DIMENSION = 512  # Realistic embedding dimension
    BATCH_SIZE = 1000  # Process in batches
    SEARCH_SAMPLES = 100  # Number of search queries to test

    # Step 1: Create production database
    print("📚 Step 1: Create Production Database")
    db_path = "production_vectors.omen"

    if os.path.exists(db_path):
        os.remove(db_path)

    db = omendb.DB(db_path)
    print(f"   Database: {db_path}")
    print(f"   Target scale: {PRODUCTION_SCALE:,} vectors")
    print(f"   Embedding dimension: {EMBEDDING_DIMENSION}")
    print(f"   Batch size: {BATCH_SIZE}")
    print()

    # Step 2: Generate and insert production data
    print("📥 Step 2: Production Data Insertion")

    num_batches = PRODUCTION_SCALE // BATCH_SIZE
    num_clusters = 20  # Realistic number of semantic clusters

    insertion_metrics = []

    print(f"   Generating {num_batches} batches of {BATCH_SIZE} vectors each...")

    overall_start = time.time()

    for batch_idx in range(num_batches):
        print(f"   Batch {batch_idx + 1}/{num_batches}...", end=" ")

        # Generate batch of vectors with realistic patterns
        batch_vectors = []
        batch_metadata = []

        for i in range(BATCH_SIZE):
            vector_id = batch_idx * BATCH_SIZE + i
            cluster_id = vector_id % num_clusters

            # Generate realistic embedding
            vector = generate_realistic_embedding(EMBEDDING_DIMENSION, cluster_id)

            # Generate production metadata
            model_version = f"v1.{(vector_id // 2000) + 1}"  # Version every 2K vectors
            metadata = generate_production_metadata(vector_id, model_version, batch_idx)

            batch_vectors.append(vector)
            batch_metadata.append(metadata)

        # Benchmark batch insertion
        metrics = benchmark_batch_insertion(
            db, batch_vectors, batch_metadata, batch_idx * BATCH_SIZE
        )
        insertion_metrics.append(metrics)

        print(f"✅ {metrics.throughput:.0f} vectors/sec, {metrics.memory_mb:.1f}MB")

    overall_insert_time = time.time() - overall_start

    print(f"   Total insertion time: {overall_insert_time:.2f} seconds")
    print(
        f"   Overall throughput: {PRODUCTION_SCALE / overall_insert_time:.0f} vectors/sec"
    )
    print()

    # Step 3: Production search benchmarking
    print("🔍 Step 3: Production Search Performance")

    # Generate diverse search queries
    search_queries = []
    for i in range(SEARCH_SAMPLES):
        # Mix of cluster-centered and random queries
        if i % 3 == 0:
            # Query near cluster centers
            cluster_id = i % num_clusters
            query = generate_realistic_embedding(
                EMBEDDING_DIMENSION, cluster_id, noise_level=0.05
            )
        else:
            # Random queries
            query = generate_realistic_embedding(
                EMBEDDING_DIMENSION, random.randint(0, num_clusters), noise_level=0.3
            )

        search_queries.append(query)

    # Benchmark different k values
    k_values = [1, 5, 10, 20]
    search_metrics = {}

    for k in k_values:
        print(f"   Testing k={k}...")
        metrics = benchmark_search_performance(db, search_queries, k)
        search_metrics[k] = metrics

        print(f"      Average latency: {metrics.avg_latency_ms:.2f}ms")
        print(f"      Throughput: {metrics.throughput:.0f} queries/sec")

    print()

    # Step 4: Memory and performance analysis
    print("📊 Step 4: Performance Analysis")

    if hasattr(db, "get_stats"):
        stats = db.info()
        print(f"   Total vectors: {stats['total_vectors']:,}")
        print(f"   Vector dimension: {stats['dimension']}")
        print(f"   Estimated memory: {stats['estimated_memory_mb']:.1f}MB")
        print(f"   Database uptime: {stats['uptime_seconds']:.1f}s")

    # Calculate performance summary
    avg_insert_throughput = sum(m.throughput for m in insertion_metrics) / len(
        insertion_metrics
    )
    avg_search_latency = sum(m.avg_latency_ms for m in search_metrics.values()) / len(
        search_metrics
    )

    print()
    print("   📈 Performance Summary:")
    print(f"      Average insert throughput: {avg_insert_throughput:.0f} vectors/sec")
    print(f"      Average search latency: {avg_search_latency:.2f}ms")

    # Get database stats for memory info
    stats = db.info()
    print(
        f"      Memory efficiency: {stats.get('estimated_memory_mb', 0) / (PRODUCTION_SCALE / 1000):.1f}MB per 1K vectors"
    )

    # Step 5: Production validation
    print()
    print("🎯 Step 5: Production Validation")

    # Validate against production targets
    targets = {
        "search_latency_ms": 2.0,  # <2ms target
        "memory_per_1k_vectors_mb": 5.0,  # <5MB per 1K vectors
        "insert_throughput": 500.0,  # >500 vectors/sec
    }

    actual_memory_per_1k = stats.get("estimated_memory_mb", 0) / (
        PRODUCTION_SCALE / 1000
    )

    validations = {
        "search_latency_ms": avg_search_latency <= targets["search_latency_ms"],
        "memory_per_1k_vectors_mb": actual_memory_per_1k
        <= targets["memory_per_1k_vectors_mb"],
        "insert_throughput": avg_insert_throughput >= targets["insert_throughput"],
    }

    print("   Production target validation:")
    for metric, passed in validations.items():
        status = "✅ PASS" if passed else "❌ FAIL"
        target_val = targets[metric]

        if metric == "search_latency_ms":
            actual_val = avg_search_latency
        elif metric == "memory_per_1k_vectors_mb":
            actual_val = actual_memory_per_1k
        else:  # insert_throughput
            actual_val = avg_insert_throughput

        print(f"      {metric}: {actual_val:.2f} (target: {target_val}) {status}")

    overall_validation = all(validations.values())
    print()
    if overall_validation:
        print("   🎉 PRODUCTION VALIDATION: PASSED")
        print("      All performance targets met for production deployment")
    else:
        print("   ⚠️  PRODUCTION VALIDATION: NEEDS IMPROVEMENT")
        print("      Some targets not met - review performance optimization")

    # Step 6: Cleanup and next steps
    print()
    print("🧹 Step 6: Cleanup and Next Steps")

    # OmenDB automatically saves on del, no explicit close needed
    del db

    if os.path.exists(db_path):
        file_size_mb = os.path.getsize(db_path) / (1024 * 1024)
        print(f"   Database file size: {file_size_mb:.1f}MB")

    print()
    print("🎯 Next Steps for Production:")
    print("   1. Monitor performance with real embedding models")
    print("   2. Test with your specific vector dimensions")
    print("   3. Validate with production query patterns")
    print("   4. Set up monitoring and alerting")
    print("   5. Consider horizontal scaling for larger datasets")
    print()

    if NATIVE_AVAILABLE:
        print("✅ Production scale testing completed with native implementation")
    else:
        print("📝 Production scale testing completed with stub implementation")
        print("   Compile native OmenDB library for actual production performance")


if __name__ == "__main__":
    main()
