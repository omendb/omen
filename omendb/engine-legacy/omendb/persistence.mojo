"""Persistence layer for OmenDB - save/load functionality.

Optimized to avoid Lists and use raw pointers for maximum performance.

Binary format specification:
- Header (64 bytes)
  - Magic number: "OMENDB01" (8 bytes)
  - Version: UInt32 (4 bytes) 
  - Dimension: UInt32 (4 bytes)
  - Vector count: UInt32 (4 bytes)
  - ID count: UInt32 (4 bytes)
  - HNSW M parameter: UInt32 (4 bytes)
  - HNSW ef_construction: UInt32 (4 bytes)
  - HNSW entry point: Int32 (4 bytes)
  - Reserved: 28 bytes
- Vectors section
  - Raw float32 data: [vector_count * dimension * 4 bytes]
- ID mappings section
  - For each mapping:
    - String ID length: UInt32
    - String ID: [length bytes]
    - Numeric ID: UInt32
- HNSW graph section
  - Node count: UInt32
  - For each node:
    - Level: UInt32
    - Connections data...
"""

from memory import UnsafePointer, memcpy, memset_zero
from sys.ffi import external_call
from python import Python

alias MAGIC_NUMBER = "OMENDB01"
alias VERSION = 1
alias HEADER_SIZE = 64

# Low-level file I/O functions (using C FFI)
fn c_fopen(filename: UnsafePointer[UInt8], mode: UnsafePointer[UInt8]) -> UnsafePointer[UInt8]:
    return external_call["fopen", UnsafePointer[UInt8]](filename, mode)

fn c_fwrite(ptr: UnsafePointer[UInt8], size: Int, count: Int, file: UnsafePointer[UInt8]) -> Int:
    return external_call["fwrite", Int](ptr, size, count, file)

fn c_fread(ptr: UnsafePointer[UInt8], size: Int, count: Int, file: UnsafePointer[UInt8]) -> Int:
    return external_call["fread", Int](ptr, size, count, file)

fn c_fclose(file: UnsafePointer[UInt8]) -> Int:
    return external_call["fclose", Int](file)

fn save_database_fast(
    filepath: UnsafePointer[UInt8],  # C-string path
    vectors: UnsafePointer[Float32],
    vector_count: Int,
    dimension: Int,
    # ID mappings as separate arrays (no Lists!)
    id_strings: UnsafePointer[UnsafePointer[UInt8]],  # Array of string pointers
    id_lengths: UnsafePointer[Int],                    # Array of string lengths  
    numeric_ids: UnsafePointer[Int],                   # Array of numeric IDs
    id_count: Int,
    # HNSW graph data
    entry_point: Int,
    node_levels: UnsafePointer[Int],                  # Array of node levels
    node_connections: UnsafePointer[UnsafePointer[Int]], # Jagged array of connections
    node_count: Int
) -> Bool:
    """Save database using direct C file I/O for maximum performance.
    
    No Python, no Lists, just raw speed.
    """
    # Open file for binary writing
    var mode = UnsafePointer[UInt8].alloc(3)
    mode[0] = ord('w')
    mode[1] = ord('b')
    mode[2] = 0
    
    var file = c_fopen(filepath, mode)
    mode.free()
    
    if not file:
        return False
    
    # Prepare header (stack allocated, no heap)
    var header = UnsafePointer[UInt8].alloc(HEADER_SIZE)
    memset_zero(header, HEADER_SIZE)
    
    # Magic number
    memcpy(header, MAGIC_NUMBER.unsafe_ptr(), 8)
    
    # Header fields
    header.offset(8).bitcast[UInt32]()[0] = VERSION
    header.offset(12).bitcast[UInt32]()[0] = UInt32(dimension)
    header.offset(16).bitcast[UInt32]()[0] = UInt32(vector_count)
    header.offset(20).bitcast[UInt32]()[0] = UInt32(id_count)
    header.offset(24).bitcast[UInt32]()[0] = 16  # M parameter
    header.offset(28).bitcast[UInt32]()[0] = 200  # ef_construction
    header.offset(32).bitcast[Int32]()[0] = Int32(entry_point)
    
    # Write header
    if c_fwrite(header, HEADER_SIZE, 1, file) != 1:
        header.free()
        _ = c_fclose(file)
        return False
    header.free()
    
    # Write vectors in one shot (no iteration!)
    var vector_bytes = vector_count * dimension * 4
    if c_fwrite(vectors.bitcast[UInt8](), vector_bytes, 1, file) != 1:
        _ = c_fclose(file)
        return False
    
    # Write ID mappings
    for i in range(id_count):
        # Write string length
        var len_val = UInt32(id_lengths[i])
        if c_fwrite(UnsafePointer.address_of(len_val).bitcast[UInt8](), 4, 1, file) != 1:
            _ = c_fclose(file)
            return False
        
        # Write string data
        if c_fwrite(id_strings[i], id_lengths[i], 1, file) != 1:
            _ = c_fclose(file)
            return False
        
        # Write numeric ID
        var id_val = UInt32(numeric_ids[i])
        if c_fwrite(UnsafePointer.address_of(id_val).bitcast[UInt8](), 4, 1, file) != 1:
            _ = c_fclose(file)
            return False
    
    # Write HNSW graph
    var node_count_val = UInt32(node_count)
    if c_fwrite(UnsafePointer.address_of(node_count_val).bitcast[UInt8](), 4, 1, file) != 1:
        _ = c_fclose(file)
        return False
    
    # TODO: Write actual graph structure (levels and connections)
    # For now, just close
    
    _ = c_fclose(file)
    return True

