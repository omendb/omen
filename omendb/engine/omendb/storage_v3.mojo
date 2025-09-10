"""
Direct mmap storage with integrated PQ compression.
Target: 5,000+ vec/s throughput with 96x compression.
"""

from memory import UnsafePointer, memcpy, memset_zero
from sys.ffi import external_call
from collections import Dict, List
from math import ceil
from algorithm import parallelize
from sys.intrinsics import sizeof
# PQ compression integrated inline to avoid import issues

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
alias BATCH_SIZE = 1000  # Vectors per batch write

# Simple PQ implementation
struct SimplePQ:
    """Simplified PQ32 compression."""
    var dimension: Int
    var trained: Bool
    
    fn __init__(out self, dimension: Int):
        self.dimension = dimension
        self.trained = False
    
    fn train(mut self, vectors: List[UnsafePointer[Float32]], count: Int):
        """Train PQ (simplified - just marks as trained)."""
        self.trained = True
    
    fn compress(self, vector: UnsafePointer[Float32]) -> UnsafePointer[UInt8]:
        """Compress to PQ32 (simplified quantization)."""
        var compressed = UnsafePointer[UInt8].alloc(PQ_VECTOR_SIZE)
        
        # Simple quantization: map each group of dimensions to 1 byte
        var dims_per_byte = self.dimension // PQ_VECTOR_SIZE
        for i in range(PQ_VECTOR_SIZE):
            var sum: Float32 = 0.0
            for j in range(dims_per_byte):
                var idx = i * dims_per_byte + j
                if idx < self.dimension:
                    sum += vector[idx]
            
            # Quantize to 0-255
            var avg = sum / Float32(dims_per_byte)
            var quantized = Int((avg + 1.0) * 127.5)  # Map [-1,1] to [0,255]
            if quantized < 0:
                quantized = 0
            elif quantized > 255:
                quantized = 255
            compressed[i] = UInt8(quantized)
        
        return compressed
    
    fn decompress(self, compressed: UnsafePointer[UInt8]) -> List[Float32]:
        """Decompress from PQ32."""
        var result = List[Float32]()
        var dims_per_byte = self.dimension // PQ_VECTOR_SIZE
        
        for i in range(PQ_VECTOR_SIZE):
            var val = Float32(Int(compressed[i])) / 127.5 - 1.0
            for j in range(dims_per_byte):
                result.append(val)
        
        # Pad if needed
        while len(result) < self.dimension:
            result.append(0.0)
        
        return result
    
    fn compressed_distance(self, q: UnsafePointer[UInt8], v: UnsafePointer[UInt8]) -> Float32:
        """Compute distance in compressed space."""
        var sum: Float32 = 0.0
        for i in range(PQ_VECTOR_SIZE):
            var diff = Float32(Int(q[i])) - Float32(Int(v[i]))
            sum += diff * diff
        return sum

