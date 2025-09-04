#!/usr/bin/env python3
"""
Mixed Precision API Test and Demonstration

Tests the new MixedPrecisionDB API and demonstrates practical usage
for memory optimization in production scenarios.
"""

import time
import numpy as np
import sys
from pathlib import Path

# Add python path for testing
python_path = Path(__file__).parent / "python"
if str(python_path) not in sys.path:
    sys.path.insert(0, str(python_path))

try:
    from omendb.mixed_precision_api import (
        MixedPrecisionDB,
        PrecisionType,
        PrecisionConfig,
        create_mixed_precision_db,
        analyze_vectors_for_precision,
    )

    MIXED_PRECISION_AVAILABLE = True
except ImportError as e:
    print(f"⚠️  Mixed precision API not available: {e}")
    MIXED_PRECISION_AVAILABLE = False


def generate_realistic_embeddings(num_vectors, dimension=384):
    """Generate realistic embedding-like vectors."""
    vectors = []

    for i in range(num_vectors):
        # Generate normalized embeddings similar to Sentence-BERT
        vector = np.random.normal(0, 0.3, dimension)

        # Normalize to unit length
        norm = np.linalg.norm(vector)
        if norm > 0:
            vector = vector / norm

        vectors.append(vector.tolist())

    return vectors


def generate_quantization_friendly_vectors(num_vectors, dimension=128):
    """Generate vectors that are well-suited for int8 quantization."""
    vectors = []

    for i in range(num_vectors):
        # Generate vectors with values in [-1, 1] range
        vector = np.random.uniform(-0.8, 0.8, dimension)

        # Add some structure to make it more realistic
        vector = np.tanh(vector)  # Squash to (-1, 1)

        vectors.append(vector.tolist())

    return vectors


def test_mixed_precision_api():
    """Test the mixed precision API functionality."""
    print("🧪 Testing Mixed Precision API")
    print("-" * 40)

    if not MIXED_PRECISION_AVAILABLE:
        print("❌ Mixed precision API not available")
        return False

    # Test 1: Create mixed precision database
    print("🔧 Test 1: Creating mixed precision database...")
    try:
        db = create_mixed_precision_db(
            precision_type=PrecisionType.AUTO,
            accuracy_threshold=0.95,
            memory_priority=True,
        )
        print("✅ Mixed precision database created successfully")
    except Exception as e:
        print(f"❌ Failed to create database: {e}")
        return False

    # Test 2: Analyze vectors for precision
    print("\n🔍 Test 2: Analyzing vectors for optimal precision...")
    test_vectors = generate_realistic_embeddings(100, 384)

    try:
        analysis = analyze_vectors_for_precision(test_vectors)
        print(f"✅ Analysis complete:")
        print(f"   Recommended precision: {analysis['recommended_precision']}")
        print(f"   Estimated accuracy: {analysis['estimated_accuracy']:.3f}")
        print(f"   Memory savings: {analysis['memory_savings_percent']:.1f}%")
    except Exception as e:
        print(f"❌ Analysis failed: {e}")
        return False

    # Test 3: Add vectors with batch optimization
    print("\n📦 Test 3: Adding vectors with batch optimization...")
    try:
        batch_vectors = [
            (f"doc_{i}", vector, {"type": "embedding"})
            for i, vector in enumerate(test_vectors[:50])
        ]

        start_time = time.time()
        db.add_batch(batch_vectors)
        batch_time = time.time() - start_time

        print(f"✅ Batch added successfully in {batch_time:.3f}s")
    except Exception as e:
        print(f"❌ Batch addition failed: {e}")
        return False

    # Test 4: Query vectors
    print("\n🔍 Test 4: Querying vectors...")
    try:
        query_vector = test_vectors[0]
        results = db.search(query_vector, limit=5)
        print(f"✅ Query successful, found {len(results)} results")
        if results:
            print(f"   Top result similarity: {results[0].score:.4f}")
    except Exception as e:
        print(f"❌ Query failed: {e}")
        return False

    # Test 5: Get memory statistics
    print("\n💾 Test 5: Memory statistics...")
    try:
        stats = db.get_memory_stats()
        print(f"✅ Memory stats:")
        print(f"   Total vectors: {stats.total_vectors}")
        print(f"   Float32 memory: {stats.float32_memory_mb:.2f} MB")
        print(f"   Optimized memory: {stats.optimized_memory_mb:.2f} MB")
        print(f"   Memory savings: {stats.memory_savings_percent:.1f}%")
    except Exception as e:
        print(f"❌ Memory stats failed: {e}")
        return False

    # Test 6: Optimization report
    print("\n📊 Test 6: Optimization report...")
    try:
        db.print_optimization_report()
        print("✅ Optimization report generated")
    except Exception as e:
        print(f"❌ Optimization report failed: {e}")
        return False

    print("\n🎉 All mixed precision API tests passed!")
    return True


