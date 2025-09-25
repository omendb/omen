"""OmenDB Storage Engine - Phase 1: Basic File-Based Persistence

This module implements persistent storage for vectors using Mojo's file I/O.
The design follows a phased approach:
- Phase 1: Basic file storage (this implementation)
- Phase 2: Write-ahead logging for crash recovery
- Phase 3: Concurrent access with locks
- Phase 4: Memory mapping optimization
"""

from python import Python, PythonObject
from sys import sizeof
from memory import memcpy, memset, UnsafePointer
from utils import Index
from math import ceil
from os import Atomic
from utils.lock import BlockingSpinLock, BlockingScopedLock
from collections import Dict, List


alias MAGIC_BYTES: String = "OMEN"
alias VERSION: Int = 1
alias BLOCK_SIZE: Int = 4096
alias HEADER_SIZE: Int = 512


@value
struct StorageHeader:
    """Storage file header containing metadata."""
    var magic: String
    var version: Int
    var block_size: Int
    var total_blocks: Int
    var free_blocks: Int
    var vector_count: Int
    var dimension: Int
    
    fn __init__(out self):
        self.magic = MAGIC_BYTES
        self.version = VERSION
        self.block_size = BLOCK_SIZE
        self.total_blocks = 0
        self.free_blocks = 0
        self.vector_count = 0
        self.dimension = 0


@value
struct IndexEntry:
    """Index entry mapping ID to storage location."""
    var id_hash: Int
    var block_id: Int
    var offset: Int
    var size: Int
    
    fn __init__(out self, id_hash: Int, block_id: Int, offset: Int, size: Int):
        self.id_hash = id_hash
        self.block_id = block_id
        self.offset = offset
        self.size = size


