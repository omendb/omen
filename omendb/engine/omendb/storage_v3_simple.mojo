"""
Simplified direct mmap storage without parallelization.
Testing basic mmap functionality.
"""

from memory import UnsafePointer, memcpy, memset_zero
from sys.ffi import external_call
from collections import Dict, List
from math import ceil

# LibC constants
alias O_RDWR = 0x0002
alias O_CREAT = 0x0200  # macOS value
alias PROT_READ = 0x01
alias PROT_WRITE = 0x02
alias MAP_SHARED = 0x01
alias MS_SYNC = 4  # Synchronous sync

# Storage constants
alias HEADER_SIZE = 256  # Bytes for metadata
alias PQ_VECTOR_SIZE = 32  # Bytes per PQ32 vector

struct SimpleMMapStorage:
    """Simplified mmap storage for testing."""
    
    var path: String
    var fd: Int32
    var ptr: UnsafePointer[UInt8]
    var file_size: Int
    var num_vectors: Int
    var dimension: Int
    var is_open: Bool
    
    fn __init__(out self, path: String, dimension: Int) raises:
        """Initialize direct mmap storage."""
        self.path = path
        self.dimension = dimension
        self.num_vectors = 0
        self.fd = -1
        self.ptr = UnsafePointer[UInt8]()
        self.file_size = HEADER_SIZE
        self.is_open = False
        
        # Open or create file
        self._open_file()
    
    fn _open_file(mut self) raises:
        """Open or create memory-mapped file."""
        print("Opening file: ", self.path)
        
        # Convert path to C string
        var path_bytes = self.path.as_bytes()
        var c_path = UnsafePointer[UInt8].alloc(len(path_bytes) + 1)
        memcpy(c_path, path_bytes.unsafe_ptr(), len(path_bytes))
        c_path[len(path_bytes)] = 0  # Null terminate
        
        # Open or create file
        self.fd = external_call["open", Int32, UnsafePointer[UInt8], Int32, UInt32](
            c_path,
            O_RDWR | O_CREAT,
            UInt32(0x1B4)  # 0644 permissions
        )
        
        c_path.free()
        
        if self.fd < 0:
            raise Error("Failed to open file")
        
        print("File opened, fd=", self.fd)
        
        # Get current file size
        var current_size = external_call["lseek", Int, Int32, Int, Int32](
            self.fd, 0, 2  # SEEK_END
        )
        
        print("Current file size: ", current_size)
        
        # Initialize file with header
        if current_size == 0:
            # Extend to header size
            _ = external_call["ftruncate", Int32, Int32, Int](
                self.fd, HEADER_SIZE
            )
            self.file_size = HEADER_SIZE
        else:
            self.file_size = current_size
        
        # Map file
        self.ptr = external_call["mmap", UnsafePointer[UInt8], 
                                UnsafePointer[UInt8], Int, Int32, Int32, Int32, Int](
            UnsafePointer[UInt8](),  # NULL
            self.file_size,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            self.fd,
            0  # offset
        )
        
        if not self.ptr:
            raise Error("mmap failed")
        
        print("File mapped at address")
        
        # Write header if new file
        if current_size == 0:
            self._write_header()
        else:
            self._read_header()
        
        self.is_open = True
    
    fn _write_header(self):
        """Write file header."""
        if not self.ptr:
            return
        
        print("Writing header...")
        # Magic number "TEST"
        self.ptr.offset(0).bitcast[UInt32]()[] = 0x54455354
        # Version
        self.ptr.offset(4).bitcast[UInt32]()[] = 1
        # Dimension
        self.ptr.offset(8).bitcast[UInt32]()[] = UInt32(self.dimension)
        # Number of vectors
        self.ptr.offset(12).bitcast[UInt32]()[] = UInt32(self.num_vectors)
    
    fn _read_header(mut self) raises:
        """Read file header."""
        if not self.ptr:
            raise Error("File not mapped")
        
        print("Reading header...")
        # Check magic number
        var magic = self.ptr.offset(0).bitcast[UInt32]()[]
        if magic != 0x54455354:
            raise Error("Invalid file format")
        
        # Read dimension and vector count
        self.dimension = Int(self.ptr.offset(8).bitcast[UInt32]()[])
        self.num_vectors = Int(self.ptr.offset(12).bitcast[UInt32]()[])
        
        print("Header: dim=", self.dimension, " vectors=", self.num_vectors)
    
    fn save_vector(mut self, id: Int, data: UnsafePointer[UInt8]) raises:
        """Save a single compressed vector."""
        # Calculate offset
        var offset = HEADER_SIZE + id * PQ_VECTOR_SIZE
        
        # Extend file if needed
        var needed_size = offset + PQ_VECTOR_SIZE
        if needed_size > self.file_size:
            print("Extending file from ", self.file_size, " to ", needed_size)
            
            # Unmap current
            _ = external_call["munmap", Int32, UnsafePointer[UInt8], Int](
                self.ptr, self.file_size
            )
            
            # Extend file
            _ = external_call["ftruncate", Int32, Int32, Int](
                self.fd, needed_size
            )
            
            # Remap
            self.ptr = external_call["mmap", UnsafePointer[UInt8], 
                                    UnsafePointer[UInt8], Int, Int32, Int32, Int32, Int](
                UnsafePointer[UInt8](),
                needed_size,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                self.fd,
                0
            )
            
            if not self.ptr:
                raise Error("Remap failed")
            
            self.file_size = needed_size
        
        # Write data
        memcpy(self.ptr.offset(offset), data, PQ_VECTOR_SIZE)
        
        # Update count if new vector
        if id >= self.num_vectors:
            self.num_vectors = id + 1
            self.ptr.offset(12).bitcast[UInt32]()[] = UInt32(self.num_vectors)
    
    fn load_vector(self, id: Int) raises -> UnsafePointer[UInt8]:
        """Load a compressed vector."""
        if id >= self.num_vectors:
            raise Error("Vector not found")
        
        var offset = HEADER_SIZE + id * PQ_VECTOR_SIZE
        var result = UnsafePointer[UInt8].alloc(PQ_VECTOR_SIZE)
        memcpy(result, self.ptr.offset(offset), PQ_VECTOR_SIZE)
        return result
    
    fn sync(self):
        """Sync to disk."""
        if self.is_open and self.ptr:
            _ = external_call["msync", Int32, UnsafePointer[UInt8], Int, Int32](
                self.ptr, self.file_size, MS_SYNC
            )
            print("Synced to disk")
    
    fn close(mut self):
        """Close file."""
        if self.is_open:
            self.sync()
            
            if self.ptr:
                _ = external_call["munmap", Int32, UnsafePointer[UInt8], Int](
                    self.ptr, self.file_size
                )
                self.ptr = UnsafePointer[UInt8]()
            
            if self.fd >= 0:
                _ = external_call["close", Int32, Int32](self.fd)
                self.fd = -1
            
            self.is_open = False
            print("File closed")
    
    fn get_stats(self) -> String:
        """Get storage statistics."""
        return "SimpleMMapStorage: " + String(self.num_vectors) + " vectors, " + 
               String(self.file_size) + " bytes"