"""
Mixed Precision Processing for OmenDB

Implements float16 and int8 support for memory and speed optimization.
Provides automatic precision detection, conversion, and validation.

Memory Benefits:
- float16: 50% memory reduction vs float32
- int8: 75% memory reduction vs float32  
- Maintains accuracy through careful quantization

Performance Benefits:
- Hardware acceleration on modern CPUs/GPUs
- Increased memory bandwidth utilization
- Support for larger datasets in memory-constrained environments
"""

from math import sqrt, pow
from memory import UnsafePointer
from algorithm import vectorize
from collections import List
from sys.info import simdwidthof
from .vector import Vector
from .distance import DistanceMetric


# ========================================
# Precision Management Types
# ========================================

struct PrecisionType(Copyable, Movable):
    """Enumeration of supported precision types."""
    
    alias FLOAT32 = 0  # Standard precision (baseline)
    alias FLOAT16 = 1  # Half precision (50% memory reduction)
    alias INT8 = 2     # 8-bit integer (75% memory reduction)
    alias INT16 = 3    # 16-bit integer (50% memory reduction)
    
    var value: Int
    
    fn __init__(out self, value: Int = 0):
        self.value = value
    
    fn __eq__(self, other: Self) -> Bool:
        return self.value == other.value
    
    fn get_memory_factor(self) -> Float32:
        """Get memory usage factor relative to float32."""
        if self.value == Self.FLOAT32:
            return 1.0
        elif self.value == Self.FLOAT16:
            return 0.5
        elif self.value == Self.INT8:
            return 0.25
        elif self.value == Self.INT16:
            return 0.5
        else:
            return 1.0


struct QuantizationConfig(Copyable, Movable):
    """Configuration for quantization parameters."""
    
    var scale: Float32
    var zero_point: Float32
    var min_value: Float32
    var max_value: Float32
    var precision_type: PrecisionType
    
    fn __init__(
        out self,
        scale: Float32 = 1.0,
        zero_point: Float32 = 0.0,
        min_value: Float32 = -1.0,
        max_value: Float32 = 1.0,
        precision_type: PrecisionType = PrecisionType(PrecisionType.FLOAT32)
    ):
        self.scale = scale
        self.zero_point = zero_point
        self.min_value = min_value
        self.max_value = max_value
        self.precision_type = precision_type


# ========================================
# Mixed Precision Vector Support
# ========================================

struct MixedPrecisionVector[base_dtype: DType = DType.float32](Copyable, Movable):
    """
    Vector that supports multiple precision formats for memory optimization.
    
    Features:
    - Automatic quantization and dequantization
    - Memory usage tracking and optimization
    - Precision-aware distance calculations
    - Fallback to higher precision when needed
    """
    
    var data_f32: UnsafePointer[Scalar[DType.float32]]
    var data_f16: UnsafePointer[Scalar[DType.float16]]  
    var data_i8: UnsafePointer[Scalar[DType.int8]]
    var dim: Int
    var active_precision: PrecisionType
    var quantization_config: QuantizationConfig
    
    fn __init__(out self, dimension: Int):
        """Initialize mixed precision vector with given dimension."""
        self.dim = dimension
        self.active_precision = PrecisionType(PrecisionType.FLOAT32)
        self.quantization_config = QuantizationConfig()
        
        # Allocate memory for all precision types
        self.data_f32 = UnsafePointer[Scalar[DType.float32]].alloc(dimension)
        self.data_f16 = UnsafePointer[Scalar[DType.float16]].alloc(dimension)
        self.data_i8 = UnsafePointer[Scalar[DType.int8]].alloc(dimension)
        
        # Initialize with zeros
        for i in range(dimension):
            self.data_f32[i] = 0.0
            self.data_f16[i] = 0.0
            self.data_i8[i] = 0
    
    fn __init__(out self, from_vector: Vector[base_dtype]):
        """Initialize from existing Vector."""
        self.dim = from_vector.dimension()
        self.active_precision = PrecisionType(PrecisionType.FLOAT32)
        self.quantization_config = QuantizationConfig()
        
        # Allocate memory
        self.data_f32 = UnsafePointer[Scalar[DType.float32]].alloc(self.dim)
        self.data_f16 = UnsafePointer[Scalar[DType.float16]].alloc(self.dim)
        self.data_i8 = UnsafePointer[Scalar[DType.int8]].alloc(self.dim)
        
        # Copy data from source vector
        for i in range(self.dim):
            var value = from_vector[i].cast[DType.float32]()
            self.data_f32[i] = value
            self.data_f16[i] = value.cast[DType.float16]()
            self.data_i8[i] = Int8(value * 127.0).cast[DType.int8]()  # Simple quantization
    
    fn __del__(owned self):
        """Clean up allocated memory."""
        if self.data_f32:
            self.data_f32.free()
        if self.data_f16:
            self.data_f16.free()
        if self.data_i8:
            self.data_i8.free()
    
    fn dimension(self) -> Int:
        """Get vector dimension."""
        return self.dim
    
    fn get_memory_usage(self) -> Int:
        """Get current memory usage in bytes."""
        var base_size = self.dim * 4  # float32 baseline
        if self.active_precision.value == PrecisionType.FLOAT16:
            return self.dim * 2
        elif self.active_precision.value == PrecisionType.INT8:
            return self.dim * 1
        else:
            return base_size
    
    fn get_memory_savings(self) -> Float32:
        """Get memory savings percentage vs float32."""
        var base_memory = self.dim * 4  # float32
        var current_memory = self.get_memory_usage()
        return (1.0 - Float32(current_memory) / Float32(base_memory)) * 100.0


