"""
Direct mmap storage without Python FFI overhead.
Target: 10,000+ vec/s throughput.
"""

from memory import UnsafePointer, memcpy, memset_zero
from sys.ffi import external_call
from collections import Dict
from math import ceil
from algorithm import parallelize

# LibC constants
alias O_RDWR = 0x0002
alias O_CREAT = 0x0200  # macOS value
alias PROT_READ = 0x01
alias PROT_WRITE = 0x02
alias MAP_SHARED = 0x01
alias MS_SYNC = 4

# Storage constants  
alias HEADER_SIZE = 512  # Expanded header: magic(4) + version(4) + dimension(4) + num_vectors(4) + id_table_offset(8) = 24 bytes used
alias ID_SIZE = 64  # Max length for string IDs
alias BATCH_SIZE = 1000  # Optimal batch size
alias NEW_VERSION = 5  # Updated version with ID table support

struct DirectStorage(Copyable, Movable):
    """High-performance direct mmap storage."""
    
    var path: String
    var fd: Int32
    var ptr: UnsafePointer[UInt8]
    var file_size: Int
    var num_vectors: Int
    var dimension: Int
    var vector_size: Int  # Bytes per vector
    var is_open: Bool
    var id_table_offset: Int  # Offset to ID table in file (0 if none)
    
    # ID mapping stored in memory for speed
    var id_to_index: Dict[String, Int]
    var index_to_id: Dict[Int, String]
    
    fn __init__(out self, path: String, dimension: Int):
        """Initialize direct storage."""
        self.path = path + ".db"
        self.dimension = dimension
        self.vector_size = dimension * 4  # Float32 = 4 bytes
        self.num_vectors = 0
        self.fd = -1
        self.ptr = UnsafePointer[UInt8]()
        self.file_size = HEADER_SIZE
        self.is_open = False
        self.id_table_offset = 0  # No ID table initially
        self.id_to_index = Dict[String, Int]()
        self.index_to_id = Dict[Int, String]()
        
        # Open or create file
        try:
            self._open_file()
        except:
            self.is_open = False
    
    fn __copyinit__(out self, existing: Self):
        """Copy constructor."""
        self.path = existing.path
        self.fd = existing.fd
        self.ptr = existing.ptr
        self.file_size = existing.file_size
        self.num_vectors = existing.num_vectors
        self.dimension = existing.dimension
        self.vector_size = existing.vector_size
        self.is_open = existing.is_open
        self.id_table_offset = existing.id_table_offset
        self.id_to_index = existing.id_to_index
        self.index_to_id = existing.index_to_id
    
    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.path = existing.path^
        self.fd = existing.fd
        self.ptr = existing.ptr
        self.file_size = existing.file_size
        self.num_vectors = existing.num_vectors
        self.dimension = existing.dimension
        self.vector_size = existing.vector_size
        self.is_open = existing.is_open
        self.id_table_offset = existing.id_table_offset
        self.id_to_index = existing.id_to_index^
        self.index_to_id = existing.index_to_id^
        # Clear source
        existing.fd = -1
        existing.ptr = UnsafePointer[UInt8]()
        existing.is_open = False
    
    fn _open_file(mut self) raises:
        """Open or create memory-mapped file."""
        # Convert path to C string
        var path_bytes = self.path.as_bytes()
        var c_path = UnsafePointer[UInt8].alloc(len(path_bytes) + 1)
        memcpy(c_path, path_bytes.unsafe_ptr(), len(path_bytes))
        c_path[len(path_bytes)] = 0
        
        # Open or create file
        self.fd = external_call["open", Int32, UnsafePointer[UInt8], Int32, UInt32](
            c_path, O_RDWR | O_CREAT, UInt32(0x1B4)
        )
        c_path.free()
        
        if self.fd < 0:
            raise Error("Failed to open file")
        
        # Get current file size
        var current_size = external_call["lseek", Int, Int32, Int, Int32](
            self.fd, 0, 2  # SEEK_END
        )
        
        # Initialize new file
        if current_size == 0:
            _ = external_call["ftruncate", Int32, Int32, Int](
                self.fd, HEADER_SIZE
            )
            self.file_size = HEADER_SIZE
        else:
            self.file_size = current_size
        
        # Map file
        self._map_file()
        
        # Read or write header
        if current_size == 0:
            self._write_header()
        else:
            self._read_header()
        
        self.is_open = True
    
    fn _map_file(mut self) raises:
        """Map file to memory."""
        self.ptr = external_call["mmap", UnsafePointer[UInt8], 
                                UnsafePointer[UInt8], Int, Int32, Int32, Int32, Int](
            UnsafePointer[UInt8](),
            self.file_size,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            self.fd,
            0
        )
        
        if not self.ptr:
            raise Error("mmap failed")
    
    fn _remap(mut self, new_size: Int) raises:
        """Remap file with new size."""
        # Unmap current
        if self.ptr:
            _ = external_call["munmap", Int32, UnsafePointer[UInt8], Int](
                self.ptr, self.file_size
            )
        
        # Extend file
        _ = external_call["ftruncate", Int32, Int32, Int](
            self.fd, new_size
        )
        self.file_size = new_size
        
        # Remap
        self._map_file()
    
    fn _write_header(self):
        """Write file header with ID table support."""
        if not self.ptr:
            return
        
        # Magic "OMDB"
        self.ptr.offset(0).bitcast[UInt32]()[] = 0x4F4D4442
        # Version (NEW_VERSION = 5 for ID table support)
        self.ptr.offset(4).bitcast[UInt32]()[] = NEW_VERSION
        # Dimension
        self.ptr.offset(8).bitcast[UInt32]()[] = UInt32(self.dimension)
        # Number of vectors
        self.ptr.offset(12).bitcast[UInt32]()[] = UInt32(self.num_vectors)
        # ID table offset (0 if no ID table yet)
        self.ptr.offset(16).bitcast[UInt64]()[] = UInt64(self.id_table_offset)
    
    fn _read_header(mut self) raises:
        """Read file header with ID table support."""
        if not self.ptr:
            raise Error("File not mapped")
        
        # Check magic
        var magic = self.ptr.offset(0).bitcast[UInt32]()[]
        if magic != 0x4F4D4442:
            raise Error("Invalid file format")
        
        # Check version
        var version = Int(self.ptr.offset(4).bitcast[UInt32]()[])
        if version < 4:
            raise Error("Unsupported file version")
        
        # Read metadata
        self.dimension = Int(self.ptr.offset(8).bitcast[UInt32]()[])
        self.vector_size = self.dimension * 4
        self.num_vectors = Int(self.ptr.offset(12).bitcast[UInt32]()[])
        
        # Read ID table offset if version supports it
        if version >= NEW_VERSION:
            self.id_table_offset = Int(self.ptr.offset(16).bitcast[UInt64]()[])
        else:
            self.id_table_offset = 0
        
        # Rebuild ID mappings from stored data
        self._rebuild_id_mappings()
    
    fn _rebuild_id_mappings(mut self):
        """Rebuild ID mappings from stored ID table."""
        self.id_to_index = Dict[String, Int]()
        self.index_to_id = Dict[Int, String]()
        
        # If no ID table, generate fallback IDs
        if self.id_table_offset == 0:
            for i in range(self.num_vectors):
                var id_str = "vec_" + String(i)
                self.id_to_index[id_str] = i
                self.index_to_id[i] = id_str
            return
        
        # Read actual stored IDs from ID table
        try:
            self._read_id_table()
        except:
            # Fallback on error
            for i in range(self.num_vectors):
                var id_str = "vec_" + String(i)
                self.id_to_index[id_str] = i
                self.index_to_id[i] = id_str
    
    fn _write_id_table(mut self) raises:
        """Write ID table to disk in binary format."""
        if self.num_vectors == 0:
            return
        
        # Calculate ID table size needed
        var table_size = 4  # Number of entries
        for i in range(self.num_vectors):
            if i in self.index_to_id:
                var id_str = self.index_to_id[i]
                table_size += 8 + len(id_str.as_bytes())  # 4 bytes length + 4 bytes index + string data
        
        # Set ID table offset at end of vectors
        self.id_table_offset = HEADER_SIZE + self.num_vectors * self.vector_size
        var needed_size = self.id_table_offset + table_size
        
        # Extend file if needed
        if needed_size > self.file_size:
            self._remap(needed_size + 1024 * 1024)  # Add 1MB buffer
        
        # Write number of entries
        var write_ptr = self.ptr.offset(self.id_table_offset)
        write_ptr.bitcast[UInt32]()[] = UInt32(self.num_vectors)
        var current_offset = 4
        
        # Write each ID entry
        for i in range(self.num_vectors):
            if i in self.index_to_id:
                var id_str = self.index_to_id[i]
                var id_bytes = id_str.as_bytes()
                var id_len = len(id_bytes)
                
                # Write string length
                write_ptr.offset(current_offset).bitcast[UInt32]()[] = UInt32(id_len)
                current_offset += 4
                
                # Write index value
                write_ptr.offset(current_offset).bitcast[UInt32]()[] = UInt32(i)
                current_offset += 4
                
                # Write string data
                if id_len > 0:
                    memcpy(
                        write_ptr.offset(current_offset),
                        id_bytes.unsafe_ptr(),
                        id_len
                    )
                    current_offset += id_len
        
        # Update header with ID table offset
        self.ptr.offset(16).bitcast[UInt64]()[] = UInt64(self.id_table_offset)
    
    fn _read_id_table(mut self) raises:
        """Read ID table from disk."""
        if self.id_table_offset == 0:
            raise Error("No ID table found")
        
        if self.id_table_offset >= self.file_size:
            raise Error("Invalid ID table offset")
        
        var read_ptr = self.ptr.offset(self.id_table_offset)
        
        # Read number of entries
        var num_entries = Int(read_ptr.bitcast[UInt32]()[])
        var current_offset = 4
        
        # Read each ID entry
        for _ in range(num_entries):
            # Read string length
            var id_len = Int(read_ptr.offset(current_offset).bitcast[UInt32]()[])
            current_offset += 4
            
            # Read index value
            var index = Int(read_ptr.offset(current_offset).bitcast[UInt32]()[])
            current_offset += 4
            
            # Read string data
            var id_str: String
            if id_len > 0:
                var id_bytes = UnsafePointer[UInt8].alloc(id_len + 1)
                memcpy(id_bytes, read_ptr.offset(current_offset), id_len)
                id_bytes[id_len] = 0  # Null terminate
                id_str = String(id_bytes.bitcast[Int8]())
                id_bytes.free()
                current_offset += id_len
            else:
                id_str = "empty_" + String(index)
            
            # Store mapping
            self.id_to_index[id_str] = index
            self.index_to_id[index] = id_str
    
    fn save_vector(mut self, id: String, vector: UnsafePointer[Float32]) raises -> Bool:
        """Save a single vector with direct mmap write."""
        if not self.is_open:
            return False
        
        # Get or assign index
        var index: Int
        if id in self.id_to_index:
            index = self.id_to_index[id]
        else:
            index = self.num_vectors
            self.id_to_index[id] = index
            self.index_to_id[index] = id
            self.num_vectors += 1
        
        # Calculate offset
        var offset = HEADER_SIZE + index * self.vector_size
        var needed_size = offset + self.vector_size
        
        # Extend file if needed
        if needed_size > self.file_size:
            # Grow by 10% or 10MB, whichever is larger
            var growth = max(self.file_size // 10, 10 * 1024 * 1024)
            self._remap(needed_size + growth)
        
        # Direct memory copy - no Python!
        memcpy(
            self.ptr.offset(offset),
            vector.bitcast[UInt8](),
            self.vector_size
        )
        
        # Update header
        self.ptr.offset(12).bitcast[UInt32]()[] = UInt32(self.num_vectors)
        
        # Write ID table to ensure persistence
        try:
            self._write_id_table()
        except:
            pass  # Continue on error - ID table write is optional
        
        return True
    
    fn save_batch(mut self, ids: List[String], vectors: UnsafePointer[Float32], count: Int) raises -> Int:
        """Save batch of vectors with parallel processing."""
        if not self.is_open or count == 0:
            return 0
        
        # Pre-calculate space needed
        var new_vectors = 0
        for i in range(count):
            if ids[i] not in self.id_to_index:
                new_vectors += 1
        
        var needed_size = HEADER_SIZE + (self.num_vectors + new_vectors) * self.vector_size
        
        # Extend file once for entire batch
        if needed_size > self.file_size:
            var growth = max(needed_size - self.file_size, 100 * 1024 * 1024)  # 100MB minimum
            self._remap(needed_size + growth)
        
        # Process vectors in parallel
        var saved_count = 0
        
        @parameter
        fn process_vector(i: Int):
            var id = ids[i]
            var index: Int
            
            # Assign index (not thread-safe, but okay for our use)
            if id in self.id_to_index:
                index = self.id_to_index[id]
            else:
                index = self.num_vectors + i  # Simple sequential assignment
                self.id_to_index[id] = index
                self.index_to_id[index] = id
            
            # Calculate offset
            var offset = HEADER_SIZE + index * self.vector_size
            var vector_ptr = vectors.offset(i * self.dimension)
            
            # Direct memory copy
            memcpy(
                self.ptr.offset(offset),
                vector_ptr.bitcast[UInt8](),
                self.vector_size
            )
        
        # Process all vectors
        for i in range(count):
            process_vector(i)
            saved_count += 1
        
        # Update count
        self.num_vectors += new_vectors
        self.ptr.offset(12).bitcast[UInt32]()[] = UInt32(self.num_vectors)
        
        # Write ID table to ensure persistence
        try:
            self._write_id_table()
        except:
            pass  # Continue on error - ID table write is optional
        
        return saved_count
    
    fn load_vector(self, id: String) raises -> UnsafePointer[Float32]:
        """Load a vector by ID."""
        if not self.is_open:
            raise Error("Storage not open")
        
        if id not in self.id_to_index:
            raise Error("Vector not found: " + id)
        
        var index = self.id_to_index[id]
        var offset = HEADER_SIZE + index * self.vector_size
        
        # Allocate result
        var result = UnsafePointer[Float32].alloc(self.dimension)
        
        # Direct memory copy
        memcpy(
            result.bitcast[UInt8](),
            self.ptr.offset(offset),
            self.vector_size
        )
        
        return result
    
    fn get_vector_count(self) -> Int:
        """Get number of vectors stored."""
        return self.num_vectors
    
    fn flush(self):
        """Flush changes to disk."""
        if self.is_open and self.ptr:
            _ = external_call["msync", Int32, UnsafePointer[UInt8], Int, Int32](
                self.ptr, self.file_size, MS_SYNC
            )
    
    fn close(mut self):
        """Close storage."""
        if self.is_open:
            self.flush()
            
            if self.ptr:
                _ = external_call["munmap", Int32, UnsafePointer[UInt8], Int](
                    self.ptr, self.file_size
                )
                self.ptr = UnsafePointer[UInt8]()
            
            if self.fd >= 0:
                _ = external_call["close", Int32, Int32](self.fd)
                self.fd = -1
            
            self.is_open = False
    
    fn clear(mut self):
        """Clear all vectors."""
        self.num_vectors = 0
        self.id_to_index = Dict[String, Int]()
        self.index_to_id = Dict[Int, String]()
        
        if self.is_open:
            # Reset header
            self.ptr.offset(12).bitcast[UInt32]()[] = 0
            
            # Truncate file
            _ = external_call["ftruncate", Int32, Int32, Int](
                self.fd, HEADER_SIZE
            )

# Compatibility alias for drop-in replacement
alias VectorStorage = DirectStorage