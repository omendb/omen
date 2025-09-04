"""
Product Quantization (PQ) for state-of-the-art vector compression.

Implements Microsoft DiskANN-style PQ32 compression:
- 128D vectors compressed to 32 bytes (4D per subspace, 32 subspaces)
- K=256 centroids per subspace (1 byte code)
- Total storage: 32 bytes + metadata vs 512 bytes for Float32
- Achieves 16x compression with minimal accuracy loss

Memory target: 148 bytes/vector total (32 PQ + 100 graph + 16 metadata)
"""

from collections import List
from algorithm import vectorize
from sys.info import simdwidthof
from memory import UnsafePointer, memcpy, memset_zero
from math import sqrt, pow
from random import seed, random_float64, random_ui64

alias PQ_K = 256  # Number of centroids per subspace (1 byte codes)
alias SIMD_WIDTH = simdwidthof[DType.float32]()

struct PQVector(Copyable, Movable):
    """Product Quantized vector - compressed representation."""
    
    var codes: List[UInt8]     # PQ codes (M bytes total)
    var dimension: Int         # Original dimension
    var M: Int                 # Number of subspaces
    
    fn __init__(out self, codes: List[UInt8], dimension: Int, M: Int):
        """Initialize PQ vector with codes."""
        self.codes = codes
        self.dimension = dimension
        self.M = M
    
    fn memory_usage(self) -> Int:
        """Memory usage in bytes."""
        return self.M + 8  # codes + metadata (dimension, M)

