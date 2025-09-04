#!/usr/bin/env python3
"""Test upsert functionality in isolation."""

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


# Test 1: Upsert new vector
test1_code = """
db = omendb.DB()
success = db.upsert("vec1", [1.0, 2.0, 3.0], {"type": "test"})
assert success == True
assert db.count() == 1
assert db.exists("vec1") == True
result = db.get("vec1")
assert result is not None
vector, metadata = result
assert vector == [1.0, 2.0, 3.0]
assert metadata == {"type": "test"}
print("Test passed!")
"""

# Test 2: Upsert existing vector
test2_code = """
db = omendb.DB()
db.add("vec1", [1.0, 2.0, 3.0], {"type": "original"})
assert db.count() == 1
success = db.upsert("vec1", [4.0, 5.0, 6.0], {"type": "updated"})
assert success == True
assert db.count() == 1
result = db.get("vec1")
assert result is not None
vector, metadata = result
assert vector == [4.0, 5.0, 6.0]
assert metadata == {"type": "updated"}
print("Test passed!")
"""

# Test 3: Upsert batch
test3_code = """
db = omendb.DB()
db.add("vec1", [1.0, 2.0, 3.0], {"type": "original"})
db.add("vec2", [4.0, 5.0, 6.0], {"type": "original"})

vectors = [
    [7.0, 8.0, 9.0],    # Update vec1
    [10.0, 11.0, 12.0], # Update vec2
    [13.0, 14.0, 15.0]  # New vec3
]
ids = ["vec1", "vec2", "vec3"]
metadata = [
    {"type": "updated"},
    {"type": "updated"}, 
    {"type": "new"}
]

result_ids = db.upsert_batch(vectors=vectors, ids=ids, metadata=metadata)
assert len(result_ids) == 3
assert set(result_ids) == {"vec1", "vec2", "vec3"}
assert db.count() == 3

# Verify vec1
result1 = db.get("vec1")
assert result1 is not None
vec1, meta1 = result1
assert vec1 == [7.0, 8.0, 9.0], f"Expected [7.0, 8.0, 9.0], got {vec1}"
assert meta1 == {"type": "updated"}

# Verify vec3
result3 = db.get("vec3")
assert result3 is not None
vec3, meta3 = result3
assert vec3 == [13.0, 14.0, 15.0]
assert meta3 == {"type": "new"}
print("Test passed!")
"""

# Test 4: Upsert empty database
test4_code = """
db = omendb.DB()
success = db.upsert("vec1", [1.0, 2.0, 3.0])
assert success == True
assert db.count() == 1
print("Test passed!")
"""


if __name__ == "__main__":
    print("Running upsert tests in isolation...\n")

    tests = [
        ("upsert_new_vector", test1_code),
        ("upsert_existing_vector", test2_code),
        ("upsert_batch", test3_code),
        ("upsert_empty_database", test4_code),
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
        print("\n🎉 All upsert tests passed in isolation!")
        print("The failures in the original test were due to global state issues.")