struct StorageEngine:
    """Main storage engine for persistent vector storage."""
    var data_file: PythonObject
    var index_file: PythonObject
    var header: StorageHeader
    var lock: BlockingSpinLock
    var free_blocks: List[Int]
    var next_block_id: Atomic[DType.int64]
    var initialized: Bool
    var py_module: PythonObject
    
    fn __init__(out self, path: String) raises:
        """Initialize storage engine with given path."""
        self.py_module = Python.import_module("builtins")
        
        # Open data file
        self.data_file = self.py_module.open(path + ".db", "wb+")
        self.index_file = self.py_module.open(path + ".idx", "wb+")
        
        self.header = StorageHeader()
        self.lock = BlockingSpinLock()
        self.free_blocks = List[Int]()
        self.next_block_id = Atomic[DType.int64](1)  # Block 0 is header
        self.initialized = False
        
        # Write initial header
        self.write_header()
        _ = self.data_file.flush()
        self.initialized = True
    
    fn __del__(owned self):
        """Close files on destruction."""
        if self.initialized:
            try:
                _ = self.data_file.close()
                _ = self.index_file.close()
            except:
                pass
    
    fn write_header(self) raises:
        """Write header to data file."""
        _ = self.data_file.seek(0)
        
        # Write magic string through Python
        var py_str = PythonObject(self.header.magic)
        _ = self.data_file.write(py_str.encode("utf-8"))
        
        # Write version
        _ = self.data_file.write(self.py_module.bytes([self.header.version]))
        
        # Write other fields as bytes
        var struct_module = Python.import_module("struct")
        var packed = struct_module.pack(
            "<IQQQI",  # Little-endian: uint32, uint64, uint64, uint64, uint32
            self.header.block_size,
            self.header.total_blocks,
            self.header.free_blocks,
            self.header.vector_count,
            self.header.dimension
        )
        _ = self.data_file.write(packed)
        
        # Pad to HEADER_SIZE
        var current_size = 4 + 1 + 32  # magic + version + packed struct
        if current_size < HEADER_SIZE:
            var padding = self.py_module.bytes(HEADER_SIZE - current_size)
            _ = self.data_file.write(padding)
    
    fn allocate_block(mut self) -> Int:
        """Allocate a new block, returning its ID."""
        # Check free list first
        if len(self.free_blocks) > 0:
            return self.free_blocks.pop()
        
        # Allocate new block
        var block_id = Int(self.next_block_id.fetch_add(1))
        self.header.total_blocks += 1
        return block_id
    
    fn hash_id(self, id: String) -> Int:
        """Simple hash function for string IDs."""
        var hash: Int = 5381
        var id_bytes = id.as_bytes()
        for i in range(len(id_bytes)):
            hash = ((hash << 5) + hash) + Int(id_bytes[i])
        return hash
    
    fn write_vector(mut self, id: String, vector: UnsafePointer[Float32], size: Int) raises:
        """Write a vector to storage."""
        with BlockingScopedLock(self.lock):
            # Calculate storage requirements
            var bytes_needed = size * sizeof[Float32]()
            
            # Allocate block
            var block_id = self.allocate_block()
            var block_offset = block_id * BLOCK_SIZE
            
            # Seek to block position
            _ = self.data_file.seek(block_offset)
            
            # Convert vector to bytes and write
            var bytes_list = Python.import_module("array").array("f")
            for i in range(size):
                _ = bytes_list.append(Float64(vector[i]))
            
            _ = self.data_file.write(bytes_list.tobytes())
            
            # Pad to block boundary if needed
            if bytes_needed < BLOCK_SIZE:
                var padding_size = BLOCK_SIZE - bytes_needed
                var padding = self.py_module.bytes(padding_size)
                _ = self.data_file.write(padding)
            
            # Write index entry
            self.write_index_entry(id, block_id, 0, bytes_needed)
            
            # Update header
            self.header.vector_count += 1
            if self.header.dimension == 0:
                self.header.dimension = size
            
            # Flush for durability
            _ = self.data_file.flush()
            _ = self.index_file.flush()
    
    fn write_index_entry(self, id: String, block_id: Int, offset: Int, size: Int) raises:
        """Write index entry to index file."""
        var struct_module = Python.import_module("struct")
        
        # Write ID string length and data through Python
        var py_id = PythonObject(id)
        var id_bytes = py_id.encode("utf-8")
        var id_len = Int(self.py_module.len(id_bytes))
        
        # Pack entry: id_length, id_string, hash, block_id, offset, size
        _ = self.index_file.write(struct_module.pack("<I", id_len))
        _ = self.index_file.write(id_bytes)
        
        var id_hash = self.hash_id(id)
        var entry_data = struct_module.pack("<QIII", id_hash, block_id, offset, size)
        _ = self.index_file.write(entry_data)
    
    fn read_vector(self, id: String, dimension: Int) raises -> UnsafePointer[Float32]:
        """Read a vector from storage."""
        # For now, scan index file to find entry
        _ = self.index_file.seek(0)
        var struct_module = Python.import_module("struct")
        var found = False
        var block_id = 0
        var offset = 0
        var size = 0
        
        try:
            while True:
                # Read entry
                var len_bytes = self.index_file.read(4)
                if len(len_bytes) < 4:
                    break
                    
                var id_len = Int(struct_module.unpack("<I", len_bytes)[0])
                var stored_id = self.index_file.read(id_len).decode("utf-8")
                var entry_bytes = self.index_file.read(20)  # hash(8) + block(4) + offset(4) + size(4)
                
                if stored_id == id:
                    var unpacked = struct_module.unpack("<QIII", entry_bytes)
                    block_id = Int(unpacked[1])
                    offset = Int(unpacked[2])
                    size = Int(unpacked[3])
                    found = True
                    break
        except:
            pass
        
        if not found:
            raise Error("Vector not found: " + id)
        
        # Read vector data
        var block_offset = block_id * BLOCK_SIZE + offset
        _ = self.data_file.seek(block_offset)
        
        var array_module = Python.import_module("array")
        var bytes_data = self.data_file.read(size)
        var float_array = array_module.array("f")
        _ = float_array.frombytes(bytes_data)
        
        # Convert to Mojo pointer
        var result = UnsafePointer[Float32].alloc(dimension)
        for i in range(dimension):
            result[i] = Float32(Float64(float_array[i]))
        
        return result
    
    fn flush(mut self) raises:
        """Flush all pending writes to disk."""
        _ = self.data_file.flush()
        _ = self.index_file.flush()
        
        # Update header on disk
        self.write_header()
        _ = self.data_file.flush()
    
    fn close(mut self) raises:
        """Close the storage engine."""
        self.flush()
        _ = self.data_file.close()
        _ = self.index_file.close()
        self.initialized = False


fn test_storage() raises:
    """Test basic storage operations."""
    print("Testing storage engine...")
    
    var storage = StorageEngine("test_storage")
    
    # Create test vector
    var dim = 128
    var test_vector = UnsafePointer[Float32].alloc(dim)
    for i in range(dim):
        test_vector[i] = Float32(i) * 0.1
    
    # Write vector
    storage.write_vector("test_1", test_vector, dim)
    print("Wrote vector 'test_1'")
    
    # Read it back
    var retrieved = storage.read_vector("test_1", dim)
    print("Read vector back")
    
    # Verify
    var matches = True
    for i in range(dim):
        if abs(retrieved[i] - test_vector[i]) > 0.0001:
            matches = False
            break
    
    if matches:
        print("✓ Storage test passed")
    else:
        print("✗ Storage test failed - data mismatch")
    
    test_vector.free()
    retrieved.free()
    storage.close()


fn main() raises:
    """Run storage tests."""
    test_storage()