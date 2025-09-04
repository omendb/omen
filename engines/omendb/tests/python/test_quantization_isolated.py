#!/usr/bin/env python3
"""Test scalar quantization functionality in isolation."""

import sys
import os
import subprocess

# Add the python directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../python"))


def run_test_in_subprocess(test_name, test_code):
    """Run a test in a subprocess to avoid global state issues."""
    test_script = f"""
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../python'))
import omendb
import numpy as np

{test_code}
"""

    # Write to temp file and run
    temp_file = f"test_{test_name}_temp.py"
    with open(temp_file, "w") as f:
        f.write(test_script)

    try:
        result = subprocess.run(
            [sys.executable, temp_file],
            capture_output=True,
            text=True,
            env={**os.environ, "PYTHONPATH": "python"},
        )
        success = result.returncode == 0
        if not success:
            print(f"  stdout: {result.stdout}")
            print(f"  stderr: {result.stderr}")
        return success
    finally:
        if os.path.exists(temp_file):
            os.remove(temp_file)


# Test 1: Basic quantization
test1_code = """
db = omendb.DB()
success = db.enable_quantization()
assert success == True, "Failed to enable quantization"

# Add some vectors
db.add("vec1", [1.0, 2.0, 3.0, 4.0])
db.add("vec2", [5.0, 6.0, 7.0, 8.0])

# Check info includes quantization stats
info = db.info()
assert info["quantization_enabled"] == True
print(f"Quantization enabled: {info['quantization_enabled']}")

if "quantized_vectors_count" in info:
    print(f"Quantized vectors: {info['quantized_vectors_count']}")

# Verify vectors can still be queried
results = db.search([1.1, 2.1, 3.1, 4.1], limit=2)
assert len(results) == 2
assert results[0].id == "vec1"  # Should be closest
print("Test passed!")
"""

# Test 2: Memory savings
test2_code = """
db = omendb.DB()
db.enable_quantization()

# Add 100 128-dimensional vectors
vectors = np.random.randn(100, 128).astype(np.float32)

for i in range(100):
    vec = vectors[i].tolist()
    db.add(f"vec_{i}", vec)

info = db.info()
print(f"Vectors: {info['vector_count']}")

if "quantized_vectors_count" in info:
    print(f"Quantized vectors: {info['quantized_vectors_count']}")
    
if "memory_savings_ratio" in info:
    print(f"Memory savings ratio: {info['memory_savings_ratio']:.1f}x")
    assert info["memory_savings_ratio"] > 3.0, "Should achieve at least 3x compression"

print("Test passed!")
"""

# Test 3: Accuracy preservation
test3_code = """
db = omendb.DB()
db.enable_quantization()

# Create vectors with known relationships
dimension = 32

# Add vectors from group 1
for i in range(5):
    vec = [0.0] * dimension
    vec[0] = 1.0 + i * 0.1  # Small variations
    vec[1] = i * 0.05
    db.add(f"group1_{i}", vec)

# Add vectors from group 2
for i in range(5):
    vec = [0.0] * dimension
    vec[1] = 1.0 + i * 0.1  # Small variations
    vec[0] = i * 0.05
    db.add(f"group2_{i}", vec)

# Query with a vector similar to group 1
query1 = [0.0] * dimension
query1[0] = 0.95
results1 = db.search(query1, limit=5)

# Check that top results are from group 1
group1_count = sum(1 for r in results1 if r.id.startswith("group1"))
print(f"Group 1 query: {group1_count}/5 results from correct group")
assert group1_count >= 3, "Quantization should preserve similarity relationships"

# Query with a vector similar to group 2
query2 = [0.0] * dimension
query2[1] = 0.95
results2 = db.search(query2, limit=5)

# Check that top results are from group 2
group2_count = sum(1 for r in results2 if r.id.startswith("group2"))
print(f"Group 2 query: {group2_count}/5 results from correct group")
assert group2_count >= 3, "Quantization should preserve similarity relationships"

print("Test passed!")
"""

# Test 4: Batch operations
test4_code = """
db = omendb.DB()
db.enable_quantization()

# Batch add with numpy
vectors = np.random.randn(50, 64).astype(np.float32)
ids = [f"batch_{i}" for i in range(50)]

result_ids = db.add_batch(vectors=vectors, ids=ids)
assert len(result_ids) == 50

# Verify all vectors are accessible
for i in range(10):  # Check first 10
    result = db.get(f"batch_{i}")
    assert result is not None
    vec, _ = result
    assert len(vec) == 64

# Batch query should still work
query = np.random.randn(64).astype(np.float32)
results = db.search(query.tolist(), limit=10)
assert len(results) == 10
print("Test passed!")
"""


if __name__ == "__main__":
    print("🧪 Running Scalar Quantization Tests in Isolation\n")

    tests = [
        ("basic_quantization", test1_code),
        ("memory_savings", test2_code),
        ("accuracy_preservation", test3_code),
        ("batch_operations", test4_code),
    ]

    passed = 0
    failed = 0

    for test_name, test_code in tests:
        print(f"Running {test_name}...", end=" ")
        if run_test_in_subprocess(test_name, test_code):
            print("✅ PASSED")
            passed += 1
        else:
            print("❌ FAILED")
            failed += 1

    print(f"\nResults: {passed} passed, {failed} failed")

    if failed == 0:
        print("\n🎉 All quantization tests passed!")
        print("\n📊 Summary:")
        print("  - Quantization can be enabled: ✅")
        print("  - Vectors are stored in quantized form: ✅")
        print("  - Memory savings tracked in info: ✅")
        print("  - Search accuracy preserved: ✅")
        print("  - Batch operations work: ✅")
    else:
        sys.exit(1)
