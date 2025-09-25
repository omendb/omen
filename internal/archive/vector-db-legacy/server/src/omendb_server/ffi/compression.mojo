"""
ZSTD compression FFI interface for OmenDB
Provides high-performance compression with pure Mojo fallback
"""

from sys.ffi import external_call
from memory import Pointer, UnsafePointer
from builtin import UInt8, Int, Bool
from collections import List


struct CompressedBuffer:
    """Buffer holding compressed data with metadata"""
    
    var data: UnsafePointer[UInt8]
    var size: Int
    var original_size: Int
    var compression_ratio: Float32
    
    fn __init__(inout self, data: UnsafePointer[UInt8], size: Int, original_size: Int):
        self.data = data
        self.size = size
        self.original_size = original_size
        self.compression_ratio = Float32(original_size) / Float32(size) if size > 0 else 1.0
    
    fn __del__(owned self):
        """Free compressed buffer memory"""
        if self.data:
            self.data.free()


struct ZSTDCompressor:
    """ZSTD compression interface with fallback to pure Mojo"""
    
    var compression_level: Int
    var use_ffi: Bool
    
    fn __init__(inout self, compression_level: Int = 3):
        """Initialize ZSTD compressor
        
        Args:
            compression_level: ZSTD compression level (1-22, default 3)
        """
        self.compression_level = compression_level
        self.use_ffi = self._check_zstd_availability()
    
    fn _check_zstd_availability(self) -> Bool:
        """Check if ZSTD FFI is available"""
        try:
            # Attempt to call ZSTD version function to test availability
            let version = external_call["ZSTD_versionNumber", Int]()
            return version > 0
        except:
            return False
    
    fn compress(self, data: UnsafePointer[UInt8], size: Int) -> CompressedBuffer:
        """Compress data using ZSTD or pure Mojo fallback
        
        Args:
            data: Input data buffer
            size: Size of input data
            
        Returns:
            CompressedBuffer with compressed data
        """
        if self.use_ffi:
            return self._compress_zstd(data, size)
        else:
            return self._compress_fallback(data, size)
    
    fn _compress_zstd(self, data: UnsafePointer[UInt8], size: Int) -> CompressedBuffer:
        """Compress using ZSTD FFI"""
        try:
            # Get maximum compressed size bound
            let max_compressed_size = external_call["ZSTD_compressBound", Int](size)
            
            # Allocate output buffer
            let compressed_data = UnsafePointer[UInt8].alloc(max_compressed_size)
            
            # Perform compression
            let compressed_size = external_call["ZSTD_compress", Int](
                compressed_data.address,
                max_compressed_size,
                data.address,
                size,
                self.compression_level
            )
            
            if compressed_size <= 0:
                # Compression failed, use fallback
                compressed_data.free()
                return self._compress_fallback(data, size)
            
            # Reallocate to actual size to save memory
            let final_data = UnsafePointer[UInt8].alloc(compressed_size)
            for i in range(compressed_size):
                final_data[i] = compressed_data[i]
            compressed_data.free()
            
            return CompressedBuffer(final_data, compressed_size, size)
            
        except:
            # FFI call failed, use fallback
            return self._compress_fallback(data, size)
    
    fn _compress_fallback(self, data: UnsafePointer[UInt8], size: Int) -> CompressedBuffer:
        """Pure Mojo compression fallback (simple RLE for now)"""
        # For now, implement a simple run-length encoding as fallback
        # In production, this could be a more sophisticated pure Mojo compressor
        
        let compressed_data = UnsafePointer[UInt8].alloc(size * 2)  # Worst case
        var write_pos = 0
        var read_pos = 0
        
        while read_pos < size:
            let current_byte = data[read_pos]
            var run_length = 1
            
            # Count consecutive identical bytes
            while read_pos + run_length < size and data[read_pos + run_length] == current_byte and run_length < 255:
                run_length += 1
            
            if run_length >= 3:  # Use RLE for runs of 3 or more
                compressed_data[write_pos] = 0xFF  # RLE marker
                compressed_data[write_pos + 1] = UInt8(run_length)
                compressed_data[write_pos + 2] = current_byte
                write_pos += 3
            else:  # Store literally
                for i in range(run_length):
                    compressed_data[write_pos] = data[read_pos + i]
                    write_pos += 1
            
            read_pos += run_length
        
        # Reallocate to actual size
        let final_data = UnsafePointer[UInt8].alloc(write_pos)
        for i in range(write_pos):
            final_data[i] = compressed_data[i]
        compressed_data.free()
        
        return CompressedBuffer(final_data, write_pos, size)
    
    fn decompress(self, compressed: CompressedBuffer) -> UnsafePointer[UInt8]:
        """Decompress data back to original form
        
        Args:
            compressed: CompressedBuffer to decompress
            
        Returns:
            Pointer to decompressed data
        """
        if self.use_ffi:
            return self._decompress_zstd(compressed)
        else:
            return self._decompress_fallback(compressed)
    
    fn _decompress_zstd(self, compressed: CompressedBuffer) -> UnsafePointer[UInt8]:
        """Decompress using ZSTD FFI"""
        try:
            let decompressed_data = UnsafePointer[UInt8].alloc(compressed.original_size)
            
            let result_size = external_call["ZSTD_decompress", Int](
                decompressed_data.address,
                compressed.original_size,
                compressed.data.address,
                compressed.size
            )
            
            if result_size != compressed.original_size:
                decompressed_data.free()
                return self._decompress_fallback(compressed)
            
            return decompressed_data
            
        except:
            return self._decompress_fallback(compressed)
    
    fn _decompress_fallback(self, compressed: CompressedBuffer) -> UnsafePointer[UInt8]:
        """Pure Mojo decompression fallback (RLE decoder)"""
        let decompressed_data = UnsafePointer[UInt8].alloc(compressed.original_size)
        var write_pos = 0
        var read_pos = 0
        
        while read_pos < compressed.size and write_pos < compressed.original_size:
            if compressed.data[read_pos] == 0xFF:  # RLE marker
                let run_length = Int(compressed.data[read_pos + 1])
                let byte_value = compressed.data[read_pos + 2]
                
                for i in range(run_length):
                    if write_pos < compressed.original_size:
                        decompressed_data[write_pos] = byte_value
                        write_pos += 1
                
                read_pos += 3
            else:  # Literal byte
                decompressed_data[write_pos] = compressed.data[read_pos]
                write_pos += 1
                read_pos += 1
        
        return decompressed_data


fn get_compression_stats() -> String:
    """Get compression library status and capabilities"""
    let compressor = ZSTDCompressor()
    if compressor.use_ffi:
        return "ZSTD FFI available - high-performance compression enabled"
    else:
        return "ZSTD FFI unavailable - using pure Mojo fallback compression"


# Global compressor instance for convenience
let DEFAULT_COMPRESSOR = ZSTDCompressor()