struct PQCompressor(Copyable, Movable):
    """Product Quantization compressor with codebook training."""
    
    var M: Int                  # Number of subspaces
    var K: Int                  # Number of centroids per subspace (256)
    var dim: Int                # Original dimension
    var subspace_dim: Int       # Dimensions per subspace (dim / M)
    var codebooks: UnsafePointer[Float32]  # M * K * subspace_dim
    var distance_table: UnsafePointer[Float32]  # Precomputed distances for search
    var trained: Bool           # Whether codebooks are trained
    
    fn __init__(out self, M: Int, dim: Int, K: Int = PQ_K):
        """Initialize PQ compressor.
        
        Args:
            M: Number of subspaces (e.g., 32 for PQ32)
            dim: Original vector dimension (must be divisible by M)
            K: Centroids per subspace (256 for byte codes)
        """
        if dim % M != 0:
            print("Error: dimension", dim, "must be divisible by M =", M)
        
        self.M = M
        self.K = K
        self.dim = dim
        self.subspace_dim = dim // M
        self.trained = False
        
        # Allocate codebooks: M subspaces * K centroids * subspace_dim
        var codebook_size = M * K * self.subspace_dim
        self.codebooks = UnsafePointer[Float32].alloc(codebook_size)
        memset_zero(self.codebooks, codebook_size)
        
        # Allocate distance table for fast search
        var table_size = M * K  # For one query vector
        self.distance_table = UnsafePointer[Float32].alloc(table_size)
        memset_zero(self.distance_table, table_size)
    
    fn __del__(owned self):
        """Cleanup allocated memory."""
        if self.codebooks:
            self.codebooks.free()
        if self.distance_table:
            self.distance_table.free()
    
    fn train(mut self, vectors: UnsafePointer[Float32], n: Int, iterations: Int = 20):
        """Train codebooks using k-means clustering.
        
        Args:
            vectors: Training vectors (n * dim)
            n: Number of training vectors
            iterations: K-means iterations per subspace
        """
        if n < self.K:
            print("Warning: Need at least", self.K, "training vectors, got", n)
            return
        
        seed(42)  # Deterministic training
        
        # Train each subspace independently
        for m in range(self.M):
            var start_dim = m * self.subspace_dim
            var end_dim = start_dim + self.subspace_dim
            
            # Extract subspace data
            var subspace_data = UnsafePointer[Float32].alloc(n * self.subspace_dim)
            for i in range(n):
                for j in range(self.subspace_dim):
                    subspace_data[i * self.subspace_dim + j] = vectors[i * self.dim + start_dim + j]
            
            # Initialize centroids randomly from data points
            var centroids = self.codebooks + (m * self.K * self.subspace_dim)
            for k in range(self.K):
                var random_idx = Int(random_ui64(0, UInt64(n - 1)))
                for j in range(self.subspace_dim):
                    centroids[k * self.subspace_dim + j] = subspace_data[random_idx * self.subspace_dim + j]
            
            # K-means iterations
            var assignments = UnsafePointer[Int].alloc(n)
            for iter in range(iterations):
                # Assignment step
                for i in range(n):
                    var best_k = 0
                    var best_dist = Float32.MAX
                    
                    for k in range(self.K):
                        var dist = Float32(0)
                        for j in range(self.subspace_dim):
                            var diff = subspace_data[i * self.subspace_dim + j] - centroids[k * self.subspace_dim + j]
                            dist += diff * diff
                        
                        if dist < best_dist:
                            best_dist = dist
                            best_k = k
                    
                    assignments[i] = best_k
                
                # Update step
                var counts = UnsafePointer[Int].alloc(self.K)
                memset_zero(counts, self.K)
                
                # Zero centroids
                for k in range(self.K):
                    for j in range(self.subspace_dim):
                        centroids[k * self.subspace_dim + j] = 0.0
                
                # Accumulate
                for i in range(n):
                    var k = assignments[i]
                    counts[k] += 1
                    for j in range(self.subspace_dim):
                        centroids[k * self.subspace_dim + j] += subspace_data[i * self.subspace_dim + j]
                
                # Average
                for k in range(self.K):
                    if counts[k] > 0:
                        var inv_count = 1.0 / Float32(counts[k])
                        for j in range(self.subspace_dim):
                            centroids[k * self.subspace_dim + j] *= inv_count
                
                counts.free()
            
            assignments.free()
            subspace_data.free()
        
        self.trained = True
        print("PQ training complete:", self.M, "subspaces,", self.K, "centroids each")
    
    fn compress(self, vector: UnsafePointer[Float32]) -> PQVector:
        """Compress a vector using trained codebooks.
        
        Args:
            vector: Input vector (dim elements)
            
        Returns:
            PQVector with quantized codes
        """
        if not self.trained:
            print("Error: Codebooks not trained")
            return PQVector(List[UInt8](), self.dim, self.M)
        
        var codes = List[UInt8]()
        
        # Quantize each subspace
        for m in range(self.M):
            var start_dim = m * self.subspace_dim
            var centroids = self.codebooks + (m * self.K * self.subspace_dim)
            
            var best_k = 0
            var best_dist = Float32.MAX
            
            # Find nearest centroid in this subspace
            for k in range(self.K):
                var dist = Float32(0)
                for j in range(self.subspace_dim):
                    var diff = vector[start_dim + j] - centroids[k * self.subspace_dim + j]
                    dist += diff * diff
                
                if dist < best_dist:
                    best_dist = dist
                    best_k = k
            
            codes.append(UInt8(best_k))
        
        return PQVector(codes, self.dim, self.M)
    
    fn decompress(self, pq: PQVector) -> List[Float32]:
        """Reconstruct approximate vector from PQ codes.
        
        Args:
            pq: Product quantized vector
            
        Returns:
            Reconstructed Float32 vector (approximation)
        """
        if not self.trained or len(pq.codes) != self.M:
            return List[Float32]()
        
        var result = List[Float32]()
        
        # Reconstruct each subspace
        for m in range(self.M):
            var code = Int(pq.codes[m])
            var centroids = self.codebooks + (m * self.K * self.subspace_dim)
            var centroid = centroids + (code * self.subspace_dim)
            
            # Copy centroid values
            for j in range(self.subspace_dim):
                result.append(centroid[j])
        
        return result
    
    fn compute_distance_table(mut self, query: UnsafePointer[Float32]):
        """Precompute distance table for fast PQ distance computation.
        
        Args:
            query: Query vector to compute distances against
        """
        if not self.trained:
            return
        
        # For each subspace and centroid, compute squared distance
        for m in range(self.M):
            var start_dim = m * self.subspace_dim
            var centroids = self.codebooks + (m * self.K * self.subspace_dim)
            
            for k in range(self.K):
                var dist = Float32(0)
                var centroid = centroids + (k * self.subspace_dim)
                
                # Compute squared distance for this subspace
                for j in range(self.subspace_dim):
                    var diff = query[start_dim + j] - centroid[j]
                    dist += diff * diff
                
                self.distance_table[m * self.K + k] = dist
    
    fn compute_distance(self, pq: PQVector, query: UnsafePointer[Float32]) -> Float32:
        """Compute approximate L2 distance using precomputed table.
        
        This is the fast path for search - O(M) instead of O(dim).
        
        Args:
            pq: Product quantized vector
            query: Query vector (must call compute_distance_table first)
            
        Returns:
            Approximate L2 distance
        """
        if not self.trained or len(pq.codes) != self.M:
            return Float32.MAX
        
        var sum = Float32(0)
        
        # Sum distances from precomputed table
        for m in range(self.M):
            var code = Int(pq.codes[m])
            sum += self.distance_table[m * self.K + code]
        
        return sqrt(sum)
    
    fn compute_distance_direct(self, pq: PQVector, query: UnsafePointer[Float32]) -> Float32:
        """Compute distance without precomputed table (slower but works anywhere).
        
        Args:
            pq: Product quantized vector
            query: Query vector
            
        Returns:
            Approximate L2 distance
        """
        if not self.trained or len(pq.codes) != self.M:
            return Float32.MAX
        
        var sum = Float32(0)
        
        # Compute distance for each subspace
        for m in range(self.M):
            var start_dim = m * self.subspace_dim
            var code = Int(pq.codes[m])
            var centroids = self.codebooks + (m * self.K * self.subspace_dim)
            var centroid = centroids + (code * self.subspace_dim)
            
            # Add subspace distance
            for j in range(self.subspace_dim):
                var diff = query[start_dim + j] - centroid[j]
                sum += diff * diff
        
        return sqrt(sum)
    
    fn compression_ratio(self) -> Float32:
        """Get compression ratio vs Float32 storage."""
        var original_bytes = self.dim * 4  # Float32 = 4 bytes
        var compressed_bytes = self.M * 1  # UInt8 = 1 byte per code
        return Float32(original_bytes) / Float32(compressed_bytes)
    
    fn memory_overhead_per_vector(self) -> Int:
        """Memory overhead for PQ vs original vectors."""
        var codebook_memory = self.M * self.K * self.subspace_dim * 4  # Float32 codebooks
        var per_vector_overhead = self.M  # PQ codes per vector
        return per_vector_overhead + (codebook_memory // 1000)  # Amortized codebook cost

# Batch operations for efficient PQ processing
struct PQVectorBatch(Copyable, Movable):
    """Batch of PQ vectors for efficient search operations."""
    
    var vectors: List[PQVector]
    var compressor: PQCompressor
    var dimension: Int
    
    fn __init__(out self, compressor: PQCompressor):
        """Initialize batch with PQ compressor."""
        self.vectors = List[PQVector]()
        self.compressor = compressor
        self.dimension = compressor.dim
    
    fn add(mut self, pq: PQVector) -> Bool:
        """Add PQ vector to batch."""
        if pq.dimension != self.dimension:
            return False
        self.vectors.append(pq)
        return True
    
    fn search_batch(mut self, query: UnsafePointer[Float32], top_k: Int) -> List[Tuple[Int, Float32]]:
        """Fast batch search using precomputed distance tables.
        
        Args:
            query: Query vector
            top_k: Number of results to return
            
        Returns:
            List of (index, distance) tuples, sorted by distance
        """
        if len(self.vectors) == 0:
            return List[Tuple[Int, Float32]]()
        
        # Precompute distance table for this query
        self.compressor.compute_distance_table(query)
        
        var results = List[Tuple[Int, Float32]]()
        
        # Compute distances for all vectors
        for i in range(len(self.vectors)):
            var distance = self.compressor.compute_distance(self.vectors[i], query)
            results.append((i, distance))
        
        # Sort by distance (simple selection sort for now)
        for i in range(min(top_k, len(results))):
            var min_idx = i
            for j in range(i + 1, len(results)):
                if results[j][1] < results[min_idx][1]:
                    min_idx = j
            
            if min_idx != i:
                var temp = results[i]
                results[i] = results[min_idx]
                results[min_idx] = temp
        
        # Return top-k results
        var top_results = List[Tuple[Int, Float32]]()
        for i in range(min(top_k, len(results))):
            top_results.append(results[i])
        
        return top_results
    
    fn memory_usage(self) -> Int:
        """Total memory usage of batch."""
        var total = 0
        for i in range(len(self.vectors)):
            total += self.vectors[i].memory_usage()
        return total + self.compressor.memory_overhead_per_vector()

# Factory functions for common PQ configurations
fn create_pq32_compressor(dim: Int = 128) -> PQCompressor:
    """Create PQ32 compressor (Microsoft DiskANN standard).
    
    Args:
        dim: Vector dimension (default 128)
        
    Returns:
        PQCompressor configured for PQ32
    """
    return PQCompressor(M=32, dim=dim, K=256)

fn create_pq16_compressor(dim: Int = 128) -> PQCompressor:
    """Create PQ16 compressor (higher quality, less compression).
    
    Args:
        dim: Vector dimension (default 128)
        
    Returns:
        PQCompressor configured for PQ16
    """
    return PQCompressor(M=16, dim=dim, K=256)

fn create_pq64_compressor(dim: Int = 256) -> PQCompressor:
    """Create PQ64 compressor for higher dimensional vectors.
    
    Args:
        dim: Vector dimension (default 256)
        
    Returns:
        PQCompressor configured for PQ64
    """
    return PQCompressor(M=64, dim=dim, K=256)