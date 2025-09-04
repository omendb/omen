#!/usr/bin/env python3
"""Test cold start performance - first operation after import."""

import time
import subprocess
import sys

def test_cold_start():
    """Test the very first operation in a fresh process."""
    
    code = """
import time
import sys
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')

# Time the import
start = time.perf_counter()
import omendb
import_time = time.perf_counter() - start

# Time DB creation
start = time.perf_counter()
db = omendb.DB()
db_time = time.perf_counter() - start

# Time first operation
import numpy as np
vector = np.random.rand(128).astype(np.float32)
start = time.perf_counter()
db.add("test", vector)
first_op_time = time.perf_counter() - start

print(f"Import:     {import_time*1000:.1f} ms")
print(f"DB create:  {db_time*1000:.1f} ms")
print(f"First op:   {first_op_time*1000:.1f} ms")
print(f"Total:      {(import_time + db_time + first_op_time)*1000:.1f} ms")
"""
    
    # Run in fresh process
    result = subprocess.run(
        [sys.executable, "-c", code],
        capture_output=True,
        text=True
    )
    
    return result.stdout

def test_warm_operations():
    """Test operations after warmup."""
    
    import sys
    sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
    import omendb
    import numpy as np
    
    db = omendb.DB()
    vector = np.random.rand(128).astype(np.float32)
    
    # Warmup
    db.add("warmup", vector)
    
    # Time subsequent operations
    times = []
    for i in range(10):
        start = time.perf_counter()
        db.add(f"test_{i}", vector)
        times.append(time.perf_counter() - start)
    
    avg_time = sum(times) / len(times)
    return avg_time * 1000  # Convert to ms

if __name__ == "__main__":
    print("OmenDB Cold Start Analysis")
    print("=" * 50)
    
    print("\n1. Cold Start (fresh process):")
    cold_output = test_cold_start()
    print(cold_output)
    
    print("\n2. Warm Operations (after warmup):")
    warm_time = test_warm_operations()
    print(f"   Average add time: {warm_time:.2f} ms")
    
    print("\n3. Multiple Cold Starts:")
    total_times = []
    for i in range(3):
        print(f"\n   Run {i+1}:")
        output = test_cold_start()
        for line in output.strip().split('\n'):
            print(f"      {line}")
        # Extract total time
        if "Total:" in output:
            total = float(output.split("Total:")[1].split("ms")[0].strip())
            total_times.append(total)
    
    if total_times:
        avg_cold = sum(total_times) / len(total_times)
        print(f"\n   Average cold start: {avg_cold:.1f} ms")