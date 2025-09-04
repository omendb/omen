#!/usr/bin/env python3
"""Test the new Redis-style API."""

import numpy as np
from python.omendb import DB

def test_new_api():
    """Test set/get and from_numpy/to_numpy methods."""
    
    print("Testing new Redis-style API...")
    
    # Create database
    db = DB()
    
    # Test single set/get
    print("\n1. Testing single set/get:")
    vec1 = [0.1, 0.2, 0.3] + [0.0] * 125  # 128 dimensions
    db.set("id1", vec1)
    retrieved = db.get_vector("id1")
    print(f"  Set vector id1, retrieved: {retrieved[:3]}...")
    
    # Test batch set via dict
    print("\n2. Testing batch set via dict:")
    batch = {
        "id2": [0.4, 0.5, 0.6] + [0.0] * 125,
        "id3": [0.7, 0.8, 0.9] + [0.0] * 125,
    }
    ids = db.set(batch)
    print(f"  Set batch, returned IDs: {ids}")
    
    # Test from_numpy
    print("\n3. Testing from_numpy:")
    vectors = np.random.rand(100, 128).astype(np.float32)
    ids = [f"numpy_{i}" for i in range(100)]
    imported_ids = db.from_numpy(vectors, ids)
    print(f"  Imported {len(imported_ids)} vectors from numpy")
    
    # Test to_numpy
    print("\n4. Testing to_numpy:")
    export_ids = ["numpy_0", "numpy_1", "numpy_2"]
    exported_vectors, exported_ids = db.to_numpy(export_ids)
    print(f"  Exported shape: {exported_vectors.shape}")
    print(f"  Exported IDs: {exported_ids}")
    
    # Test search with k parameter (future)
    print("\n5. Testing search:")
    query = np.random.rand(128).astype(np.float32)
    results = db.search(query, limit=5)  # Currently uses limit, will change to k
    print(f"  Found {len(results)} results")
    
    print("\n✅ New API test complete!")

if __name__ == "__main__":
    test_new_api()