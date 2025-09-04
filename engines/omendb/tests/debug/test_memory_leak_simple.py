#!/usr/bin/env python3
"""
Simple memory leak test to isolate the issue.
"""

import gc
import psutil
import os
import sys

# Add parent directory to path for imports
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "../../")))


def get_memory_mb():
    """Get current process memory usage in MB."""
    return psutil.Process().memory_info().rss / 1024 / 1024


print("OmenDB Simple Memory Leak Test")
print("=" * 50)

# Test 1: Just import
initial = get_memory_mb()
print(f"\nInitial memory: {initial:.1f} MB")

import omendb

after_import = get_memory_mb()
print(f"After import: {after_import:.1f} MB (delta: {after_import - initial:.1f} MB)")

# Test 2: Create empty DB
db = omendb.DB()
after_create = get_memory_mb()
print(
    f"After DB creation: {after_create:.1f} MB (delta: {after_create - after_import:.1f} MB)"
)

# Test 3: Add ONE vector
db.add("test1", [0.1] * 128)
after_one = get_memory_mb()
print(f"After 1 vector: {after_one:.1f} MB (delta: {after_one - after_create:.1f} MB)")

# Test 4: Add 10 more vectors
for i in range(2, 12):
    db.add(f"test{i}", [0.1] * 128)
after_ten = get_memory_mb()
print(
    f"After 11 vectors total: {after_ten:.1f} MB (delta: {after_ten - after_one:.1f} MB)"
)

# Test 5: Delete DB
del db
gc.collect()
after_delete = get_memory_mb()
print(
    f"After DB delete: {after_delete:.1f} MB (delta: {after_delete - after_ten:.1f} MB)"
)

# Test 6: Create another DB
db2 = omendb.DB()
after_second = get_memory_mb()
print(
    f"After 2nd DB creation: {after_second:.1f} MB (delta: {after_second - after_delete:.1f} MB)"
)

# Calculate per-vector memory
per_vector = (after_ten - after_one) / 10
print(f"\nEstimated memory per vector: {per_vector:.1f} MB")
print(f"Total leaked memory: {after_delete - initial:.1f} MB")
