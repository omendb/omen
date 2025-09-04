// lib.rs - Safe, fast storage layer for OmenDB
// Designed to be replaced with pure Mojo when language matures

use memmap2::{MmapMut, MmapOptions};
use parking_lot::{RwLock, Mutex};
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use thiserror::Error;

// Use mimalloc for better performance if enabled
#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod ffi;  // C FFI exports
mod pool;     // Memory pool implementation

use pool::MemoryPool;

// Error types
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid index: {index} >= {capacity}")]
    InvalidIndex { index: usize, capacity: usize },
    
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    
    #[error("Storage is read-only")]
    ReadOnly,
}

type Result<T> = std::result::Result<T, StorageError>;

// Header stored at beginning of mmap file
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct StorageHeader {
    magic: u32,           // 0x4F4D454E ('OMEN')
    version: u32,         // Format version (1)
    capacity: u64,        // Max vectors
    count: u64,           // Current vector count
    dimension: u64,       // Vector dimension
    metadata_offset: u64, // Offset to metadata section
    _padding: [u64; 2],   // Future expansion
}

impl StorageHeader {
    const MAGIC: u32 = 0x4F4D454E;  // 'OMEN' in hex
    const VERSION: u32 = 1;
    const SIZE: usize = std::mem::size_of::<Self>();
}

// Main storage structure
pub struct Storage {
    mmap: Arc<RwLock<MmapMut>>,
    file: Arc<Mutex<File>>,
    header: Arc<RwLock<StorageHeader>>,
    pool: Arc<MemoryPool>,
    read_only: bool,
    
    // Cached values for fast access
    dimension: usize,
    capacity: usize,
    count: Arc<AtomicUsize>,
}

impl Storage {
    /// Create or open a storage file
    pub fn create<P: AsRef<Path>>(path: P, capacity: usize, dimension: usize) -> Result<Self> {
        let path = path.as_ref();
        
        // Open or create file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        
        // Calculate layout
        let header_size = StorageHeader::SIZE;
        let vectors_size = capacity * dimension * std::mem::size_of::<f32>();
        let metadata_size = capacity * 256;  // 256 bytes per vector for metadata
        let total_size = header_size + vectors_size + metadata_size;
        
        // Ensure file is correct size
        file.set_len(total_size as u64)?;
        
        // Memory map the file
        let mut mmap = unsafe { MmapOptions::new().map_mut(&file)? };
        
        // Initialize or read header
        let header = {
            let header_bytes = &mut mmap[0..header_size];
            let header: &mut StorageHeader = bytemuck::from_bytes_mut(header_bytes);
            
            if header.magic != StorageHeader::MAGIC {
                // New file, initialize header
                *header = StorageHeader {
                    magic: StorageHeader::MAGIC,
                    version: StorageHeader::VERSION,
                    capacity: capacity as u64,
                    count: 0,
                    dimension: dimension as u64,
                    metadata_offset: (header_size + vectors_size) as u64,
                    _padding: [0; 2],
                };
            }
            
            *header
        };
        
        // Create storage instance
        Ok(Storage {
            mmap: Arc::new(RwLock::new(mmap)),
            file: Arc::new(Mutex::new(file)),
            header: Arc::new(RwLock::new(header)),
            pool: Arc::new(MemoryPool::new()),
            read_only: false,
            dimension: header.dimension as usize,
            capacity: header.capacity as usize,
            count: Arc::new(AtomicUsize::new(header.count as usize)),
        })
    }
    
    /// Open storage in read-only mode
    pub fn open_readonly<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        
        // Read header
        let header = {
            let header_bytes = &mmap[0..StorageHeader::SIZE];
            let header: &StorageHeader = bytemuck::from_bytes(header_bytes);
            *header
        };
        
        // Convert to mutable mmap (but we won't write)
        let mmap = unsafe { MmapOptions::new().map_mut(&file)? };
        
