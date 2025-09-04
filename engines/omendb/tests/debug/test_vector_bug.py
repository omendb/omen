#!/usr/bin/env python3
"""Test script to reproduce the vector retrieval bug"""

import sys

sys.path.insert(0, "python")

import omendb
import numpy as np

# Test the vector retrieval bug
db = omendb.DB()
vec = np.arange(64, dtype=np.float32)  # [0, 1, 2, ..., 63]

print("Original vector (first 16):", vec[:16])

# Add vector
db.add("test", vec.tolist())

# Get vector back
result = db.get("test")
retrieved = np.array(result[0])

print("Retrieved vector (first 16):", retrieved[:16])

# Check how many values match
matches = np.sum(vec == retrieved)
print(f"Matching elements: {matches}/{len(vec)} ({100 * matches / len(vec):.1f}%)")

# Check specifically for the pattern described - elements at 0, 8, 16, 24, etc
print("Pattern check - values at indices 0, 8, 16, 24:")
for i in [0, 8, 16, 24, 32, 40, 48, 56]:
    if i < len(retrieved):
        print(
            f"  Index {i}: original={vec[i]}, retrieved={retrieved[i]}, match={vec[i] == retrieved[i]}"
        )
