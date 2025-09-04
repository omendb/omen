# Storage Specification - OmenDB Server

## Overview

OmenDB implements a sophisticated tiered storage architecture optimized specifically for vector data, leveraging SIMD operations, intelligent quantization, and adaptive algorithms to achieve industry-leading performance with cost efficiency.

## Architecture

### Storage Tiers

#### Hot Tier (1% of data)
- **Storage**: In-memory with scalar quantization
- **Index**: HNSW with optimized parameters
- **Access Time**: <1μs
- **Quantization**: 8-bit scalar (4x compression, <2% accuracy loss)
- **Cache**: L1 ultra-hot cache for top 0.1%
- **Implementation**: `hnsw.mojo` with SIMD optimizations

#### Warm Tier (9% of data)
- **Storage**: NVMe SSD with product quantization
- **Index**: IVF-PQ (better than HNSW for this tier)
- **Access Time**: <1ms
- **Quantization**: Product quantization (8-16x compression)
- **Compression**: Optional ZSTD with trained dictionaries
- **Implementation**: Asynchronous I/O with prefetching

#### Cold Tier (90% of data)
- **Storage**: S3-compatible object storage
- **Index**: Binary quantization + learned indices
- **Access Time**: <10ms
- **Quantization**: Binary (32x compression)
- **Compression**: ZSTD level 3 optimized for float arrays
- **Implementation**: Batched retrieval with local caching

### Index Algorithms

#### Algorithm Selection by Tier

```mojo
@value
struct IndexAlgorithm:
    var algorithm_type: String
    
    # Free tier algorithms (open source)
    alias BRUTE_FORCE = "brute_force"  # <1K vectors (embedded)
    alias HNSW = "hnsw"                # 1K-100K vectors (hot tier)
    
    # Platform tier algorithms (proprietary)
    alias ROARGRAPH = "roargraph"      # 5-10x faster batch construction
    alias IVF_PQ = "ivf_pq"            # Memory-efficient warm tier
    
    # Enterprise tier algorithms (proprietary)
    alias SPANN = "spann"              # Billion-scale cold tier

    @staticmethod
    fn auto_select(num_vectors: Int, tier: String, license: String = "free") -> IndexAlgorithm:
        if tier == "hot" and num_vectors < 1000:
            return IndexAlgorithm(Self.BRUTE_FORCE)
        elif tier == "hot":
            # Use RoarGraph for batch workloads in platform tier
            if license != "free" and num_vectors > 10000:
                return IndexAlgorithm(Self.ROARGRAPH)
            else:
                return IndexAlgorithm(Self.HNSW)
        elif tier == "warm" and license != "free":
            return IndexAlgorithm(Self.IVF_PQ)
        elif tier == "cold" and license == "enterprise":
            return IndexAlgorithm(Self.SPANN)
        else:
            # Fallback to HNSW for unsupported configurations
            return IndexAlgorithm(Self.HNSW)
```

#### HNSW Tuning Parameters

```mojo
struct HNSWParams:
    var M: Int                  # Number of bi-directional links
    var ef_construction: Int    # Size of dynamic candidate list
    var ef_search: Int          # Size of search candidate list
    
    @staticmethod
    fn optimized_for(num_vectors: Int, recall_target: Float32) -> HNSWParams:
        if recall_target > 0.95:
            return HNSWParams(M=32, ef_construction=500, ef_search=200)
        elif num_vectors > 1_000_000:
            return HNSWParams(M=16, ef_construction=200, ef_search=100)
        else:
            return HNSWParams(M=24, ef_construction=300, ef_search=150)
```

### Memory Layout

#### SIMD-Optimized Storage

