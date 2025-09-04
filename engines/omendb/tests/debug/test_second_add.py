#!/usr/bin/env python3
"""Test second add operation"""

import sys

sys.path.insert(0, "python")

import omendb
import numpy as np

print("=== Testing Second Add ===")

db = omendb.DB()

# First add
vec1 = np.arange(64, dtype=np.float32)
print("First add...")
db.add("test1", vec1.tolist())

# Test retrieval after first add
result1 = db.get("test1")
retrieved1 = np.array(result1[0])
matches1 = np.sum(vec1 == retrieved1)
print(f"First retrieval: {matches1}/64 matches ({100 * matches1 / 64:.1f}%)")

# Second add (should skip warmup)
vec2 = np.arange(64, dtype=np.float32) + 100  # Different values
print("\nSecond add...")
db.add("test2", vec2.tolist())

# Test retrieval after second add
result2 = db.get("test2")
retrieved2 = np.array(result2[0])
matches2 = np.sum(vec2 == retrieved2)
print(f"Second retrieval: {matches2}/64 matches ({100 * matches2 / 64:.1f}%)")

# Check if first vector is still corrupted
result1_again = db.get("test1")
retrieved1_again = np.array(result1_again[0])
matches1_again = np.sum(vec1 == retrieved1_again)
print(
    f"First vector again: {matches1_again}/64 matches ({100 * matches1_again / 64:.1f}%)"
)
