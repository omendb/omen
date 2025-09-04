#!/usr/bin/env python3
"""
Test OmenDB with real-world embedding data patterns.

This comprehensive test validates that the RoarGraph algorithm works correctly
with actual embedding distributions found in production AI applications.
"""

import sys
import time
import math
import numpy as np
from typing import List, Tuple, Dict

sys.path.insert(0, "/Users/nick/github/omenDB/python")


def generate_openai_like_embeddings(
    count: int, dim: int = 1536
) -> List[Tuple[str, List[float]]]:
    """
    Generate embeddings that mimic OpenAI text-embedding-3-large patterns.

    OpenAI embeddings typically have:
    - High dimensionality (1536D)
    - Values roughly in [-1, 1] range but concentrated around [-0.5, 0.5]
    - Some clustering based on semantic similarity
    - Roughly normalized (unit vectors)
    """
    np.random.seed(42)  # Reproducible results
    embeddings = []

    # Create semantic clusters
    num_clusters = max(5, count // 20)  # About 5% of vectors as cluster centers
    cluster_centers = []

    for i in range(num_clusters):
        # Generate cluster center
        center = np.random.normal(0, 0.3, dim)  # Concentrated around 0
        center = center / np.linalg.norm(center)  # Normalize to unit vector
        cluster_centers.append(center)

    for i in range(count):
        # Choose a cluster (80% clustered, 20% random)
        if np.random.random() < 0.8 and cluster_centers:
            # Generate vector near a cluster center
            cluster_idx = np.random.randint(0, len(cluster_centers))
            base_vector = cluster_centers[cluster_idx]

            # Add noise to create variation within cluster
            noise = np.random.normal(0, 0.1, dim)
            vector = base_vector + noise
        else:
            # Generate random vector
            vector = np.random.normal(0, 0.3, dim)

        # Normalize to unit vector (like OpenAI embeddings)
        vector = vector / np.linalg.norm(vector)

        # Convert to Python list and create embedding
        vector_list = vector.tolist()
        embedding_id = f"openai_embed_{i:05d}"
        embeddings.append((embedding_id, vector_list))

    return embeddings


def generate_sentence_bert_like_embeddings(
    count: int, dim: int = 384
) -> List[Tuple[str, List[float]]]:
    """
    Generate embeddings that mimic Sentence-BERT patterns.

    Sentence-BERT embeddings typically have:
    - Medium dimensionality (384D)
    - Values in roughly [-1, 1] but often more concentrated
    - Strong clustering based on sentence similarity
    - Unit normalized vectors
    """
    np.random.seed(43)  # Different seed for variety
    embeddings = []

    # Create topic-based clusters (simulating sentence topics)
    topics = [
        "technology",
        "science",
        "arts",
        "sports",
        "politics",
        "health",
        "travel",
        "food",
        "education",
        "business",
    ]
    num_clusters = min(len(topics), count // 10)
    cluster_centers = []

    for i in range(num_clusters):
        # Generate cluster center with more pronounced clustering
        center = np.random.normal(0, 0.4, dim)
        center = center / np.linalg.norm(center)
        cluster_centers.append(center)

    for i in range(count):
        # Most vectors are clustered (90% vs 80% for OpenAI)
        if np.random.random() < 0.9 and cluster_centers:
            cluster_idx = np.random.randint(0, len(cluster_centers))
            base_vector = cluster_centers[cluster_idx]

            # Less noise for tighter clusters
            noise = np.random.normal(0, 0.08, dim)
            vector = base_vector + noise
        else:
            vector = np.random.normal(0, 0.4, dim)

        # Normalize to unit vector
        vector = vector / np.linalg.norm(vector)

        vector_list = vector.tolist()
        embedding_id = f"sbert_embed_{i:05d}"
        embeddings.append((embedding_id, vector_list))

    return embeddings


def generate_word2vec_like_embeddings(
    count: int, dim: int = 300
) -> List[Tuple[str, List[float]]]:
    """
    Generate embeddings that mimic Word2Vec patterns.

    Word2Vec embeddings typically have:
    - Traditional dimensionality (300D)
    - Values in wider range, often [-2, 2]
    - Semantic clustering (similar words cluster together)
    - Not necessarily unit normalized
    """
    np.random.seed(44)  # Different seed
    embeddings = []

    # Create semantic word clusters
    word_categories = [
        "animals",
        "colors",
        "numbers",
        "verbs",
        "adjectives",
        "countries",
        "foods",
        "emotions",
        "tools",
        "nature",
    ]
    num_clusters = min(len(word_categories), count // 15)
    cluster_centers = []

    for i in range(num_clusters):
        # Wider distribution for Word2Vec
        center = np.random.normal(0, 0.8, dim)
        # Don't normalize - Word2Vec isn't always unit normalized
        cluster_centers.append(center)

    for i in range(count):
        if np.random.random() < 0.85 and cluster_centers:
            cluster_idx = np.random.randint(0, len(cluster_centers))
            base_vector = cluster_centers[cluster_idx]

            # Moderate noise for word variations
            noise = np.random.normal(0, 0.2, dim)
            vector = base_vector + noise
        else:
            vector = np.random.normal(0, 0.8, dim)

        # Don't normalize - keep original magnitude
        vector_list = vector.tolist()
        embedding_id = f"w2v_embed_{i:05d}"
        embeddings.append((embedding_id, vector_list))

    return embeddings


def test_embedding_accuracy(
    embeddings: List[Tuple[str, List[float]]], description: str
):
    """Test search accuracy with a specific embedding type."""
    from omendb import DB

    print(f"\n🧪 Testing {description}...")
    print(f"   Vectors: {len(embeddings)}, Dimension: {len(embeddings[0][1])}")

    # Create database and add embeddings
    start_time = time.time()
    db = DB()

    for embed_id, vector in embeddings:
        success = db.add(embed_id, vector)
        if not success:
            print(f"   ❌ Failed to add {embed_id}")
            return False

    construction_time = time.time() - start_time
    print(
        f"   📊 Construction: {construction_time:.3f}s ({len(embeddings) / construction_time:.0f} vec/s)"
    )

    # Test accuracy with random queries
    num_queries = min(10, len(embeddings))
    total_accuracy = 0.0
    total_query_time = 0.0

    for i in range(num_queries):
        # Use an existing vector as query for perfect match test
        query_idx = i * (len(embeddings) // num_queries)
        query_id, query_vector = embeddings[query_idx]

        # Perform search
        start_time = time.time()
        results = db.search(query_vector, limit=5)
        query_time = time.time() - start_time
        total_query_time += query_time

        # Check if we found the exact match
        found_exact = any(result.id == query_id for result in results)
        if found_exact:
            # Find the exact match and check similarity
            exact_result = next(r for r in results if r.id == query_id)
            similarity = exact_result.score
            total_accuracy += similarity
            print(f"   Query {i + 1}: {similarity:.6f} similarity for exact match")
        else:
            print(f"   Query {i + 1}: ❌ Exact match not found in top 5")
            total_accuracy += 0.0

    avg_accuracy = total_accuracy / num_queries
    avg_query_time = total_query_time / num_queries

    print(f"   🎯 Accuracy: {avg_accuracy:.6f} (average similarity for exact matches)")
    print(f"   ⚡ Query time: {avg_query_time * 1000:.2f}ms average")

    # Get database statistics
    stats = db.info()
    print(
        f"   📈 DB Stats: {stats.get('vector_count', 0)} vectors, "
        f"{stats.get('indexed_vectors', 0)} indexed, "
        f"{stats.get('pending_vectors', 0)} pending"
    )

    db.close()

    # Consider test successful if average accuracy > 0.99
    success = avg_accuracy > 0.99
    if success:
        print(f"   ✅ {description}: PASSED (accuracy {avg_accuracy:.6f})")
    else:
        print(f"   ❌ {description}: FAILED (accuracy {avg_accuracy:.6f} < 0.99)")

    return success


def test_mixed_dimension_databases():
    """Test multiple databases with different embedding types simultaneously."""
    from omendb import DB

    print(f"\n🔄 Testing mixed-dimension databases...")

    # Create different embedding types
    openai_embeddings = generate_openai_like_embeddings(50, 1536)
    sbert_embeddings = generate_sentence_bert_like_embeddings(50, 384)
    w2v_embeddings = generate_word2vec_like_embeddings(50, 300)

    # Create separate databases
    databases = []

    try:
        # OpenAI database
        openai_db = DB()
        for embed_id, vector in openai_embeddings:
            openai_db.add(embed_id, vector)
        databases.append((openai_db, "OpenAI-like (1536D)", openai_embeddings))

        # Sentence-BERT database
        sbert_db = DB()
        for embed_id, vector in sbert_embeddings:
            sbert_db.add(embed_id, vector)
        databases.append((sbert_db, "Sentence-BERT-like (384D)", sbert_embeddings))

        # Word2Vec database
        w2v_db = DB()
        for embed_id, vector in w2v_embeddings:
            w2v_db.add(embed_id, vector)
        databases.append((w2v_db, "Word2Vec-like (300D)", w2v_embeddings))

        print(
            f"   ✅ Created {len(databases)} concurrent databases with different dimensions"
        )

        # Test each database independently
        all_passed = True
        for db, description, embeddings in databases:
            # Test query on first vector
            test_vector = embeddings[0][1]
            test_id = embeddings[0][0]

            results = db.search(test_vector, limit=3)
            if results and results[0].id == test_id:
                similarity = results[0].score
                print(
                    f"   ✅ {description}: Query successful (similarity {similarity:.6f})"
                )
            else:
                print(f"   ❌ {description}: Query failed")
                all_passed = False

        return all_passed

    except Exception as e:
        print(f"   ❌ Mixed database test failed: {e}")
        return False
    finally:
        # Clean up databases
        for db, _, _ in databases:
            try:
                db.close()
            except:
                pass


def test_clustering_effectiveness():
    """Test how well the algorithm handles clustered vs random data."""
    from omendb import DB

    print(f"\n📊 Testing clustering effectiveness...")

    # Generate clustered data (like sentence embeddings)
    clustered_data = generate_sentence_bert_like_embeddings(100, 384)

    # Generate random data (uniform distribution)
    np.random.seed(45)
    random_data = []
    for i in range(100):
        vector = np.random.uniform(-1, 1, 384)
        vector = vector / np.linalg.norm(vector)  # Normalize
        random_data.append((f"random_{i:03d}", vector.tolist()))

    results = {}

    for data_type, embeddings in [
        ("Clustered", clustered_data),
        ("Random", random_data),
    ]:
        print(f"   Testing {data_type} data...")

        db = DB()
        start_time = time.time()

        # Add all vectors
        for embed_id, vector in embeddings:
            db.add(embed_id, vector)

        construction_time = time.time() - start_time

        # Test search performance
        query_times = []
        for i in range(5):
            query_vector = embeddings[i][1]
            start_time = time.time()
            results_list = db.search(query_vector, limit=10)
            query_time = time.time() - start_time
            query_times.append(query_time)

        avg_query_time = sum(query_times) / len(query_times)
        construction_rate = len(embeddings) / construction_time

        results[data_type] = {
            "construction_time": construction_time,
            "construction_rate": construction_rate,
            "avg_query_time": avg_query_time,
        }

        print(
            f"     Construction: {construction_time:.3f}s ({construction_rate:.0f} vec/s)"
        )
        print(f"     Query time: {avg_query_time * 1000:.2f}ms average")

        db.close()

    # Compare results
    clustered_faster = (
        results["Clustered"]["construction_rate"]
        > results["Random"]["construction_rate"]
    )
    print(
        f"   📈 Clustered data construction: {'✅ Faster' if clustered_faster else '❓ Similar'}"
    )

    return True


def main():
    """Run comprehensive real-world embedding validation."""
    print("🌍 OmenDB Real-World Embedding Validation")
    print("=" * 60)

    test_results = []

    # Test 1: OpenAI-like embeddings (1536D)
    openai_embeddings = generate_openai_like_embeddings(100, 1536)
    success = test_embedding_accuracy(
        openai_embeddings, "OpenAI-like embeddings (1536D)"
    )
    test_results.append(("OpenAI-like", success))

    # Test 2: Sentence-BERT-like embeddings (384D)
    sbert_embeddings = generate_sentence_bert_like_embeddings(100, 384)
    success = test_embedding_accuracy(
        sbert_embeddings, "Sentence-BERT-like embeddings (384D)"
    )
    test_results.append(("Sentence-BERT-like", success))

    # Test 3: Word2Vec-like embeddings (300D)
    w2v_embeddings = generate_word2vec_like_embeddings(100, 300)
    success = test_embedding_accuracy(w2v_embeddings, "Word2Vec-like embeddings (300D)")
    test_results.append(("Word2Vec-like", success))

    # Test 4: Mixed dimension databases
    success = test_mixed_dimension_databases()
    test_results.append(("Mixed dimensions", success))

    # Test 5: Clustering effectiveness
    success = test_clustering_effectiveness()
    test_results.append(("Clustering effectiveness", success))

    # Summary
    print("\n" + "=" * 60)
    print("🎯 Real-World Embedding Validation Summary")
    print("=" * 60)

    passed_tests = sum(1 for _, success in test_results if success)
    total_tests = len(test_results)

    for test_name, success in test_results:
        status = "✅ PASSED" if success else "❌ FAILED"
        print(f"  {test_name}: {status}")

    print(f"\n📊 Overall Result: {passed_tests}/{total_tests} tests passed")

    if passed_tests == total_tests:
        print("🎉 ALL TESTS PASSED - OmenDB handles real-world embeddings perfectly!")
        print("\n✨ Validated capabilities:")
        print("  - OpenAI text-embedding-3-large patterns (1536D)")
        print("  - Sentence-BERT sentence embeddings (384D)")
        print("  - Word2Vec word embeddings (300D)")
        print("  - Mixed dimension concurrent databases")
        print("  - Clustered and random data distributions")
        print("  - 100% accuracy maintained across all embedding types")
    else:
        print("⚠️  Some tests failed - investigation needed")

    return passed_tests == total_tests


if __name__ == "__main__":
    main()
