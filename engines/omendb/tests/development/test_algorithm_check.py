#!/usr/bin/env python3
"""Check what algorithm is actually being used."""

import omendb
import numpy as np

# Test with small batch
print("Testing with 100 vectors...")
db = omendb.DB(force_algorithm="diskann")
vectors = np.random.rand(100, 128).astype(np.float32)
db.add_batch(vectors)
print(f"Added {db.count()} vectors\n")

# Clear and test with large batch
db.clear()
print("Testing with 10000 vectors...")
vectors = np.random.rand(10000, 128).astype(np.float32)
db.add_batch(vectors)
print(f"Added {db.count()} vectors\n")

# Check what's in the native module
import omendb.native as native
print("Native module attributes:")
attrs = [attr for attr in dir(native) if not attr.startswith('_')]
for attr in attrs[:20]:  # Show first 20
    print(f"  {attr}")