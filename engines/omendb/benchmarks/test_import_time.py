#!/usr/bin/env python3
"""Profile import time to identify bottlenecks."""

import time
import sys

def profile_import():
    """Profile the import time of omendb."""
    
    # Track individual import times
    times = {}
    
    # Test 1: Time the full import
    start = time.perf_counter()
    import omendb
    total_time = time.perf_counter() - start
    times['total'] = total_time
    
    # Test 2: Import just the native module
    if 'omendb.native' in sys.modules:
        del sys.modules['omendb.native']
    if 'omendb' in sys.modules:
        del sys.modules['omendb']
    
    start = time.perf_counter()
    from omendb import native
    native_time = time.perf_counter() - start
    times['native'] = native_time
    
    # Test 3: Import just the API module (without native)
    if 'omendb.api' in sys.modules:
        del sys.modules['omendb.api']
    if 'omendb' in sys.modules:
        del sys.modules['omendb']
    
    start = time.perf_counter()
    # Temporarily skip native validation
    import omendb.api
    omendb.api._native_validated = True
    api_time = time.perf_counter() - start
    times['api'] = api_time
    
    return times

def profile_operations():
    """Profile common operations after import."""
    import omendb
    import numpy as np
    
    times = {}
    
    # DB creation
    start = time.perf_counter()
    db = omendb.DB()
    times['db_creation'] = time.perf_counter() - start
    
    # First vector add
    vector = np.random.rand(128).astype(np.float32)
    start = time.perf_counter()
    db.add("test1", vector)
    times['first_add'] = time.perf_counter() - start
    
    # Second vector add (should be faster)
    start = time.perf_counter()
    db.add("test2", vector)
    times['second_add'] = time.perf_counter() - start
    
    # Search
    start = time.perf_counter()
    results = db.search(vector, limit=1)
    times['search'] = time.perf_counter() - start
    
    return times

if __name__ == "__main__":
    print("OmenDB Import Time Profiling")
    print("=" * 50)
    
    # Profile imports
    print("\n1. Import Times:")
    import_times = profile_import()
    for name, time_ms in import_times.items():
        print(f"   {name:15s}: {time_ms*1000:7.1f} ms")
    
    # Profile operations
    print("\n2. Operation Times (after import):")
    op_times = profile_operations()
    for name, time_ms in op_times.items():
        print(f"   {name:15s}: {time_ms*1000:7.1f} ms")
    
    print("\n3. Analysis:")
    overhead = import_times['total'] - import_times['native']
    print(f"   Python overhead: {overhead*1000:.1f} ms")
    print(f"   Native module:   {import_times['native']*1000:.1f} ms")
    
    if import_times['native'] > 0.5:
        print("\n   ⚠️ Native module import is slow!")
        print("   This is likely due to Mojo runtime initialization.")
        print("   Consider using lazy loading or warmup strategies.")