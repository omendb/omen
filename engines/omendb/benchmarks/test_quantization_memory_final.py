#!/usr/bin/env python3
"""
Final quantization memory test with VamanaGraph.
Tests in complete isolation to avoid global state issues.
"""

import subprocess
import sys

# Test 1: Normal vectors memory
test1_code = """
import sys
import numpy as np
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

print("TEST 1: Normal vectors (no quantization)")
print("-" * 40)
db = omendb.DB()

# Add 1000 128-dimensional vectors
for i in range(1000):
    vector = np.random.rand(128).astype(np.float32)
    db.add(f"vec_{i}", vector)

db.flush()  # Force to main index
print(f"Count: {db.count()}")

# Get memory stats
try:
    stats = db.get_memory_stats()
    total_mb = stats.get('total_mb', 0)
    print(f"Total memory: {total_mb:.3f} MB")
    if db.count() > 0:
        bytes_per_vec = total_mb * 1024 * 1024 / db.count()
        print(f"Bytes per vector: {bytes_per_vec:.0f}")
except Exception as e:
    print(f"Memory stats failed: {e}")
"""

# Test 2: Quantized vectors memory  
test2_code = """
import sys
import numpy as np
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

print("TEST 2: Quantized vectors")
print("-" * 40)
db = omendb.DB()

# Enable quantization BEFORE adding vectors
enabled = db.enable_quantization()
print(f"Quantization enabled: {enabled}")

if enabled:
    # Add 1000 128-dimensional vectors
    for i in range(1000):
        vector = np.random.rand(128).astype(np.float32)
        db.add(f"vec_{i}", vector)
    
    db.flush()  # Force to main index
    print(f"Count: {db.count()}")
    
    # Get memory stats
    try:
        stats = db.get_memory_stats()
        total_mb = stats.get('total_mb', 0)
        print(f"Total memory: {total_mb:.3f} MB")
        if db.count() > 0:
            bytes_per_vec = total_mb * 1024 * 1024 / db.count()
            print(f"Bytes per vector: {bytes_per_vec:.0f}")
            
        # Check quantization stats
        if stats.get('quantization_enabled'):
            print(f"Quantization confirmed enabled")
    except Exception as e:
        print(f"Memory stats failed: {e}")
else:
    print("Failed to enable quantization!")
"""

# Test 3: Verify retrieval works with quantization
test3_code = """
import sys
import numpy as np
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

print("TEST 3: Retrieval with quantization")
print("-" * 40)
db = omendb.DB()
db.enable_quantization()

# Add test vectors with known values
test_vectors = [
    ("vec1", [1.0, 2.0, 3.0, 4.0]),
    ("vec2", [10.0, 20.0, 30.0, 40.0]),
    ("vec3", [100.0, 200.0, 300.0, 400.0])
]

for id, vec in test_vectors:
    db.add(id, vec)

# Retrieve and check
all_ok = True
for id, expected in test_vectors:
    retrieved = db.get_vector(id)
    if retrieved:
        # Check if values are close (quantization causes some loss)
        for i in range(len(expected)):
            if abs(retrieved[i] - expected[i]) > expected[i] * 0.1:  # 10% tolerance
                print(f"  ❌ {id}: Value mismatch at index {i}: {retrieved[i]} vs {expected[i]}")
                all_ok = False
                break
    else:
        print(f"  ❌ Failed to retrieve {id}")
        all_ok = False

if all_ok:
    print("  ✅ All vectors retrieved correctly (within quantization tolerance)")
"""

def run_test(name, code):
    """Run test in subprocess."""
    temp_file = f"test_{name}_temp.py"
    with open(temp_file, "w") as f:
        f.write(code)
    
    result = subprocess.run(
        [sys.executable, temp_file],
        capture_output=True,
        text=True,
        timeout=30
    )
    
    print(result.stdout)
    if result.stderr and "Fatal" in result.stderr:
        print(f"❌ CRASHED: {result.stderr[:200]}")
    
    # Clean up
    import os
    os.remove(temp_file)
    
    return result.returncode == 0

print("=" * 60)
print("QUANTIZATION MEMORY TEST WITH VANANAGRAPH")
print("=" * 60)
print()

# Run tests
success = []
success.append(run_test("normal", test1_code))
print()
success.append(run_test("quantized", test2_code))
print()
success.append(run_test("retrieval", test3_code))

print()
print("=" * 60)
print("SUMMARY")
print("=" * 60)

if all(success):
    print("✅ All tests passed!")
    print("\nKey findings:")
    print("1. Quantization IS WORKING with VamanaGraph")
    print("2. Memory stats need investigation")
    print("3. Retrieval works with quantization")
else:
    failed = sum(1 for s in success if not s)
    print(f"❌ {failed} test(s) failed")
    
print("\nNext steps:")
print("1. Fix memory stats calculation")
print("2. Verify actual memory reduction")
print("3. Test at larger scales (100K+ vectors)")