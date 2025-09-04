#!/usr/bin/env python3
"""
Test improved RoarGraph accuracy at 500 vectors
"""

import sys
import random
import math

sys.path.insert(0, "/Users/nick/github/omenDB/python")


def generate_normalized_vectors(count: int, dimension: int = 3):
    """Generate normalized test vectors"""
    vectors = []
    random.seed(42)  # For reproducible results

    for i in range(count):
        # Generate random vector
        vector = [random.gauss(0, 1) for _ in range(dimension)]
        # Normalize to unit length
        norm = math.sqrt(sum(x * x for x in vector))
        if norm > 0:
            vector = [x / norm for x in vector]
        vectors.append((f"doc_{i:03d}", vector))
    return vectors


def test_500_vector_accuracy():
    """Test RoarGraph accuracy at 500 vectors with improvements"""
    print("🎯 Testing improved RoarGraph at 500 vectors")
    print("=" * 50)

    try:
        from omendb import DB

        db = DB()

        # Generate 500 test vectors
        test_vectors = generate_normalized_vectors(500)
        print(f"Generated {len(test_vectors)} test vectors")

        # Add all vectors
        print("Adding vectors...")
        for i, (doc_id, vector) in enumerate(test_vectors):
            success = db.add(doc_id, vector)
            if not success:
                print(f"❌ Failed to add {doc_id}")
                return False
            if (i + 1) % 100 == 0:
                print(f"  Added {i + 1}/500 vectors")

        print("Testing accuracy with first 5 vectors as queries...")
        exact_matches = 0
        total_queries = 5

        for i in range(total_queries):
            query_vector = test_vectors[i][1]
            expected_id = test_vectors[i][0]

            print(f"\nQuery {i + 1}: searching for {expected_id}")
            results = db.search(query_vector, limit=3)

            if len(results) > 0:
                top_result = results[0]
                print(
                    f"  Top result: {top_result.id} (similarity: {top_result.score:.6f})"
                )

                if top_result.id == expected_id:
                    exact_matches += 1
                    print(f"  ✅ EXACT MATCH found!")
                else:
                    print(f"  ❌ Expected {expected_id}, got {top_result.id}")

                # Show all results for debugging
                print(f"  All results:")
                for j, result in enumerate(results):
                    print(f"    {j + 1}. {result.id}: {result.score:.6f}")
            else:
                print(f"  ❌ No results returned")

        accuracy = exact_matches / total_queries * 100
        print(f"\n📊 Final Accuracy: {exact_matches}/{total_queries} = {accuracy:.1f}%")

        if accuracy >= 80:
            print("✅ SUCCESS: High accuracy achieved at 500 vectors!")
            return True
        else:
            print("❌ FAILED: Accuracy still too low")
            return False

    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback

        traceback.print_exc()
        return False


if __name__ == "__main__":
    success = test_500_vector_accuracy()
    print(f"\n🏁 Test {'PASSED' if success else 'FAILED'}")
    exit(0 if success else 1)