```mojo
@align(64)  # Cache-line aligned
struct AlignedVectorBlock:
    var data: UnsafePointer[Float32]
    var dimension: Int
    var capacity: Int
    var padding: Int
    
    fn __init__(out self, dimension: Int, capacity: Int):
        # Ensure 64-byte alignment for AVX-512
        self.dimension = dimension
        self.capacity = capacity
        # Round up to 16 floats (64 bytes) for SIMD alignment
        var padded_dim = (dimension + 15) & ~15
        self.padding = padded_dim - dimension
        
        # Allocate aligned memory
        var total_floats = padded_dim * capacity
        self.data = UnsafePointer[Float32].aligned_alloc(64, total_floats)
        memset_zero(self.data, total_floats)
```

#### Memory Access Patterns
- **Contiguous storage** for sequential SIMD operations
- **64-byte alignment** for cache-line boundaries
- **Prefetching** for predictable access patterns
- **NUMA-aware** allocation for multi-socket systems

### Quantization Schemes

#### Scalar Quantization (Hot Tier)

```mojo
struct ScalarQuantizedVector:
    var values: List[UInt8]     # Quantized values
    var scale: Float32          # Scaling factor
    var offset: Float32         # Zero point
    var dimension: Int
    
    @staticmethod
    fn quantize(vector: List[Float32]) -> ScalarQuantizedVector:
        var min_val: Float32 = Float32.MAX
        var max_val: Float32 = Float32.MIN
        
        # Find min/max for quantization range
        for i in range(len(vector)):
            if vector[i] < min_val:
                min_val = vector[i]
            if vector[i] > max_val:
                max_val = vector[i]
        
        var scale = (max_val - min_val) / 255.0
        var offset = min_val
        
        # Quantize to 8-bit
        var quantized = List[UInt8]()
        for i in range(len(vector)):
            var normalized = (vector[i] - offset) / scale
            var quantized_val = round(normalized).cast[DType.uint8]()
            quantized.append(quantized_val)
        
        return ScalarQuantizedVector(quantized, scale, offset, len(vector))
    
    fn distance_to(self, other: Self) -> Float32:
        # SIMD-optimized distance computation on quantized values
        var sum: Float32 = 0.0
        
        @parameter
        fn simd_distance[simd_width: Int](idx: Int):
            var a = SIMD[DType.uint8, simd_width].load(self.values.data + idx)
            var b = SIMD[DType.uint8, simd_width].load(other.values.data + idx)
            var diff = (a - b).cast[DType.float32]()
            sum += (diff * diff).reduce_add()
        
        vectorize[simd_distance, simd_width](self.dimension)
        return sqrt(sum)
```

#### Product Quantization (Warm Tier)

```mojo
struct ProductQuantizer:
    var num_subvectors: Int
    var codebook_size: Int
    var codebooks: List[List[List[Float32]]]  # [subvector][code][dim]
    var subvector_dim: Int
    
    fn __init__(out self, dimension: Int, num_subvectors: Int = 8, codebook_size: Int = 256):
        self.num_subvectors = num_subvectors
        self.codebook_size = codebook_size
        self.subvector_dim = dimension // num_subvectors
        self.codebooks = List[List[List[Float32]]]()
    
    fn train(mut self, vectors: List[List[Float32]]):
        # K-means clustering to learn optimal codebooks per subvector
        for subvec_idx in range(self.num_subvectors):
            var subvector_data = self.extract_subvectors(vectors, subvec_idx)
            var codebook = self.kmeans_train(subvector_data, self.codebook_size)
            self.codebooks.append(codebook)
    
    fn encode(self, vector: List[Float32]) -> List[UInt8]:
        var codes = List[UInt8]()
        
        # Split vector and find nearest codebook entry per subvector
        for subvec_idx in range(self.num_subvectors):
            var start = subvec_idx * self.subvector_dim
            var end = start + self.subvector_dim
            
            var min_dist = Float32.MAX
            var best_code: UInt8 = 0
            
            # Find nearest codebook entry
            for code_idx in range(self.codebook_size):
                var dist = self.subvector_distance(
                    vector[start:end], 
                    self.codebooks[subvec_idx][code_idx]
                )
                if dist < min_dist:
                    min_dist = dist
                    best_code = code_idx
            
            codes.append(best_code)
        
        return codes
```