def benchmark_mixed_precision():
    """Benchmark mixed precision against standard precision."""
    print("\n⚡ Mixed Precision Benchmark")
    print("-" * 40)

    if not MIXED_PRECISION_AVAILABLE:
        print("❌ Mixed precision API not available")
        return

    # Test different data types
    test_scenarios = [
        {
            "name": "Realistic Embeddings (384D)",
            "generator": lambda n: generate_realistic_embeddings(n, 384),
            "count": 500,
        },
        {
            "name": "Quantization-Friendly Vectors (128D)",
            "generator": lambda n: generate_quantization_friendly_vectors(n, 128),
            "count": 1000,
        },
    ]

    for scenario in test_scenarios:
        print(f"\n📊 Benchmarking: {scenario['name']}")
        print("-" * 30)

        # Generate test data
        vectors = scenario["generator"](scenario["count"])
        print(f"Generated {len(vectors)} test vectors")

        # Analyze optimal precision
        analysis = analyze_vectors_for_precision(vectors)
        print(f"Recommended precision: {analysis['recommended_precision']}")
        print(f"Potential memory savings: {analysis['memory_savings_percent']:.1f}%")

        # Test with mixed precision database
        try:
            db = create_mixed_precision_db(precision_type=PrecisionType.AUTO)

            # Prepare batch data
            batch_data = [
                (f"vec_{i}", vector, {"index": i}) for i, vector in enumerate(vectors)
            ]

            # Benchmark batch addition
            start_time = time.time()
            db.add_batch(batch_data)
            batch_time = time.time() - start_time

            # Benchmark queries
            query_times = []
            for i in range(10):
                query_vector = vectors[i % len(vectors)]
                start_time = time.time()
                results = db.search(query_vector, limit=10)
                query_time = (time.time() - start_time) * 1000  # ms
                query_times.append(query_time)

            avg_query_time = sum(query_times) / len(query_times)

            # Get final stats
            stats = db.get_memory_stats()

            # Report results
            print(f"⚡ Performance Results:")
            print(
                f"   Batch addition: {batch_time:.3f}s ({len(vectors) / batch_time:.0f} vec/s)"
            )
            print(f"   Average query time: {avg_query_time:.3f}ms")
            print(f"   Memory usage: {stats.optimized_memory_mb:.2f} MB")
            print(f"   Memory savings: {stats.memory_savings_percent:.1f}%")

        except Exception as e:
            print(f"❌ Benchmark failed: {e}")


def demonstrate_production_usage():
    """Demonstrate practical production usage scenarios."""
    print("\n🚀 Production Usage Demonstration")
    print("-" * 40)

    if not MIXED_PRECISION_AVAILABLE:
        print("❌ Mixed precision API not available")
        return

    # Scenario 1: Large-scale embeddings with automatic optimization
    print("📈 Scenario 1: Large-scale embedding storage")

    # Simulate realistic embeddings (like OpenAI or Sentence-BERT)
    print("Generating 2000 realistic embeddings...")
    embeddings = generate_realistic_embeddings(2000, 384)

    # Create mixed precision database with automatic optimization
    db = create_mixed_precision_db(
        precision_type=PrecisionType.AUTO,
        accuracy_threshold=0.98,  # High accuracy requirement
        memory_priority=True,
    )

    # Add embeddings in batches (realistic production pattern)
    batch_size = 100
    total_time = 0

    for i in range(0, len(embeddings), batch_size):
        batch = embeddings[i : i + batch_size]
        batch_data = [
            (f"doc_{j}", vector, {"batch": i // batch_size, "doc_id": j})
            for j, vector in enumerate(batch)
        ]

        start_time = time.time()
        db.add_batch(batch_data)
        batch_time = time.time() - start_time
        total_time += batch_time

        if i == 0:  # Print details for first batch
            print(f"First batch processed in {batch_time:.3f}s")

    print(
        f"All embeddings processed in {total_time:.3f}s ({len(embeddings) / total_time:.0f} vec/s)"
    )

    # Query performance test
    print("Testing query performance...")
    query_times = []
    for i in range(20):
        query_vector = embeddings[i]
        start_time = time.time()
        results = db.search(query_vector, limit=10)
        query_time = (time.time() - start_time) * 1000
        query_times.append(query_time)

    avg_query_time = sum(query_times) / len(query_times)
    print(f"Average query time: {avg_query_time:.3f}ms")

    # Show final optimization report
    db.print_optimization_report()

    # Scenario 2: Memory-constrained environment
    print(f"\n💾 Scenario 2: Memory-constrained deployment")

    # Force aggressive quantization for memory savings
    memory_db = create_mixed_precision_db(
        precision_type=PrecisionType.INT8,  # Force int8 for maximum savings
        accuracy_threshold=0.90,  # Lower accuracy threshold
        memory_priority=True,
    )

    # Use quantization-friendly vectors
    quant_vectors = generate_quantization_friendly_vectors(1000, 256)
    batch_data = [
        (f"item_{i}", vector, {"category": f"cat_{i % 5}"})
        for i, vector in enumerate(quant_vectors)
    ]

    memory_db.add_batch(batch_data)

    memory_stats = memory_db.get_memory_stats()
    print(f"Memory-optimized deployment:")
    print(f"   Vectors stored: {memory_stats.total_vectors}")
    print(f"   Memory usage: {memory_stats.optimized_memory_mb:.2f} MB")
    print(f"   Memory savings: {memory_stats.memory_savings_percent:.1f}%")

    # Test query accuracy
    query_vector = quant_vectors[0]
    results = memory_db.search(query_vector, limit=5)
    print(f"   Query results: {len(results)} found")
    if results:
        print(f"   Top result similarity: {results[0].score:.4f}")


def main():
    """Main mixed precision API test function."""
    print("🔬 OmenDB Mixed Precision API Test Suite")
    print("Advanced memory optimization for vector databases")
    print("=" * 55)

    # Run API tests
    api_success = test_mixed_precision_api()

    if api_success:
        # Run benchmarks
        benchmark_mixed_precision()

        # Demonstrate production usage
        demonstrate_production_usage()

        print("\n🎉 Mixed Precision API Testing Complete!")
        print("Ready for production deployment with memory optimization.")
    else:
        print("\n❌ Mixed precision API tests failed")
        print("Check the errors above for debugging information.")

    return api_success


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
