"""
Compression Manager for OmenDB.

Provides unified compression interface supporting multiple algorithms
with dual-mode architecture (embedded + server) and automatic optimization.
"""

from memory import UnsafePointer, memcpy
from collections import List
from core.vector import Vector
from compression.binary_quantization import BinaryQuantizedVector, BinaryQuantizer
from storage.embedded_db import CompressionType


struct CompressedVector[dtype: DType = DType.float32](Copyable, Movable):
    """
    Unified compressed vector supporting multiple compression algorithms.
    
    Provides consistent interface for all compression types while maintaining
    dual-mode compatibility (embedded + server).
    """
    
    var compression_type: CompressionType
    var original_dim: Int
    var data: UnsafePointer[UInt8]  # Raw compressed data
    var data_size: Int              # Size of compressed data in bytes
    var metadata: UnsafePointer[UInt8]  # Algorithm-specific metadata
    var metadata_size: Int          # Size of metadata in bytes
    
    fn __init__(out self, compression_type: CompressionType, dim: Int):
        """Initialize empty compressed vector."""
        self.compression_type = compression_type
        self.original_dim = dim
        self.data = UnsafePointer[UInt8]()
        self.data_size = 0
        self.metadata = UnsafePointer[UInt8]()
        self.metadata_size = 0
        
    fn __copyinit__(out self, other: Self):
        """Copy constructor."""
        self.compression_type = other.compression_type
        self.original_dim = other.original_dim
        self.data_size = other.data_size
        self.metadata_size = other.metadata_size
        
        # Copy compressed data
        if other.data_size > 0:
            self.data = UnsafePointer[UInt8].alloc(self.data_size)
            memcpy(self.data, other.data, self.data_size)
        else:
            self.data = UnsafePointer[UInt8]()
            
        # Copy metadata
        if other.metadata_size > 0:
            self.metadata = UnsafePointer[UInt8].alloc(self.metadata_size)
            memcpy(self.metadata, other.metadata, self.metadata_size)
        else:
            self.metadata = UnsafePointer[UInt8]()
            
    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.compression_type = existing.compression_type
        self.original_dim = existing.original_dim
        self.data = existing.data
        self.data_size = existing.data_size
        self.metadata = existing.metadata
        self.metadata_size = existing.metadata_size
        
    fn __del__(owned self):
        """Destructor."""
        if self.data:
            self.data.free()
        if self.metadata:
            self.metadata.free()
            
    fn total_size(self) -> Int:
        """Calculate total memory usage."""
        return (
            8 +                        # compression_type + padding
            8 +                        # original_dim
            8 +                        # data pointer
            8 +                        # data_size
            8 +                        # metadata pointer
            8 +                        # metadata_size
            self.data_size +           # actual compressed data
            self.metadata_size         # algorithm metadata
        )
        
    fn compression_ratio(self, original_size: Int) -> Float64:
        """Calculate compression ratio."""
        if self.total_size() > 0:
            return Float64(original_size) / Float64(self.total_size())
        return 0.0


