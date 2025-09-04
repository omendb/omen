#!/usr/bin/env python3
"""Test to reproduce and verify the DiskANN batch_build partial processing bug."""

import numpy as np
import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def test_diskann_batch_processing():
    """Test that DiskANN processes all vectors in a batch, not just 1000."""

    import omendb

    # Create database with DiskANN algorithm
    db = omendb.DB(algorithm="diskann")

    # Create 9000 test vectors
    num_vectors = 9000
    dimension = 128

    print(f"Creating {num_vectors} test vectors...")
    ids = [f"vec_{i}" for i in range(num_vectors)]
    vectors = np.random.randn(num_vectors, dimension).astype(np.float32)

    # Add batch
    print(f"Adding batch of {num_vectors} vectors...")
    results = db.add_batch(ids, vectors)

    # Check results
    successful = sum(1 for r in results if r)
    print(f"Successfully added: {successful}/{num_vectors}")

    # Get stats to verify
    stats = db.get_stats()
    print(f"Database stats: {stats}")

    # Search to verify vectors are retrievable
    query = vectors[0]
    results = db.search(query, limit=10)
    print(f"Search returned {len(results)} results")

    # Verify the bug
    if successful < num_vectors:
        print(
            f"\n❌ BUG CONFIRMED: Only {successful} out of {num_vectors} vectors were processed!"
        )
        print("This confirms the batch_build partial processing bug.")
    else:
        print(f"\n✅ All {num_vectors} vectors were processed successfully!")

    return successful == num_vectors


if __name__ == "__main__":
    success = test_diskann_batch_processing()
    sys.exit(0 if success else 1)
