#!/usr/bin/env python3
"""Test Rust storage integration for OmenDB."""

import numpy as np
import time
import os
import sys
from ctypes import *

# Load the Rust library
lib_path = "./storage-rs/target/release/libomendb_storage.dylib"
if not os.path.exists(lib_path):
    # Try Linux path
    lib_path = "./storage-rs/target/release/libomendb_storage.so"
    
if not os.path.exists(lib_path):
    print(f"Error: Rust library not found at {lib_path}")
    print("Please run: cd storage-rs && cargo build --release")
    sys.exit(1)

lib = CDLL(lib_path)

# Define FFI function signatures
lib.storage_create.argtypes = [c_char_p, c_size_t, c_size_t]
lib.storage_create.restype = c_void_p

lib.storage_destroy.argtypes = [c_void_p]
lib.storage_destroy.restype = None

lib.storage_set_vector.argtypes = [c_void_p, c_size_t, POINTER(c_float), c_size_t]
lib.storage_set_vector.restype = c_bool

lib.storage_get_vector.argtypes = [c_void_p, c_size_t]
lib.storage_get_vector.restype = POINTER(c_float)

lib.storage_capacity.argtypes = [c_void_p]
lib.storage_capacity.restype = c_size_t

lib.storage_count.argtypes = [c_void_p]
lib.storage_count.restype = c_size_t

lib.storage_sync.argtypes = [c_void_p]
lib.storage_sync.restype = c_bool

# NumPy array info structure
class NumpyArrayInfo(Structure):
    _fields_ = [
        ("data", c_void_p),
        ("shape_ptr", POINTER(c_size_t)),
        ("strides_ptr", POINTER(c_size_t)),
        ("ndim", c_size_t),
        ("itemsize", c_size_t),
    ]

lib.storage_set_from_numpy.argtypes = [c_void_p, c_size_t, POINTER(NumpyArrayInfo)]
lib.storage_set_from_numpy.restype = c_bool

def test_basic_operations():
    """Test basic storage operations."""
    print("Testing basic Rust storage operations...")
    
    # Create storage
    path = b"/tmp/test_rust.omen"
    capacity = 1000
    dimension = 128
    
    storage = lib.storage_create(path, capacity, dimension)
    assert storage != 0, "Failed to create storage"
    print(f"✓ Created storage at {path.decode()}")
    
    # Check capacity
    cap = lib.storage_capacity(storage)
    assert cap == capacity, f"Capacity mismatch: {cap} != {capacity}"
    print(f"✓ Capacity: {cap}")
    
    # Test single vector operations
    vec = np.random.rand(dimension).astype(np.float32)
    vec_ptr = vec.ctypes.data_as(POINTER(c_float))
    
    success = lib.storage_set_vector(storage, 0, vec_ptr, dimension)
    assert success, "Failed to set vector"
    print("✓ Set single vector")
    
    # Get vector back
    result_ptr = lib.storage_get_vector(storage, 0)
    assert result_ptr, "Failed to get vector"
    
    # Convert to numpy array for comparison
    result = np.ctypeslib.as_array(result_ptr, shape=(dimension,))
    np.testing.assert_array_almost_equal(vec, result, decimal=5)
    print("✓ Retrieved vector matches")
    
    # Clean up
    lib.storage_destroy(storage)
    print("✓ Destroyed storage")
    
    return True

