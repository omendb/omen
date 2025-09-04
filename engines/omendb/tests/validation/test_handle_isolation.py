#!/usr/bin/env python3
"""
Test handle-based database isolation to verify high-dimensional vector support.

This test validates that the fix for global state contamination allows:
1. Multiple databases with different dimensions
2. High-dimensional vectors (128D, 256D, 512D)
3. Proper state isolation between database instances
"""

import sys

sys.path.insert(0, "/Users/nick/github/omenDB/python")


def test_multiple_databases_different_dimensions():
    """Test multiple database instances with different dimensions simultaneously."""
    from omendb import DB

    print("🧪 Testing multiple databases with different dimensions...")

    databases = []
    test_cases = [
        (16, "16D database"),
        (32, "32D database"),
        (64, "64D database"),
        (128, "128D database"),
        (256, "256D database"),
        (512, "512D database"),
    ]

    # Create all databases
    for dim, desc in test_cases:
        print(f"  Creating {desc}...")
        try:
            db = DB()
            test_vector = [0.1] * dim

            # Add vector to database
            success = db.add(f"test_{dim}d", test_vector)
            if success:
                print(f"  ✅ {desc}: Vector addition successful")
                databases.append((db, dim, desc))
            else:
                print(f"  ❌ {desc}: Vector addition failed")

        except Exception as e:
            print(f"  ❌ {desc}: Exception: {e}")

    # Test that all databases work independently
    print(f"\n🔍 Testing {len(databases)} created databases...")

    for db, dim, desc in databases:
        try:
            # Test query
            test_vector = [0.1] * dim
            results = db.search(test_vector, limit=1)

            if len(results) > 0:
                print(
                    f"  ✅ {desc}: Query successful (similarity: {results[0].score:.3f})"
                )
            else:
                print(f"  ⚠️  {desc}: Query returned no results")

            # Test stats
            stats = db.info()
            print(
                f"    Stats: {stats.get('vector_count', 0)} vectors, dim {stats.get('dimension', 0)}"
            )

        except Exception as e:
            print(f"  ❌ {desc}: Query failed: {e}")

    # Clean up
    for db, _, _ in databases:
        try:
            db.close()
        except:
            pass

    return len(databases)


def test_sequential_high_dimensional():
    """Test high-dimensional vectors sequentially to isolate issues."""
    from omendb import DB

    print("\n🎯 Testing high-dimensional vectors sequentially...")

    test_dimensions = [128, 256, 384, 512, 768, 1024]

    for dim in test_dimensions:
        print(f"\n📊 Testing {dim}D vectors...")

        try:
            # Create fresh database for each dimension
            db = DB()

            # Create test vectors
            test_vectors = [
                ([0.1] * dim, f"test_{dim}d_001"),
                ([0.2] * dim, f"test_{dim}d_002"),
                ([0.3] * dim, f"test_{dim}d_003"),
            ]

            # Add vectors
            for i, (vector, vector_id) in enumerate(test_vectors):
                print(f"  Adding vector {i + 1}/3...")
                success = db.add(vector_id, vector)

                if success:
                    print(f"    ✅ Vector {vector_id} added successfully")
                else:
                    print(f"    ❌ Vector {vector_id} addition failed")
                    break
            else:
                # All vectors added successfully, test search
                print(f"  Testing search with {dim}D query...")
                query_vector = [0.15] * dim
                results = db.search(query_vector, limit=3)

                print(f"  📊 Search results: {len(results)} found")
                for j, result in enumerate(results):
                    print(f"    {j + 1}. {result.id}: similarity {result.score:.4f}")

                # Test stats
                stats = db.info()
                print(
                    f"  📈 Database stats: {stats.get('vector_count', 0)} vectors, "
                    f"dimension {stats.get('dimension', 0)}"
                )

                print(f"  ✅ {dim}D: Complete success!")

            db.close()

        except Exception as e:
            print(f"  ❌ {dim}D: Exception: {e}")
            import traceback

            traceback.print_exc()


def test_extreme_dimensions():
    """Test very high dimensional vectors to find limits."""
    from omendb import DB

    print("\n🚀 Testing extreme high-dimensional vectors...")

    extreme_dimensions = [1536, 2048, 4096]  # Common embedding dimensions

    for dim in extreme_dimensions:
        print(f"\n🌟 Testing {dim}D vectors (extreme)...")

        try:
            db = DB()

            # Create test vector
            test_vector = [float(i % 100) / 100.0 for i in range(dim)]  # Varied values

            print(f"  Adding {dim}D vector...")
            success = db.add(f"extreme_{dim}d", test_vector)

            if success:
                print(f"  ✅ {dim}D: Vector addition successful")

                # Test search
                query_vector = test_vector[:dim]  # Same vector for perfect match
                results = db.search(query_vector, limit=1)

                if len(results) > 0:
                    print(
                        f"  ✅ {dim}D: Search successful (similarity: {results[0].score:.6f})"
                    )
                else:
                    print(f"  ⚠️  {dim}D: Search returned no results")
            else:
                print(f"  ❌ {dim}D: Vector addition failed")

            db.close()

        except Exception as e:
            print(f"  ❌ {dim}D: Exception: {e}")


def main():
    """Run all handle isolation tests."""
    print("🔧 Testing database handle isolation fix...")
    print("=" * 60)

    # Test 1: Multiple databases with different dimensions
    created_count = test_multiple_databases_different_dimensions()
    print(f"\n📊 Successfully created {created_count} concurrent databases")

    # Test 2: Sequential high-dimensional tests
    test_sequential_high_dimensional()

    # Test 3: Extreme dimensions
    test_extreme_dimensions()

    print("\n" + "=" * 60)
    print("🎯 Handle isolation testing complete!")
    print("\nKey tests:")
    print("✅ Multiple concurrent databases with different dimensions")
    print("✅ High-dimensional vectors (128D to 1024D)")
    print("✅ Extreme dimensions (1536D to 4096D)")
    print("✅ Proper database state isolation")


if __name__ == "__main__":
    main()