# ========================================
# Quantization Operations
# ========================================

struct QuantizationOperations:
    """Operations for quantizing and dequantizing vectors."""
    
    @staticmethod
    fn analyze_data_range[
        dtype: DType
    ](data: UnsafePointer[Scalar[dtype]], dim: Int) -> QuantizationConfig:
        """Analyze data range to determine optimal quantization parameters."""
        var min_val = Float32.MAX
        var max_val = Float32.MIN
        
        # Find min/max values
        for i in range(dim):
            var value = data[i].cast[DType.float32]()
            if value < min_val:
                min_val = value
            if value > max_val:
                max_val = value
        
        # Calculate scale and zero point for int8 quantization
        var range_val = max_val - min_val
        var scale = range_val / 255.0  # 8-bit range
        var zero_point = -min_val / scale
        
        return QuantizationConfig(
            scale=scale,
            zero_point=zero_point,
            min_value=min_val,
            max_value=max_val,
            precision_type=PrecisionType(PrecisionType.INT8)
        )
    
    @staticmethod
    fn quantize_to_int8[
        dtype: DType
    ](
        source: UnsafePointer[Scalar[dtype]],
        target: UnsafePointer[Scalar[DType.int8]],
        dim: Int,
        config: QuantizationConfig
    ):
        """Quantize float32/16 data to int8."""
        alias simd_width = simdwidthof[DType.float32]()
        
        @parameter
        fn quantize_chunk[width: Int](offset: Int):
            # Load source data
            var source_vec = source.load[width=width](offset).cast[DType.float32]()
            
            # Apply quantization: (value - min) / scale
            var normalized = (source_vec - config.min_value) / config.scale
            
            # Clamp to [0, 255] and convert to int8
            var clamped = normalized.clamp(0.0, 255.0)
            var quantized = clamped.cast[DType.int8]()
            
            # Store result
            target.store(offset, quantized)
        
        # Vectorized quantization
        var vectorized_end = (dim // simd_width) * simd_width
        vectorize[quantize_chunk, simd_width](vectorized_end)
        
        # Handle remaining elements
        for i in range(vectorized_end, dim):
            var value = source[i].cast[DType.float32]()
            var normalized = (value - config.min_value) / config.scale
            var clamped = max(0.0, min(255.0, normalized))
            target[i] = Int8(clamped).cast[DType.int8]()
    
    @staticmethod
    fn dequantize_from_int8(
        source: UnsafePointer[Scalar[DType.int8]],
        target: UnsafePointer[Scalar[DType.float32]],
        dim: Int,
        config: QuantizationConfig
    ):
        """Dequantize int8 data back to float32."""
        alias simd_width = simdwidthof[DType.float32]()
        
        @parameter
        fn dequantize_chunk[width: Int](offset: Int):
            # Load quantized data
            var quantized_vec = source.load[width=width](offset).cast[DType.float32]()
            
            # Apply dequantization: value * scale + min
            var dequantized = quantized_vec * config.scale + config.min_value
            
            # Store result
            target.store(offset, dequantized.cast[DType.float32]())
        
        # Vectorized dequantization
        var vectorized_end = (dim // simd_width) * simd_width
        vectorize[dequantize_chunk, simd_width](vectorized_end)
        
        # Handle remaining elements
        for i in range(vectorized_end, dim):
            var quantized = source[i].cast[DType.float32]()
            var dequantized = quantized * config.scale + config.min_value
            target[i] = dequantized


# ========================================
# Mixed Precision Distance Calculations
# ========================================

struct MixedPrecisionDistance:
    """Distance calculations optimized for mixed precision data."""
    
    @staticmethod
    fn l2_distance_int8(
        a: UnsafePointer[Scalar[DType.int8]],
        b: UnsafePointer[Scalar[DType.int8]],
        dim: Int,
        config_a: QuantizationConfig,
        config_b: QuantizationConfig
    ) -> Float32:
        """Fast L2 distance calculation for int8 vectors."""
        var distance_squared = Float32(0.0)
        alias simd_width = simdwidthof[DType.int8]()
        
        @parameter
        fn distance_chunk[width: Int](offset: Int):
            # Load int8 data
            var a_vec = a.load[width=width](offset).cast[DType.float32]()
            var b_vec = b.load[width=width](offset).cast[DType.float32]()
            
            # Dequantize during calculation
            var a_dequant = a_vec * config_a.scale + config_a.min_value
            var b_dequant = b_vec * config_b.scale + config_b.min_value
            
            # Compute squared differences
            var diff = a_dequant - b_dequant
            var diff_sq = diff * diff
            
            # Accumulate (manual reduction)
            for i in range(width):
                distance_squared += diff_sq[i]
        
        # Vectorized calculation
        var vectorized_end = (dim // simd_width) * simd_width
        vectorize[distance_chunk, simd_width](vectorized_end)
        
        # Handle remaining elements
        for i in range(vectorized_end, dim):
            var a_val = a[i].cast[DType.float32]()
            var b_val = b[i].cast[DType.float32]()
            
            # Dequantize
            var a_dequant = a_val * config_a.scale + config_a.min_value
            var b_dequant = b_val * config_b.scale + config_b.min_value
            
            var diff = a_dequant - b_dequant
            distance_squared += diff * diff
        
        return sqrt(distance_squared)
    
    @staticmethod
    fn cosine_similarity_int8(
        a: UnsafePointer[Scalar[DType.int8]],
        b: UnsafePointer[Scalar[DType.int8]],
        dim: Int,
        config_a: QuantizationConfig,
        config_b: QuantizationConfig
    ) -> Float32:
        """Fast cosine similarity for int8 vectors."""
        var dot_product = Float32(0.0)
        var norm_a_squared = Float32(0.0)
        var norm_b_squared = Float32(0.0)
        
        # Calculate dot product and norms simultaneously
        for i in range(dim):
            var a_val = a[i].cast[DType.float32]()
            var b_val = b[i].cast[DType.float32]()
            
            # Dequantize
            var a_dequant = a_val * config_a.scale + config_a.min_value
            var b_dequant = b_val * config_b.scale + config_b.min_value
            
            dot_product += a_dequant * b_dequant
            norm_a_squared += a_dequant * a_dequant
            norm_b_squared += b_dequant * b_dequant
        
        # Avoid division by zero
        var norm_product = sqrt(norm_a_squared * norm_b_squared)
        if norm_product < 1e-8:
            return 0.0
        
        return dot_product / norm_product


# ========================================
# Precision Optimization Analysis
# ========================================

struct PrecisionAnalyzer:
    """Analyzes optimal precision settings for given data."""
    
    @staticmethod
    fn analyze_vector[
        dtype: DType
    ](vector: Vector[dtype]) -> PrecisionType:
        """Analyze vector to recommend optimal precision."""
        var data = vector.data
        var dim = vector.dimension()
        
        # Calculate statistics
        var min_val = Float32.MAX
        var max_val = Float32.MIN
        var mean = Float32(0.0)
        
        for i in range(dim):
            var value = data[i].cast[DType.float32]()
            min_val = min(min_val, value)
            max_val = max(max_val, value)
            mean += value
        
        mean /= Float32(dim)
        
        # Calculate variance
        var variance = Float32(0.0)
        for i in range(dim):
            var value = data[i].cast[DType.float32]()
            var diff = value - mean
            variance += diff * diff
        
        variance /= Float32(dim)
        var std_dev = sqrt(variance)
        
        # Decision logic based on data characteristics
        var range_val = max_val - min_val
        
        # If values are small and range is limited, int8 is suitable
        if range_val <= 2.0 and abs(mean) <= 1.0 and std_dev <= 0.5:
            return PrecisionType(PrecisionType.INT8)
        
        # If values have moderate range, float16 is suitable
        elif range_val <= 10.0 and abs(mean) <= 5.0:
            return PrecisionType(PrecisionType.FLOAT16)
        
        # Otherwise, keep float32
        else:
            return PrecisionType(PrecisionType.FLOAT32)
    
    @staticmethod
    fn calculate_accuracy_loss[
        dtype: DType
    ](
        original: Vector[dtype],
        precision_type: PrecisionType
    ) -> Float32:
        """Calculate accuracy loss when using lower precision."""
        if precision_type.value == PrecisionType.FLOAT32:
            return 0.0  # No loss
        
        # Create mixed precision version
        var mixed = MixedPrecisionVector(original)
        
        # Calculate MSE between original and quantized
        var mse = Float32(0.0)
        var dim = original.dimension()
        
        for i in range(dim):
            var orig_val = original[i].cast[DType.float32]()
            var quantized_val: Float32
            
            if precision_type.value == PrecisionType.FLOAT16:
                quantized_val = mixed.data_f16[i].cast[DType.float32]()
            else:  # INT8
                quantized_val = mixed.data_i8[i].cast[DType.float32]()
            
            var diff = orig_val - quantized_val
            mse += diff * diff
        
        mse /= Float32(dim)
        return sqrt(mse)  # RMSE


# ========================================
# Convenience Functions
# ========================================

fn create_mixed_precision_vector[
    dtype: DType = DType.float32
](data: List[Float32]) -> MixedPrecisionVector[dtype]:
    """Create mixed precision vector from Python list."""
    var dim = len(data)
    var vector = MixedPrecisionVector[dtype](dim)
    
    for i in range(dim):
        vector.data_f32[i] = data[i]
        vector.data_f16[i] = data[i].cast[DType.float16]()
        vector.data_i8[i] = Int8(data[i] * 127.0).cast[DType.int8]()
    
    return vector


fn analyze_precision_benefits[
    dtype: DType = DType.float32
](vectors: List[Vector[dtype]]) -> String:
    """Analyze potential benefits of mixed precision for a dataset."""
    if len(vectors) == 0:
        return "No vectors to analyze"
    
    var int8_suitable = 0
    var float16_suitable = 0
    var total_memory_f32 = 0
    var total_memory_optimized = 0
    
    for i in range(len(vectors)):
        var vector = vectors[i]
        var recommended = PrecisionAnalyzer.analyze_vector(vector)
        var memory_f32 = vector.dimension() * 4
        
        total_memory_f32 += memory_f32
        
        if recommended.value == PrecisionType.INT8:
            int8_suitable += 1
            total_memory_optimized += vector.dimension() * 1
        elif recommended.value == PrecisionType.FLOAT16:
            float16_suitable += 1
            total_memory_optimized += vector.dimension() * 2
        else:
            total_memory_optimized += memory_f32
    
    var memory_savings = Float32(total_memory_f32 - total_memory_optimized) / Float32(total_memory_f32) * 100.0
    
    return "Memory Analysis: " + str(memory_savings) + "% potential savings, " + str(int8_suitable) + " vectors suitable for int8, " + str(float16_suitable) + " suitable for float16"