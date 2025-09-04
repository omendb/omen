#!/usr/bin/env python3
"""Test script with clear"""

import sys

sys.path.insert(0, "python")

import omendb
import numpy as np

# Clear any existing data first
db = omendb.DB()
db.clear()

print("After clear, trying to get stats...")
try:
    stats = db.get_stats()
    print("Stats after clear:", stats)
except:
    print("Could not get stats after clear")

# Now test again
vec = np.arange(64, dtype=np.float32)
print("Adding vector with shape:", vec.shape)
db.add("test", vec.tolist())

result = db.get("test")
retrieved = np.array(result[0])
print("Retrieved shape:", retrieved.shape)
print("First 16 elements:", retrieved[:16])
matches = np.sum(vec == retrieved)
print(f"Matching elements: {matches}/{len(vec)} ({100 * matches / len(vec):.1f}%)")