struct CompressionManager[dtype: DType = DType.float32]:
    """
    Unified compression manager supporting multiple algorithms.
    
    Provides automatic algorithm selection, dual-mode optimization,
    and consistent interface for all compression operations.
    """
    
    var default_compression: CompressionType
    var binary_quantizer: BinaryQuantizer[dtype]
    var auto_select: Bool           # Automatic algorithm selection
    var embedded_mode: Bool         # Optimize for embedded deployment
    
    fn __init__(out self, 
                compression_type: CompressionType = CompressionType.Binary,
                auto_select: Bool = True,
                embedded_mode: Bool = True):
        """Initialize compression manager."""
        self.default_compression = compression_type
        self.auto_select = auto_select
        self.embedded_mode = embedded_mode
        
        # Configure binary quantizer based on mode
        var threshold_mode = "balanced"
        var batch_size = 1000
        
        if embedded_mode:
            threshold_mode = "balanced"  # Optimal for accuracy + compression
            batch_size = 100            # Smaller batches for embedded
        else:
            threshold_mode = "median"   # Optimize for server throughput
            batch_size = 10000         # Larger batches for server
            
        self.binary_quantizer = BinaryQuantizer[dtype](threshold_mode, batch_size)
        
    fn select_compression(self, vector: Vector[dtype]) -> CompressionType:
        """Automatically select optimal compression algorithm."""
        if not self.auto_select:
            return self.default_compression
            
        # Analyze vector characteristics for optimal compression
        var sparsity = self._calculate_sparsity(vector)
        var dynamic_range = self._calculate_dynamic_range(vector)
        
        # Selection logic based on vector characteristics
        if sparsity > 0.7:  # High sparsity - binary quantization works well
            return CompressionType.Binary
        elif dynamic_range < 2.0:  # Low dynamic range - scalar quantization
            return CompressionType.Scalar
        elif vector.dim > 512:  # High-dimensional - product quantization
            return CompressionType.Product
        else:
            return CompressionType.Binary  # Default to binary for general case
            
    fn compress_vector(self, vector: Vector[dtype], 
                      compression_type: CompressionType = CompressionType.None) -> CompressedVector[dtype]:
        """Compress vector using specified or auto-selected algorithm."""
        var comp_type = compression_type
        if comp_type == CompressionType.None:
            comp_type = self.select_compression(vector)
            
        if comp_type == CompressionType.Binary:
            return self._compress_binary(vector)
        elif comp_type == CompressionType.Scalar:
            return self._compress_scalar(vector)
        elif comp_type == CompressionType.Product:
            return self._compress_product(vector)
        else:
            # No compression - store as-is
            return self._store_uncompressed(vector)
            
    fn decompress_vector(self, compressed: CompressedVector[dtype]) -> Vector[dtype]:
        """Decompress vector using appropriate algorithm."""
        if compressed.compression_type == CompressionType.Binary:
            return self._decompress_binary(compressed)
        elif compressed.compression_type == CompressionType.Scalar:
            return self._decompress_scalar(compressed)
        elif compressed.compression_type == CompressionType.Product:
            return self._decompress_product(compressed)
        else:
            return self._decompress_uncompressed(compressed)
            
    fn compress_batch(self, vectors: List[Vector[dtype]]) -> List[CompressedVector[dtype]]:
        """Compress batch of vectors for server mode efficiency."""
        var compressed = List[CompressedVector[dtype]]()
        
        if self.embedded_mode:
            # Process individually for embedded mode
            for i in range(len(vectors)):
                compressed.append(self.compress_vector(vectors[i]))
        else:
            # Batch processing for server mode
            # For now, process individually but could be optimized for batch algorithms
            for i in range(len(vectors)):
                compressed.append(self.compress_vector(vectors[i]))
                
        return compressed
        
    fn _calculate_sparsity(self, vector: Vector[dtype]) -> Float64:
        """Calculate vector sparsity (percentage of near-zero values)."""
        var near_zero_count = 0
        var threshold = Float64(0.01)  # Values below this considered near-zero
        
        for i in range(vector.dim):
            if abs(Float64(vector.data[i])) < threshold:
                near_zero_count += 1
                
        return Float64(near_zero_count) / Float64(vector.dim)
        
    fn _calculate_dynamic_range(self, vector: Vector[dtype]) -> Float64:
        """Calculate vector dynamic range (max/min ratio)."""
        var min_val = Float64(vector.data[0])
        var max_val = Float64(vector.data[0])
        
        for i in range(1, vector.dim):
            var val = Float64(vector.data[i])
            if val < min_val:
                min_val = val
            if val > max_val:
                max_val = val
                
        if abs(min_val) > 0.0:
            return max_val / abs(min_val)
        else:
            return max_val
            
    fn _compress_binary(self, vector: Vector[dtype]) -> CompressedVector[dtype]:
        """Compress using binary quantization."""
        var binary_quantized = self.binary_quantizer.quantize_vector(vector)
        var compressed = CompressedVector[dtype](CompressionType.Binary, vector.dim)
        
        # Calculate data size needed
        var binary_data_size = binary_quantized.num_words * 8  # 8 bytes per UInt64
        var metadata_size = 16  # 8 bytes for norm + 8 bytes for num_words
        
        # Allocate and copy binary data
        compressed.data_size = binary_data_size
        compressed.data = UnsafePointer[UInt8].alloc(binary_data_size)
        var binary_ptr = binary_quantized.packed_data.bitcast[UInt8]()
        memcpy(compressed.data, binary_ptr, binary_data_size)
        
        # Store metadata (norm and num_words)
        compressed.metadata_size = metadata_size
        compressed.metadata = UnsafePointer[UInt8].alloc(metadata_size)
        var norm_ptr = UnsafePointer[Float64](compressed.metadata.bitcast[Float64]())
        norm_ptr[0] = binary_quantized.norm
        var words_ptr = UnsafePointer[Int](compressed.metadata.offset(8).bitcast[Int]())
        words_ptr[0] = binary_quantized.num_words
        
        return compressed
        
    fn _decompress_binary(self, compressed: CompressedVector[dtype]) -> Vector[dtype]:
        """Decompress binary quantized vector."""
        # Extract metadata
        var norm_ptr = UnsafePointer[Float64](compressed.metadata.bitcast[Float64]())
        var norm = norm_ptr[0]
        var words_ptr = UnsafePointer[Int](compressed.metadata.offset(8).bitcast[Int]())
        var num_words = words_ptr[0]
        
        # Reconstruct binary quantized vector
        var binary_quantized = BinaryQuantizedVector[dtype](compressed.original_dim)
        binary_quantized.norm = norm
        binary_quantized.num_words = num_words
        
        # Copy packed data
        var binary_ptr = compressed.data.bitcast[UInt64]()
        memcpy(binary_quantized.packed_data, binary_ptr, num_words * 8)
        
        # Convert to approximate dense vector (simplified reconstruction)
        var result = Vector[dtype](compressed.original_dim)
        var scale = norm / sqrt(Float64(compressed.original_dim))  # Approximate scaling
        
        for i in range(compressed.original_dim):
            var word_idx = i // 64
            var bit_idx = i % 64
            var word = binary_quantized.packed_data[word_idx]
            var bit_set = (word >> UInt64(bit_idx)) & UInt64(1)
            
            # Simple reconstruction: 1 -> +scale, 0 -> -scale
            if bit_set:
                result.data[i] = Scalar[dtype](scale)
            else:
                result.data[i] = Scalar[dtype](-scale)
        
        return result
        
    fn _compress_scalar(self, vector: Vector[dtype]) -> CompressedVector[dtype]:
        """Compress using scalar quantization (placeholder)."""
        # TODO: Implement scalar quantization
        return self._store_uncompressed(vector)
        
    fn _decompress_scalar(self, compressed: CompressedVector[dtype]) -> Vector[dtype]:
        """Decompress scalar quantized vector (placeholder)."""
        # TODO: Implement scalar decompression
        return self._decompress_uncompressed(compressed)
        
    fn _compress_product(self, vector: Vector[dtype]) -> CompressedVector[dtype]:
        """Compress using product quantization (placeholder)."""
        # TODO: Implement product quantization
        return self._store_uncompressed(vector)
        
    fn _decompress_product(self, compressed: CompressedVector[dtype]) -> Vector[dtype]:
        """Decompress product quantized vector (placeholder)."""
        # TODO: Implement product decompression
        return self._decompress_uncompressed(compressed)
        
    fn _store_uncompressed(self, vector: Vector[dtype]) -> CompressedVector[dtype]:
        """Store vector without compression."""
        var compressed = CompressedVector[dtype](CompressionType.None, vector.dim)
        
        var data_size = vector.dim * sizeof[Scalar[dtype]]()
        compressed.data_size = data_size
        compressed.data = UnsafePointer[UInt8].alloc(data_size)
        
        var vector_ptr = vector.data.bitcast[UInt8]()
        memcpy(compressed.data, vector_ptr, data_size)
        
        return compressed
        
    fn _decompress_uncompressed(self, compressed: CompressedVector[dtype]) -> Vector[dtype]:
        """Restore uncompressed vector."""
        var result = Vector[dtype](compressed.original_dim)
        var result_ptr = result.data.bitcast[UInt8]()
        memcpy(result_ptr, compressed.data, compressed.data_size)
        return result


# Convenience functions for dual-mode usage
fn create_embedded_compression_manager[dtype: DType]() -> CompressionManager[dtype]:
    """Create compression manager optimized for embedded mode."""
    return CompressionManager[dtype](
        CompressionType.Binary,
        auto_select=True,
        embedded_mode=True
    )

fn create_server_compression_manager[dtype: DType]() -> CompressionManager[dtype]:
    """Create compression manager optimized for server mode."""
    return CompressionManager[dtype](
        CompressionType.Binary,
        auto_select=True,
        embedded_mode=False
    )