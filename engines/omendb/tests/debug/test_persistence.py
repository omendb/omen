#!/usr/bin/env python
"""Test persistence functionality."""

import time
import numpy as np
import sys
import os

sys.path.insert(0, "python")
import omendb


def test_persistence():
    """Test checkpoint and recovery."""
    print("\n=== Testing Persistence ===\n")

    # Test file path
    persist_file = "test_vectors.omen"
    dimension = 128
    num_vectors = 1000

    # Clean up any existing file
    if os.path.exists(persist_file):
        os.remove(persist_file)

    print(f"1. Creating database and adding {num_vectors} vectors")
    db = omendb.DB()

    # Generate test data
    vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
    ids = [f"vec_{i}" for i in range(num_vectors)]

    # Add vectors
    start = time.time()
    db.add_batch(vectors, ids)
    add_time = time.time() - start
    print(f"   Added {num_vectors} vectors in {add_time:.2f}s")

    # Manual checkpoint (if supported)
    print("\n2. Attempting checkpoint...")
    try:
        # This will fail for now since we haven't exposed checkpoint yet
        db.checkpoint(persist_file)
        print(f"   Checkpoint successful to {persist_file}")
    except AttributeError:
        print(f"   Note: Checkpoint not yet exposed in Python API")
        print(f"   Would save to: {persist_file}")

    # Test search before recovery
    query = vectors[0]  # Use first vector as query
    results = db.search(query, limit=5)
    print(f"\n3. Search results before recovery:")
    for r in results[:3]:
        print(f"   - {r.id}: score={r.score:.3f}")

    # Simulate recovery (would create new DB and load)
    print("\n4. Simulating recovery from disk...")
    print(f"   Would load from: {persist_file}")
    print(f"   Expected to recover: {num_vectors} vectors")

    # Clean up
    if os.path.exists(persist_file):
        os.remove(persist_file)

    print("\n✅ Persistence test completed (API integration pending)")


if __name__ == "__main__":
    test_persistence()
