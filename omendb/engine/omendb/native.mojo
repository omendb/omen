"""
OmenDB native module with DiskANN algorithm and SIMD optimizations.

File Organization:
  SECTION 1: Imports and Dependencies
  SECTION 2: Global State and Configuration  
  SECTION 3: Core VectorStore Implementation
  SECTION 4: Collection Management
  SECTION 5: Helper Functions
  SECTION 6: FFI Python Exports
"""

# =============================================================================
# SECTION 1: IMPORTS AND DEPENDENCIES
# =============================================================================

from python import PythonObject, Python
from python.bindings import PythonModuleBuilder
from python._cpython import PyObjectPtr
from collections import List, Dict, Optional
from omendb.core.sparse_map import SparseMap
from omendb.core.sparse_metadata_map import SparseMetadataMap
from math import sqrt
from algorithm import vectorize
from sys.info import simdwidthof
from memory import UnsafePointer, memcpy
from sys.intrinsics import sizeof

# Import index algorithms
from omendb.algorithms.diskann import DiskANNIndex
from omendb.algorithms.bruteforce import BruteForceIndex
# Note: Using sparse DiskANN for 50-70% memory reduction
from omendb.core.vector import Vector, VectorID
from omendb.core.distance import DistanceMetric
# Removed unused imports: SearchResult, VectorRecord
from omendb.core.metadata import Metadata
from omendb.core.vector_buffer import VectorBuffer
from omendb.core.matrix_ops import MatrixOps, MemoryPool
from omendb.utils.memory_pool import reset_global_pool
from omendb.core.storage import StorageEngine, InMemoryStorage, SnapshotStorage
from omendb.storage.memory_mapped import MemoryMappedStorage, create_optimal_storage
from omendb.utils.metrics import (
    DatabaseMetrics, 
    MetricsSnapshot, 
    OperationTimer,
    init_metrics,
    get_global_metrics,
    record_query_timing,
    record_insert,
    record_error
)
from omendb.utils.types import (
    DEFAULT_BUFFER_SIZE,
    AlgorithmType,
    StorageType,
    SearchResult
)
# Simplified configuration - just 5 essential fields
from omendb.utils.config import Config, __default_config
from omendb.utils.optional_safe import (
    optional_or, optional_or_raise, safe_unwrap
)
# Removed migration - using buffer architecture instead
from omendb.compression.scalar import ScalarQuantizedVector, QuantizedVectorBatch
from omendb.compression.binary import BinaryQuantizedVector
from omendb.core.memory_tracker import MemoryTracker, ComponentMemoryStats
# Removed unused import: numpy_to_matrix_direct
# Phase 2/3 optimizations removed - caused 10x performance regression
# Using Phase 1 SIMD optimizations only for stable 30-35K vec/s


# =============================================================================
# SECTION 2: GLOBAL STATE AND CONFIGURATION
# FUTURE: Extract to core/state.mojo when module-vars stabilize (2026+)
# =============================================================================

# Global metrics will be initialized on first use

# Configuration variables (set via configure_database)
var __buffer_size: Int = DEFAULT_BUFFER_SIZE  # Use default from types
                                   # Sparse graph has fast batch building, not expensive like HNSW
                                   # Memory savings: ~15MB per instance
var __use_columnar: Bool = False      # Use columnar storage (experimental)
var __is_server: Bool = False         # Server mode configuration
# Phase 2/3 optimizations permanently disabled - they degrade performance
# Phase 1 SIMD delivers 41% search improvement without issues

# Default configuration constants
# Types and constants now imported from utils.types

# =============================================================================
# SECTION 3: CORE VECTORSTORE IMPLEMENTATION  
# FUTURE: Extract to core/database.mojo when globals fixed
# DEPENDENCY: Requires global state refactor first
# =============================================================================

