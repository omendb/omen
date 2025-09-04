#!/usr/bin/env python3
"""Test for duplicate detection issue at 15K+ scale."""

import numpy as np
import sys
import os
import time

# Add the local development path for omendb
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)

import omendb


def test_duplicate_detection():
    """Test that duplicate detection works correctly at various scales."""

    print("Testing Duplicate Detection at Scale")
    print("=" * 60)

    db = omendb.DB()

    # Test at different scales
    test_sizes = [1000, 5000, 10000, 15000, 20000, 25000]
    dimension = 128

    for size in test_sizes:
        print(f"\n{'=' * 60}")
        print(f"Testing with {size:,} vectors...")
        print(f"{'=' * 60}")

        # Clear database
        db.clear()

        # Generate test data
        ids = [f"vec_{i}" for i in range(size)]
        vectors = np.random.randn(size, dimension).astype(np.float32)

        # Normalize vectors
        norms = np.linalg.norm(vectors, axis=1, keepdims=True)
        vectors = vectors / norms

        # Add vectors in batches
        batch_size = 1000
        total_added = 0
        start_time = time.time()

        for i in range(0, size, batch_size):
            end_idx = min(i + batch_size, size)
            batch_ids = ids[i:end_idx]
            batch_vecs = vectors[i:end_idx]

            results = db.add_batch(batch_vecs, batch_ids)
            successful = sum(1 for r in results if r)
            total_added += successful

            if (i + batch_size) % 5000 == 0:
                print(f"  Progress: {i + batch_size:,}/{size:,} vectors added")

        add_time = time.time() - start_time
        print(
            f"✅ Added {total_added}/{size} vectors in {add_time:.2f}s ({total_added / add_time:.0f} vec/s)"
        )

        # Test for duplicates by searching for each vector
        print(f"\nTesting for duplicate detection...")

        # Sample 100 random vectors to check
        sample_size = min(100, size)
        sample_indices = np.random.choice(size, sample_size, replace=False)

        duplicates_found = []
        for idx in sample_indices:
            query = vectors[idx]
            expected_id = ids[idx]

            # Search for top 10 results
            results = db.search(query, limit=10)

            # Check if the same ID appears multiple times
            result_ids = [r.id for r in results]
            id_counts = {}
            for rid in result_ids:
                id_counts[rid] = id_counts.get(rid, 0) + 1

            # Check for duplicates
            for rid, count in id_counts.items():
                if count > 1:
                    duplicates_found.append((rid, count))
                    print(
                        f"  ⚠️ Duplicate found: ID '{rid}' appears {count} times in results"
                    )

            # Also check if the expected vector is found
            if expected_id not in result_ids:
                print(f"  ❌ Vector '{expected_id}' not found in search results!")

        if duplicates_found:
            print(f"\n❌ DUPLICATE DETECTION ISSUE at {size:,} vectors!")
            print(f"  Found {len(duplicates_found)} duplicate entries")
            return False
        else:
            print(f"✅ No duplicates found at {size:,} vectors")

        # Try to add the same vectors again to test duplicate handling
        print(f"\nTesting duplicate insertion prevention...")
        duplicate_test_size = min(100, size)
        dup_ids = ids[:duplicate_test_size]
        dup_vecs = vectors[:duplicate_test_size]

        # Try to add them again
        results = db.add_batch(dup_vecs, dup_ids)
        successful = sum(1 for r in results if r)

        if successful > 0:
            print(
                f"  ⚠️ {successful}/{duplicate_test_size} duplicate vectors were added (should be 0)"
            )
        else:
            print(f"  ✅ Duplicate insertion correctly prevented")

        # Search again to verify no new duplicates
        test_vec = vectors[0]
        results = db.search(test_vec, limit=20)
        result_ids = [r.id for r in results]
        id_counts = {}
        for rid in result_ids:
            id_counts[rid] = id_counts.get(rid, 0) + 1

        duplicates_after = [
            (rid, count) for rid, count in id_counts.items() if count > 1
        ]
        if duplicates_after:
            print(f"  ❌ Duplicates found after re-insertion: {duplicates_after}")
        else:
            print(f"  ✅ No duplicates after re-insertion attempt")

    print(f"\n{'=' * 60}")
    print("✅ All duplicate detection tests completed")
    print(f"{'=' * 60}")
    return True


if __name__ == "__main__":
    try:
        success = test_duplicate_detection()
        if success:
            print("\n🎉 Duplicate detection working correctly!")
            sys.exit(0)
        else:
            print("\n❌ Duplicate detection issues found")
            sys.exit(1)
    except Exception as e:
        print(f"\n❌ Test failed with error: {e}")
        import traceback

        traceback.print_exc()
        sys.exit(1)
