#!/usr/bin/env python3
"""
Test quantization memory usage after double storage fix.
Creates separate database instances to avoid state issues.
"""

import sys
import numpy as np
import subprocess
import os

# Add python directory to path
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')

def test_in_subprocess(test_name, code):
    """Run test in subprocess to avoid global state issues."""
    script = f"""
import sys
import numpy as np
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

{code}
"""
    
    # Write to temp file and run
    temp_file = f"test_{test_name}_temp.py"
    with open(temp_file, "w") as f:
        f.write(script)
    
    try:
        result = subprocess.run(
            [sys.executable, temp_file],
            capture_output=True,
            text=True,
            timeout=30
        )
        if result.returncode != 0:
            print(f"❌ {test_name} failed:")
            print(f"  stdout: {result.stdout}")
            print(f"  stderr: {result.stderr}")
            return False
        else:
            print(f"✅ {test_name} output:")
            print(result.stdout)
            return True
    except subprocess.TimeoutExpired:
        print(f"❌ {test_name} timed out")
        return False
    finally:
        if os.path.exists(temp_file):
            os.remove(temp_file)

# Test 1: Normal vectors memory usage
test1_code = """
print("Testing WITHOUT quantization:")
db = omendb.DB()

# Add 1000 vectors
for i in range(1000):
    vector = np.random.rand(128).astype(np.float32)
    db.add(f"vec_{i}", vector)

db.flush()  # Force to main index
stats = db.get_memory_stats()

print(f"  Count: {db.count()}")
print(f"  Memory: {stats.get('total_mb', 0):.3f} MB")
print(f"  Bytes per vector: {stats.get('total_mb', 0) * 1024 * 1024 / db.count():.0f}")
"""

# Test 2: Quantized vectors memory usage  
test2_code = """
print("Testing WITH quantization:")
db = omendb.DB()

# Enable quantization BEFORE adding vectors
success = db.enable_quantization()
print(f"  Quantization enabled: {success}")

if not success:
    print("  ❌ Failed to enable quantization!")
else:
    # Add 1000 vectors
    for i in range(1000):
        vector = np.random.rand(128).astype(np.float32) 
        db.add(f"vec_{i}", vector)
    
    db.flush()  # Force to main index
    stats = db.get_memory_stats()
    
    print(f"  Count: {db.count()}")
    print(f"  Memory: {stats.get('total_mb', 0):.3f} MB")
    print(f"  Bytes per vector: {stats.get('total_mb', 0) * 1024 * 1024 / db.count():.0f}")
"""

# Test 3: Verify retrieval still works
test3_code = """
print("Testing retrieval after fix:")
db = omendb.DB()

# Add test vectors
test_vectors = [
    [1.0, 2.0, 3.0, 4.0],
    [5.0, 6.0, 7.0, 8.0],
    [9.0, 10.0, 11.0, 12.0]
]

for i, vec in enumerate(test_vectors):
    db.add(f"vec_{i}", vec)

# Try to retrieve each vector
all_retrieved = True
for i, expected in enumerate(test_vectors):
    retrieved = db.get_vector(f"vec_{i}")
    if retrieved is None:
        print(f"  ❌ Failed to retrieve vec_{i}")
        all_retrieved = False
    else:
        # Check values match (approximately)
        for j in range(len(expected)):
            if abs(retrieved[j] - expected[j]) > 0.001:
                print(f"  ❌ vec_{i} value mismatch at index {j}: {retrieved[j]} vs {expected[j]}")
                all_retrieved = False
                break

if all_retrieved:
    print("  ✅ All vectors retrieved correctly")
else:
    print("  ❌ Retrieval failed after double storage fix")
"""

print("=" * 60)
print("QUANTIZATION FIX TEST SUITE")
print("=" * 60)
print()

# Run tests
results = []
results.append(test_in_subprocess("normal_memory", test1_code))
results.append(test_in_subprocess("quantized_memory", test2_code))
results.append(test_in_subprocess("retrieval", test3_code))

print()
print("=" * 60)
print("TEST SUMMARY")
print("=" * 60)

if all(results):
    print("✅ All tests passed!")
else:
    failed = sum(1 for r in results if not r)
    print(f"❌ {failed} test(s) failed")

print()
print("Next steps:")
if results[1]:  # If quantization test passed
    print("1. ✅ Quantization is now enabled properly")
    print("2. Check memory usage to verify reduction")
    print("3. Test at larger scales (100K+ vectors)")
else:
    print("1. ❌ Quantization still not working")
    print("2. Need to debug why enable_quantization fails")
    print("3. Check if CSRGraph is properly initialized with quantization flag")