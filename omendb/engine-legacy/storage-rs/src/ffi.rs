// ffi.rs - C FFI exports that solve Mojo's limitations
use crate::Storage;
use std::ffi::{c_char, CStr};
use std::ptr;
use std::slice;
use libc::{c_void, size_t};

/// Opaque handle for C/Mojo
pub struct OpaqueStorage {
    inner: Storage,
}

/// Create storage - called from Mojo
#[no_mangle]
pub extern "C" fn storage_create(path: *const c_char, capacity: size_t, dimension: size_t) -> *mut OpaqueStorage {
    let path = unsafe {
        if path.is_null() { return ptr::null_mut(); }
        CStr::from_ptr(path)
    };
    
    let path_str = match path.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    match Storage::create(path_str, capacity, dimension) {
        Ok(storage) => Box::into_raw(Box::new(OpaqueStorage { inner: storage })),
        Err(_) => ptr::null_mut(),
    }
}

/// Destroy storage
#[no_mangle]
pub extern "C" fn storage_destroy(storage: *mut OpaqueStorage) {
    if !storage.is_null() {
        unsafe { drop(Box::from_raw(storage)); }
    }
}

/// Get vector - returns pointer directly into mmap (zero-copy!)
#[no_mangle]
pub extern "C" fn storage_get_vector(storage: *const OpaqueStorage, index: size_t) -> *const f32 {
    if storage.is_null() { return ptr::null(); }
    
    let storage = unsafe { &(*storage).inner };
    
    match unsafe { storage.get_vector_raw(index) } {
        Ok(ptr) => ptr,
        Err(_) => ptr::null(),
    }
}

/// Set vector from Mojo
#[no_mangle]
pub extern "C" fn storage_set_vector(
    storage: *mut OpaqueStorage, 
    index: size_t,
    data: *const f32,
    dimension: size_t
) -> bool {
    if storage.is_null() || data.is_null() { return false; }
    
    let storage = unsafe { &(*storage).inner };
    let data = unsafe { slice::from_raw_parts(data, dimension) };
    
    storage.set_vector(index, data).is_ok()
}

// ============================================================================
// THE BIG SOLUTION: Direct NumPy Support!
// ============================================================================

/// NumPy array info passed from Python
#[repr(C)]
pub struct NumpyArrayInfo {
    pub data: *const c_void,  // The data pointer from __array_interface__
    pub shape_ptr: *const size_t,
    pub strides_ptr: *const size_t,
    pub ndim: size_t,
    pub itemsize: size_t,
}

/// Accept NumPy array DIRECTLY from Python/Mojo - zero copy!
/// This solves the pointer casting issue - Rust handles it!
#[no_mangle]
pub extern "C" fn storage_set_from_numpy(
    storage: *mut OpaqueStorage,
    start_idx: size_t,
    array_info: *const NumpyArrayInfo
) -> bool {
    if storage.is_null() || array_info.is_null() { return false; }
    
    let storage = unsafe { &(*storage).inner };
    let info = unsafe { &*array_info };
    
    // Extract shape (assuming 2D array: [n_vectors, dimension])
    let shape = unsafe { slice::from_raw_parts(info.shape_ptr, info.ndim) };
    if info.ndim != 2 { return false; }
    
    let n_vectors = shape[0];
    let dimension = shape[1];
    
    // Verify dimension matches
    if dimension != storage.dimension() { return false; }
    
    // Get data as f32 slice (numpy should be float32)
    let total_elements = n_vectors * dimension;
    let data = unsafe {
        slice::from_raw_parts(info.data as *const f32, total_elements)
    };
    
    // Batch set - this is zero-copy from numpy!
    storage.set_batch(start_idx, data).is_ok()
}

/// Get vectors into pre-allocated numpy array - zero copy!
#[no_mangle]
pub extern "C" fn storage_get_to_numpy(
    storage: *const OpaqueStorage,
    start_idx: size_t,
    count: size_t,
    output: *mut f32
) -> bool {
    if storage.is_null() || output.is_null() { return false; }
    
    let storage = unsafe { &(*storage).inner };
    let dimension = storage.dimension();
    
    // Copy vectors to output buffer
    let output_slice = unsafe {
        slice::from_raw_parts_mut(output, count * dimension)
    };
    
    for i in 0..count {
        match storage.get_vector(start_idx + i) {
            Ok(vector) => {
                let offset = i * dimension;
                output_slice[offset..offset + dimension].copy_from_slice(&vector);
            }
            Err(_) => return false,
        }
    }
    
    true
}

// ============================================================================
// Thread-Safe Memory Pool Access
// ============================================================================

/// Allocate aligned memory from pool (for SIMD)
#[no_mangle]
pub extern "C" fn storage_alloc_aligned(storage: *mut OpaqueStorage, size: size_t, align: size_t) -> *mut c_void {
    if storage.is_null() { return ptr::null_mut(); }
    
    let storage = unsafe { &(*storage).inner };
    storage.alloc(size, align) as *mut c_void
}

/// Free memory back to pool
#[no_mangle]
pub extern "C" fn storage_free_aligned(storage: *mut OpaqueStorage, ptr: *mut c_void) {
    if storage.is_null() || ptr.is_null() { return; }
    
    let storage = unsafe { &(*storage).inner };
    unsafe { storage.free(ptr as *mut u8); }
}

// ============================================================================
// Persistence Operations
// ============================================================================

/// Sync to disk
#[no_mangle]
pub extern "C" fn storage_sync(storage: *mut OpaqueStorage) -> bool {
    if storage.is_null() { return false; }
    
    let storage = unsafe { &(*storage).inner };
    storage.sync().is_ok()
}

/// Resize storage
#[no_mangle]
pub extern "C" fn storage_resize(storage: *mut OpaqueStorage, new_capacity: size_t) -> bool {
    if storage.is_null() { return false; }
    
    let storage = unsafe { &mut (*storage).inner };
    storage.resize(new_capacity).is_ok()
}

// ============================================================================
// Info Functions
// ============================================================================

#[no_mangle]
pub extern "C" fn storage_capacity(storage: *const OpaqueStorage) -> size_t {
    if storage.is_null() { return 0; }
    unsafe { (*storage).inner.capacity() }
}

#[no_mangle]
pub extern "C" fn storage_count(storage: *const OpaqueStorage) -> size_t {
    if storage.is_null() { return 0; }
    unsafe { (*storage).inner.count() }
}

#[no_mangle]
pub extern "C" fn storage_dimension(storage: *const OpaqueStorage) -> size_t {
    if storage.is_null() { return 0; }
    unsafe { (*storage).inner.dimension() }
}