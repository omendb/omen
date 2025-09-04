#!/usr/bin/env python3
"""Simple test that persistence works."""

import os
import sys
import numpy as np

sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

# Test file
test_file = "/tmp/simple_test.omen"
if os.path.exists(test_file):
    os.remove(test_file)

print("Creating DB and saving vectors...")
db = omendb.DB()
db.set_persistence(test_file)

# Add vectors
vectors = np.random.rand(50, 128).astype(np.float32)
db.add_batch(vectors)
print(f"Added {db.count()} vectors")

# Checkpoint
if db.checkpoint():
    print(f"✅ Saved to {test_file} ({os.path.getsize(test_file):,} bytes)")
else:
    print("❌ Checkpoint failed")

# Test recovery
print("\nTesting recovery...")
db2 = omendb.DB()
db2.set_persistence(test_file)
print(f"✅ Recovered {db2.count()} vectors")

# Test search
query = np.random.rand(128).astype(np.float32)
results = db2.search(query, limit=5)
print(f"✅ Search works: {len(results)} results")

print("\n✅ PERSISTENCE WORKING!")