// pool.rs - Thread-safe memory pool for SIMD-aligned allocations
use parking_lot::Mutex;
use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::ptr;

pub struct MemoryPool {
    // Track allocations for proper cleanup
    allocations: Mutex<HashMap<usize, Layout>>,
    
    // Statistics
    total_allocated: Mutex<usize>,
    total_freed: Mutex<usize>,
}

impl MemoryPool {
    pub fn new() -> Self {
        MemoryPool {
            allocations: Mutex::new(HashMap::new()),
            total_allocated: Mutex::new(0),
            total_freed: Mutex::new(0),
        }
    }
    
    /// Allocate aligned memory (for SIMD operations)
    pub fn alloc(&self, size: usize, align: usize) -> *mut u8 {
        // Ensure alignment is power of 2 and at least pointer-sized
        let align = align.max(std::mem::align_of::<usize>());
        let align = align.next_power_of_two();
        
        let layout = match Layout::from_size_align(size, align) {
            Ok(layout) => layout,
            Err(_) => return ptr::null_mut(),
        };
        
        let ptr = unsafe { alloc(layout) };
        
        if !ptr.is_null() {
            let mut allocations = self.allocations.lock();
            allocations.insert(ptr as usize, layout);
            
            let mut total = self.total_allocated.lock();
            *total += size;
        }
        
        ptr
    }
    
    /// Free allocated memory
    pub unsafe fn free(&self, ptr: *mut u8) {
        if ptr.is_null() { return; }
        
        let mut allocations = self.allocations.lock();
        if let Some(layout) = allocations.remove(&(ptr as usize)) {
            dealloc(ptr, layout);
            
            let mut total = self.total_freed.lock();
            *total += layout.size();
        }
    }
    
    /// Get statistics
    pub fn stats(&self) -> (usize, usize) {
        let allocated = *self.total_allocated.lock();
        let freed = *self.total_freed.lock();
        (allocated, freed)
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        // Clean up any remaining allocations
        let allocations = self.allocations.lock();
        for (&ptr, &layout) in allocations.iter() {
            unsafe {
                dealloc(ptr as *mut u8, layout);
            }
        }
    }
}

// Thread-safe
unsafe impl Send for MemoryPool {}
unsafe impl Sync for MemoryPool {}