fn load_database_fast(
    filepath: UnsafePointer[UInt8]  # C-string path
) -> LoadedDatabase:
    """Load database using direct C file I/O.
    
    Returns a struct with raw pointers, no Lists.
    """
    var result = LoadedDatabase()
    
    # Open file for binary reading
    var mode = UnsafePointer[UInt8].alloc(3)
    mode[0] = ord('r')
    mode[1] = ord('b')
    mode[2] = 0
    
    var file = c_fopen(filepath, mode)
    mode.free()
    
    if not file:
        return result
    
    # Read header
    var header = UnsafePointer[UInt8].alloc(HEADER_SIZE)
    if c_fread(header, HEADER_SIZE, 1, file) != 1:
        header.free()
        _ = c_fclose(file)
        return result
    
    # Verify magic number
    var magic_ok = True
    for i in range(8):
        if header[i] != ord(MAGIC_NUMBER[i]):
            magic_ok = False
            break
    
    if not magic_ok:
        header.free()
        _ = c_fclose(file)
        return result
    
    # Parse header
    result.dimension = Int(header.offset(12).bitcast[UInt32]()[0])
    result.vector_count = Int(header.offset(16).bitcast[UInt32]()[0])
    result.id_count = Int(header.offset(20).bitcast[UInt32]()[0])
    result.entry_point = Int(header.offset(32).bitcast[Int32]()[0])
    
    header.free()
    
    # Allocate and read vectors
    var vector_size = result.vector_count * result.dimension
    result.vectors = UnsafePointer[Float32].alloc(vector_size)
    
    if c_fread(result.vectors.bitcast[UInt8](), vector_size * 4, 1, file) != 1:
        result.vectors.free()
        result.vectors = UnsafePointer[Float32]()
        _ = c_fclose(file)
        return result
    
    # Allocate arrays for ID mappings
    result.id_strings = UnsafePointer[UnsafePointer[UInt8]].alloc(result.id_count)
    result.id_lengths = UnsafePointer[Int].alloc(result.id_count)
    result.numeric_ids = UnsafePointer[Int].alloc(result.id_count)
    
    # Read ID mappings
    for i in range(result.id_count):
        # Read string length
        var len_val: UInt32 = 0
        if c_fread(UnsafePointer.address_of(len_val).bitcast[UInt8](), 4, 1, file) != 1:
            # Cleanup on error
            for j in range(i):
                result.id_strings[j].free()
            result.id_strings.free()
            result.id_lengths.free()
            result.numeric_ids.free()
            result.vectors.free()
            _ = c_fclose(file)
            return LoadedDatabase()  # Return empty
        
        result.id_lengths[i] = Int(len_val)
        
        # Allocate and read string
        result.id_strings[i] = UnsafePointer[UInt8].alloc(result.id_lengths[i] + 1)
        if c_fread(result.id_strings[i], result.id_lengths[i], 1, file) != 1:
            # Cleanup on error
            for j in range(i + 1):
                result.id_strings[j].free()
            result.id_strings.free()
            result.id_lengths.free()
            result.numeric_ids.free()
            result.vectors.free()
            _ = c_fclose(file)
            return LoadedDatabase()
        
        result.id_strings[i][result.id_lengths[i]] = 0  # Null terminate
        
        # Read numeric ID
        var id_val: UInt32 = 0
        if c_fread(UnsafePointer.address_of(id_val).bitcast[UInt8](), 4, 1, file) != 1:
            # Cleanup on error
            for j in range(i + 1):
                result.id_strings[j].free()
            result.id_strings.free()
            result.id_lengths.free()
            result.numeric_ids.free()
            result.vectors.free()
            _ = c_fclose(file)
            return LoadedDatabase()
        
        result.numeric_ids[i] = Int(id_val)
    
    # TODO: Read HNSW graph structure
    
    _ = c_fclose(file)
    result.valid = True
    return result

struct LoadedDatabase:
    """Container for loaded database data - no Lists, just pointers."""
    var valid: Bool
    var dimension: Int
    var vector_count: Int
    var id_count: Int
    var entry_point: Int
    var vectors: UnsafePointer[Float32]
    var id_strings: UnsafePointer[UnsafePointer[UInt8]]
    var id_lengths: UnsafePointer[Int]
    var numeric_ids: UnsafePointer[Int]
    
    fn __init__(out self):
        self.valid = False
        self.dimension = 0
        self.vector_count = 0
        self.id_count = 0
        self.entry_point = -1
        self.vectors = UnsafePointer[Float32]()
        self.id_strings = UnsafePointer[UnsafePointer[UInt8]]()
        self.id_lengths = UnsafePointer[Int]()
        self.numeric_ids = UnsafePointer[Int]()
    
    fn cleanup(mut self):
        """Free all allocated memory."""
        if self.vectors:
            self.vectors.free()
        
        if self.id_strings:
            for i in range(self.id_count):
                if self.id_strings[i]:
                    self.id_strings[i].free()
            self.id_strings.free()
        
        if self.id_lengths:
            self.id_lengths.free()
        
        if self.numeric_ids:
            self.numeric_ids.free()