def test_numpy_zero_copy():
    """Test zero-copy numpy integration."""
    print("\nTesting numpy zero-copy integration...")
    
    # Create storage
    path = b"/tmp/test_numpy.omen"
    capacity = 10000
    dimension = 128
    n_vectors = 1000
    
    storage = lib.storage_create(path, capacity, dimension)
    assert storage != 0, "Failed to create storage"
    
    # Create batch of vectors
    vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
    
    # Get numpy array interface
    array_interface = vectors.__array_interface__
    data_ptr = array_interface['data'][0]
    shape = vectors.shape
    strides = vectors.strides
    
    # Create info structure
    shape_array = (c_size_t * len(shape))(*shape)
    strides_array = (c_size_t * len(strides))(*[s//4 for s in strides])  # Convert byte strides to element strides
    
    info = NumpyArrayInfo()
    info.data = c_void_p(data_ptr)
    info.shape_ptr = cast(shape_array, POINTER(c_size_t))
    info.strides_ptr = cast(strides_array, POINTER(c_size_t))
    info.ndim = len(shape)
    info.itemsize = 4  # float32
    
    # Measure time for batch insert
    start = time.perf_counter()
    success = lib.storage_set_from_numpy(storage, 0, pointer(info))
    elapsed = time.perf_counter() - start
    
    assert success, "Failed to set from numpy"
    
    vec_per_sec = n_vectors / elapsed
    print(f"✓ Set {n_vectors} vectors in {elapsed:.3f}s = {vec_per_sec:,.0f} vec/s")
    
    # Verify count
    count = lib.storage_count(storage)
    assert count == n_vectors, f"Count mismatch: {count} != {n_vectors}"
    print(f"✓ Count: {count}")
    
    # Sync to disk
    success = lib.storage_sync(storage)
    assert success, "Failed to sync"
    print("✓ Synced to disk")
    
    # Clean up
    lib.storage_destroy(storage)
    
    # Test reopening
    storage2 = lib.storage_create(path, capacity, dimension)
    count2 = lib.storage_count(storage2)
    assert count2 == n_vectors, f"Count after reopen: {count2} != {n_vectors}"
    print(f"✓ Reopened with {count2} vectors")
    
    lib.storage_destroy(storage2)
    
    return vec_per_sec

def test_performance_benchmark():
    """Run performance benchmark."""
    print("\nRunning performance benchmark...")
    
    path = b"/tmp/bench_rust.omen"
    capacity = 1000000
    dimension = 128
    
    # Test different batch sizes
    batch_sizes = [100, 1000, 10000, 50000]
    
    for batch_size in batch_sizes:
        storage = lib.storage_create(path, capacity, dimension)
        
        # Create vectors
        vectors = np.random.rand(batch_size, dimension).astype(np.float32)
        
        # Setup numpy info
        array_interface = vectors.__array_interface__
        data_ptr = array_interface['data'][0]
        shape = vectors.shape
        strides = vectors.strides
        
        shape_array = (c_size_t * len(shape))(*shape)
        strides_array = (c_size_t * len(strides))(*[s//4 for s in strides])
        
        info = NumpyArrayInfo()
        info.data = c_void_p(data_ptr)
        info.shape_ptr = cast(shape_array, POINTER(c_size_t))
        info.strides_ptr = cast(strides_array, POINTER(c_size_t))
        info.ndim = len(shape)
        info.itemsize = 4
        
        # Measure
        start = time.perf_counter()
        success = lib.storage_set_from_numpy(storage, 0, pointer(info))
        elapsed = time.perf_counter() - start
        
        vec_per_sec = batch_size / elapsed
        print(f"  Batch {batch_size:6d}: {vec_per_sec:8,.0f} vec/s ({elapsed*1000:.2f}ms)")
        
        lib.storage_destroy(storage)
        os.remove(path)
    
    return True

def main():
    """Run all tests."""
    print("=" * 60)
    print("OmenDB Rust Storage Integration Test")
    print("=" * 60)
    
    try:
        # Run tests
        test_basic_operations()
        vec_per_sec = test_numpy_zero_copy()
        test_performance_benchmark()
        
        print("\n" + "=" * 60)
        print("✅ All tests passed!")
        print(f"Performance: {vec_per_sec:,.0f} vec/s with zero-copy numpy")
        
        if vec_per_sec > 40000:
            print("🎉 Achieved target of 40K+ vec/s!")
        else:
            print(f"⚠️  Below target of 40K vec/s (currently {vec_per_sec/1000:.1f}K)")
        
        print("=" * 60)
        
    except Exception as e:
        print(f"\n❌ Test failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    main()