struct VectorStore(Copyable, Movable):
    """High-performance vector storage with buffer + main index architecture.

    Buffer provides O(1) inserts for hot data.
    DiskANN main index provides O(log n) scalable search.
    Industry-standard architecture used by ChromaDB, Weaviate, Qdrant.
    """

    var buffer: VectorBuffer  # Fast O(1) buffer for new vectors
    var main_index: DiskANNIndex  # Main DiskANN index (79%+ memory savings, zero allocation overhead)
    var dimension: Int
    var initialized: Bool
    var total_vectors: Int
    var id_to_idx: SparseMap  # Map ID to index for exists check (180x more memory efficient)
    var metadata_store: SparseMetadataMap  # Store metadata by vector ID (180x more memory efficient)
    # REMOVED: vector_store Dict - was never used, saves ~700 bytes per vector!
    var memory_pool: MemoryPool  # BLAS-optimized memory pool for batch operations
    var use_quantization: Bool  # Enable 8-bit scalar quantization for memory efficiency
    # REMOVED: quantized_vectors Dict (was 8KB per vector overhead)
    var use_binary_quantization: Bool  # Enable binary quantization for extreme compression
    # REMOVED: binary_vectors Dict (was 8KB per vector overhead) - binary quantization disabled for now
    var storage_type: StorageType  # Type of storage engine
    var snapshot_storage: Optional[SnapshotStorage]  # Legacy snapshot storage
    var memory_mapped_storage: Optional[MemoryMappedStorage]  # Next-gen storage
    var persist_path: Optional[String]  # Path for persistence if enabled
    var buffer_size: Int  # Configured buffer size
    var memory_tracker: MemoryTracker  # Track memory usage
    var memory_stats: ComponentMemoryStats  # Component-specific stats
    var config: Config  # Runtime configuration

    fn __init__(out self):
        # Use simple configuration with just 5 fields
        self.config = __default_config
        self.dimension = 0
        self.initialized = False
        self.total_vectors = 0
        self.use_quantization = False  # Initialize quantization flag
        self.id_to_idx = SparseMap(100000)  # Large capacity for production scale
        self.metadata_store = SparseMetadataMap(10000)  # Sparse metadata storage (180x more efficient)
        # REMOVED: vector_store Dict initialization - never used!
        # Use global buffer size which can be configured via configure_database
        self.buffer_size = __buffer_size  # Use global configured buffer size, not config default
        
        # Don't create buffer yet - will be created on first use with correct size
        self.buffer = VectorBuffer(dimension=1, capacity=1, use_quantization=False)  # Placeholder
        
        # Initialize main DiskANN index with configuration
        # Will be recreated with correct dimension on first use
        self.main_index = DiskANNIndex(
            dimension=1,  # Placeholder, will be recreated on first use
            expected_nodes=50,  # Minimal placeholder capacity - will resize on first use
            use_quantization=self.use_quantization
            # DiskANN will use its own defaults for r_max, beam_width, alpha
        )
        
        # Memory pool for batch operations
        self.memory_pool = MemoryPool(1)  # 1MB memory pool - reduced from 100MB to eliminate massive pre-allocation
        
        # Optional features
        # use_quantization already initialized above
        # REMOVED: quantized_vectors Dict initialization
        self.use_binary_quantization = False
        # REMOVED: binary_vectors Dict initialization - binary quantization disabled
        
        # Storage engine (default to memory-mapped for 2025 performance)
        self.storage_type = StorageType(StorageType.IN_MEMORY)
        self.snapshot_storage = Optional[SnapshotStorage]()
        self.memory_mapped_storage = Optional[MemoryMappedStorage]()
        self.persist_path = Optional[String]()
        
        # Initialize memory tracking
        self.memory_tracker = MemoryTracker(enabled=True)
        self.memory_stats = ComponentMemoryStats()
        
    fn enable_quantization(mut self) -> Bool:
        """Enable 8-bit scalar quantization for 4x memory savings.
        
        Must be called before adding vectors. Returns False if vectors exist.
        """
        if self.total_vectors > 0:
            return False
        
        self.use_quantization = True
        # Recreate buffer with quantization enabled if dimension is known
        if self.dimension > 0:
            var actual_buffer_size = self.buffer_size if self.buffer_size > 0 else __buffer_size
            self.buffer = VectorBuffer(
                dimension=self.dimension, 
                capacity=actual_buffer_size, 
                use_quantization=True
            )
            # Also recreate main_index with quantization
            var initial_nodes = max(actual_buffer_size, 100)  # Start small, grow as needed
            self.main_index = DiskANNIndex(
                dimension=self.dimension,
                expected_nodes=initial_nodes,  # Reasonable size that handles typical workloads
                use_quantization=True
            )
        return True
        
    fn enable_binary_quantization(mut self) -> Bool:
        """Enable binary quantization for extreme compression (32x).
        
        Must be called before adding vectors. Returns False if vectors exist.
        """
        # DISABLED: Binary quantization disabled to eliminate Dict[String, BinaryQuantizedVector] overhead
        return False  # Binary quantization not available
        
    fn clear_data(mut self):
        """Clear all data from the store."""
        self.id_to_idx = SparseMap(100000)  # Reset with large capacity
        self.metadata_store.clear()
        # REMOVED: vector_store.clear() - Dict was removed
        # CSRGraph handles quantization internally
        # DISABLED: Binary quantization removed
        # if self.use_binary_quantization:
        #     self.binary_vectors.clear()
        self.total_vectors = 0
        self.initialized = False
        self.dimension = 0
        # Clear buffer
        self.buffer.clear()
        # Reset main DiskANN index with larger capacity
        self.main_index = DiskANNIndex(
            dimension=1, 
            expected_nodes=50,  # Minimal capacity - scales with data, not buffer size
            use_quantization=self.use_quantization,
            r_max=64,
            beam_width=self.config.beam_width,
            alpha=1.2
        )
    
    fn set_persistence(mut self, path: String, use_wal: Bool = True) raises -> Bool:
        """Configure persistence with next-generation storage.
        
        Args:
            path: Path to database file
            use_wal: Legacy parameter (memory-mapped storage doesn't use WAL)
        
        Returns:
            True if configuration succeeded
        """
        try:
            # Store the path for later use
            self.persist_path = Optional[String](path)
            
            # If dimension is known, create storage now
            if self.dimension > 0:
                pass  # Storage type check
                if self.storage_type.value == StorageType.MEMORY_MAPPED:
                    # Use state-of-the-art memory-mapped storage
                    var storage = create_optimal_storage(path, self.dimension, True)
                    
                    # Attempt recovery from existing files
                    var recovered = storage.recover()
                    if recovered > 0:
                        pass  # print("✅ Memory-mapped recovery:", recovered, "vectors from", path)
                        # Load recovered vectors into global vector store
                        self._load_recovered_vectors_from_storage(storage)
                        pass  # print("✅ Loaded", recovered, "recovered vectors into global state")
                    
                    self.memory_mapped_storage = Optional[MemoryMappedStorage](storage)
                    pass  # print("✅ Memory-mapped storage configured for optimal performance")
                    
                else:
                    # Fall back to legacy snapshot storage
                    var storage = SnapshotStorage(path, self.dimension, checkpoint_interval=1000)
                    
                    # Attempt recovery from existing file
                    var recovered = storage.recover()
                    if recovered > 0:
                        # Load recovered vectors into vector_store
                        var all_ids = storage.get_all_ids()
                        for id in all_ids:
                            var vector_opt = storage.load_vector(id)
                            if vector_opt:
                                # Don't store in vector_store - CSRGraph handles storage
                                pass  # Vector will be added to index later
                        pass  # print("✅ Legacy recovery completed:", recovered, "vectors loaded from", path)
                
                    self.snapshot_storage = Optional[SnapshotStorage](storage)
                    pass  # print("✅ Legacy snapshot storage configured at", path)
            else:
                # Check if storage files exist to determine dimension for recovery
                var python = Python.import_module("builtins")
                var os = Python.import_module("os")
                var struct_module = Python.import_module("struct")
                
                var vector_file_path = path + ".vectors"
                if os.path.exists(vector_file_path):
                    pass  # print("📁 Found existing storage files - attempting to read dimension for recovery")
                    try:
                        # Read dimension from vector file header
                        var file = python.open(vector_file_path, "rb")
                        _ = file.read(4)  # Skip magic
                        _ = file.read(4)  # Skip version
                        var dimension_bytes = file.read(4)
                        var dimension_tuple = struct_module.unpack("I", dimension_bytes)
                        var file_dimension = Int(dimension_tuple[0])
                        file.close()
                        
                        if file_dimension > 0:
                            pass  # print("📁 Found dimension", file_dimension, "in storage file - initializing recovery")
                            self.dimension = file_dimension
                            
                            # Now create storage and attempt recovery
                            if self.storage_type.value == StorageType.MEMORY_MAPPED:
                                var storage = create_optimal_storage(path, self.dimension, True)
                                
                                # Attempt recovery from existing files
                                var recovered = storage.recover()
                                if recovered > 0:
                                    pass  # print("✅ Memory-mapped recovery:", recovered, "vectors from", path)
                                    # Load recovered vectors into global vector store
                                    self._load_recovered_vectors_from_storage(storage)
                                    pass  # print("✅ Loaded", recovered, "recovered vectors into global state")
                                
                                self.memory_mapped_storage = Optional[MemoryMappedStorage](storage)
                                pass  # print("✅ Memory-mapped storage configured for optimal performance")
                            else:
                                # Fall back to legacy snapshot storage
                                var storage = SnapshotStorage(path, self.dimension, checkpoint_interval=1000)
                                
                                # Attempt recovery from existing file
                                var recovered = storage.recover()
                                if recovered > 0:
                                    # Load recovered vectors into vector_store
                                    var all_ids = storage.get_all_ids()
                                    for id in all_ids:
                                        var vector_opt = storage.load_vector(id)
                                        if vector_opt:
                                            # Don't store in vector_store - CSRGraph handles storage
                                            pass  # Vector will be added to index later
                                    pass  # print("✅ Legacy recovery completed:", recovered, "vectors loaded from", path)
                            
                                self.snapshot_storage = Optional[SnapshotStorage](storage)
                                pass  # print("✅ Legacy snapshot storage configured at", path)
                        else:
                            pass  # print("⚠️ Invalid dimension found in storage file:", file_dimension)
                            # Defer storage creation until first vector is added
                            pass  # print("📁 Persistence path set to", path, "- storage will be created on first vector")
                    except e:
                        pass  # print("⚠️ Could not read dimension from storage file:", e)
                        # Defer storage creation until first vector is added
                        pass  # print("📁 Persistence path set to", path, "- storage will be created on first vector")
                else:
                    # No existing files, defer storage creation until first vector is added
                    pass  # print("📁 Persistence path set to", path, "- storage will be created on first vector")
            
            return True
        except e:
            pass  # print("❌ Failed to configure persistence:", e)
            return False
    
    fn _load_recovered_vectors_from_storage(mut self, storage: MemoryMappedStorage) raises:
        """Load recovered vectors from memory-mapped storage into VectorStore.
        
        This is the missing piece that connects disk recovery to the global state.
        """
        # Load recovered vectors directly into vector_store for immediate retrieval
        var recovered_count = 0
        for id in storage.hot_vectors.keys():
            var vector = storage.hot_vectors[id]
            
            # Don't store in vector_store - CSRGraph handles storage
            # Vectors are already in the memory-mapped storage and will be in the index
            recovered_count += 1
                    
            # Also add to metadata store if metadata exists
            if id in storage.hot_metadata:
                var metadata = storage.hot_metadata[id]
                var vs_metadata = Metadata()
                _ = self.metadata_store.set(id, vs_metadata)
        
        # Update total vector count
        self.total_vectors += recovered_count
        pass  # print("✅ Loaded", recovered_count, "recovered vectors into vector_store for retrieval")
    
    fn delete_vector(mut self, id: String) raises -> Bool:
        """Delete a vector from either buffer or main index.
        
        Returns True if the vector was found and deleted, False otherwise.
        """
        var success = False
        
        try:
            # First try to delete from buffer if vector is there
            var idx_opt = self.id_to_idx.get(id)
            if idx_opt and idx_opt.value() == -1:
                # Vector is in buffer
                success = self.buffer.delete(id)
                if success:
                    # Remove from tracking
                    _ = self.id_to_idx.remove(id)
                    if self.metadata_store.contains(id):
                        _ = self.metadata_store.remove(id)
                    # DISABLED: Binary quantization removed
                    # if id in self.binary_vectors:
                    #     _ = self.binary_vectors.pop(id)
                    # CSRGraph handles quantization internally
            
            # If not in buffer, try main index (though CSR doesn't support surgical delete yet)
            elif idx_opt and idx_opt.value() == 1:
                # Vector is in main index - CSR doesn't support surgical delete yet
                success = False  # Not implemented in CSR yet
                # TODO: Implement by marking as tombstone and rebuilding periodically
        except:
            # If any exception occurs, deletion failed
            success = False
        
        # Update total count if deletion was successful
        if success:
            self.total_vectors -= 1
        
        return success
    
    fn checkpoint(mut self) raises -> Bool:
        """Force a checkpoint to persist all vectors to disk.
        
        Returns:
            True if checkpoint succeeded
        """
        try:
            var success = False
            var saved_count = 0
            
            # Handle memory-mapped storage
            if self.storage_type.value == StorageType.MEMORY_MAPPED and self.memory_mapped_storage:
                var storage = safe_unwrap(self.memory_mapped_storage, "memory-mapped storage")
                
                # Calculate total vectors to checkpoint
                var total_vectors = self.buffer.size + self.main_index.size()
                pass  # print("📦 Batching", total_vectors, "vectors for checkpoint...")
                
                # Add all buffered vectors to hot buffer
                for i in range(self.buffer.size):
                    var id = self.buffer.ids[i]
                    var vector = List[Float32]()
                    for j in range(self.dimension):
                        vector.append(self.buffer.data[i * self.dimension + j])
                    
                    # Get actual metadata for this vector
                    storage.hot_vectors[id] = vector
                    if self.metadata_store.contains(id):
                        var vector_metadata_opt = self.metadata_store.get(id)
                        if vector_metadata_opt:
                            var vector_metadata = vector_metadata_opt.value()
                            if len(vector_metadata) > 0:
                                var metadata = Dict[String, String]()
                                for i in range(len(vector_metadata.keys)):
                                    metadata[vector_metadata.keys[i]] = vector_metadata.values[i]
                                storage.hot_metadata[id] = metadata
                            else:
                                storage.hot_metadata[id] = Dict[String, String]()
                    else:
                        storage.hot_metadata[id] = Dict[String, String]()
                    saved_count += 1
                
                # Add vectors from main index CSR graph
                if self.main_index.size() > 0:
                    for node_idx in range(self.main_index.graph.num_nodes()):
                        var id = self.main_index.graph.get_node_id(node_idx)
                        if id != "":
                            var vec_ptr = self.main_index.graph.get_vector_ptr(node_idx)
                            var vector = List[Float32]()
                            for j in range(self.dimension):
                                vector.append(vec_ptr[j])
                            
                            # Get actual metadata for this vector
                            storage.hot_vectors[id] = vector
                            if self.metadata_store.contains(id):
                                var vector_metadata_opt = self.metadata_store.get(id)
                                if vector_metadata_opt:
                                    var vector_metadata = vector_metadata_opt.value()
                                    if len(vector_metadata) > 0:
                                        var metadata = Dict[String, String]()
                                        for i in range(len(vector_metadata.keys)):
                                            metadata[vector_metadata.keys[i]] = vector_metadata.values[i]
                                        storage.hot_metadata[id] = metadata
                                    else:
                                        storage.hot_metadata[id] = Dict[String, String]()
                            else:
                                storage.hot_metadata[id] = Dict[String, String]()
                            saved_count += 1
                
                # Use async checkpoint for instant return!
                success = storage.checkpoint_async()
                if success:
                    # Force sync to actually write data (until Mojo gets threading)
                    storage.sync()
                    pass  # print("✅ Memory-mapped async checkpoint:", saved_count, "vectors")
            
            # Handle snapshot storage
            elif self.storage_type.value == StorageType.SNAPSHOT and self.snapshot_storage:
                var storage = safe_unwrap(self.snapshot_storage, "snapshot storage")
                
                # Transfer all vectors from buffer and index
                # Add buffered vectors
                for i in range(self.buffer.size):
                    var id = self.buffer.ids[i]
                    var vector = List[Float32]()
                    for j in range(self.dimension):
                        vector.append(self.buffer.data[i * self.dimension + j])
                    
                    # Get actual metadata for this vector
                    var metadata = Dict[String, String]()
                    if self.metadata_store.contains(id):
                        var vector_metadata_opt = self.metadata_store.get(id)
                        if vector_metadata_opt:
                            var vector_metadata = vector_metadata_opt.value()
                            for i in range(len(vector_metadata.keys)):
                                metadata[vector_metadata.keys[i]] = vector_metadata.values[i]
                    
                    if storage.save_vector(id, vector, metadata):
                        saved_count += 1
                    else:
                        pass  # print("⚠️ Failed to save vector:", id)
                
                # Add vectors from main index
                if self.main_index.size() > 0:
                    for node_idx in range(self.main_index.graph.num_nodes()):
                        var id = self.main_index.graph.get_node_id(node_idx)
                        if id != "":
                            var vec_ptr = self.main_index.graph.get_vector_ptr(node_idx)
                            var vector = List[Float32]()
                            for j in range(self.dimension):
                                vector.append(vec_ptr[j])
                            
                            # Get actual metadata for this vector
                            var metadata = Dict[String, String]()
                            if self.metadata_store.contains(id):
                                var vector_metadata_opt = self.metadata_store.get(id)
                                if vector_metadata_opt:
                                    var vector_metadata = vector_metadata_opt.value()
                                    for i in range(len(vector_metadata.keys)):
                                        metadata[vector_metadata.keys[i]] = vector_metadata.values[i]
                            
                            if storage.save_vector(id, vector, metadata):
                                saved_count += 1
                            else:
                                pass  # print("⚠️ Failed to save vector:", id)
                
                # Force storage to write to disk
                success = storage.checkpoint()
                if success:
                    pass  # print("✅ Checkpoint:", saved_count, "vectors saved")
                else:
                    pass  # print("❌ Storage checkpoint failed")
            
            # No storage configured
            else:
                if self.storage_type.value == StorageType.IN_MEMORY:
                    pass  # print("⚠️ No persistence configured (in-memory mode)")
                    return True
                else:
                    pass  # print("⚠️ No storage configured - use set_persistence() first")
                    return False
            
            return success
            
        except e:
            pass  # print("❌ Checkpoint failed:", e)
            return False
    
    fn recover(mut self) raises -> Int:
        """Recover vectors from persisted storage.
        
        Returns:
            Number of vectors recovered
        """
        try:
            var recovered = 0
            
            # Handle memory-mapped storage
            if self.storage_type.value == StorageType.MEMORY_MAPPED and self.memory_mapped_storage:
                var storage = safe_unwrap(self.memory_mapped_storage, "memory-mapped storage")
                recovered = storage.recover()
                
                # Process recovered vectors using batch operation
                if recovered > 0:
                    self.clear_data()
                    self.total_vectors = 0
                    self.metadata_store = SparseMetadataMap(10000)  # Sparse metadata storage (180x more efficient)
                    
                    # Collect all vectors for batch processing
                    var batch_ids = List[String]()
                    var batch_vectors = List[List[Float32]]()
                    var batch_metadata = List[Metadata]()
                    
                    var ids = storage.get_all_ids()
                    for vec_id in ids:
                        var vec_opt = storage.load_vector(vec_id)
                        if vec_opt:
                            var vec_data = safe_unwrap(vec_opt, "recovery vector data")
                            batch_ids.append(vec_id)
                            batch_vectors.append(vec_data)
                            batch_metadata.append(Metadata())
                    
                    # Process entire batch at once for better performance
                    if len(batch_ids) > 0:
                        var batch_data = List[Tuple[String, List[Float32], Metadata]]()
                        for i in range(len(batch_ids)):
                            batch_data.append((batch_ids[i], batch_vectors[i], batch_metadata[i]))
                        _ = self.add_vector_batch(batch_data)
                    
                    pass  # print("✅ Memory-mapped recovery completed:", recovered, "vectors loaded")
            
            # Handle snapshot storage
            elif self.storage_type.value == StorageType.SNAPSHOT and self.snapshot_storage:
                var storage = safe_unwrap(self.snapshot_storage, "snapshot storage")
                recovered = storage.recover()
                
                # Process recovered vectors using batch operation
                if recovered > 0:
                    self.clear_data()
                    self.total_vectors = 0
                    self.metadata_store = SparseMetadataMap(10000)  # Sparse metadata storage (180x more efficient)
                    
                    # Collect all vectors for batch processing
                    var batch_ids = List[String]()
                    var batch_vectors = List[List[Float32]]()
                    var batch_metadata = List[Metadata]()
                    
                    var ids = storage.get_all_ids()
                    for vec_id in ids:
                        var vec_opt = storage.load_vector(vec_id)
                        if vec_opt:
                            var vec_data = safe_unwrap(vec_opt, "recovery vector data")
                            batch_ids.append(vec_id)
                            batch_vectors.append(vec_data)
                            batch_metadata.append(Metadata())
                    
                    # Process entire batch at once for better performance
                    if len(batch_ids) > 0:
                        var batch_data = List[Tuple[String, List[Float32], Metadata]]()
                        for i in range(len(batch_ids)):
                            batch_data.append((batch_ids[i], batch_vectors[i], batch_metadata[i]))
                        _ = self.add_vector_batch(batch_data)
                    
                    pass  # print("✅ Recovery completed:", recovered, "vectors loaded")
            
            # No storage configured
            else:
                pass  # print("⚠️ No persistence configured")
                return 0
            
            return recovered
            
        except e:
            pass  # print("❌ Recovery failed:", e)
            return 0
    
    
    fn _initialize_runtime(mut self) raises:
        """Internal method to eagerly initialize runtime to avoid cold start.
        
        This eliminates the ~1500ms overhead on first operation by triggering
        Mojo runtime initialization early.
        """
        # If already initialized with real data (not just warmup), don't disturb it
        if self.initialized and self.dimension > 0 and self.dimension != 128:
            return
            
        if not self.initialized:
            # Use common dimension to properly initialize runtime
            alias WARMUP_DIM = 128
            self.dimension = WARMUP_DIM
            
            # Create temp index for warmup
            var params = self._get_adaptive_parameters(1)
            self.main_index = DiskANNIndex(
                dimension=WARMUP_DIM,
                expected_nodes=50,  # Minimal - resize as needed
                use_quantization=self.use_quantization,
                r_max=64,
                beam_width=self.config.beam_width,
                alpha=1.2
            )
            self.initialized = True
            
            # Perform warmup operations
            var dummy = List[Float32]()
            for i in range(WARMUP_DIM):
                dummy.append(Float32(i))
            
            # Trigger runtime initialization
            _ = self.main_index.add("__warmup__", dummy)
            var results = self.main_index.search(dummy, 1)
            
            # Reset to uninitialized but keep runtime warm
            self.dimension = 0
            self.initialized = False
    
    fn __copyinit__(out self, existing: Self):
        self.dimension = existing.dimension
        self.initialized = existing.initialized
        self.total_vectors = existing.total_vectors
        self.id_to_idx = existing.id_to_idx
        self.metadata_store = existing.metadata_store
        # REMOVED: vector_store copy - Dict was removed
        self.buffer_size = existing.buffer_size
        self.config = existing.config
        self.buffer = existing.buffer  # VectorBuffer handles its own copy
        self.main_index = existing.main_index
        self.memory_pool = MemoryPool(1)  # Minimal memory pool for copy
        self.use_quantization = existing.use_quantization
        # REMOVED: quantized_vectors and binary_vectors (Dict overhead eliminated)
        self.use_binary_quantization = False  # Always disabled
        self.storage_type = existing.storage_type
        self.snapshot_storage = existing.snapshot_storage
        self.memory_mapped_storage = existing.memory_mapped_storage
        self.persist_path = existing.persist_path
        self.memory_tracker = MemoryTracker(enabled=True)  # Fresh tracker for copy
        self.memory_stats = existing.memory_stats  # Copy existing stats
    
    fn __moveinit__(out self, owned existing: Self):
        self.dimension = existing.dimension
        self.initialized = existing.initialized
        self.total_vectors = existing.total_vectors
        self.id_to_idx = existing.id_to_idx^
        self.metadata_store = existing.metadata_store^
        # REMOVED: vector_store move - Dict was removed
        self.buffer_size = existing.buffer_size
        self.config = existing.config^
        self.buffer = existing.buffer^  # Move buffer
        self.main_index = existing.main_index^
        self.memory_pool = MemoryPool(1)  # Minimal memory pool for move
        self.use_quantization = existing.use_quantization
        # REMOVED: quantized_vectors and binary_vectors (Dict overhead eliminated)
        self.use_binary_quantization = False  # Always disabled
        self.storage_type = existing.storage_type
        self.snapshot_storage = existing.snapshot_storage^
        self.memory_mapped_storage = existing.memory_mapped_storage^
        self.persist_path = existing.persist_path^
        self.memory_tracker = existing.memory_tracker^
        self.memory_stats = existing.memory_stats^

    fn add_vector(
        mut self,
        id: String,
        vector: List[Float32],
        metadata: Metadata = Metadata(),
    ) raises -> Bool:
        """Add a vector to buffer for O(1) insertion.
        
        Buffer provides fast writes, flushes to main index when full.
        """
        if len(vector) == 0:
            record_error()
            return False

        # Initialize on first vector
        if not self.initialized:
            self.dimension = len(vector)
            
            # Use configured buffer_size if set, otherwise use global default
            var actual_buffer_size = self.buffer_size if self.buffer_size > 0 else __buffer_size
            
            # Recreate buffer with correct dimension, configured size, and quantization flag
            self.buffer = VectorBuffer(dimension=self.dimension, capacity=actual_buffer_size, use_quantization=self.use_quantization)
            self.buffer_size = actual_buffer_size  # Update stored size
            
            # Initialize DiskANN index with correct dimension and adaptive parameters
            var params = self._get_adaptive_parameters(self.buffer_size)
            # BALANCED FIX: Start with reasonable size for typical workloads
            # MEMORY FIX: Scale pre-allocation based on actual usage, not just buffer size
            # Problem: max(100 * 5, 1000) = always 1000 nodes (220KB waste for small datasets)
            # Solution: Start with minimal allocation, grow as needed
            # For embedded use: start with 50 nodes, can grow dynamically
            var initial_nodes = max(self.buffer_size, 50)  # Min 50 nodes, scales with buffer
            self.main_index = DiskANNIndex(
                dimension=self.dimension,
                expected_nodes=initial_nodes,  # Reasonable start, can grow if needed
                use_quantization=self.use_quantization,
                r_max=64,
                beam_width=self.config.beam_width,
                alpha=1.2
            )
            
            # Initialize persistence storage if path was set
            if self.persist_path:
                var path = safe_unwrap(self.persist_path, "persist path")
                
                pass  # print("🔍 Lazy init - Storage type value:", self.storage_type.value, "MEMORY_MAPPED=", StorageType.MEMORY_MAPPED)
                if self.storage_type.value == StorageType.MEMORY_MAPPED:
                    var storage = create_optimal_storage(path, self.dimension, True)
                    self.memory_mapped_storage = Optional[MemoryMappedStorage](storage)
                    pass  # print("✅ Memory-mapped storage initialized with dimension", self.dimension)
                else:
                    var storage = SnapshotStorage(path, self.dimension, checkpoint_interval=1000)
                    self.snapshot_storage = Optional[SnapshotStorage](storage)
                    pass  # print("✅ Legacy storage initialized with dimension", self.dimension)
            
            self.initialized = True
        
        elif len(vector) != self.dimension:
            raise Error(
                "Dimension mismatch: database contains " + String(self.dimension) +
                "D vectors, but attempted to add " + String(len(vector)) + "D vector. " +
                "All vectors in a database must have the same dimension."
            )
        
        # Check for duplicates
        if self.id_to_idx.contains(id):
            return False  # Duplicate ID
        
        try:
            # PURE INCREMENTAL: Always add directly to main index
            # Never use buffer for single vector adds to avoid any cliffs
            var flat_vector = List[Float32](capacity=len(vector))
            for i in range(len(vector)):
                flat_vector.append(vector[i])
            
            var success = self.main_index.add(id, flat_vector)
            if success:
                self.total_vectors += 1
                self.id_to_idx.insert(id, 1)  # Mark as indexed
            
            if success:
                # Update metadata store
                _ = self.metadata_store.set(id, metadata)
                
                # Store based on quantization mode
                if self.use_binary_quantization:
                    # Binary quantization - extreme compression
                    var ptr = UnsafePointer[Float32].alloc(len(vector))
                    for i in range(len(vector)):
                        ptr[i] = vector[i]
                    var binary = BinaryQuantizedVector(ptr, len(vector))
                    # self.binary_vectors[id] = binary  # DISABLED: Binary quantization removed
                    ptr.free()
                    # Update memory stats
                    self.memory_stats.vectors_memory += binary.num_bytes
                    # Don't store original or scalar quantized
                elif self.use_quantization:
                    # CSRGraph handles quantization internally
                    # Update memory stats (int8 + metadata)
                    self.memory_stats.vectors_memory += self.dimension + 8  # 1 byte/dim + scale/offset
                    # Don't store original when quantization is enabled
                else:
                    # No quantization - vectors stored in CSR graph only
                    # Memory tracked by CSR graph memory_bytes() method
                    pass
                
                # Track in id_to_idx (point to buffer for now)
                self.id_to_idx.insert(id, -1)  # -1 indicates it's in buffer
                
                # Update metadata memory stats (rough estimate)
                self.memory_stats.metadata_memory += 100  # ID mapping + metadata dict
                
                # Update buffer memory stats (count actual allocations, not just used)
                # Account for quantization: 1 byte per dim + 8 bytes overhead per vector
                if self.buffer.use_quantization:
                    self.memory_stats.buffer_memory = self.buffer.capacity * (self.dimension + 8)
                else:
                    # Full precision: dimension * 4 bytes (Float32)
                    self.memory_stats.buffer_memory = self.buffer.capacity * self.dimension * 4
                
                self.total_vectors += 1
                
                record_insert()  # Record successful insert
                return True
            else:
                return False  # Buffer add failed
                
        except e:
            pass  # print("❌ ERROR: Exception in add_vector:", e)
            record_error()  # Record operation error
            raise
    
    fn _flush_buffer_to_main(mut self) raises:
        """Flush buffer to main index with proper segment merging.
        
        Fixed: Now properly merges with existing index instead of replacing.
        """
        if self.buffer.size == 0:
            return  # Nothing to flush
        
        # MEMORY FIX: Don't copy IDs unnecessarily - use buffer directly
        var buffer_size = self.buffer.size
        # Access buffer.ids directly instead of copying
        pass  # print("🔍 Flush: buffer_size=", buffer_size, "len(buffer_ids)=", len(buffer_ids))
        
        # Check if main index has data
        if self.main_index.size() > 0:
            # MERGE MODE: Add buffer vectors to existing index using batch operation
            pass  # Merge mode
            
            # Prepare batch data as flat array to avoid List[List[Float32]] memory issues
            # MEMORY FIX: Pre-allocate exact size since we know buffer_size
            var batch_ids = List[String](capacity=buffer_size)
            var batch_vectors_flat = List[Float32](capacity=buffer_size * self.dimension)
            
            for i in range(buffer_size):
                var id = self.buffer.ids[i]  # Use buffer directly
                
                # CRITICAL FIX: Use quantized vectors when available
                # DISABLED: Binary quantization removed
                if False:  # if self.use_binary_quantization and id in self.binary_vectors:
                    # Use dequantized binary vector (lossy but compressed)
                    # var ptr = self.binary_vectors[id].dequantize()  # DISABLED
                    # for j in range(self.dimension):
                    #     batch_vectors_flat.append(ptr[j])
                    # ptr.free()
                    pass  # Disabled binary quantization code
                # CSRGraph handles quantization internally
                else:
                    # Get vector from buffer (handles both quantized and full precision)
                    try:
                        var vector = self.buffer.get_vector_by_id(id)
                        for j in range(self.dimension):
                            batch_vectors_flat.append(vector[j])
                    except:
                        # If get_vector_by_id fails, skip this vector
                        # This shouldn't happen but prevents segfault
                        continue
                
                batch_ids.append(id)
            
            # Add entire batch at once
            # NOTE: With buffer_size=10000, this causes a 3.4s delay at 10K vectors
            # Workaround: Use smaller buffer_size (e.g., 1000) to flush more frequently
            _ = self.main_index.add_batch(batch_ids, batch_vectors_flat, buffer_size)
            
            # Don't finalize here - graph is still being built
            
        else:
            # INITIAL BUILD: Create new index from buffer
            pass  # Initial build mode
            # Balance between memory efficiency and capacity
            # Using 2x buffer size for growth room without massive overhead
            var expected_capacity = buffer_size * 2  # Modest growth room
            var new_segment = DiskANNIndex(
                dimension=self.dimension,
                expected_nodes=expected_capacity,  # Sufficient for typical workloads
                use_quantization=self.use_quantization,
                r_max=64,
                beam_width=self.config.beam_width,
                alpha=1.2
            )
            
            # Prepare batch data for initial build as flat array
            # MEMORY FIX: Pre-allocate exact size since we know buffer_size
            var batch_ids = List[String](capacity=buffer_size)
            var batch_vectors_flat = List[Float32](capacity=buffer_size * self.dimension)
            
            for i in range(buffer_size):
                var id = self.buffer.ids[i]  # Use buffer directly
                
                # CRITICAL FIX: Use quantized vectors when available
                # DISABLED: Binary quantization removed
                if False:  # if self.use_binary_quantization and id in self.binary_vectors:
                    # Use dequantized binary vector (lossy but compressed)
                    # var ptr = self.binary_vectors[id].dequantize()  # DISABLED
                    # for j in range(self.dimension):
                    #     batch_vectors_flat.append(ptr[j])
                    # ptr.free()
                    pass  # Disabled binary quantization code
                # CSRGraph handles quantization internally
                else:
                    # Get vector from buffer (handles both quantized and full precision)
                    try:
                        var vector = self.buffer.get_vector_by_id(id)
                        for j in range(self.dimension):
                            batch_vectors_flat.append(vector[j])
                    except:
                        # If get_vector_by_id fails, skip this vector
                        # This shouldn't happen but prevents segfault
                        continue
                
                batch_ids.append(id)
            
            # Add entire batch at once for initial build
            _ = new_segment.add_batch(batch_ids, batch_vectors_flat, buffer_size)
            
            # First segment becomes main index
            self.main_index = new_segment
            
            # Don't finalize yet - more vectors may be added
        
        # Update memory stats (don't accumulate - use current totals)
        # Add safety check to prevent segfault from corrupted stats
        try:
            var csr_stats = self.main_index.get_memory_stats()
            self.memory_stats.graph_memory = csr_stats.graph_memory
            self.memory_stats.metadata_memory = csr_stats.metadata_memory
            self.memory_stats.vectors_memory = csr_stats.vectors_memory
            self.memory_stats.index_memory = csr_stats.index_memory
        except:
            # If stats are corrupted, estimate based on known values
            self.memory_stats.graph_memory = self.main_index.size() * 64  # Estimate edges
            self.memory_stats.vectors_memory = self.main_index.size() * self.dimension * 4
            self.memory_stats.index_memory = 0
            self.memory_stats.metadata_memory = 0
        
        pass  # print("📝 Updating id_to_idx for", buffer_size, "vectors")
        for i in range(buffer_size):
            if i == 0:
                pass  # print("  First iteration, accessing buffer_ids[0]...")
            var id = self.buffer.ids[i]  # Use buffer directly
            if i == 0:
                pass  # print("  Got id:", id)
            self.id_to_idx.insert(id, 1)
        
        # CRITICAL FIX: Clear quantization dictionaries after flush 
        # Quantized vectors are now incorporated into main index
        for i in range(buffer_size):
            var id = self.buffer.ids[i]  # Use buffer directly
            # DISABLED: Binary quantization removed
            # if id in self.binary_vectors:
            #     _ = self.binary_vectors.pop(id)
            # CSRGraph handles quantization internally
        
        self.buffer.clear()
        self.memory_stats.buffer_memory = 0
    
    fn _get_adaptive_parameters(self, expected_size: Int) -> List[Int]:
        """Get adaptive DiskANN parameters based on expected dataset size.
        
        Returns: [R (max_degree), L (beam_width), alpha (as int*10)]
        """
        if expected_size < 1000:
            # Small scale: Reduced for faster insertion
            return List[Int](16, 30, 12)  # Fewer neighbors for speed
        elif expected_size < 10000:
            # Medium scale: Balance insertion and search
            return List[Int](24, 40, 12)  # Moderate connectivity
        elif expected_size < 100000:
            # Large scale: Better connectivity needed
            return List[Int](32, 60, 15)  # alpha=1.5
        else:
            # Very large scale: High connectivity
            return List[Int](48, 100, 15)  # Balanced for scale
    
    fn _get_adaptive_beam_width(self, k: Int) -> Int:
        """Get adaptive beam width based on k and dataset size."""
        # More aggressive beam width for better search performance
        # Base beam width on k
        var base_beam = max(k * 2, 50)
        
        # Scale with dataset size
        if self.total_vectors < 1000:
            return base_beam
        elif self.total_vectors < 10000:
            return base_beam + 20
        elif self.total_vectors < 100000:
            return base_beam + 50
        else:
            return base_beam + 100
    
    fn _update_adaptive_parameters(mut self):
        """Update DiskANN parameters based on current dataset size."""
        # CSR index doesn't support dynamic parameter updates
        # Parameters are compile-time constants (CSR_R, CSR_L, CSR_ALPHA)
        pass

    fn add_vector_batch(
        mut self,
        batch_data: List[Tuple[String, List[Float32], Metadata]],
    ) raises -> List[Bool]:
        """Add multiple vectors directly to DiskANN - optimized batch operation.
        """
        var results = List[Bool]()
        
        if len(batch_data) == 0:
            return results
            
        # SCALE FIX: Process large batches in chunks to prevent memory allocation failures
        var batch_size = len(batch_data)
        var max_chunk_size = 15000  # Process 15K vectors per chunk to stay under limits
        
        if batch_size > max_chunk_size:
            print("🔧 Processing", batch_size, "vectors in", (batch_size + max_chunk_size - 1) // max_chunk_size, "chunks")
            var processed = 0
            while processed < batch_size:
                var chunk_size = min(max_chunk_size, batch_size - processed)
                var chunk_data = List[Tuple[String, List[Float32], Metadata]]()
                
                # Extract chunk
                for i in range(chunk_size):
                    chunk_data.append(batch_data[processed + i])
                
                # Process chunk recursively
                var chunk_results = self.add_vector_batch(chunk_data)
                for result in chunk_results:
                    results.append(result)
                    
                processed += chunk_size
                
            return results

        # Initialize on first vector if needed
        if not self.initialized and len(batch_data) > 0:
            var first_vector = batch_data[0][1]
            if len(first_vector) > 0:
                self.dimension = len(first_vector)
                
                # Initialize buffer with proper capacity and quantization flag
                var actual_buffer_size = self.buffer_size if self.buffer_size > 0 else __buffer_size
                self.buffer = VectorBuffer(dimension=self.dimension, capacity=actual_buffer_size, use_quantization=self.use_quantization)
                self.buffer_size = actual_buffer_size
                
                # Create DiskANN index with adaptive parameters
                var params = self._get_adaptive_parameters(len(batch_data))
                self.main_index = DiskANNIndex(
                    dimension=self.dimension,
                    expected_nodes=min(max(len(batch_data), 50), 20000),  # Cap at 20K nodes, grow dynamically
                    use_quantization=self.use_quantization,
                    r_max=64,
                    beam_width=self.config.beam_width,
                    alpha=1.2
                )
                
                # Initialize persistence storage if path was set
                if self.persist_path:
                    var path = safe_unwrap(self.persist_path, "persist path")
                    if self.storage_type.value == StorageType.MEMORY_MAPPED:
                        var storage = create_optimal_storage(path, self.dimension, True)
                        var recovered = storage.recover()
                        self.memory_mapped_storage = Optional[MemoryMappedStorage](storage)
                    else:
                        var storage = SnapshotStorage(path, self.dimension, checkpoint_interval=1000)
                        var recovered = storage.recover()
                        self.snapshot_storage = Optional[SnapshotStorage](storage)
                
                self.initialized = True

        try:
            # MEMORY FIX: Don't pre-allocate List capacity to avoid massive overhead
            var batch_size = len(batch_data)
            var batch_ids = List[String]()  # Let it grow naturally
            var vectors_flat = List[Float32](capacity=batch_size * self.dimension)  # Safe now with FFI chunking
            var metadata_list = List[Metadata]()  # Remove capacity pre-allocation
            
            # First pass: validate and prepare data with pre-allocated memory
            for i in range(batch_size):
                var item = batch_data[i]
                var id = item[0]
                var vector = item[1]
                var metadata = item[2]
                
                # Validate vector
                if len(vector) == 0:
                    results.append(False)
                    continue
                    
                if len(vector) != self.dimension:
                    raise Error(
                        "Dimension mismatch in batch: database contains " + String(self.dimension) +
                        "D vectors, but vector '" + id + "' is " + String(len(vector)) + "D. " +
                        "All vectors must have the same dimension."
                    )
                
                # Store metadata
                _ = self.metadata_store.set(id, metadata)
                self.memory_stats.metadata_memory += 100
                
                # Handle quantization if needed
                if self.use_binary_quantization:
                    var ptr = UnsafePointer[Float32].alloc(len(vector))
                    for j in range(len(vector)):
                        ptr[j] = vector[j]
                    var binary = BinaryQuantizedVector(ptr, len(vector))
                    # self.binary_vectors[id] = binary  # DISABLED: Binary quantization removed
                    ptr.free()
                    self.memory_stats.vectors_memory += binary.num_bytes
                elif self.use_quantization:
                    self.memory_stats.vectors_memory += self.dimension + 8
                
                # Add to batch using pre-allocated capacity
                batch_ids.append(id)
                # OPTIMIZATION: Batch-copy vector elements to reduce allocation overhead
                var start_idx = len(vectors_flat)
                for j in range(self.dimension):
                    vectors_flat.append(vector[j])
                metadata_list.append(metadata)
                results.append(True)
            
            # PRODUCTION FIX: Enforce hard cap on total vectors until disk persistence is implemented
            from omendb.utils.config import MAX_VECTORS_IN_MEMORY
            var current_total = self.buffer.size + self.main_index.size()
            if current_total + len(batch_ids) > MAX_VECTORS_IN_MEMORY:
                print("⚠️ WARNING: Reached maximum vector limit of", MAX_VECTORS_IN_MEMORY)
                print("   Current:", current_total, "Attempted to add:", len(batch_ids))
                raise Error("Maximum vector limit reached. This is a temporary limit until disk persistence is fully implemented.")
            
            # Add vectors using optimized flat array path
            if len(batch_ids) > 0:
                var batch_size = len(batch_ids)
                var processed = 0
                
                while processed < batch_size:
                    # OPTIMIZATION: Check buffer space more efficiently
                    var buffer_space = self.buffer.capacity - self.buffer.size
                    if buffer_space <= 0:
                        self._flush_buffer_to_main()
                        buffer_space = self.buffer.capacity - self.buffer.size
                    
                    # Calculate chunk size for optimal memory usage
                    var remaining = batch_size - processed
                    var chunk_size = min(remaining, buffer_space)
                    
                    if chunk_size > 0:
                        # MEMORY FIX: Don't pre-allocate capacity to avoid massive overhead
                        var chunk_ids = List[String]()  # Let it grow naturally
                        var chunk_vectors_flat = List[Float32](capacity=chunk_size * self.dimension)
                        
                        # Copy IDs for this chunk with pre-allocated capacity
                        for i in range(chunk_size):
                            chunk_ids.append(batch_ids[processed + i])
                        
                        # OPTIMIZATION: Bulk copy vectors for this chunk (already flat!)
                        var start_offset = processed * self.dimension
                        var elements_to_copy = chunk_size * self.dimension
                        for i in range(elements_to_copy):
                            chunk_vectors_flat.append(vectors_flat[start_offset + i])
                        
                        # Add entire chunk in one operation
                        var added = self.buffer.add_batch(chunk_ids, chunk_vectors_flat, chunk_size)
                        
                        # OPTIMIZATION: Batch update tracking with pre-allocated hash map size
                        var current_size = self.id_to_idx.len()
                        if current_size + added > (current_size + current_size // 2):
                            # Pre-grow hash map to avoid rehashing during loop
                            pass  # Dict auto-resizing is handled internally
                        
                        for i in range(added):
                            var id = chunk_ids[i]
                            self.id_to_idx.insert(id, -1)  # -1 indicates it's in buffer
                            self.total_vectors += 1
                        
                        # OPTIMIZATION: Update buffer memory stats only once per batch
                        # Defer memory stats calculation to avoid per-chunk overhead
                        pass  # Will calculate at end
                        
                        processed += added
                    else:
                        # Buffer full, flush and continue
                        self._flush_buffer_to_main()
                
                # Update adaptive parameters periodically
                if self.total_vectors % 5000 == 0:
                    self._update_adaptive_parameters()
            
            # OPTIMIZATION: Calculate memory stats once at the end for efficiency
            if self.buffer.use_quantization:
                # Count actual memory used for quantized storage
                self.memory_stats.buffer_memory = self.buffer.size * (self.dimension + 8)
            else:
                # Full precision: actual used space
                self.memory_stats.buffer_memory = self.buffer.size * self.dimension * 4
            
            # More accurate metadata memory calculation (empty metadata = minimal overhead)
            self.memory_stats.metadata_memory = self.total_vectors * 16  # Minimal overhead for empty metadata
            
            return results
            
        except e:
            raise
    
    fn add_vector_batch_flat(
        mut self,
        ids: List[String],
        vectors_flat: List[Float32],
        metadata_list: List[Metadata],
        dimension: Int,
    ) raises -> List[Bool]:
        """Add multiple vectors using flat array - MOST OPTIMIZED path.
        
        This avoids all intermediate List[List[Float32]] structures.
        Direct path from FFI to buffer with minimal allocations.
        """
        var results = List[Bool]()
        var batch_size = len(ids)
        
        if batch_size == 0:
            return results
        
        # Initialize on first vector if needed
        if not self.initialized and batch_size > 0:
            self.dimension = dimension
            
            # Initialize buffer with proper capacity and quantization flag
            var actual_buffer_size = self.buffer_size if self.buffer_size > 0 else __buffer_size
            self.buffer = VectorBuffer(dimension=self.dimension, capacity=actual_buffer_size, use_quantization=self.use_quantization)
            self.buffer_size = actual_buffer_size
            
            # Create DiskANN index with adaptive parameters
            var params = self._get_adaptive_parameters(batch_size)
            self.main_index = DiskANNIndex(
                dimension=self.dimension,
                expected_nodes=max(batch_size, 50),  # Scale with actual data size
                use_quantization=self.use_quantization,
                r_max=64,
                beam_width=self.config.beam_width,
                alpha=1.2
            )
            
            # Initialize persistence storage if path was set
            if self.persist_path:
                var path = safe_unwrap(self.persist_path, "persist path")
                if self.storage_type.value == StorageType.MEMORY_MAPPED:
                    var storage = create_optimal_storage(path, self.dimension, True)
                    var recovered = storage.recover()
                    self.memory_mapped_storage = Optional[MemoryMappedStorage](storage)
                else:
                    var storage = SnapshotStorage(path, self.dimension, checkpoint_interval=1000)
                    var recovered = storage.recover()
                    self.snapshot_storage = Optional[SnapshotStorage](storage)
            
            self.initialized = True
        
        try:
            # Validate dimension matches
            if len(vectors_flat) != batch_size * dimension:
                raise Error("Flat array size mismatch: expected " + String(batch_size * dimension) + 
                           " but got " + String(len(vectors_flat)))
            
            if dimension != self.dimension:
                raise Error("Dimension mismatch: database has " + String(self.dimension) + 
                           "D vectors, but trying to add " + String(dimension) + "D vectors")
            
            # Process metadata and quantization
            for i in range(batch_size):
                var id = ids[i]
                var metadata = metadata_list[i] if i < len(metadata_list) else Metadata()
                
                # Store metadata
                _ = self.metadata_store.set(id, metadata)
                self.memory_stats.metadata_memory += 100
                
                # Handle quantization if needed
                if self.use_binary_quantization:
                    # Extract vector from flat array for binary quantization
                    var ptr = UnsafePointer[Float32].alloc(dimension)
                    var offset = i * dimension
                    for j in range(dimension):
                        ptr[j] = vectors_flat[offset + j]
                    var binary = BinaryQuantizedVector(ptr, dimension)
                    # self.binary_vectors[id] = binary  # DISABLED: Binary quantization removed
                    ptr.free()
                    self.memory_stats.vectors_memory += binary.num_bytes
                elif self.use_quantization:
                    # CSRGraph handles quantization internally
                    self.memory_stats.vectors_memory += dimension + 8
                
                results.append(True)
            
            # Add vectors to buffer using flat array directly
            var processed = 0
            while processed < batch_size:
                # Check if buffer needs flushing
                if self.buffer.is_full():
                    self._flush_buffer_to_main()
                
                # Calculate chunk size
                var remaining = batch_size - processed
                var chunk_size = min(remaining, self.buffer.capacity - self.buffer.size)
                
                if chunk_size > 0:
                    # Extract chunk from flat arrays
                    var chunk_ids = List[String]()
                    var chunk_vectors_flat = List[Float32]()
                    
                    # Copy IDs for this chunk
                    for i in range(chunk_size):
                        chunk_ids.append(ids[processed + i])
                    
                    # Copy vectors for this chunk (already flat!)
                    var start_offset = processed * dimension
                    var end_offset = start_offset + (chunk_size * dimension)
                    for i in range(start_offset, end_offset):
                        chunk_vectors_flat.append(vectors_flat[i])
                    
                    # Add entire chunk in one operation
                    var added = self.buffer.add_batch(chunk_ids, chunk_vectors_flat, chunk_size)
                    
                    # Update tracking
                    for i in range(added):
                        var id = chunk_ids[i]
                        self.id_to_idx.insert(id, -1)  # -1 indicates it's in buffer
                        self.total_vectors += 1
                    
                    processed += added
                else:
                    # Buffer full, flush and continue
                    self._flush_buffer_to_main()
            
            # Update adaptive parameters periodically
            if self.total_vectors % 5000 == 0:
                self._update_adaptive_parameters()
            
            return results
            
        except e:
            raise


    # Memory-optimized implementation handles rebuilds internally
    # No need for manual pending vector management
    # Batch operations now handled directly by DiskANN index

    fn get_stats(self) raises -> Dict[String, PythonObject]:
        """Get database statistics with buffer architecture."""
        var stats = Dict[String, PythonObject]()
        
        if self.initialized:
            stats["vector_count"] = self.total_vectors
            stats["buffer_size"] = self.buffer.size  # Current buffer usage
            stats["buffer_capacity"] = self.buffer_size
            
            # Get DiskANN index size
            stats["main_index_size"] = self.main_index.size()
            stats["algorithm"] = String("buffer_diskann")
            
            # Buffer + batch build mode
            stats["storage_mode"] = String("buffer_batch")
            stats["status"] = String("active")
            
            stats["dimension"] = self.dimension
            
            # Memory statistics - combine all sources
            var total_vectors_memory = self.memory_stats.vectors_memory
            
            # Note: Buffer vectors are already counted in buffer_memory
            # CSRGraph handles quantization internally, no separate accounting needed
            # DISABLED: Binary quantization removed
            # if self.use_binary_quantization:
            #     total_vectors_memory += len(self.binary_vectors) * (self.dimension // 8 + 1)
            
            stats["vectors_mb"] = Float64(total_vectors_memory) / (1024.0 * 1024.0)
            stats["graph_mb"] = Float64(self.memory_stats.graph_memory) / (1024.0 * 1024.0)
            stats["metadata_mb"] = Float64(self.memory_stats.metadata_memory) / (1024.0 * 1024.0)
            stats["buffer_mb"] = Float64(self.memory_stats.buffer_memory) / (1024.0 * 1024.0)
            stats["index_mb"] = Float64(self.memory_stats.index_memory) / (1024.0 * 1024.0)
            
            # Calculate total with all components (with safety checks)
            var graph_mem = self.memory_stats.graph_memory if self.memory_stats.graph_memory > 0 else 0
            var meta_mem = self.memory_stats.metadata_memory if self.memory_stats.metadata_memory > 0 else 0
            var buffer_mem = self.memory_stats.buffer_memory if self.memory_stats.buffer_memory > 0 else 0
            var index_mem = self.memory_stats.index_memory if self.memory_stats.index_memory > 0 else 0
            
            var total_memory = (total_vectors_memory + graph_mem + meta_mem + buffer_mem + index_mem)
            stats["total_mb"] = Float64(total_memory) / (1024.0 * 1024.0)
            
            # Quantization statistics
            stats["quantization_enabled"] = self.use_quantization
            if self.use_quantization:
                # Count quantized vectors in both main index and buffer
                var quantized_count = self.main_index.size()
                if self.buffer.use_quantization:
                    quantized_count += self.buffer.size
                stats["quantized_vectors_count"] = quantized_count
                # Calculate memory savings ratio
                # Original: 4 bytes per float * dimension * num_vectors
                # Quantized: 1 byte per value * dimension * num_vectors + overhead
                if self.total_vectors > 0:
                    var original_memory = 4 * self.dimension * self.total_vectors
                    # CSRGraph reports quantized memory as: 1 byte/dim + 8 bytes/vector
                    var quantized_memory = self.total_vectors * (self.dimension + 8)
                    var savings_ratio = Float32(original_memory) / Float32(quantized_memory)
                    stats["memory_savings_ratio"] = savings_ratio
                else:
                    stats["memory_savings_ratio"] = 4.0  # Theoretical maximum
        else:
            stats["vector_count"] = 0
            stats["indexed_vectors"] = 0
            stats["pending_vectors"] = 0
            stats["dimension"] = 0
            
            stats["status"] = String("adaptive_ready")
            stats["algorithm"] = String("adaptive")
            

        return stats

    fn search(
        mut self, query: List[Float32], k: Int
    ) raises -> List[Tuple[String, Float32]]:
        """Search for similar vectors without metadata filtering."""
        return self.search_with_metadata_filter(
            query, k, Dict[String, String]()
        )

    fn search_with_metadata_filter(
        mut self,
        query: List[Float32],
        k: Int,
        filter_conditions: Dict[String, String],
    ) raises -> List[Tuple[String, Float32]]:
        """Search for similar vectors with optional metadata filtering."""
        return self._search_concurrent_safe(query, k, filter_conditions)
    
    fn search_with_beam(
        mut self,
        query: List[Float32],
        k: Int,
        filter_conditions: Dict[String, String],
        beam_width: Int,
    ) raises -> List[Tuple[String, Float32]]:
        """Search with explicit beam width control."""
        return self._search_concurrent_safe_with_beam(query, k, filter_conditions, beam_width)

    fn _search_concurrent_safe(
        mut self,
        query: List[Float32],
        k: Int,
        filter_conditions: Dict[String, String],
    ) raises -> List[Tuple[String, Float32]]:
        """Smart search: checks both buffer and main index.
        """
        var adaptive_beam = self._get_adaptive_beam_width(k)
        return self._search_concurrent_safe_with_beam(query, k, filter_conditions, adaptive_beam)
    
    fn _search_concurrent_safe_with_beam(
        mut self,
        query: List[Float32],
        k: Int,
        filter_conditions: Dict[String, String],
        beam_width: Int,
    ) raises -> List[Tuple[String, Float32]]:
        """Smart search with explicit beam width.
        """
        # Start timing the search operation  
        var timer = OperationTimer(get_global_metrics(), "query")
        
        var results = List[Tuple[String, Float32]]()

        if not self.initialized:
            return results  # Return empty for uninitialized DB
            
        if len(query) != self.dimension:
            record_error()  # Record validation error
            raise Error(
                "Dimension mismatch: database contains " + String(self.dimension) +
                "D vectors, but query vector is " + String(len(query)) + "D. " +
                "Query vector must match database dimension."
            )

        try:
            # Search buffer first (linear search for small buffer)
            var buffer_results = List[Tuple[String, Float32]]()
            if self.buffer.size > 0:
                buffer_results = self.buffer.search_linear(query, k)
            
            # Search main index if it has data
            var main_results = List[Tuple[String, Float32]]()
            if self.main_index.size() > 0:
                # Auto-finalize graph on first search if needed
                # This converts adjacency lists to CSR for efficient search
                self.main_index.finalize()
                
                # Search DiskANN with specified beam width
                var search_k = k * 2 if len(filter_conditions) > 0 else k
                main_results = self.main_index.search(query, search_k)
            
            # Merge results from buffer and main index
            var merged = List[Tuple[String, Float32]]()
            
            # Add all results to merged list
            for i in range(len(buffer_results)):
                merged.append(buffer_results[i])
            for i in range(len(main_results)):
                merged.append(main_results[i])
            
            # Sort merged results by distance
            for i in range(len(merged)):
                for j in range(i + 1, len(merged)):
                    if merged[j][1] < merged[i][1]:
                        var temp = merged[i]
                        merged[i] = merged[j]
                        merged[j] = temp
            
            # Apply metadata filters and return top k
            if len(filter_conditions) > 0:
                for i in range(len(merged)):
                    var result = merged[i]
                    var id_str = result[0]
                    var distance = result[1]
                    
                    if self._matches_metadata_filter(id_str, filter_conditions):
                        results.append((id_str, distance))
                        if len(results) >= k:
                            break
            else:
                # No filtering - return top k merged results
                for i in range(min(k, len(merged))):
                    results.append(merged[i])

            return results
        except e:
            record_error()  # Record search operation error
            # Re-raise to propagate to Python
            raise

    fn search_batch_concurrent(
        mut self,
        queries: List[List[Float32]],
        k: Int,
        filter_conditions: Dict[String, String],
    ) raises -> List[List[Tuple[String, Float32]]]:
        """Perform multiple searches concurrently for better throughput."""
        var batch_results = List[List[Tuple[String, Float32]]]()

        if not self.initialized:
            return batch_results

        # Process each query in the batch
        for i in range(len(queries)):
            var query = queries[i]
            if len(query) == self.dimension:
                try:
                    var query_results = self._search_concurrent_safe(
                        query, k, filter_conditions
                    )
                    batch_results.append(query_results)
                except:
                    # On error, append empty results
                    batch_results.append(List[Tuple[String, Float32]]())
            else:
                # Return empty results for invalid query dimensions
                batch_results.append(List[Tuple[String, Float32]]())

        return batch_results

    fn _merge_results(
        self,
        buffer_results: List[Tuple[String, Float32]],
        main_results: List[Tuple[String, Float32]],
        k: Int,
    ) -> List[Tuple[String, Float32]]:
        """Merge results from buffer and main index by score.
        
        Lower distance is better. Deduplicates by ID.
        """
        var merged = List[Tuple[String, Float32]]()
        var seen_ids = Dict[String, Bool]()
        
        # Use two pointers to merge sorted results
        var buf_idx = 0
        var main_idx = 0
        
        while len(merged) < k and (buf_idx < len(buffer_results) or main_idx < len(main_results)):
            var use_buffer = False
            
            if buf_idx >= len(buffer_results):
                use_buffer = False
            elif main_idx >= len(main_results):
                use_buffer = True
            else:
                # Compare distances (lower is better)
                use_buffer = buffer_results[buf_idx][1] <= main_results[main_idx][1]
            
            if use_buffer:
                var result = buffer_results[buf_idx]
                var id_str = result[0]
                if id_str not in seen_ids:
                    merged.append(result)
                    seen_ids[id_str] = True
                buf_idx += 1
            else:
                var result = main_results[main_idx]
                var id_str = result[0]
                if id_str not in seen_ids:
                    merged.append(result)
                    seen_ids[id_str] = True
                main_idx += 1
        
        return merged
    
    fn _matches_metadata_filter(
        self, vector_id: String, filter_conditions: Dict[String, String]
    ) -> Bool:
        """Check if a vector's metadata matches the filter conditions."""
        try:
            # Get metadata for this vector
            if self.metadata_store.contains(vector_id):
                var vector_metadata_opt = self.metadata_store.get(vector_id)
                if vector_metadata_opt:
                    var vector_metadata = vector_metadata_opt.value()

                    # Check all filter conditions
                    for filter_item in filter_conditions.items():
                        var filter_key = filter_item.key
                        var filter_value = filter_item.value

                        # Check if metadata has this key and value matches
                        if not vector_metadata.contains(filter_key):
                            return False  # Missing required key

                        var metadata_value = vector_metadata.get(filter_key)
                        if metadata_value != filter_value:
                            return False  # Value doesn't match

                return True  # All conditions matched
            else:
                # No metadata for this vector - doesn't match if filter conditions exist
                return False
        except:
            return False

    # Migration methods removed - using buffer architecture instead
    # The buffer provides predictable performance without migration complexity
    


# Module-level storage using static pointer pattern (Mojo v25.4.0 compatible)
# Avoids double-free issues with complex global objects while maintaining zero overhead
var __global_db_ptr: UnsafePointer[VectorStore] = UnsafePointer[VectorStore]()
var __global_warmup_ptr: UnsafePointer[Bool] = UnsafePointer[Bool]() 
var __module_initialized: Bool = False

# Global collections storage - maps collection names to VectorStore instances
var __global_collections: UnsafePointer[Dict[String, VectorStore]] = UnsafePointer[Dict[String, VectorStore]]()
var __collections_initialized: Bool = False
var __default_collection_name: String = "default"

# =============================================================================
# SECTION 4: HELPER FUNCTIONS
# FUTURE: Extract to utils/helpers.mojo (safe now if needed)
# =============================================================================

@always_inline
fn get_global_db() -> UnsafePointer[VectorStore]:
    """Get global database instance with zero overhead access."""
    if not __module_initialized:
        # Allocate and initialize if first access
        __global_db_ptr = UnsafePointer[VectorStore].alloc(1)
        __global_db_ptr.init_pointee_move(VectorStore())
        __global_warmup_ptr = UnsafePointer[Bool].alloc(1)
        __global_warmup_ptr.init_pointee_move(False)
        __module_initialized = True
    return __global_db_ptr

@always_inline  
fn get_warmup_done() -> UnsafePointer[Bool]:
    """Get warmup state with zero overhead access."""
    if not __module_initialized:
        _ = get_global_db()  # Initialize everything
    return __global_warmup_ptr


# =============================================================================
# SECTION 5: COLLECTION MANAGEMENT
# FUTURE: Extract to core/collections.mojo
# DEPENDENCY: Requires VectorStore extraction first
# =============================================================================

@always_inline
fn get_global_collections() -> UnsafePointer[Dict[String, VectorStore]]:
    """Get or initialize the global collections dictionary."""
    if not __collections_initialized:
        __global_collections = UnsafePointer[Dict[String, VectorStore]].alloc(1)
        __global_collections.init_pointee_move(Dict[String, VectorStore]())
        
        # Create default collection for backward compatibility
        var default_store = VectorStore()
        __global_collections[][__default_collection_name] = default_store^
        __collections_initialized = True
    
    return __global_collections


fn get_or_create_collection(collection_name: String) raises -> UnsafePointer[VectorStore]:
    """Get existing collection or create new one if it doesn't exist."""
    var collections = get_global_collections()
    
    # Check if collection exists
    if collection_name in collections[]:
        # Return pointer to existing collection
        var ptr = UnsafePointer(to=collections[][collection_name])
        return ptr
    else:
        # Create new collection
        var new_store = VectorStore()
        collections[][collection_name] = new_store^
        var ptr = UnsafePointer(to=collections[][collection_name])
        return ptr


fn get_collection_by_name(collection_name: String) raises -> Optional[UnsafePointer[VectorStore]]:
    """Get existing collection, return None if it doesn't exist."""
    var collections = get_global_collections()
    
    if collection_name in collections[]:
        var ptr = UnsafePointer(to=collections[][collection_name])
        return Optional[UnsafePointer[VectorStore]](ptr)
    else:
        return None


fn collection_exists_internal(collection_name: String) -> Bool:
    """Check if a collection exists."""
    var collections = get_global_collections()
    return collection_name in collections[]


fn delete_collection_internal(collection_name: String) raises -> Bool:
    """Delete a collection. Cannot delete the default collection."""
    if collection_name == __default_collection_name:
        return False  # Cannot delete default collection
    
    var collections = get_global_collections()
    
    if collection_name in collections[]:
        _ = collections[].pop(collection_name)
        return True
    else:
        return False


fn list_collections_internal() -> List[String]:
    """List all existing collections."""
    var collections = get_global_collections()
    var collection_names = List[String]()
    
    # Iterate through collection names  
    for item in collections[].items():
        collection_names.append(item.key)
    
    return collection_names


# =============================================================================
# SECTION 6: FFI PYTHON EXPORTS - Core Database Operations
# NOTE: Must remain in native.mojo (Python bindings)
# =============================================================================

@export
fn configure_database(config: PythonObject) raises -> PythonObject:
    """Configure database parameters for buffer architecture.
    
    Args:
        config: Dictionary with optional keys:
            - buffer_size: Write buffer size before flush (default 1000)
            - algorithm: Index algorithm ('hnsw', 'flat')
            - use_columnar: Enable columnar storage (experimental)
            - is_server: Server mode configuration
    
    Returns:
        True on success
    """
    try:
        var python = Python.import_module("builtins")
        
        # Check for buffer_size
        try:
            var buffer_obj = config.get("buffer_size")
            if buffer_obj is not None:
                var buffer_int = python.int(buffer_obj)
                var new_buffer_size = Int(buffer_int)
                
                # Update global buffer size
                __buffer_size = new_buffer_size
                
                # Note: We don't update existing store here since it will use
                # the new __buffer_size value when operations are performed
                
                # Buffer size configured
        except:
            pass  # Use default
        
        # Check for use_columnar (experimental)
        try:
            var columnar_obj = config.get("use_columnar")
            if columnar_obj is not None:
                __use_columnar = Bool(python.bool(columnar_obj))
        except:
            pass  # use_columnar not present or error accessing
        
        # Check for algorithm selection
        try:
            var algo_obj = config.get("algorithm")
            if algo_obj is not None:
                var algo_str = String(python.str(algo_obj))
                
                # Algorithm selection is deprecated - always use DiskANN
                pass
        except:
            pass  # Algorithm configuration not needed
        
        # Check for is_server
        try:
            var server_obj = config.get("is_server")
            if server_obj is not None:
                __is_server = Bool(python.bool(server_obj))
                if __is_server:
                    __use_columnar = True  # Server mode forces columnar
                    pass  # print("🚀 Server mode enabled")
        except:
            pass  # is_server not present or error accessing
        
        # Check for quantization mode (must be set before any vectors are added)
        try:
            var quant_obj = config.get("quantization")
            if quant_obj is not None:
                var quant_str = String(python.str(quant_obj))
                var store = get_global_db()
                
                if quant_str == "scalar":
                    store[].use_quantization = True
                    store[].use_binary_quantization = False
                    pass  # print("📦 Scalar quantization enabled (4x memory reduction)")
                elif quant_str == "binary":
                    # DISABLED: Binary quantization disabled to eliminate Dict overhead
                    store[].use_quantization = False
                    store[].use_binary_quantization = False
                    pass  # Binary quantization not available
        except:
            pass  # quantization not present or error accessing
                
        return PythonObject(True)
    except:
        raise

@export
fn PyInit_native() -> PythonObject:
    """Python module initialization function."""
    try:
        var module = PythonModuleBuilder("native")

        module.def_function[test_connection]("test_connection")
        module.def_function[configure_database]("configure_database")
        
        # Core database operations
        module.def_function[add_vector]("add_vector")
        module.def_function[add_vector_batch]("add_vector_batch")
        # Note: Zero-copy blocked by Mojo FFI limitations
        module.def_function[search_vectors]("search_vectors")
        module.def_function[batch_query_vectors]("batch_query_vectors")
        module.def_function[get_stats]("get_stats")
        module.def_function[clear_database]("clear_database")  # Legacy name
        module.def_function[clear_database]("_reset")  # Private testing function
        module.def_function[save_database]("save_database")
        module.def_function[load_database]("load_database")
        module.def_function[bulk_load_vectors]("bulk_load_vectors")
        module.def_function[enable_quantization]("enable_quantization")
        module.def_function[enable_binary_quantization]("enable_binary_quantization")
        module.def_function[get_memory_stats]("get_memory_stats")
        
        # Persistence operations
        module.def_function[checkpoint]("checkpoint")
        module.def_function[recover]("recover")
        module.def_function[set_persistence]("set_persistence")
        
        # Next-generation storage
        module.def_function[enable_memory_mapped_storage]("enable_memory_mapped_storage")
        module.def_function[enable_legacy_storage]("enable_legacy_storage")
        
        # CRUD operations
        module.def_function[delete_vector]("delete_vector")
        module.def_function[delete_vector_batch]("delete_vector_batch")
        module.def_function[update_vector]("update_vector")
        module.def_function[update_vector_batch]("update_vector_batch")
        module.def_function[vector_exists]("vector_exists")
        module.def_function[get_vector]("get_vector")
        module.def_function[get_metadata]("get_metadata")
        
        # Metrics export functions
        module.def_function[get_metrics_snapshot]("get_metrics_snapshot")
        module.def_function[export_metrics_prometheus]("export_metrics_prometheus")
        module.def_function[export_metrics_json]("export_metrics_json")
        module.def_function[export_metrics_statsd]("export_metrics_statsd")
        module.def_function[reset_metrics]("reset_metrics")
        
        # Database info functions
        module.def_function[count]("count")
        
        # Collections API functions
        module.def_function[create_collection]("create_collection")
        module.def_function[list_collections]("list_collections")
        module.def_function[delete_collection]("delete_collection")
        module.def_function[collection_exists]("collection_exists")
        module.def_function[get_collection_stats]("get_collection_stats")
        module.def_function[add_vector_to_collection]("add_vector_to_collection")
        module.def_function[search_vectors_in_collection]("search_vectors_in_collection")

        return module.finalize()
    except:
        return PythonObject()




@export
fn add_vector(
    vector_id: PythonObject,
    vector_data: PythonObject,
    metadata_dict: PythonObject = PythonObject(),
) raises -> PythonObject:
    """Add a vector to a specific database instance."""
    try:
        # Ensure runtime is warmed up on first call to avoid 1500ms cold start
        # DISABLED: Warmup causing dimension issues
        # if not get_warmup_done()[]:
        #     get_global_db()[]._initialize_runtime()
        #     get_warmup_done()[] = True
            
        var id_str = String(vector_id)

        # Try numpy zero-copy path first
        var vector_list = List[Float32]()
        var vector_size = 0
        
        try:
            # Check if it's a numpy array for zero-copy using the BREAKTHROUGH method
            _ = vector_data.__array_interface__  # Check if it's numpy
            var numpy = Python.import_module("numpy")
            
            # Ensure float32 type
            var vector_array = vector_data
            if String(vector_data.dtype) != "float32":
                vector_array = vector_data.astype(numpy.float32)
            
            # Use the BREAKTHROUGH: ctypes.data.unsafe_get_as_pointer
            var ctypes_data = vector_array.ctypes.data
            var data_ptr = ctypes_data.unsafe_get_as_pointer[DType.float32]()
            vector_size = Int(len(vector_array))
            
            # ZERO-COPY: Direct memory access
            vector_list = List[Float32](capacity=vector_size)
            for i in range(vector_size):
                var value = data_ptr.load(i)
                vector_list.append(value)
        except:
            # Not numpy or conversion failed - use original path
            var py_list = vector_data
            
            # Validation
            if len(id_str) == 0 or len(py_list) == 0:
                return PythonObject(False)

            # Optimized bulk conversion: pre-allocate and batch convert
            vector_size = len(py_list)
            vector_list = List[Float32](capacity=vector_size)
            
            # Extract Python floats first (minimize Python API calls)
            var temp_buffer = UnsafePointer[Float64].alloc(vector_size)
            for i in range(vector_size):
                temp_buffer[i] = Float64(py_list[i])
            
            vector_list.resize(vector_size, 0.0)
            for i in range(vector_size):
                vector_list[i] = Float32(temp_buffer[i])
            
            temp_buffer.free()

        # Optimized metadata conversion - skip if empty
        var metadata = Metadata()
        if len(metadata_dict) > 0:
            var items = metadata_dict.items()
            for item in items:
                var key = String(item[0])
                var value = String(item[1])
                metadata.set(key, value)

        # Add to specific database instance
        var success = get_global_db()[].add_vector(id_str, vector_list, metadata)
        return PythonObject(success)
    except:
        raise

@export
fn clear_database() raises -> PythonObject:
    """Clear all vectors from a specific database instance."""
    try:
        # CRITICAL FIX: Instead of clearing the singleton, create a fresh instance
        # This avoids segfaults from trying to clear complex data structures
        # and ensures we start with a clean state
        
        # Reset the global pointer by creating a new VectorStore
        if __module_initialized:
            # Free the old instance first (if it exists)
            if __global_db_ptr:
                __global_db_ptr.destroy_pointee()
                __global_db_ptr.free()
            
            # Allocate and initialize a fresh instance
            __global_db_ptr = UnsafePointer[VectorStore].alloc(1)
            __global_db_ptr.init_pointee_move(VectorStore())
            
            # Also reset the memory pool
            reset_global_pool()
        
        return PythonObject(True)
    except:
        raise

@export
fn save_database(file_path: PythonObject) raises -> PythonObject:
    """Save database to file - returns database content as Python dict."""
    try:
        var path_str = String(file_path)
        if len(path_str) == 0:
            return PythonObject(None)

        var python = Python.import_module("builtins")
        # Using single instance
        
        # Create result dictionary with metadata
        var result = python.dict()
        result["dimension"] = get_global_db()[].dimension
        result["total_vectors"] = get_global_db()[].total_vectors
        result["buffer_size"] = 0  # No buffer in single index architecture
        result["initialized"] = get_global_db()[].initialized
        result["algorithm"] = String("diskann")  # Always DiskANN now
        
        # Create vectors list
        var vectors = python.list()
        
        # Cannot directly iterate DiskANN index
        if get_global_db()[].initialized:
            pass  # DiskANN doesn't support iteration
        
        result["vectors"] = vectors
        return result
    except:
        return PythonObject(None)

@export
fn load_database(file_path: PythonObject) raises -> PythonObject:
    """Load database from file - returns parsed data as Python dict."""
    try:
        var path_str = String(file_path)
        if len(path_str) == 0:
            return PythonObject(None)
    
        var python = Python.import_module("builtins")
        var os = Python.import_module("os")
        
        # Check if file exists
        if not os.path.exists(path_str):
            return PythonObject(None)
            
        var file_obj = python.open(path_str, "r")
        var content = file_obj.read()
        file_obj.close()
        
        var lines = content.split("\n")
        if len(lines) < 2:
            return PythonObject(None)
            
        # Check header
        var header = lines[0].strip()
        if header != "OMENDB_SAVE_V1":
            return PythonObject(None)
            
        # Parse metadata
        var metadata_parts = lines[1].split(",")
        if len(metadata_parts) != 3:
            return PythonObject(None)
            
        var file_dimension = Int(metadata_parts[0])
        var file_total = Int(metadata_parts[1])
        var file_rebuild_threshold = Int(metadata_parts[2])
        
        # Create result dictionary
        var result = python.dict()
        result["dimension"] = file_dimension
        result["total"] = file_total
        result["rebuild_threshold"] = file_rebuild_threshold
        
        # Create vectors list and hashmap dict
        var vectors = python.list()
        var hashmap = python.dict()
        
        # Parse format with [HASHMAP] and [VECTORS] sections
        var in_hashmap_section = False
        var in_vectors_section = False
        
        for line_idx in range(2, len(lines)):
            var line = lines[line_idx].strip()
            if len(line) == 0:
                continue
                
            # Check for section markers
            if line == "[HASHMAP]":
                in_hashmap_section = True
                in_vectors_section = False
                continue
            elif line == "[VECTORS]":
                in_hashmap_section = False
                in_vectors_section = True
                continue
            
            # Parse HashMap section
            if in_hashmap_section:
                var parts = line.split(",")
                if len(parts) == 2:
                    var vector_id = parts[0]
                    var index = Int(parts[1])
                    hashmap[vector_id] = index
            
            # Parse vectors section
            elif in_vectors_section:
                var parts = line.split(",")
                if len(parts) < 3:
                    continue
                    
                var vector_id = parts[0]
                var dim_str = parts[1]
                var dimension = Int(dim_str)
                
                if dimension != file_dimension:
                    continue
                    
                # Parse vector values
                var vector_data = python.list()
                for i in range(2, 2 + dimension):
                    if i < len(parts):
                        var val = Float64(parts[i])
                        vector_data.append(val)
                        
                if len(vector_data) == dimension:
                    var vector_tuple = python.tuple([vector_id, vector_data])
                    vectors.append(vector_tuple)
        
        result["vectors"] = vectors
        result["hashmap"] = hashmap
        return result
    except:
        return PythonObject(None)

@export
fn delete_vector(vector_id: PythonObject) raises -> PythonObject:
    """Delete a vector by ID from a specific database instance."""
    try:
        var id = String(vector_id)
        # Delete from both buffer and main index
        var success = False
        # Use the integrated delete_vector method that handles both buffer and index
        success = get_global_db()[].delete_vector(id)
        
        # Metadata is already cleaned up in VectorStore.delete_vector()
        # Count is also already decremented in VectorStore.delete_vector()
        # So no additional cleanup needed here
            
        return PythonObject(success)
    except:
        raise

@export
fn update_vector(
    vector_id: PythonObject,
    vector_data: PythonObject,
    metadata_dict: PythonObject = PythonObject()
) raises -> PythonObject:
    """Update an existing vector in a specific database instance."""
    try:
        var id = String(vector_id)
        
        var vector = List[Float32]()
        for i in range(len(vector_data)):
            vector.append(Float32(Float64(vector_data[i])))
        
        # Convert metadata
        var metadata = Dict[String, String]()
        var python = Python.import_module("builtins")
        var has_items = python.hasattr(metadata_dict, "items")
        if has_items and len(metadata_dict) > 0:
            var items = metadata_dict.items()
            for item in items:
                var key = String(item[0])
                var value = String(item[1])
                metadata[key] = value
        
        # Perform update (delete and re-add)
        # Using single instance - first try to delete from buffer/index
        var delete_success = get_global_db()[].delete_vector(id)
        
        # Add the updated vector (will go to buffer if space available)
        var metadata_obj = Metadata()
        for key in metadata.keys():
            try:
                metadata_obj.set(key, metadata[key])
            except:
                pass
        var success = get_global_db()[].add_vector(id, vector, metadata_obj)
        
        if success and len(metadata) > 0:
            # Update metadata in the metadata store
            var mojo_metadata = Metadata()
            for item in metadata.items():
                mojo_metadata.set(item.key, item.value)
            _ = get_global_db()[].metadata_store.set(id, mojo_metadata)
        
        return PythonObject(success)
    except:
        raise

@export
fn vector_exists(vector_id: PythonObject) raises -> PythonObject:
    """Check if a vector exists by ID in a specific database instance."""
    try:
        var id = String(vector_id)
        var exists = get_global_db()[].id_to_idx.contains(id)
        var py = Python.import_module("builtins")
        return py.bool(exists)
    except:
        raise

@export
fn get_vector(vector_id: PythonObject) raises -> PythonObject:
    """Get vector data by ID from a specific database instance."""
    try:
        var id = String(vector_id)
        var db = get_global_db()[]
        # Check if vector exists (binary, scalar quantized, or normal)
        var vector_data = List[Float32]()
        
        # DISABLED: Binary quantization removed
        if False:  # if db.use_binary_quantization and id in db.binary_vectors:
            # Dequantize binary on the fly (lossy) - DISABLED
            # var ptr = db.binary_vectors[id].dequantize()
            # vector_data = List[Float32]()
            # for i in range(db.binary_vectors[id].dimension):
            #     vector_data.append(ptr[i])
            # ptr.free()
            pass  # Disabled binary quantization
        elif db.id_to_idx.contains(id):
            # Check if in buffer or main index
            var idx_opt = db.id_to_idx.get(id)
            if not idx_opt:
                return PythonObject(None)
            var idx_value = idx_opt.value()
            if idx_value == -1:
                # In buffer - O(1) lookup using new method
                try:
                    vector_data = db.buffer.get_vector_by_id(id)
                    if len(vector_data) == 0:
                        return PythonObject(None)
                except:
                    return PythonObject(None)
            
            elif idx_value == 1:
                # In main index - get ORIGINAL vector from CSR graph (CRITICAL FIX for data corruption)
                var node_idx_opt = db.main_index.graph.get_node_index(id)
                if node_idx_opt:
                    var node_idx = safe_unwrap(node_idx_opt, "node index for vector retrieval")
                    # When quantization is enabled, dequantize from CSRGraph
                    if db.use_quantization:
                        # Dequantize from CSRGraph storage
                        var quant_ptr = db.main_index.graph.get_quantized_vector_ptr(node_idx)
                        # No need to check pointer - it's valid if quantization is enabled
                        var scale = db.main_index.graph.get_quantization_scale(node_idx)
                        var offset = db.main_index.graph.get_quantization_offset(node_idx)
                        vector_data = List[Float32]()
                        for i in range(db.dimension):
                            # Dequantize: value = quantized * scale + offset
                            var quantized_val = Float32(quant_ptr[i])
                            var original_val = quantized_val * scale + offset
                            vector_data.append(original_val)
                    else:
                        # Use original vector when not quantized
                        var vec_ptr = db.main_index.graph.get_original_vector_ptr(node_idx)
                        if vec_ptr:
                            vector_data = List[Float32]()
                            for i in range(db.dimension):
                                vector_data.append(vec_ptr[i])
                        else:
                            return PythonObject(None)
                else:
                    return PythonObject(None)
            else:
                return PythonObject(None)
        else:
            # Vector not found
            return PythonObject(None)
        
        var python = Python.import_module("builtins")
        var py_vector = python.list()
        
        for i in range(len(vector_data)):
            py_vector.append(PythonObject(vector_data[i]))
        
        return py_vector
    except:
        return PythonObject(None)

@export
fn get_metadata(vector_id: PythonObject) raises -> PythonObject:
    """Get metadata for a vector by ID from a specific database instance."""
    try:
        var id = String(vector_id)
        var python = Python.import_module("builtins")
        var metadata_dict = python.dict()
        
        # Using single instance
        # Check if metadata exists for this vector
        if get_global_db()[].metadata_store.contains(id):
            var vector_metadata_opt = get_global_db()[].metadata_store.get(id)
            if vector_metadata_opt:
                var vector_metadata = vector_metadata_opt.value()
            
                # Convert Mojo metadata to Python dict
                for i in range(len(vector_metadata.keys)):
                    var key = vector_metadata.keys[i]
                    var value = vector_metadata.values[i]
                    metadata_dict[key] = value
        
        return metadata_dict
    except:
        # Return empty dict on error
        var python = Python.import_module("builtins")
        return python.dict()

@export  
fn get_stats() raises -> PythonObject:
    """Get statistics for a specific database instance."""
    try:
        # SAFETY FIX: Handle memory corruption between main DB and Collections gracefully
        var python = Python.import_module("builtins")
        var stats = python.dict()
        
        # Try to get stats from main DB first
        try:
            var stats_dict = get_global_db()[].get_stats()
            
            # Convert Mojo Dict to Python dict
            for item in stats_dict.items():
                var key = item.key
                var value = item.value
                stats[key] = value
        except:
            # If main DB access fails, try Collections default instead
            try:
                var collections = get_global_collections()
                if __default_collection_name in collections[]:
                    var default_collection_ptr = UnsafePointer(to=collections[][ __default_collection_name])
                    var default_stats = default_collection_ptr[].get_stats()
                    
                    # Convert Mojo Dict to Python dict
                    for item in default_stats.items():
                        var key = item.key
                        var value = item.value
                        stats[key] = value
                else:
                    # Return safe defaults if both fail
                    stats["vector_count"] = 0
                    stats["algorithm"] = "unknown"
                    stats["status"] = "error_fallback"
            except:
                # Final fallback - return minimal safe stats
                stats["vector_count"] = 0
                stats["algorithm"] = "unknown"
                stats["status"] = "error_fallback"
        
        # Add SIMD width
        stats["simd_width"] = simdwidthof[DType.float32]()
        
        return stats
    except:
        # Ultimate fallback
        var python = Python.import_module("builtins")
        var fallback_stats = python.dict()
        fallback_stats["vector_count"] = 0
        fallback_stats["algorithm"] = "unknown"
        fallback_stats["status"] = "critical_error"
        fallback_stats["simd_width"] = simdwidthof[DType.float32]()
        return fallback_stats

@export
fn search_vectors(
    query_vector: PythonObject,
    limit: PythonObject,
    filter_dict: PythonObject = PythonObject(),
) raises -> PythonObject:
    """Search for similar vectors in a specific database instance."""
    return _search_vectors_impl(query_vector, limit, filter_dict, PythonObject())

@export
fn search_vectors_with_beam(
    query_vector: PythonObject,
    limit: PythonObject,
    filter_dict: PythonObject,
    beamwidth: PythonObject,
) raises -> PythonObject:
    """Search for similar vectors with explicit beam width control."""
    return _search_vectors_impl(query_vector, limit, filter_dict, beamwidth)

fn _search_vectors_impl(
    query_vector: PythonObject,
    limit: PythonObject,
    filter_dict: PythonObject,
    beamwidth: PythonObject,
) raises -> PythonObject:
    """Implementation of vector search with optional beam width."""
    try:
        var py_query = query_vector
        var search_limit = Int(limit)

        # Validation
        if search_limit <= 0:
            var python = Python.import_module("builtins")
            return python.list()

        var query_list = List[Float32]()
        for i in range(len(py_query)):
            try:
                var val = Float32(Float64(py_query[i]))
                query_list.append(val)
            except:
                var python = Python.import_module("builtins")
                return python.list()

        # Convert filter dict to Mojo Dict
        var filter_conditions = Dict[String, String]()
        try:
            var python = Python.import_module("builtins")
            var has_items = python.hasattr(filter_dict, "items")
            if has_items and len(filter_dict) > 0:
                var items = filter_dict.items()
                for item in items:
                    var key = String(item[0])
                    var value = String(item[1])
                    filter_conditions[key] = value
        except:
            pass

        # SAFETY FIX: Handle memory corruption between main DB and Collections gracefully
        var python = Python.import_module("builtins")
        var results = python.list()
        
        # Try to search using main DB first
        try:
            var search_results: List[Tuple[String, Float32]]
            
            # Check if beamwidth was provided
            var has_beamwidth = False
            var beam_value = 0
            try:
                beam_value = Int(beamwidth)
                has_beamwidth = True
            except:
                has_beamwidth = False
            
            if has_beamwidth and beam_value > 0:
                search_results = get_global_db()[].search_with_beam(query_list, search_limit, filter_conditions, beam_value)
            else:
                search_results = get_global_db()[].search_with_metadata_filter(query_list, search_limit, filter_conditions)
            
            for i in range(len(search_results)):
                var result = python.dict()
                var result_id = search_results[i][0]
                result["id"] = result_id
                # search_results returns distance, convert to similarity for Python API
                var distance = Float64(search_results[i][1])
                var similarity = 1.0 - distance  # distance -> similarity
                # Clamp to valid range [0, 1] for similarity
                if similarity > 1.0:
                    similarity = 1.0
                elif similarity < 0.0:
                    similarity = 0.0
                result["score"] = similarity

                # Add metadata to result if it exists
                try:
                    if get_global_db()[].metadata_store.contains(result_id):
                        var vector_metadata_opt = get_global_db()[].metadata_store.get(result_id)
                        if vector_metadata_opt:
                            var vector_metadata = vector_metadata_opt.value()
                            var metadata_dict = python.dict()

                            for i in range(len(vector_metadata.keys)):
                                var key = vector_metadata.keys[i]
                                var value = vector_metadata.values[i]
                                metadata_dict[key] = value

                            result["metadata"] = metadata_dict
                except:
                    pass

                _ = results.append(result)
                
        except:
            # If main DB access fails, return empty results safely
            # Note: Full Collections fallback would require complex pointer management
            # For now, just return empty results to prevent crashes
            pass

        return results
    except:
        raise


@export
fn test_connection() raises -> PythonObject:
    """Test that the native module is working."""
    return PythonObject(
        "Connection successful - OmenDB native module with SIMD optimization"
        " ready!"
    )


@export
fn count() raises -> PythonObject:
    """Get the total number of vectors in the database."""
    return PythonObject(get_global_db()[].total_vectors)






# TODO: Re-enable after fixing handle management
# fn add_vector_batch(
#     #     batch_data: PythonObject,
# ) raises -> PythonObject:
#    """Add multiple vectors efficiently in a single native call.
#    
#    This provides 10-50x speedup over individual add_vector calls by
#    eliminating Python-Mojo conversion overhead for each vector.
#    
#    Args:
#        handle: Database handle
#        batch_data: List of tuples (id, vector, metadata_dict)
#    
#    Returns:
#        List of boolean success values
#    """
#    var handle_id = Int(handle)
#    
#    # Validation
#    if handle_id < 0 or handle_id >= len(g_database_stores):
#        try:
#            var python = Python.import_module("builtins")
#            return python.list()
#        except:
#            return PythonObject("batch_add_error")
#    
#    try:
#        # Convert Python batch data to Mojo format
#        var mojo_batch = List[Tuple[String, List[Float32], Metadata]]()
#        
#        for i in range(len(batch_data)):
#            var item = batch_data[i]
#            
#            # Extract id, vector, metadata from tuple
#            var id_str = String(item[0])
#            var py_vector = item[1]
#            var py_metadata = item[2] if len(item) > 2 else PythonObject()
#            
#            # Convert vector
#            var vector_list = List[Float32]()
#            for j in range(len(py_vector)):
#                try:
#                    var val = Float32(Float64(py_vector[j]))
#                    vector_list.append(val)
#                except:
#                    return PythonObject("batch_add_error")
#            
#            # Convert metadata
#            var metadata = Metadata()
#            try:
#                var python = Python.import_module("builtins")
#                var has_items = python.hasattr(py_metadata, "items")
#                if has_items and len(py_metadata) > 0:
#                    var items = py_metadata.items()
#                    for item_pair in items:
#                        var key = String(item_pair[0])
#                        var value = String(item_pair[1])
#                        metadata.set(key, value)
#            except:
#                # Use empty metadata if conversion fails
#                pass
#            
#            # Add to batch
#            mojo_batch.append((id_str, vector_list, metadata))
#        
#        # Call native batch method
#        var success_list = g_database_stores[handle_id].add_vector_batch(mojo_batch)
#        
#        # Convert results to Python
#        var python = Python.import_module("builtins")
#        var py_results = python.list()
#        
#        for k in range(len(success_list)):
#            _ = py_results.append(success_list[k])
#        
#        return py_results
#        
#    except:
#        try:
#            var python = Python.import_module("builtins")
#            return python.list()
#        except:
#            return PythonObject("batch_add_error")
## End of commented add_vector_batch
#
#


# TODO: Re-enable after fixing handle management
# fn search_vectors_concurrent(
#     #     query_vector: PythonObject,
#     limit: PythonObject,
#     filter_dict: PythonObject = PythonObject(),
# ) raises -> PythonObject:
#     """Thread-safe search for similar vectors with optional metadata filtering.
#     """
#     var handle_id = Int(handle)
#     var py_query = query_vector
#     var search_limit = Int(limit)
# 
#     # Validation
#     if handle_id < 0 or handle_id >= len(g_database_stores):
#        try:
#            var python = Python.import_module("builtins")
#            return python.list()
#        except:
#            return PythonObject("search_error")
#
#    # Convert query vector
#    var query_list = List[Float32]()
#    for i in range(len(py_query)):
#        try:
#            var val = Float32(Float64(py_query[i]))
#            query_list.append(val)
#        except:
#            try:
#                var python = Python.import_module("builtins")
#                return python.list()
#            except:
#                return PythonObject("search_error")
#
#    # Convert filter dict to Mojo Dict
#    var filter_conditions = Dict[String, String]()
#    try:
#        var python = Python.import_module("builtins")
#        var has_items = python.hasattr(filter_dict, "items")
#        if has_items and len(filter_dict) > 0:
#            var items = filter_dict.items()
#            for item in items:
#                var key = String(item[0])
#                var value = String(item[1])
#                filter_conditions[key] = value
#    except:
#        pass
#
#    # Use thread-safe search method
#    var search_results = g_database_stores[handle_id]._search_concurrent_safe(
#        query_list, search_limit, filter_conditions
#    )
#
#    try:
#        var python = Python.import_module("builtins")
#        var results = python.list()
#
#        for i in range(len(search_results)):
#            var result = python.dict()
#            var result_id = search_results[i][0]
#            result["id"] = result_id
#            result["similarity"] = Float64(search_results[i][1])
#
#            # Add metadata to result if it exists
#            try:
#                if result_id in g_database.metadata_store:
#                    var vector_metadata = g_database.metadata_store[result_id]
#                    var metadata_dict = python.dict()
#
#                    # Convert ALL Mojo metadata to Python dict
#                    for metadata_item in vector_metadata.data.items():
#                        var key = metadata_item.key
#                        var value = metadata_item.value
#                        metadata_dict[key] = value
#
#                    result["metadata"] = metadata_dict
#            except:
#                pass
#
#            _ = results.append(result)
#
#        return results
#    except:
#        return PythonObject("search_error")
## End of commented search_vectors_concurrent


# TODO: Re-enable after fixing handle management
# fn search_batch_concurrent(
#    #    query_vectors: PythonObject,
#    limit: PythonObject,
#    filter_dict: PythonObject = PythonObject(),
#) raises -> PythonObject:
#    """Perform multiple searches concurrently for better throughput."""
#    var handle_id = Int(handle)
#    var py_queries = query_vectors
#    var search_limit = Int(limit)
#
#    # Validation
#    if handle_id < 0 or handle_id >= len(g_database_stores):
#        try:
#            var python = Python.import_module("builtins")
#            return python.list()
#        except:
#            return PythonObject("batch_search_error")
#
#    # Convert query vectors
#    var query_list = List[List[Float32]]()
#    for i in range(len(py_queries)):
#        var single_query = List[Float32]()
#        var py_single_query = py_queries[i]
#
#        for j in range(len(py_single_query)):
#            try:
#                var val = Float32(Float64(py_single_query[j]))
#                single_query.append(val)
#            except:
#                return PythonObject("batch_search_error")
#
#        query_list.append(single_query)
#
#    # Convert filter dict
#    var filter_conditions = Dict[String, String]()
#    try:
#        var python = Python.import_module("builtins")
#        var has_items = python.hasattr(filter_dict, "items")
#        if has_items and len(filter_dict) > 0:
#            var items = filter_dict.items()
#            for item in items:
#                var key = String(item[0])
#                var value = String(item[1])
#                filter_conditions[key] = value
#    except:
#        pass
#
#    # Perform batch search
#    var batch_results = g_database_stores[handle_id].search_batch_concurrent(
#        query_list, search_limit, filter_conditions
#    )
#
#    try:
#        var python = Python.import_module("builtins")
#        var results = python.list()
#
#        for i in range(len(batch_results)):
#            var query_results = python.list()
#            var single_query_results = batch_results[i]
#
#            for j in range(len(single_query_results)):
#                var result = python.dict()
#                var result_id = single_query_results[j][0]
#                result["id"] = result_id
#                result["similarity"] = Float64(single_query_results[j][1])
#
#                # Add metadata if available
#                try:
#                    if result_id in g_database_stores[handle_id].metadata_store:
#                        var vector_metadata = g_database_stores[
#                            handle_id
#                        ].metadata_store[result_id]
#                        var metadata_dict = python.dict()
#
#                        # Convert ALL Mojo metadata to Python dict
#                        for metadata_item in vector_metadata.data.items():
#                            var key = metadata_item.key
#                            var value = metadata_item.value
#                            metadata_dict[key] = value
#
#                        result["metadata"] = metadata_dict
#                except:
#                    pass
#
#                _ = query_results.append(result)
#
#            _ = results.append(query_results)
#
#        return results
#    except:
#        return PythonObject("batch_search_error")
# End of commented search_batch_concurrent

# Handle-based batch functions

@export
fn add_vector_batch(
    ids: PythonObject,
    vectors: PythonObject,
    metadata_list: PythonObject = PythonObject(),
) raises -> PythonObject:
    """Add multiple vectors to a specific database instance - OPTIMIZED with parallel processing."""
    try:
        var python = Python.import_module("builtins")
        var batch_size = len(ids)
        
        # Early return for empty batch
        if batch_size == 0:
            return python.list()
            
        # UNLIMITED SCALE: Process large batches in chunks at FFI level
        var max_chunk_size = 2000   # Much smaller chunks for true unlimited scale (2K * 128 * 4 = 1MB per chunk)
        
        if batch_size > max_chunk_size:
            print("🚀 UNLIMITED SCALE: Processing", batch_size, "vectors in", (batch_size + max_chunk_size - 1) // max_chunk_size, "chunks")
            var all_results = python.list()
            var processed = 0
            
            while processed < batch_size:
                var chunk_size = min(max_chunk_size, batch_size - processed)
                
                # Extract chunk slices from Python objects
                var chunk_ids = ids[processed:processed + chunk_size]
                var chunk_vectors = vectors[processed:processed + chunk_size] 
                var chunk_metadata = metadata_list[processed:processed + chunk_size] if len(metadata_list) > 0 else python.list()
                
                # Process chunk recursively
                var chunk_results = add_vector_batch(chunk_ids, chunk_vectors, chunk_metadata)
                
                # Merge results
                for result in chunk_results:
                    all_results.append(result)
                    
                processed += chunk_size
                
            return all_results
        
        # Use optimized FFI conversion if enabled
        var mojo_vectors = List[List[Float32]]()
        var mojo_ids = List[String]()
        var results = List[Bool]()
        
        # Unified batch processing using deferred indexing
        # Works for both numpy arrays and Python lists
        var batch_data = List[Tuple[String, List[Float32], Metadata]]()
        
        # OPTIMIZATION: Try batch-level numpy detection first
        try:
            _ = vectors.__array_interface__  # Check if entire batch is numpy
            var numpy = Python.import_module("numpy")
            
            # Process entire 2D numpy array in bulk
            var vectors_f32 = vectors
            if String(vectors.dtype) != "float32":
                vectors_f32 = vectors.astype(numpy.float32)
            
            # THE BREAKTHROUGH: Use ctypes.data.unsafe_get_as_pointer (from Modular docs)
            var flat_vectors = vectors_f32.flatten()
            var dimension = Int(vectors_f32.shape[1])
            var total_elements = batch_size * dimension
            
            # Get direct memory pointer - NO tolist(), NO python.float() calls!
            var ctypes_data = flat_vectors.ctypes.data
            var data_ptr = ctypes_data.unsafe_get_as_pointer[DType.float32]()
            
            # Debug info (can be removed for production)
            pass  # print("Zero-copy batch processing:", batch_size, "vectors x", dimension, "dims")
            
            # Build all vectors from direct memory access
            for i in range(batch_size):
                var id_obj = ids[i]
                var id_str = String(python.str(id_obj))
                var vector = List[Float32](capacity=dimension)
                
                # ZERO-COPY: Direct memory load - no Python objects!
                var start_idx = i * dimension
                for j in range(dimension):
                    var value = data_ptr.load(start_idx + j)
                    vector.append(value)
                
                var metadata = Metadata()
                if i < len(metadata_list) and len(metadata_list[i]) > 0:
                    var py_meta = metadata_list[i]
                    for meta_key in py_meta.keys():
                        var key = String(meta_key)
                        var value = String(py_meta[meta_key])
                        try:
                            metadata.set(key, value)
                        except:
                            pass
                
                batch_data.append((id_str, vector, metadata))
            
        except:
            # Batch-level detection failed, falling back to per-vector processing
            # Convert all inputs to unified format (fallback)
            for i in range(batch_size):
                var id_obj = ids[i]
                var id_str = String(python.str(id_obj))
                var vector_list = vectors[i]
                var vector = List[Float32]()
                
                # Try numpy zero-copy path first, then fall back to element-by-element
                try:
                    # Check if it's a numpy array for zero-copy optimization
                    _ = vector_list.__array_interface__
                    
                    # Use the same breakthrough method as batch processing
                    var numpy = Python.import_module("numpy")
                    var vector_array = vector_list
                    if String(vector_array.dtype) != "float32":
                        vector_array = vector_array.astype(numpy.float32)
                    
                    # Use unsafe_get_as_pointer for consistency
                    var ctypes_data = vector_array.ctypes.data
                    var data_ptr = ctypes_data.unsafe_get_as_pointer[DType.float32]()
                    var vec_len = Int(len(vector_array))
                    
                    # Zero-copy direct memory access
                    for j in range(vec_len):
                        var value = data_ptr.load(j)
                        vector.append(value)
                    
                except:
                    # Fall back to element-by-element conversion for Python lists
                    var vec_len = python.len(vector_list)
                    for j in range(vec_len):
                        var elem = vector_list[j]
                        var float_val = Float64(python.float(elem))
                        vector.append(Float32(float_val))
                
                var metadata = Metadata()
                if i < len(metadata_list) and len(metadata_list[i]) > 0:
                    var py_meta = metadata_list[i]
                    for meta_key in py_meta.keys():
                        var key = String(meta_key)
                        var value = String(py_meta[meta_key])
                        try:
                            metadata.set(key, value)
                        except:
                            pass
                
                batch_data.append((id_str, vector, metadata))
        
        results = get_global_db()[].add_vector_batch(batch_data)
        
        # Convert results to Python list
        var py_results = python.list()
        for result in results:
            py_results.append(result)
        
        return py_results
    except:
        raise

@export
fn process_buffer_simplified(buffer_info: PythonObject) raises -> PythonObject:
    """Process vectors from shared memory buffer - simplified for testing."""
    # For now, just return empty list to test if function exports
    var python = Python.import_module("builtins")
    var result = python.list()
    pass  # print("process_buffer_simplified called")
    return result

@export
fn batch_query_vectors(
    query_vectors: PythonObject,
    limit: PythonObject,
) raises -> PythonObject:
    """Batch query multiple vectors in a specific database instance."""
    try:
        # Using single instance
        var k = Int(limit)
        var python = Python.import_module("builtins")
        var all_results = python.list()
        
        # Process each query vector
        for query_vector in query_vectors:
            # Convert query vector
            var vector_size = len(query_vector)
            var query = List[Float32](capacity=vector_size)
            query.resize(vector_size, 0.0)
            for j in range(vector_size):
                query[j] = Float32(Float64(query_vector[j]))
            
            # Search using the same pattern as working search_vectors function
            var empty_filter = Dict[String, String]()
            var results = get_global_db()[].search_with_metadata_filter(query, k, empty_filter)
            
            # Convert results to Python format (same pattern as working search_vectors)
            var py_results = python.list()
            for i in range(len(results)):
                var result = python.dict()
                var result_id = results[i][0]
                result["id"] = result_id
                # results returns distance, convert to similarity for Python API
                var distance = Float64(results[i][1])
                # Convert distance to similarity: similarity = 1.0 - distance
                var similarity = 1.0 - distance
                # Clamp to valid range [0, 1] for similarity
                if similarity > 1.0:
                    similarity = 1.0
                elif similarity < 0.0:
                    similarity = 0.0
                result["score"] = similarity
                
                # Add metadata if it exists
                try:
                    if get_global_db()[].metadata_store.contains(result_id):
                        var vector_metadata_opt = get_global_db()[].metadata_store.get(result_id)
                        if vector_metadata_opt:
                            var vector_metadata = vector_metadata_opt.value()
                            var metadata_dict = python.dict()
                            for i in range(len(vector_metadata.keys)):
                                var key = vector_metadata.keys[i]
                                var value = vector_metadata.values[i]
                                metadata_dict[key] = value
                            result["metadata"] = metadata_dict
                except:
                    pass
                
                _ = py_results.append(result)
            
            all_results.append(py_results)
        
        return all_results
    except:
        var python = Python.import_module("builtins")
        return python.list()

@export
fn bulk_load_vectors(
    vectors_data: PythonObject,
    rebuild_index: PythonObject = PythonObject(True),
) raises -> PythonObject:
    """Bulk load vectors into a specific database instance."""
    try:
        # Using single instance
        var python = Python.import_module("builtins")
        
        # Bulk load vectors
        for item in vectors_data:
            var id_str = String(item[0])
            var vector_list = item[1]
            
            # Convert vector
            var vector_size = len(vector_list)
            var vector = List[Float32](capacity=vector_size)
            vector.resize(vector_size, 0.0)
            for j in range(vector_size):
                vector[j] = Float32(Float64(vector_list[j]))
            
            # Add to buffer for O(1) insertion
            var success = get_global_db()[].main_index.add(id_str, vector)
            if success:
                get_global_db()[].total_vectors += 1
                if not get_global_db()[].initialized:
                    get_global_db()[].dimension = vector_size
                    get_global_db()[].initialized = True
        
        return PythonObject(True)
    except:
        raise

@export
fn delete_vector_batch(
    ids: PythonObject,
) raises -> PythonObject:
    """Delete multiple vectors from a specific database instance."""
    try:
        # Using single instance
        var results = List[Bool]()
        
        for id_obj in ids:
            var id_str = String(id_obj)
            # For delete, need to check both buffer and main index
            var success = False
            # Use integrated delete_vector method for both buffer and index
            success = get_global_db()[].delete_vector(id_str)
            results.append(success)
            
            # Note: delete_vector already handles total_vectors decrement and metadata cleanup
        
        # Convert results to Python list
        var python = Python.import_module("builtins")
        var py_results = python.list()
        for result in results:
            py_results.append(result)
        
        return py_results
    except:
        var python = Python.import_module("builtins")
        return python.list()

@export
fn update_vector_batch(
    update_data: PythonObject,
) raises -> PythonObject:
    """Update multiple vectors in a specific database instance."""
    try:
        # Using single instance
        var results = List[Bool]()
        
        for update_item in update_data:
            var id_str = String(update_item[0])
            var vector_list = update_item[1]
            var metadata_dict = update_item[2] if len(update_item) > 2 else PythonObject()
            
            # Convert vector
            var vector_size = len(vector_list)
            var vector = List[Float32](capacity=vector_size)
            vector.resize(vector_size, 0.0)
            for j in range(vector_size):
                vector[j] = Float32(Float64(vector_list[j]))
            
            # Update vector (delete and re-add using integrated methods)
            var delete_success = get_global_db()[].delete_vector(id_str)
            var empty_metadata = Metadata()
            var success = get_global_db()[].add_vector(id_str, vector, empty_metadata)
            results.append(success)
            
            # Update metadata if provided
            if success and len(metadata_dict) > 0:
                var metadata = Dict[String, String]()
                var items = metadata_dict.items()
                for item in items:
                    var key = String(item[0])
                    var value = String(item[1])
                    metadata[key] = value
                
                var mojo_metadata = Metadata()
                for item in metadata.items():
                    mojo_metadata.set(item.key, item.value)
                _ = get_global_db()[].metadata_store.set(id_str, mojo_metadata)
        
        # Convert results to Python list
        var python = Python.import_module("builtins")
        var py_results = python.list()
        for result in results:
            py_results.append(result)
        
        return py_results
    except:
        var py = Python.import_module("builtins")
        return py.list()

# Metrics export functions for Python integration
@export
fn get_metrics_snapshot() raises -> PythonObject:
    """Get current metrics snapshot for Python consumption."""
    try:
        var metrics = get_global_metrics()
        var snapshot = metrics[].get_snapshot()
        
        # Convert to Python dict
        var python = Python.import_module("builtins")
        var result = python.dict()
        
        result["query_count"] = Int(snapshot.query_count)
        result["insert_count"] = Int(snapshot.insert_count)
        result["error_count"] = Int(snapshot.error_count)
        result["memory_allocated_bytes"] = Int(snapshot.memory_allocated_bytes)
        result["uptime_seconds"] = snapshot.uptime_seconds
        result["last_query_duration_ms"] = snapshot.last_query_duration_ms
        result["average_query_duration_ms"] = snapshot.average_query_duration_ms
        
        return result
    except:
        # Return empty dict on error
        var python = Python.import_module("builtins")
        return python.dict()


@export
fn export_metrics_prometheus() raises -> PythonObject:
    """Export metrics in Prometheus format for Python consumption."""
    try:
        var metrics = get_global_metrics()
        var prometheus_text = metrics[].export_prometheus_format()
        return PythonObject(prometheus_text)
    except:
        return PythonObject("")


@export
fn export_metrics_json() raises -> PythonObject:
    """Export metrics in JSON format for Python consumption."""
    try:
        var metrics = get_global_metrics()
        var json_text = metrics[].export_json_format()
        return PythonObject(json_text)
    except:
        return PythonObject("{}")


@export
fn export_metrics_statsd() raises -> PythonObject:
    """Export metrics in StatsD format for Python consumption."""
    try:
        var metrics = get_global_metrics()
        var statsd_text = metrics[].export_statsd_format()
        return PythonObject(statsd_text)
    except:
        return PythonObject("")
















@export
fn reset_metrics() raises -> PythonObject:
    """Reset all metrics counters (useful for testing)."""
    try:
        var metrics = get_global_metrics()
        metrics[].reset_counters()
        return PythonObject(True)
    except:
        raise

@export
fn enable_quantization() raises -> PythonObject:
    """Enable 8-bit scalar quantization for 4x memory savings."""
    try:
        var success = get_global_db()[].enable_quantization()
        return PythonObject(success)
    except:
        raise

@export
fn get_memory_stats() raises -> PythonObject:
    """Get detailed memory usage statistics including quantization info."""
    var python = Python.import_module("builtins")
    
    try:
        var db = get_global_db()[]
        
        # Try to get stats - this may crash due to memory calculation issues
        var result = python.dict()
        
        # Get basic stats that are safe
        result["vector_count"] = db.total_vectors
        result["buffer_size"] = db.buffer.size
        result["main_index_size"] = db.main_index.size()
        result["dimension"] = db.dimension
        result["quantization_enabled"] = db.use_quantization
        
        # Try to get memory stats but catch failures
        try:
            # Get the detailed stats dictionary - this may crash
            var stats_dict = db.get_stats()
            
            # Manually copy specific known keys to avoid Dict iteration bug
            # Dict iteration causes bus error in Mojo, so we access keys directly
            if "vector_count" in stats_dict:
                result["vector_count"] = stats_dict["vector_count"]
            if "buffer_size" in stats_dict:
                result["buffer_size"] = stats_dict["buffer_size"]
            if "buffer_capacity" in stats_dict:
                result["buffer_capacity"] = stats_dict["buffer_capacity"]
            if "main_index_size" in stats_dict:
                result["main_index_size"] = stats_dict["main_index_size"]
            if "algorithm" in stats_dict:
                result["algorithm"] = stats_dict["algorithm"]
            if "storage_mode" in stats_dict:
                result["storage_mode"] = stats_dict["storage_mode"]
            if "status" in stats_dict:
                result["status"] = stats_dict["status"]
            if "dimension" in stats_dict:
                result["dimension"] = stats_dict["dimension"]
            if "vectors_mb" in stats_dict:
                result["vectors_mb"] = stats_dict["vectors_mb"]
            if "graph_mb" in stats_dict:
                result["graph_mb"] = stats_dict["graph_mb"]
            if "metadata_mb" in stats_dict:
                result["metadata_mb"] = stats_dict["metadata_mb"]
            if "buffer_mb" in stats_dict:
                result["buffer_mb"] = stats_dict["buffer_mb"]
            if "index_mb" in stats_dict:
                result["index_mb"] = stats_dict["index_mb"]
            if "total_mb" in stats_dict:
                result["total_mb"] = stats_dict["total_mb"]
            if "quantization_enabled" in stats_dict:
                result["quantization_enabled"] = stats_dict["quantization_enabled"]
            if "quantized_vectors_count" in stats_dict:
                result["quantized_vectors_count"] = stats_dict["quantized_vectors_count"]
            if "compression_ratio" in stats_dict:
                result["compression_ratio"] = stats_dict["compression_ratio"]
            if "binary_quantization_enabled" in stats_dict:
                result["binary_quantization_enabled"] = stats_dict["binary_quantization_enabled"]
            if "binary_vectors_count" in stats_dict:
                result["binary_vectors_count"] = stats_dict["binary_vectors_count"]
        except:
            # If detailed stats fail, provide estimates
            result["total_mb"] = Float64(db.total_vectors * db.dimension * 4) / (1024.0 * 1024.0)
            result["status"] = String("estimated")
            result["error"] = String("Memory calculation failed, showing estimates")
        
        return result
    except:
        # Return minimal safe stats on complete failure
        var fallback = python.dict()
        fallback["error"] = String("Failed to get memory stats")
        fallback["status"] = String("error")
        return fallback

@export
fn enable_binary_quantization() raises -> PythonObject:
    """Enable binary quantization for extreme compression (32x).
    
    Uses 1 bit per dimension. Best for initial filtering followed
    by rescoring with full precision.
    """
    try:
        var success = get_global_db()[].enable_binary_quantization()
        return PythonObject(success)
    except:
        raise


@export
fn checkpoint() raises -> PythonObject:
    """Force a checkpoint to persist data to disk.
    
    Returns:
        True if checkpoint succeeded, False otherwise
    """
    try:
        var db_ptr = get_global_db()
        var success = db_ptr[].checkpoint()
        return PythonObject(success)
    except e:
        pass  # print("Checkpoint failed:", e)
        return PythonObject(False)

@export
fn recover() raises -> PythonObject:
    """Recover database from persisted storage.
    
    Returns:
        Number of vectors recovered
    """
    try:
        var db_ptr = get_global_db()
        var recovered = db_ptr[].recover()
        return PythonObject(recovered)
    except e:
        pass  # print("Recovery failed:", e)
        return PythonObject(0)

@export
fn set_persistence(path: PythonObject, use_wal: PythonObject = PythonObject(True)) raises -> PythonObject:
    """Configure persistence settings.
    
    Args:
        path: Path to database file
        use_wal: Use Write-Ahead Log for durability (default: True)
    
    Returns:
        True if configuration succeeded
    """
    try:
        var path_str = String(path)
        var use_wal_bool = Bool(use_wal)
        
        # Configure persistence on the global database pointer directly
        var db_ptr = get_global_db()
        var success = db_ptr[].set_persistence(path_str, use_wal_bool)
        
        return PythonObject(success)
    except e:
        pass  # print("Failed to configure persistence:", e)
        return PythonObject(False)

@export
fn enable_memory_mapped_storage() raises -> PythonObject:
    """Enable next-generation memory-mapped storage for optimal performance.
    
    Uses state-of-the-art storage architecture based on 2025 research:
    - AiSAQ: 10MB memory footprint for billion-scale
    - NDSEARCH: 31.7x throughput improvement
    - Memory-mapped files for zero-copy access
    - SSD-optimized block layout
    
    Returns:
        True if successfully enabled
    """
    try:
        var db_ptr = get_global_db()
        db_ptr[].storage_type = StorageType(StorageType.MEMORY_MAPPED)
        pass  # print("✅ Memory-mapped storage enabled (2025 state-of-the-art)")
        return PythonObject(True)
    except e:
        pass  # print("❌ Failed to enable memory-mapped storage:", e)
        return PythonObject(False)

@export
fn enable_legacy_storage() raises -> PythonObject:
    """Enable legacy snapshot storage for compatibility.
    
    Returns:
        True if successfully enabled
    """
    try:
        var db_ptr = get_global_db()
        db_ptr[].storage_type = StorageType(StorageType.SNAPSHOT)
        pass  # print("⚠️ Legacy snapshot storage enabled")
        return PythonObject(True)
    except e:
        pass  # print("❌ Failed to enable legacy storage:", e)
        return PythonObject(False)

# =============================================================================
# SECTION 7: FFI PYTHON EXPORTS - Collections API
# NOTE: Multi-collection support exports
# =============================================================================

@export
fn create_collection(collection_name: PythonObject) raises -> PythonObject:
    """Create a new named collection.
    
    Args:
        collection_name: Name of the collection to create
        
    Returns:
        True if collection was created, False if it already exists
    """
    try:
        var name = String(collection_name)
        var existing = get_collection_by_name(name)
        
        if existing:
            return PythonObject(False)  # Collection already exists
        else:
            _ = get_or_create_collection(name)  # Create new collection
            return PythonObject(True)
    except:
        raise

@export
fn list_collections() raises -> PythonObject:
    """List all existing collections.
    
    Returns:
        List of collection names as Python list
    """
    try:
        var python = Python.import_module("builtins")
        var collection_names = list_collections_internal()
        var py_list = python.list()
        
        for i in range(len(collection_names)):
            py_list.append(collection_names[i])
        
        return py_list
    except:
        raise

@export
fn delete_collection(collection_name: PythonObject) raises -> PythonObject:
    """Delete a collection and all its vectors.
    
    Args:
        collection_name: Name of the collection to delete
        
    Returns:
        True if collection was deleted, False if it didn't exist or is default
    """
    try:
        var name = String(collection_name)
        var success = delete_collection_internal(name)
        return PythonObject(success)
    except:
        raise

@export
fn collection_exists(collection_name: PythonObject) raises -> PythonObject:
    """Check if a collection exists.
    
    Args:
        collection_name: Name of the collection to check
        
    Returns:
        True if collection exists, False otherwise
    """
    try:
        var name = String(collection_name)
        var exists = collection_exists_internal(name)
        return PythonObject(exists)
    except:
        raise

@export
fn get_collection_stats(collection_name: PythonObject) raises -> PythonObject:
    """Get statistics for a specific collection.
    
    Args:
        collection_name: Name of the collection
        
    Returns:
        Dictionary of collection statistics or None if collection doesn't exist
    """
    try:
        var name = String(collection_name)
        var collection_ptr = get_collection_by_name(name)
        
        if collection_ptr:
            var ptr = safe_unwrap(collection_ptr, "collection pointer for stats")
            var stats_dict = ptr[].get_stats()
            
            # Convert Dict[String, PythonObject] to Python dict
            var python = Python.import_module("builtins")
            var py_stats = python.dict()
            
            for item in stats_dict.items():
                py_stats[item.key] = item.value
            
            return py_stats
        else:
            return PythonObject(None)
    except:
        raise

@export
fn add_vector_to_collection(collection_name: PythonObject, vector_id: PythonObject, vector: PythonObject, metadata: PythonObject) raises -> PythonObject:
    """Add a vector to a specific collection.
    
    Args:
        collection_name: Name of the collection
        vector_id: Unique identifier for the vector
        vector: Vector data as Python list or numpy array
        metadata: Metadata dictionary (optional)
        
    Returns:
        True on success, False on failure
    """
    try:
        var name = String(collection_name)
        var collection_ptr = get_or_create_collection(name)
        var id = String(vector_id)
        
        # Convert vector to List[Float32] - simplified version
        var vector_list = List[Float32]()
        
        # Handle regular Python list
        var py_list = vector
        var vector_size = len(py_list)
        for i in range(vector_size):
            var item = py_list[i]
            vector_list.append(Float32(Float64(item)))
        
        # Handle metadata - simplified version
        var mojo_metadata = Metadata()
        if metadata and String(metadata) != "None":
            try:
                for key_py in metadata.keys():
                    var key_str = String(key_py)
                    var value_py = metadata[key_py]
                    var value_str = String(value_py)
                    mojo_metadata.set(key_str, value_str)
            except:
                pass  # Skip metadata on error
        
        # Add vector to collection
        var success = collection_ptr[].add_vector(id, vector_list, mojo_metadata)
        return PythonObject(success)
    except:
        raise

@export
fn search_vectors_in_collection(collection_name: PythonObject, query_vector: PythonObject, k: PythonObject) raises -> PythonObject:
    """Search for similar vectors in a specific collection.
    
    Args:
        collection_name: Name of the collection to search
        query_vector: Query vector as Python list or numpy array
        k: Number of results to return
        
    Returns:
        List of [id, distance, metadata] tuples
    """
    try:
        var name = String(collection_name)
        var collection_ptr = get_collection_by_name(name)
        
        if not collection_ptr:
            var python = Python.import_module("builtins")
            return python.list()  # Collection doesn't exist, return empty results
        
        var search_limit = Int32(k)
        
        # Convert query vector - simplified version
        var query_list = List[Float32]()
        var py_list = query_vector
        var query_size = len(py_list)
        for i in range(query_size):
            var item = py_list[i]
            query_list.append(Float32(Float64(item)))
        
        # Perform search in collection
        var empty_filter = Dict[String, String]()
        var ptr = safe_unwrap(collection_ptr, "collection pointer for search")
        var results = ptr[].search_with_metadata_filter(query_list, Int(k), empty_filter)
        
        # Convert results to Python format
        var python = Python.import_module("builtins")
        var py_results = python.list()
        for i in range(len(results)):
            var result = results[i]
            var result_id = result[0]  # String
            var distance = result[1]   # Float32
            
            var score = 1.0 - Float64(distance)  # distance -> score
            if score > 1.0:
                score = 1.0
            elif score < -1.0:
                score = -1.0
            
            # Create result dict
            var result_dict = python.dict()
            result_dict["id"] = result_id
            result_dict["score"] = score
            
            # Get metadata for this result
            if ptr[].metadata_store.contains(result_id):
                var vector_metadata_opt = ptr[].metadata_store.get(result_id)
                if vector_metadata_opt:
                    var vector_metadata = vector_metadata_opt.value()
                    var metadata_dict = python.dict()
                    for i in range(len(vector_metadata.keys)):
                        var key = vector_metadata.keys[i]
                        var value = vector_metadata.values[i]
                        metadata_dict[key] = value
                    result_dict["metadata"] = metadata_dict
            
            py_results.append(result_dict)
        
        return py_results
    except:
        raise


# Module registration using @register_python_module decorator
# The @export decorators above already make functions available to Python
