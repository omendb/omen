#!/usr/bin/env python3
"""Simple test to check DiskANN is working."""

import sys
import os

# Add the local development path for omendb
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)

import numpy as np

# Import omendb from local development
import omendb

print("OmenDB version info:")
print(f"Module path: {omendb.__file__}")
print(f"DB class: {omendb.DB}")

# Create database - test different approaches
print("\nTesting DB creation:")

# Method 1: Default (should be DiskANN now)
db1 = omendb.DB()
print(f"DB() created: {db1}")

# Method 2: Try setting algorithm in constructor
try:
    db2 = omendb.DB(algorithm="diskann")
    print(f"DB(algorithm='diskann') created: {db2}")
except Exception as e:
    print(f"DB(algorithm='diskann') failed: {e}")

# Method 3: Try using configure
try:
    db3 = omendb.DB()
    db3.configure(algorithm="diskann")
    print(f"DB().configure(algorithm='diskann') succeeded")
except Exception as e:
    print(f"DB().configure(algorithm='diskann') failed: {e}")

# Test basic operations
print("\nTesting basic operations:")
dimension = 128
num_vectors = 100

ids = [f"vec_{i}" for i in range(num_vectors)]
vectors = np.random.randn(num_vectors, dimension).astype(np.float32)

# Add vectors
print(f"Adding {num_vectors} vectors...")
results = db1.add_batch(ids, vectors)
successful = sum(1 for r in results if r)
print(f"Added: {successful}/{num_vectors}")

# Get stats
stats = db1.get_stats()
print(f"Stats: {stats}")

# Search
query = vectors[0]
search_results = db1.search(query, limit=5)
print(f"Search returned {len(search_results)} results")
