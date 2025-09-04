#!/usr/bin/env python3
"""
Test basic functionality
"""

import numpy as np
import sys
import os

# Ensure we're using the local omendb
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "python"))

import omendb

print("Testing basic functionality...")

# Create DB
db = omendb.DB()

# Test single vector add
print("\n1. Testing single vector add...")
try:
    vector = [1.0, 2.0, 3.0, 4.0]
    db.add("test1", vector)
    print(f"✅ Added single vector")
except Exception as e:
    print(f"❌ Failed: {e}")

# Test search
print("\n2. Testing search...")
try:
    results = db.search([1.0, 2.0, 3.0, 4.0], limit=1)
    print(f"✅ Search works, found {len(results)} results")
except Exception as e:
    print(f"❌ Search failed: {e}")

# Test batch with simple data
print("\n3. Testing batch add with lists...")
try:
    ids = ["a", "b", "c"]
    vectors = [[1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12]]
    db.add_batch(vectors, ids)
    print(f"✅ Batch add with lists works")
except Exception as e:
    print(f"❌ Batch failed: {e}")
    import traceback

    traceback.print_exc()

# Test count
print(f"\n4. Database contains {db.count()} vectors")

print("\nDone!")
