"""
Quick test to check if bounds checking fixes help.
"""

import sys
import numpy as np

sys.path.append('/Users/nick/github/omendb/core/omendb/engine/python/omendb')

try:
    import native
    print("✅ Module loaded")
except ImportError as e:
    print(f"❌ Import failed: {e}")
    sys.exit(1)

def quick_test():
    print("Running quick stability test...")
    native.clear_database()
    
    # Test with just a few vectors
    vectors = np.random.random((5, 768)).astype(np.float32)
    ids = [f"test_{i}" for i in range(5)]
    metadata = [{"test": True} for _ in range(5)]
    
    try:
        result = native.add_vector_batch(ids, vectors, metadata)
        print(f"✅ 5 vectors: {'SUCCESS' if result else 'FAILED'}")
        
        if result:
            # Try a search
            query = np.random.random(768).astype(np.float32)
            search_result = native.search_vectors(query, 3, {})
            print(f"✅ Search: {len(search_result) if search_result else 0} results")
            
            return True
        return False
        
    except Exception as e:
        print(f"❌ Exception: {e}")
        return False

if __name__ == "__main__":
    success = quick_test()
    if success:
        print("🎉 Basic test passed!")
    else:
        print("❌ Still has issues")