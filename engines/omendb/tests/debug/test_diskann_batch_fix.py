#!/usr/bin/env python3
"""Test to verify the DiskANN batch_build fix works correctly."""

import numpy as np
import sys
import os

# Add the local development path for omendb
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)


def test_diskann_batch_processing():
    """Test that DiskANN processes all vectors in a batch, not just 1000."""

    import omendb

    # Create database with DiskANN algorithm
    db = omendb.DB(algorithm="diskann")

    # Create test vectors
    test_sizes = [100, 1000, 2500, 5000, 9000]

    for num_vectors in test_sizes:
        print(f"\n{'=' * 60}")
        print(f"Testing with {num_vectors} vectors...")
        print(f"{'=' * 60}")

        # Clear database for fresh test
        db.clear()

        dimension = 128
        ids = [f"vec_{i}" for i in range(num_vectors)]
        vectors = np.random.randn(num_vectors, dimension).astype(np.float32)

        # Add batch - vectors first, then ids!
        print(f"Adding batch of {num_vectors} vectors...")
        results = db.add_batch(vectors, ids)

        # Check results
        successful = sum(1 for r in results if r)
        print(f"Successfully added: {successful}/{num_vectors}")

        # Get stats to verify
        stats = db.get_stats()
        print(f"Database stats: {stats}")

        # Search to verify vectors are retrievable
        query = vectors[0]
        search_results = db.search(query, limit=10)
        print(f"Search returned {len(search_results)} results")

        # Verify the fix
        if successful < num_vectors:
            print(
                f"\n❌ STILL BROKEN: Only {successful} out of {num_vectors} vectors were processed!"
            )
            return False
        else:
            print(f"\n✅ SUCCESS: All {num_vectors} vectors were processed correctly!")

    print(f"\n{'=' * 60}")
    print("🎉 ALL TESTS PASSED! Batch processing bug is FIXED!")
    print(f"{'=' * 60}")
    return True


def test_incremental_batches():
    """Test that multiple batch additions work correctly."""

    import omendb

    print(f"\n{'=' * 60}")
    print("Testing incremental batch additions...")
    print(f"{'=' * 60}")

    db = omendb.DB(algorithm="diskann")

    dimension = 128
    total_added = 0

    # Add multiple batches of different sizes
    batch_sizes = [500, 1500, 2000, 1000]

    for i, batch_size in enumerate(batch_sizes):
        print(f"\nAdding batch {i + 1}: {batch_size} vectors...")

        ids = [f"batch{i}_vec_{j}" for j in range(batch_size)]
        vectors = np.random.randn(batch_size, dimension).astype(np.float32)

        results = db.add_batch(vectors, ids)
        successful = sum(1 for r in results if r)
        total_added += successful

        print(f"Batch {i + 1} result: {successful}/{batch_size} added")
        print(f"Total vectors in DB: {total_added}")

        # Verify with stats
        stats = db.get_stats()
        print(f"Database stats: {stats}")

        if successful != batch_size:
            print(f"❌ Failed to add all vectors in batch {i + 1}")
            return False

    print(f"\n✅ Incremental batch test PASSED! Total: {total_added} vectors")
    return True


if __name__ == "__main__":
    success1 = test_diskann_batch_processing()
    success2 = test_incremental_batches()

    if success1 and success2:
        print("\n🎊 ALL DISKANN BATCH TESTS PASSED!")
        sys.exit(0)
    else:
        print("\n❌ Some tests failed - check output above")
        sys.exit(1)