### Migration Policy

**Architecture**: Rust server coordinates migrations via PyO3 → Python → Mojo engine

#### Rust Server Layer (Coordination)
```rust
// server/src/storage.rs - Coordinates tier migrations
pub struct MigrationPolicy {
    pub hot_threshold: f32,        // Top 1% access frequency
    pub warm_threshold: f32,       // Next 9% access frequency
    pub hysteresis_factor: f32,    // 20% buffer to prevent thrashing
    pub decay_rate: f32,           // 0.99 daily decay
    pub min_age_before_demotion: Duration,  // 24 hours
}

// server/src/python_ffi.rs - PyO3 bridge to Mojo
impl StorageCoordinator {
    pub async fn migrate_vector(&self, vector_id: &str, from: Tier, to: Tier) -> Result<()> {
        Python::with_gil(|py| {
            let omendb = py.import("omendb.native")?;
            omendb.call_method(
                "migrate_vector",
                (vector_id, from.to_string(), to.to_string()),
                None
            )?;
            Ok(())
        })
    }
}
```

#### Python Bindings (Bridge)
```python
# omendb/api.py - Python API that Rust calls
def migrate_vector(vector_id: str, from_tier: str, to_tier: str) -> bool:
    """Called by Rust server via PyO3"""
    return _native.migrate_vector(vector_id, from_tier, to_tier)
```

#### Mojo Engine (Implementation)
```mojo
# omendb/algorithms/tiered_storage.mojo - Actual storage operations
struct TieredStorageEngine:
    var hot_tier: HNSWIndex[DType.float32]
    var warm_tier: IVFPQIndex
    var cold_tier: BinaryIndex
    
    fn migrate_vector(mut self, vector_id: String, from_tier: String, to_tier: String) -> Bool:
        # Extract vector from source tier
        var vector = self.get_vector_from_tier(vector_id, from_tier)
        if not vector:
            return False
            
        # Apply appropriate quantization for target tier
        if to_tier == "warm":
            vector = self.apply_product_quantization(vector)
        elif to_tier == "cold":
            vector = self.apply_binary_quantization(vector)
            
        # Insert into target tier with appropriate index
        var success = self.insert_to_tier(vector_id, vector, to_tier)
        
        # Remove from source tier if successful
        if success:
            self.remove_from_tier(vector_id, from_tier)
            
        return success
```

### Compression

**Note**: Compression is handled in the Mojo engine, coordinated by Rust server.

#### Mojo Implementation
```mojo
# omendb/algorithms/compression.mojo
struct VectorCompressor:
    var compression_level: Int
    var dictionary: Optional[List[UInt8]]
    
    fn __init__(out self):
        self.compression_level = 3  # Optimal for float arrays
        self.dictionary = None
    
    fn compress_vectors(self, vectors: List[List[Float32]]) -> List[UInt8]:
        # Convert float vectors to bytes
        # Apply ZSTD compression with trained dictionary
        # Return compressed byte stream
        pass
    
    fn train_dictionary(mut self, samples: List[List[Float32]]):
        # Train ZSTD dictionary on vector samples
        # 20-30% better compression with dictionary
        # Store 64KB optimized dictionary
        pass
```

#### Rust Coordination
```rust
// server/src/storage.rs - Coordinates compression
impl StorageCoordinator {
    pub async fn compress_cold_tier(&self) -> Result<()> {
        Python::with_gil(|py| {
            let omendb = py.import("omendb.native")?;
            omendb.call_method("compress_cold_tier", (), None)?;
            Ok(())
        })
    }
}
```

### Caching Strategy

```rust
pub struct TieredCache {
    l1_ultra_hot: LruCache<VectorId, Arc<Vector>>,     // 0.1% most accessed
    l2_hot: LruCache<VectorId, Arc<Vector>>,           // 1% frequently accessed
    negative_cache: BloomFilter,                        // Non-existent vectors
}

impl TieredCache {
    pub fn new() -> Self {
        Self {
            l1_ultra_hot: LruCache::new(1000),   // ~500KB for 128D
            l2_hot: LruCache::new(10000),        // ~5MB
            negative_cache: BloomFilter::new(1_000_000, 0.01),  // 1M items, 1% FPR
        }
    }
}
```

