#!/usr/bin/env python3
"""
Working OmenDB API Example
=========================

This example demonstrates the ACTUAL current API of OmenDB with real working code.
All method calls are verified to work with the production implementation.

No external dependencies required - uses the validated API patterns.
"""

import sys
import os
import time
import random
import math

# Add python directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python"))

try:
    from omendb import DB

    print("✅ OmenDB imported successfully")
except ImportError as e:
    print(f"❌ Could not import OmenDB: {e}")
    sys.exit(1)


def generate_embedding(text: str, dimension: int = 384) -> list[float]:
    """
    Generate a simple embedding for demonstration.
    In production, you would use sentence-transformers, OpenAI API, etc.
    """
    # Simple hash-based embedding for consistent results
    hash_val = hash(text)
    random.seed(hash_val)

    # Generate normalized vector
    vector = [random.gauss(0, 1) for _ in range(dimension)]
    magnitude = math.sqrt(sum(x * x for x in vector))
    return [x / magnitude for x in vector] if magnitude > 0 else vector


def main():
    """Demonstrate the current working OmenDB API."""

    print("🚀 Working OmenDB API Demonstration")
    print("=" * 40)

    # 1. Initialize database (no context manager - not implemented)
    print("\n1. Database Initialization")
    db = DB("working_example.omen")
    print("   ✅ Database created")

    # 2. Add individual vectors (ACTUAL working method)
    print("\n2. Adding Individual Vectors")

    documents = [
        {"text": "Vector databases enable semantic search", "category": "technology"},
        {"text": "Machine learning models need vector storage", "category": "ai"},
        {"text": "Embeddings capture semantic meaning", "category": "nlp"},
        {"text": "RAG systems combine retrieval and generation", "category": "ai"},
        {"text": "OmenDB provides fast similarity search", "category": "technology"},
    ]

    for i, doc in enumerate(documents):
        # Generate embedding
        embedding = generate_embedding(doc["text"], 128)  # 128D for performance

        # Add to database (ACTUAL API: add(id, vector, metadata))
        success = db.add(f"doc_{i}", embedding, doc)

        if success:
            print(f"   ✅ Added: {doc['text'][:40]}...")
        else:
            print(f"   ❌ Failed: {doc['text'][:40]}...")

    # 3. Batch operations (ACTUAL working method)
    print("\n3. Batch Operations")

    # Modern columnar batch format
    batch_vectors = [generate_embedding(f"batch document {i}", 128) for i in range(5)]
    batch_ids = [f"batch_{i}" for i in range(5)]
    batch_metadata = [{"type": "batch"} for i in range(5)]

    result_ids = db.add_batch(
        vectors=batch_vectors, ids=batch_ids, metadata=batch_metadata
    )
    successful_batch = len(result_ids)
    print(f"   ✅ Batch added: {successful_batch}/5 vectors")

    # 4. Similarity search (ACTUAL API: search(vector, limit, filter))
    print("\n4. Similarity Search")

    query_text = "semantic search technology"
    query_embedding = generate_embedding(query_text, 128)

    # Basic search
    search_results = db.search(query_embedding, limit=3)
    print(f"   Query: '{query_text}'")
    print(f"   Found {len(search_results)} results:")

    for i, result in enumerate(search_results, 1):
        print(f"     {i}. ID: {result.id}")
        print(f"        Score: {result.score:.3f}")
        if result.metadata:
            print(f"        Text: {result.metadata.get('text', 'N/A')[:50]}...")
            print(f"        Category: {result.metadata.get('category', 'N/A')}")

    # 5. Metadata filtering (ACTUAL working feature)
    print("\n5. Metadata Filtering")

    # Search only in 'technology' category
    filtered_results = db.search(
        query_embedding, limit=5, filter={"category": "technology"}
    )

    print(f"   Filtered by category='technology': {len(filtered_results)} results")
    for result in filtered_results:
        print(f"     - {result.id}: {result.metadata.get('text', 'N/A')[:40]}...")

    # 6. Database statistics (ACTUAL working method)
    print("\n6. Database Statistics")

    stats = db.info()
    print(f"   Vector count: {stats.get('vector_count', 0)}")
    print(f"   Dimensions: {stats.get('dimension', 0)}")
    print(f"   Algorithm: {stats.get('algorithm', 'Unknown')}")
    print(f"   Status: {stats.get('status', 'Unknown')}")
    print(f"   API version: {stats.get('api_version', 'Unknown')}")

    if "memory_usage_mb" in stats:
        print(f"   Memory usage: {stats['memory_usage_mb']:.2f} MB")

    # 7. Test algorithm switching (automatic at 5K vectors)
    print("\n7. Algorithm Information")

    current_algorithm = stats.get("algorithm", "unknown")
    vector_count = stats.get("vector_count", 0)

    print(f"   Current algorithm: {current_algorithm}")
    print(f"   Vector count: {vector_count}")
    print(f"   Crossover point: 5000 vectors")

    if vector_count < 5000:
        print("   Status: Using ultra-optimized brute force (< 5K vectors)")
    else:
        print("   Status: Using DiskANN algorithm (no rebuilds needed)")

    # 8. Performance timing
    print("\n8. Performance Test")

    # Time a query
    start_time = time.time()
    perf_results = db.search(query_embedding, limit=5)
    query_time = (time.time() - start_time) * 1000  # Convert to milliseconds

    print(f"   Query latency: {query_time:.3f}ms")
    print(f"   Results returned: {len(perf_results)}")

    # Time batch insertion
    perf_vectors = [generate_embedding(f"perf {i}", 128) for i in range(100)]
    perf_ids = [f"perf_{i}" for i in range(100)]

    start_time = time.time()
    batch_result_ids = db.add_batch(vectors=perf_vectors, ids=perf_ids)
    batch_time = time.time() - start_time

    successful_perf = len(batch_result_ids)
    batch_rate = successful_perf / batch_time if batch_time > 0 else 0

    print(f"   Batch insertion: {successful_perf} vectors in {batch_time:.3f}s")
    print(f"   Insertion rate: {batch_rate:.1f} vectors/second")

    # 9. Final status
    print("\n9. Final Status")

    final_stats = db.info()
    print(f"   Total vectors: {final_stats.get('vector_count', 0)}")
    print(f"   Final algorithm: {final_stats.get('algorithm', 'unknown')}")

    # Check if database file was created
    if os.path.exists("working_example.omen"):
        file_size = os.path.getsize("working_example.omen")
        print(f"   Database file: working_example.omen ({file_size:,} bytes)")

    print("\n✅ API Demonstration Complete!")
    print("\nThis example shows:")
    print("  • All current working API methods")
    print("  • Real performance characteristics")
    print("  • Metadata filtering capabilities")
    print("  • Algorithm switching behavior")
    print("  • Production-ready patterns")

    print(f"\n📊 Performance Summary:")
    print(f"  • Query latency: {query_time:.1f}ms")
    print(f"  • Insertion rate: {batch_rate:.0f} vectors/sec")
    print(f"  • Memory efficiency: {final_stats.get('memory_usage_mb', 0):.1f}MB")
    print(f"  • Algorithm: {final_stats.get('algorithm', 'unknown')}")


if __name__ == "__main__":
    main()
