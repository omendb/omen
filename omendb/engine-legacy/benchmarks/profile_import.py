#!/usr/bin/env python3
"""Profile import memory usage."""

import psutil
import os
import gc
import sys

def get_memory_mb():
    """Get current process memory in MB."""
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / (1024 * 1024)

print("Import Memory Profiling")
print("=" * 60)

# 1. Baseline
gc.collect()
baseline = get_memory_mb()
print(f"Baseline: {baseline:.2f} MB")

# 2. Import numpy
import numpy as np
gc.collect()
numpy_mem = get_memory_mb()
print(f"After numpy: {numpy_mem:.2f} MB (+{numpy_mem - baseline:.2f} MB)")

# 3. Add path
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')

# 4. Import omendb
import omendb
gc.collect()
omendb_mem = get_memory_mb()
print(f"After omendb: {omendb_mem:.2f} MB (+{omendb_mem - numpy_mem:.2f} MB)")

# 5. Create DB instance
db = omendb.DB()
gc.collect()
db_mem = get_memory_mb()
print(f"After DB(): {db_mem:.2f} MB (+{db_mem - omendb_mem:.2f} MB)")

# 6. Check what was loaded
print("\nLoaded modules:")
mojo_modules = [m for m in sys.modules if 'omendb' in m or 'native' in m]
for m in sorted(mojo_modules):
    print(f"  {m}")

# 7. Check the native module size
import os
native_path = omendb.native.__file__
if os.path.exists(native_path):
    size_mb = os.path.getsize(native_path) / (1024 * 1024)
    print(f"\nNative module size: {size_mb:.2f} MB")

print("\n" + "=" * 60)
print("SUMMARY")
print("=" * 60)
print(f"Import overhead: {omendb_mem - numpy_mem:.2f} MB")
print(f"DB creation: {db_mem - omendb_mem:.2f} MB")
print(f"Total: {db_mem - baseline:.2f} MB")

# Theory: The native.so is large and loaded into memory
print("\nPossible causes:")
print("1. Large native.so file loaded into memory")
print("2. Global variables initialized in Mojo")
print("3. Memory pool pre-allocation")
print("4. Buffer pre-allocation (10K vectors)")