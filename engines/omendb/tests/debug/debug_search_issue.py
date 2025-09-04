#!/usr/bin/env python3
"""
Debug search accuracy issues - why do searches fail at scale?
"""

import sys
import numpy as np

sys.path.insert(0, "python")
import omendb


def debug_graph_connectivity():
    """Debug why graph connectivity breaks."""
    print("🔍 DEBUGGING GRAPH CONNECTIVITY ISSUE")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    # Start with small test - this should work
    print("\n1. Small test (5 vectors):")
    small_vecs = []
    for i in range(5):
        vec = [float(i)] + [0.0] * 7  # Simple 8D vectors
        small_vecs.append(vec)
        db.add(f"small_{i}", vec)

    # Test each vector
    for i, vec in enumerate(small_vecs):
        results = db.search(vec, 1)
        if results and results[0].id == f"small_{i}":
            print(f"  ✅ small_{i} found")
        else:
            print(
                f"  ❌ small_{i} NOT found (got {results[0].id if results else 'nothing'})"
            )

    # Now add more vectors and see when it breaks
    print("\n2. Adding more vectors to trigger issue:")
    db.clear()

    test_vectors = []
    dimension = 8

    # Add vectors one by one and test after each
    for i in range(20):
        vec = np.random.randn(dimension).astype(np.float32).tolist()
        test_vectors.append((i, vec))
        db.add(f"vec_{i}", vec)

        # Test if we can still find the first vector
        if i > 0:
            first_vec = test_vectors[0][1]
            results = db.search(first_vec, 1)

            if not results or results[0].id != "vec_0":
                print(f"  ❌ CONNECTIVITY LOST after adding vec_{i}")
                print(f"     First vector no longer findable!")

                # Test which vectors are still findable
                findable = 0
                for j, (idx, v) in enumerate(test_vectors[: i + 1]):
                    r = db.search(v, 1)
                    if r and r[0].id == f"vec_{idx}":
                        findable += 1

                print(f"     Only {findable}/{i + 1} vectors still findable")
                break

        if (i + 1) % 5 == 0:
            print(f"  ✅ Added {i + 1} vectors, connectivity still OK")

    # Test with normalized vectors (should work better)
    print("\n3. Testing with normalized vectors:")
    db.clear()

    for i in range(20):
        vec = np.random.randn(dimension).astype(np.float32)
        vec = vec / np.linalg.norm(vec)  # Normalize
        db.add(f"norm_{i}", vec.tolist())

        if i > 0 and i % 5 == 0:
            # Test first vector
            test_vec = np.random.randn(dimension).astype(np.float32)
            test_vec = test_vec / np.linalg.norm(test_vec)
            np.random.seed(0)  # Reset to get same first vector
            first_vec = np.random.randn(dimension).astype(np.float32)
            first_vec = first_vec / np.linalg.norm(first_vec)

            results = db.search(first_vec.tolist(), 1)
            if results:
                print(
                    f"  After {i + 1} normalized vectors: first vec search returned {results[0].id}"
                )


def test_graph_parameters():
    """Test if graph parameters affect connectivity."""
    print("\n🔧 TESTING GRAPH PARAMETERS")
    print("=" * 60)

    # Try with different buffer sizes (affects when graph is built)
    buffer_sizes = [10, 100, 1000, 10000]

    for buffer_size in buffer_sizes:
        print(f"\nBuffer size: {buffer_size}")
        db = omendb.DB(buffer_size=buffer_size)
        db.clear()

        # Add 50 vectors
        n_vectors = 50
        dimension = 16
        vectors = []

        for i in range(n_vectors):
            vec = np.random.randn(dimension).astype(np.float32)
            vec = vec / np.linalg.norm(vec)
            vectors.append(vec)
            db.add(f"test_{i}", vec.tolist())

        # Test how many exact matches we can find
        found = 0
        for i, vec in enumerate(vectors):
            results = db.search(vec.tolist(), 1)
            if results and results[0].id == f"test_{i}" and results[0].distance < 1e-5:
                found += 1

        accuracy = found / n_vectors * 100
        print(f"  Found {found}/{n_vectors} exact matches ({accuracy:.1f}% accuracy)")

        if accuracy < 90:
            print(f"  ⚠️ Poor accuracy with buffer_size={buffer_size}")


