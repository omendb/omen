#!/usr/bin/env python3
"""Debug script to understand the vector bug"""

import sys

sys.path.insert(0, "python")

import omendb
import numpy as np

print("=== Starting Fresh Test ===")

# Create fresh database
db = omendb.DB()

# Add debug info
print("Before adding vector:")
print(f"Database initialized: (unknown - no API to check)")

# Create test vector
vec = np.arange(64, dtype=np.float32)
print(f"Test vector shape: {vec.shape}")
print(f"Test vector first 8 elements: {vec[:8]}")

# Add the vector
print("\nAdding vector...")
success = db.add("test", vec.tolist())
print(f"Add successful: {success}")

# Get it back immediately
print("\nRetrieving vector...")
result = db.get("test")
if result:
    retrieved = np.array(result[0])
    print(f"Retrieved shape: {retrieved.shape}")
    print(f"Retrieved first 8 elements: {retrieved[:8]}")

    # Check pattern
    matches = np.sum(vec == retrieved)
    print(f"Total matches: {matches}/{len(vec)} ({100 * matches / len(vec):.1f}%)")

    # Check specific pattern - every 4th element starting from 0
    print("\nPattern Analysis:")
    for i in range(0, min(32, len(vec)), 4):
        orig = vec[i] if i < len(vec) else "N/A"
        retr = retrieved[i] if i < len(retrieved) else "N/A"
        match = orig == retr if i < len(vec) and i < len(retrieved) else False
        print(f"  Index {i:2d}: orig={orig:5.1f}, retr={retr:5.1f}, match={match}")

else:
    print("Failed to retrieve vector!")