        Ok(Storage {
            mmap: Arc::new(RwLock::new(mmap)),
            file: Arc::new(Mutex::new(file)),
            header: Arc::new(RwLock::new(header)),
            pool: Arc::new(MemoryPool::new()),
            read_only: true,
            dimension: header.dimension as usize,
            capacity: header.capacity as usize,
            count: Arc::new(AtomicUsize::new(header.count as usize)),
        })
    }
    
    /// Get a vector by index (returns copy for safety)
    pub fn get_vector(&self, index: usize) -> Result<Vec<f32>> {
        if index >= self.capacity {
            return Err(StorageError::InvalidIndex { 
                index, 
                capacity: self.capacity 
            });
        }
        
        let mmap = self.mmap.read();
        let offset = StorageHeader::SIZE + (index * self.dimension * 4);
        let end = offset + (self.dimension * 4);
        
        // Safe transmute using bytemuck
        let bytes = &mmap[offset..end];
        let slice: &[f32] = bytemuck::cast_slice(bytes);
        Ok(slice.to_vec())
    }
    
    /// Get a vector by index (zero-copy but unsafe)
    pub unsafe fn get_vector_raw(&self, index: usize) -> Result<*const f32> {
        if index >= self.capacity {
            return Err(StorageError::InvalidIndex { 
                index, 
                capacity: self.capacity 
            });
        }
        
        let mmap = self.mmap.read();
        let offset = StorageHeader::SIZE + (index * self.dimension * 4);
        let ptr = mmap.as_ptr().add(offset) as *const f32;
        Ok(ptr)
    }
    
    /// Set a vector at index
    pub fn set_vector(&self, index: usize, data: &[f32]) -> Result<()> {
        if self.read_only {
            return Err(StorageError::ReadOnly);
        }
        
        if index >= self.capacity {
            return Err(StorageError::InvalidIndex { 
                index, 
                capacity: self.capacity 
            });
        }
        
        if data.len() != self.dimension {
            return Err(StorageError::DimensionMismatch {
                expected: self.dimension,
                actual: data.len(),
            });
        }
        
        let mut mmap = self.mmap.write();
        let offset = StorageHeader::SIZE + (index * self.dimension * 4);
        let end = offset + (self.dimension * 4);
        
        // Safe copy using bytemuck
        let dest = &mut mmap[offset..end];
        let src = bytemuck::cast_slice(data);
        dest.copy_from_slice(src);
        
        // Update count if needed
        let current = self.count.load(Ordering::Relaxed);
        if index >= current {
            self.count.store(index + 1, Ordering::Relaxed);
            
            // Update header
            let mut header = self.header.write();
            header.count = (index + 1) as u64;
            let header_bytes = bytemuck::bytes_of_mut(&mut *header);
            mmap[0..StorageHeader::SIZE].copy_from_slice(header_bytes);
        }
        
        Ok(())
    }
    
    /// Batch set vectors (more efficient)
    pub fn set_batch(&self, start_idx: usize, vectors: &[f32]) -> Result<()> {
        if self.read_only {
            return Err(StorageError::ReadOnly);
        }
        
        let count = vectors.len() / self.dimension;
        if start_idx + count > self.capacity {
            return Err(StorageError::InvalidIndex {
                index: start_idx + count,
                capacity: self.capacity,
            });
        }
        
        let mut mmap = self.mmap.write();
        let offset = StorageHeader::SIZE + (start_idx * self.dimension * 4);
        let end = offset + (vectors.len() * 4);
        
        let dest = &mut mmap[offset..end];
        let src = bytemuck::cast_slice(vectors);
        dest.copy_from_slice(src);
        
        // Update count
        let end_idx = start_idx + count;
        let current = self.count.load(Ordering::Relaxed);
        if end_idx > current {
            self.count.store(end_idx, Ordering::Relaxed);
            
            let mut header = self.header.write();
            header.count = end_idx as u64;
            let header_bytes = bytemuck::bytes_of_mut(&mut *header);
            mmap[0..StorageHeader::SIZE].copy_from_slice(header_bytes);
        }
        
        Ok(())
    }
    
    /// Sync changes to disk
    pub fn sync(&self) -> Result<()> {
        if !self.read_only {
            self.mmap.read().flush()?;
        }
        Ok(())
    }
    
    /// Resize storage (grows or shrinks)
    pub fn resize(&mut self, new_capacity: usize) -> Result<()> {
        if self.read_only {
            return Err(StorageError::ReadOnly);
        }
        
        // Calculate new size
        let header_size = StorageHeader::SIZE;
        let vectors_size = new_capacity * self.dimension * std::mem::size_of::<f32>();
        let metadata_size = new_capacity * 256;
        let new_size = header_size + vectors_size + metadata_size;
        
        // Resize file
        {
            let file = self.file.lock();
            file.set_len(new_size as u64)?;
        }
        
        // Remap
        let new_mmap = unsafe {
            let file = self.file.lock();
            MmapOptions::new().map_mut(&*file)?
        };
        
        // Update mmap
        *self.mmap.write() = new_mmap;
        
        // Update header
        let mut header = self.header.write();
        header.capacity = new_capacity as u64;
        self.capacity = new_capacity;
        
        // Write header back
        let header_bytes = bytemuck::bytes_of(&*header);
        self.mmap.write()[0..StorageHeader::SIZE].copy_from_slice(header_bytes);
        
        Ok(())
    }
    
    // Getters
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn count(&self) -> usize { self.count.load(Ordering::Relaxed) }
    pub fn dimension(&self) -> usize { self.dimension }
    pub fn is_readonly(&self) -> bool { self.read_only }
    
    /// Allocate memory from pool
    pub fn alloc(&self, size: usize, align: usize) -> *mut u8 {
        self.pool.alloc(size, align)
    }
    
    /// Free memory to pool
    pub unsafe fn free(&self, ptr: *mut u8) {
        self.pool.free(ptr)
    }
}

// Ensure Send + Sync for FFI
unsafe impl Send for Storage {}
unsafe impl Sync for Storage {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_create_and_store() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.omen");
        
        // Create storage
        let storage = Storage::create(&path, 1000, 128).unwrap();
        
        // Store some vectors
        let vec1 = vec![1.0_f32; 128];
        storage.set_vector(0, &vec1).unwrap();
        
        // Retrieve and verify
        let retrieved = storage.get_vector(0).unwrap();
        assert_eq!(retrieved, &vec1[..]);
        
        // Verify persistence
        storage.sync().unwrap();
        drop(storage);
        
        // Reopen and check
        let storage2 = Storage::open_readonly(&path).unwrap();
        let retrieved2 = storage2.get_vector(0).unwrap();
        assert_eq!(retrieved2, &vec1[..]);
    }
}