#!/usr/bin/env python3
"""Debug ComponentMemoryStats issue with direct testing."""

import sys
import numpy as np
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')

def test_memory_stats_direct():
    """Test memory stats with direct validation."""
    
    print("ComponentMemoryStats Debug Test")
    print("=" * 60)
    
    import omendb
    
    # Create DB and get initial stats
    db = omendb.DB()
    
    print("\n1. Initial state:")
    stats = db.get_memory_stats()
    for key, value in stats.items():
        print(f"   {key}: {value}")
    
    # Add just enough vectors to trigger a flush
    print("\n2. Adding 10001 vectors to trigger flush...")
    
    # Add vectors in batches to see progression
    batch_size = 2000
    for batch in range(0, 10001, batch_size):
        end_batch = min(batch + batch_size, 10001)
        
        for i in range(batch, end_batch):
            vector = np.random.rand(128).astype(np.float32)
            db.add(f"vec_{i}", vector)
        
        print(f"\n   After {end_batch} vectors:")
        stats = db.get_memory_stats()
        
        # Check if any values are non-zero
        non_zero = [f"{k}: {v:.3f}" for k, v in stats.items() if isinstance(v, float) and v > 0.001]
        if non_zero:
            print(f"     Non-zero values: {', '.join(non_zero)}")
        else:
            print(f"     All values still zero")
    
    # Final check
    print("\n3. Final memory stats:")
    stats = db.get_memory_stats()
    
    has_memory_data = False
    for key, value in stats.items():
        if isinstance(value, float) and value > 0.001:
            print(f"   {key}: {value:.6f}")
            has_memory_data = True
        elif key.endswith('_mb'):
            print(f"   {key}: {value:.6f} (zero)")
    
    if not has_memory_data:
        print("\n❌ DIAGNOSIS: ComponentMemoryStats is not being updated")
        print("   Possible causes:")
        print("   1. Memory calculation code not executing")
        print("   2. Values being overwritten after calculation")
        print("   3. Copy constructor not preserving values")
        print("   4. Thread synchronization issue")
    else:
        print("\n✅ Memory tracking is working!")
    
    # Test the count to verify vectors were added
    print(f"\n4. Vector count verification: {db.count()}")
    
    return stats

if __name__ == "__main__":
    test_memory_stats_direct()