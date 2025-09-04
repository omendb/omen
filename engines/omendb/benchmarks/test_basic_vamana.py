#!/usr/bin/env python3
"""
Test basic operations with VamanaGraph implementation.
"""

import sys
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

print("Testing basic VamanaGraph operations...")

# Test 1: Create database
print("\n1. Creating database...")
db = omendb.DB()
print("   ✅ Database created")

# Test 2: Add a few vectors
print("\n2. Adding vectors...")
try:
    db.add("vec1", [1.0, 2.0, 3.0, 4.0])
    db.add("vec2", [5.0, 6.0, 7.0, 8.0])
    db.add("vec3", [9.0, 10.0, 11.0, 12.0])
    print(f"   ✅ Added 3 vectors")
    print(f"   Count: {db.count()}")
except Exception as e:
    print(f"   ❌ Error adding vectors: {e}")
    sys.exit(1)

# Test 3: Search
print("\n3. Testing search...")
try:
    results = db.search([1.1, 2.1, 3.1, 4.1], limit=2)
    print(f"   ✅ Search returned {len(results)} results")
    if results and results[0].id == "vec1":
        print("   ✅ Correct nearest neighbor found")
except Exception as e:
    print(f"   ❌ Error searching: {e}")

# Test 4: Retrieval
print("\n4. Testing retrieval...")
try:
    retrieved = db.get_vector("vec1")
    if retrieved:
        print(f"   ✅ Retrieved vec1: {retrieved}")
    else:
        print("   ❌ Failed to retrieve vec1")
except Exception as e:
    print(f"   ❌ Error retrieving: {e}")

# Test 5: Enable quantization
print("\n5. Testing quantization...")
db2 = omendb.DB()
try:
    enabled = db2.enable_quantization()
    print(f"   Quantization enabled: {enabled}")
    
    if enabled:
        db2.add("qvec1", [1.0, 2.0, 3.0, 4.0])
        print("   ✅ Added vector with quantization enabled")
        
        # Try to get memory stats (this might crash)
        print("   Trying to get memory stats...")
        try:
            stats = db2.get_memory_stats()
            print(f"   ✅ Memory stats retrieved")
            if "quantization_enabled" in stats:
                print(f"      Quantization enabled: {stats['quantization_enabled']}")
        except Exception as e:
            print(f"   ⚠️ Memory stats failed: {e}")
    else:
        print("   ❌ Failed to enable quantization")
except Exception as e:
    print(f"   ❌ Error with quantization: {e}")

print("\n✅ Basic tests completed")