struct DirectMMapStorage:
    """Direct mmap storage bypassing Python I/O completely."""
    
    var path: String
    var fd: Int32
    var ptr: UnsafePointer[UInt8]
    var file_size: Int
    var num_vectors: Int
    var dimension: Int
    var compressor: SimplePQ
    var id_map: Dict[String, Int]
    var is_open: Bool
    
    fn __init__(out self, path: String, dimension: Int) raises:
        """Initialize direct mmap storage with PQ compression."""
        self.path = path
        self.dimension = dimension
        self.num_vectors = 0
        self.fd = -1
        self.ptr = UnsafePointer[UInt8]()
        self.file_size = HEADER_SIZE  # Start with just header
        self.id_map = Dict[String, Int]()
        self.is_open = False
        
        # Initialize PQ compressor
        self.compressor = SimplePQ(dimension)
        
        # Open or create file
        self._open_file()
    
    fn _open_file(mut self) raises:
        """Open or create memory-mapped file."""
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
            raise Error("Failed to open file: " + self.path)
        
        # Get current file size
        var current_size = external_call["lseek", Int, Int32, Int, Int32](
            self.fd, 0, 2  # SEEK_END
        )
        
        # If new file, write header
        if current_size == 0:
            # Extend to header size
            _ = external_call["ftruncate", Int32, Int32, Int](
                self.fd, HEADER_SIZE
            )
            self.file_size = HEADER_SIZE
            
            # Map header and write initial values
            self._remap(HEADER_SIZE)
            self._write_header()
        else:
            # Read existing file
            self.file_size = current_size
            self._remap(self.file_size)
            self._read_header()
        
        self.is_open = True
    
    fn _remap(mut self, new_size: Int) raises:
        """Remap file with new size."""
        # Unmap if already mapped
        if self.ptr:
            _ = external_call["munmap", Int32, UnsafePointer[UInt8], Int](
                self.ptr, self.file_size
            )
        
        # Extend file if needed
        if new_size > self.file_size:
            _ = external_call["ftruncate", Int32, Int32, Int](
                self.fd, new_size
            )
            self.file_size = new_size
        
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
    
    fn _write_header(self):
        """Write file header."""
        if not self.ptr:
            return
        
        # Magic number "OMDB"
        self.ptr.offset(0).bitcast[UInt32]()[] = 0x4F4D4442
        # Version
        self.ptr.offset(4).bitcast[UInt32]()[] = 3
        # Dimension
        self.ptr.offset(8).bitcast[UInt32]()[] = UInt32(self.dimension)
        # Number of vectors
        self.ptr.offset(12).bitcast[UInt32]()[] = UInt32(self.num_vectors)
        # PQ compression enabled
        self.ptr.offset(16).bitcast[UInt32]()[] = 1
    
    fn _read_header(mut self) raises:
        """Read file header."""
        if not self.ptr:
            raise Error("File not mapped")
        
        # Check magic number
        var magic = self.ptr.offset(0).bitcast[UInt32]()[]
        if magic != 0x4F4D4442:
            raise Error("Invalid file format")
        
        # Read dimension and vector count
        self.dimension = Int(self.ptr.offset(8).bitcast[UInt32]()[])
        self.num_vectors = Int(self.ptr.offset(12).bitcast[UInt32]()[])
        
        # Rebuild ID map from stored data
        # For simplicity, using numeric IDs for now
        for i in range(self.num_vectors):
            self.id_map["vec_" + String(i)] = i
    
    fn train_compressor(mut self, training_vectors: List[UnsafePointer[Float32]], count: Int) raises:
        """Train PQ compressor on sample vectors."""
        self.compressor.train(training_vectors, count)
    
    fn save_batch(mut self, ids: List[String], vectors: List[UnsafePointer[Float32]]) raises:
        """Save batch of vectors with direct mmap writes."""
        var batch_size = len(ids)
        if batch_size == 0:
            return
        
        # Calculate space needed
        var vectors_size = batch_size * PQ_VECTOR_SIZE
        var new_size = HEADER_SIZE + (self.num_vectors + batch_size) * PQ_VECTOR_SIZE
        
        # Extend file if needed
        if new_size > self.file_size:
            self._remap(new_size)
        
        # Compress and write vectors in parallel
        @parameter
        fn compress_and_write(i: Int):
            # Compress vector
            var compressed = self.compressor.compress(vectors[i])
            
            # Calculate offset for this vector
            var offset = HEADER_SIZE + (self.num_vectors + i) * PQ_VECTOR_SIZE
            
            # Write compressed data directly to mmap
            memcpy(
                self.ptr.offset(offset),
                compressed,
                PQ_VECTOR_SIZE
            )
            
            # Update ID map
            self.id_map[ids[i]] = self.num_vectors + i
        
        # Process in parallel for speed
        parallelize[compress_and_write](batch_size)
        
        # Update header
        self.num_vectors += batch_size
        self.ptr.offset(12).bitcast[UInt32]()[] = UInt32(self.num_vectors)
        
        # Sync to disk
        self.sync()
    
    fn load_vector(self, id: String) raises -> List[Float32]:
        """Load and decompress a vector."""
        if id not in self.id_map:
            raise Error("Vector not found: " + id)
        
        var index = self.id_map[id]
        var offset = HEADER_SIZE + index * PQ_VECTOR_SIZE
        
        # Read compressed data
        var compressed = UnsafePointer[UInt8].alloc(PQ_VECTOR_SIZE)
        memcpy(compressed, self.ptr.offset(offset), PQ_VECTOR_SIZE)
        
        # Decompress
        return self.compressor.decompress(compressed)
    
    fn search_batch(self, query: UnsafePointer[Float32], top_k: Int) -> List[Tuple[String, Float32]]:
        """Search all vectors using compressed space distance computation."""
        var results = List[Tuple[String, Float32]]()
        
        # Compress query once
        var query_compressed = self.compressor.compress(query)
        
        # Compute distances in parallel
        var distances = List[Float32]()
        for _ in range(self.num_vectors):
            distances.append(Float32.MAX)
        
        @parameter
        fn compute_distance(i: Int):
            var offset = HEADER_SIZE + i * PQ_VECTOR_SIZE
            
            # Read compressed vector
            var compressed = UnsafePointer[UInt8].alloc(PQ_VECTOR_SIZE)
            memcpy(compressed, self.ptr.offset(offset), PQ_VECTOR_SIZE)
            
            # Compute distance in compressed space
            distances[i] = self.compressor.compressed_distance(
                query_compressed, compressed
            )
        
        parallelize[compute_distance](self.num_vectors)
        
        # Find top-k
        for _ in range(min(top_k, self.num_vectors)):
            var min_idx = 0
            var min_dist = distances[0]
            
            for i in range(1, self.num_vectors):
                if distances[i] < min_dist:
                    min_idx = i
                    min_dist = distances[i]
            
            # Add to results
            results.append(("vec_" + String(min_idx), min_dist))
            distances[min_idx] = Float32.MAX  # Mark as used
        
        return results
    
    fn sync(self):
        """Sync memory-mapped changes to disk."""
        if self.is_open and self.ptr:
            _ = external_call["msync", Int32, UnsafePointer[UInt8], Int, Int32](
                self.ptr, self.file_size, MS_SYNC
            )
    
    fn close(mut self):
        """Close memory-mapped file."""
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
    
    fn get_stats(self) -> String:
        """Get storage statistics."""
        var compression_ratio = Float32(self.dimension * 4) / Float32(PQ_VECTOR_SIZE)
        var actual_size = self.file_size
        var theoretical_size = self.num_vectors * self.dimension * 4
        
        return String("DirectMMapStorage Stats:\n") +
               "  Vectors: " + String(self.num_vectors) + "\n" +
               "  File size: " + String(actual_size) + " bytes\n" +
               "  Theoretical size: " + String(theoretical_size) + " bytes\n" +
               "  Compression ratio: " + String(compression_ratio) + "x\n" +
               "  Bytes per vector: " + String(PQ_VECTOR_SIZE) + "\n"