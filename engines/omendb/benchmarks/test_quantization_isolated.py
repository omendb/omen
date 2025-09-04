#!/usr/bin/env python3
"""
Test quantization in complete isolation using subprocess.
"""

import subprocess
import sys

test_code = """
import sys
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

print("Creating new database...")
db = omendb.DB()
print(f"Initial count: {db.count()}")

print("Enabling quantization...")
enabled = db.enable_quantization()
print(f"Quantization enabled: {enabled}")

if enabled:
    print("Adding vectors with quantization...")
    db.add("vec1", [1.0, 2.0, 3.0, 4.0])
    db.add("vec2", [5.0, 6.0, 7.0, 8.0])
    print(f"Count after adding: {db.count()}")
    
    # Test retrieval
    vec = db.get_vector("vec1")
    if vec:
        print(f"Retrieved vec1: {vec}")
    else:
        print("Failed to retrieve vec1")
    
    # Check stats
    info = db.info()
    print(f"Quantization in info: {info.get('quantization_enabled', 'Not present')}")
    print(f"Vector count: {info.get('vector_count', 0)}")
else:
    print("Failed to enable quantization - database may not be empty")
    # Try clearing and retrying
    print("Clearing database...")
    db.clear()
    print(f"Count after clear: {db.count()}")
    
    print("Trying to enable quantization again...")
    enabled = db.enable_quantization()
    print(f"Quantization enabled (2nd try): {enabled}")
"""

# Write to temp file and run
temp_file = "test_quant_isolated.py"
with open(temp_file, "w") as f:
    f.write(test_code)

print("Running test in subprocess...")
print("=" * 60)

result = subprocess.run(
    [sys.executable, temp_file],
    capture_output=True,
    text=True
)

print(result.stdout)
if result.stderr:
    print("STDERR:", result.stderr)

print("=" * 60)
if result.returncode == 0:
    print("✅ Test completed successfully")
else:
    print(f"❌ Test failed with code {result.returncode}")

# Clean up
import os
os.remove(temp_file)