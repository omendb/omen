#!/usr/bin/env python3
"""Test the memory stats functionality."""

import numpy as np
import sys
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

# Create DB and add some vectors
db = omendb.DB()
vectors = np.random.rand(1000, 128).astype(np.float32)
db.add_batch(vectors)

# Get memory stats
stats = db.get_memory_stats()

print("\nMemory Statistics:")
print("-" * 40)
for key, value in stats.items():
    print(f"{key:20s}: {value:.2f} MB")

print("\nAdding more vectors...")
more_vectors = np.random.rand(5000, 128).astype(np.float32)
db.add_batch(more_vectors)

# Get updated stats
stats2 = db.get_memory_stats()
print("\nUpdated Memory Statistics:")
print("-" * 40)
for key, value in stats2.items():
    print(f"{key:20s}: {value:.2f} MB")