### Concurrent Access

```rust
pub struct ConcurrentIndex {
    // Read-Copy-Update pattern for lock-free reads
    current: Arc<RwLock<IndexState>>,
    // Write queue for serialized updates
    write_queue: Arc<Mutex<VecDeque<WriteOp>>>,
}

impl ConcurrentIndex {
    pub async fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        // Lock-free read path
        let index = self.current.read().await;
        index.search_immutable(query, k)
    }
    
    pub async fn update(&self, op: WriteOp) {
        // Queue write for batch processing
        self.write_queue.lock().await.push_back(op);
        
        // Process writes in background
        if self.should_process_writes() {
            self.process_write_batch().await;
        }
    }
}
```

## Performance Characteristics

### Throughput by Tier

| Tier | Insert (vec/s) | Query (qps) | Latency P99 |
|------|----------------|-------------|-------------|
| Hot  | 100K           | 50K         | <1ms        |
| Warm | 10K            | 10K         | <5ms        |
| Cold | 1K             | 1K          | <20ms       |

### Memory Efficiency

| Optimization | Compression | Accuracy Loss | Use Case |
|--------------|-------------|---------------|----------|
| No Quantization | 1x | 0% | Development |
| Scalar (8-bit) | 4x | <2% | Hot tier |
| Product (4-bit) | 8-16x | <5% | Warm tier |
| Binary (1-bit) | 32x | <20% | Cold tier |

## Implementation Status

### Completed
- ✅ Basic tiered storage structure
- ✅ HNSW implementation with SIMD
- ✅ Memory-aligned vector storage
- ✅ Access tracking and statistics

### In Progress
- 🔄 Scalar quantization for hot tier
- 🔄 RoarGraph algorithm integration
- 🔄 Migration policy implementation

### Planned
- 📋 Product quantization for warm tier
- 📋 ZSTD compression with dictionaries
- 📋 Concurrent access with RCU
- 📋 Advanced caching strategies

## Configuration

```yaml
storage:
  hot_tier:
    algorithm: hnsw
    quantization: scalar_8bit
    max_vectors: 1_000_000
    memory_limit: 8GB
    
  warm_tier:
    algorithm: ivf_pq
    quantization: pq_4bit
    storage_path: /mnt/nvme/omendb
    max_size: 1TB
    
  cold_tier:
    algorithm: spann
    quantization: binary
    s3_bucket: omendb-cold-tier
    compression: zstd_level3
    
  migration:
    hot_threshold: 0.99      # Top 1%
    warm_threshold: 0.90     # Top 10%
    hysteresis: 0.2          # 20% buffer
    decay_rate: 0.99         # Daily
    interval: 1h             # Check hourly
```

## API Integration

The storage layer integrates seamlessly with the server API:

```rust
// Transparent tier access
let results = storage.search(query, k).await?;

// Explicit tier control (enterprise)
let results = storage.search_tier(query, k, StorageTier::Hot).await?;

// Batch operations with optimal routing
storage.insert_batch(vectors, InsertStrategy::Optimized).await?;
```

## References

- HNSW Paper: [Efficient and robust approximate nearest neighbor search](https://arxiv.org/abs/1603.09320)
- Product Quantization: [Product quantization for nearest neighbor search](https://hal.inria.fr/inria-00514462v2/document)
- RoarGraph: [A Projected Bipartite Graph for Efficient Cross-Modal Approximate Nearest Neighbor Search](https://arxiv.org/abs/2408.08933)
- DiskANN: [DiskANN: Fast Accurate Billion-point Nearest Neighbor Search on a Single Node](https://papers.nips.cc/paper/2019/file/09853c7fb1d3f8ee67a61b6bf4a7f8e6-Paper.pdf)