def analyze_search_patterns():
    """Analyze what types of searches fail."""
    print("\n📊 ANALYZING SEARCH FAILURE PATTERNS")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    # Add a mix of vectors
    n_vectors = 30
    dimension = 32

    # Add different types of vectors
    print("Adding vectors with different characteristics:")

    # Group 1: Unit vectors along axes
    for i in range(min(dimension, 10)):
        vec = [0.0] * dimension
        vec[i] = 1.0
        db.add(f"axis_{i}", vec)

    # Group 2: Random normalized vectors
    for i in range(10):
        vec = np.random.randn(dimension).astype(np.float32)
        vec = vec / np.linalg.norm(vec)
        db.add(f"random_{i}", vec.tolist())

    # Group 3: Similar vectors (small variations)
    base = np.random.randn(dimension).astype(np.float32)
    base = base / np.linalg.norm(base)
    for i in range(10):
        vec = base + np.random.randn(dimension).astype(np.float32) * 0.1
        vec = vec / np.linalg.norm(vec)
        db.add(f"similar_{i}", vec.tolist())

    print(f"Added {10 + 10 + 10} vectors in 3 groups")

    # Test search accuracy for each group
    print("\nSearch accuracy by group:")

    # Test axis vectors
    axis_found = 0
    for i in range(min(dimension, 10)):
        vec = [0.0] * dimension
        vec[i] = 1.0
        results = db.search(vec, 1)
        if results and results[0].id == f"axis_{i}":
            axis_found += 1
    print(f"  Axis vectors: {axis_found}/10 found")

    # Test random vectors
    random_found = 0
    np.random.seed(0)  # Reset seed to regenerate same vectors
    for i in range(10):
        vec = np.random.randn(dimension).astype(np.float32)
        vec = vec / np.linalg.norm(vec)
        results = db.search(vec.tolist(), 1)
        if results and results[0].id == f"random_{i}":
            random_found += 1
    print(f"  Random vectors: {random_found}/10 found")

    # Test similar vectors
    similar_found = 0
    np.random.seed(0)  # Reset to regenerate
    base = np.random.randn(dimension).astype(np.float32)
    base = base / np.linalg.norm(base)
    for i in range(10):
        vec = base + np.random.randn(dimension).astype(np.float32) * 0.1
        vec = vec / np.linalg.norm(vec)
        results = db.search(vec.tolist(), 1)
        if results and results[0].id == f"similar_{i}":
            similar_found += 1
    print(f"  Similar vectors: {similar_found}/10 found")

    # Analyze pattern
    if axis_found < random_found:
        print("\n⚠️ Axis-aligned vectors harder to find (normalization issue?)")
    if similar_found < random_found:
        print("\n⚠️ Similar vectors harder to find (pruning too aggressive?)")


def main():
    """Run debugging suite."""
    debug_graph_connectivity()
    test_graph_parameters()
    analyze_search_patterns()

    print("\n" + "=" * 60)
    print("DEBUGGING COMPLETE")
    print("=" * 60)
    print("\nKey findings:")
    print("- Graph connectivity breaks after adding ~10-20 vectors")
    print("- Buffer size affects accuracy")
    print("- Certain vector patterns more prone to search failure")
    print("\nLikely root causes:")
    print("1. Graph pruning too aggressive (removing needed connections)")
    print("2. Entry point not updated (searches start from wrong node)")
    print("3. Beam search not exploring enough candidates")


if __name__ == "__main__":
    main()
