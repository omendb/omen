"""
Hybrid RoarGraph Implementation - CPU Optimized with GPU Readiness

Designed for development on Mac with CPU optimization,
but architected to seamlessly translate to GPU acceleration on RTX 4090.

Target: Perfect algorithm on CPU, then 100x speedup on GPU
"""

from collections import List
from core.vector import Vector
from core.distance import cosine_similarity
import time
from algorithm import sort
from math import min

alias SIMD_WIDTH = simdwidthof[DType.float32]()

struct DeviceContext:
    """Unified device context for CPU/GPU abstraction."""
    var device_type: String  # "cpu", "cuda", "metal"
    var device_id: Int
    var is_available: Bool
    
    fn __init__(inout self, preferred_device: String = "cpu"):
        self.device_type = preferred_device
        self.device_id = 0
        self.is_available = True
        
        if preferred_device == "cuda":
            # Will be implemented for RTX 4090
            self.is_available = False  # Not available on Mac
            self.device_type = "cpu"   # Fallback
        elif preferred_device == "metal":
            # Mac Metal (limited MAX support)
            self.is_available = False  # Not fully supported
            self.device_type = "cpu"   # Fallback
        
        print(f"🔧 Device context: {self.device_type}")


struct BipartiteGraph:
    """
    CPU-optimized bipartite graph with GPU-ready memory layout.
    
    Memory layout optimized for:
    - SIMD operations on CPU (vectorized processing)
    - Coalesced memory access patterns (GPU-ready)
    - Cache-friendly data structures
    """
    
    var left_nodes: List[Int]      # Vector indices (cache-aligned)
    var right_nodes: List[Int]     # Query indices (cache-aligned) 
    var edge_weights: List[Float32]  # Similarity scores (SIMD-friendly)
    var edge_count: Int
    var adjacency_list: List[List[Int]]  # Fast neighbor lookup
    
    fn __init__(inout self, max_vectors: Int, max_queries: Int):
        """Initialize with GPU-friendly memory layout."""
        self.left_nodes = List[Int]()
        self.right_nodes = List[Int]()
        self.edge_weights = List[Float32]()
        self.edge_count = 0
        
        # Pre-allocate adjacency list for fast lookup
        self.adjacency_list = List[List[Int]]()
        for i in range(max_vectors):
            self.adjacency_list.append(List[Int]())
    
    fn add_edge(inout self, vector_idx: Int, query_idx: Int, weight: Float32):
        """Add edge with cache-friendly storage."""
        self.left_nodes.append(vector_idx)
        self.right_nodes.append(query_idx)
        self.edge_weights.append(weight)
        self.edge_count += 1
        
        # Update adjacency list for O(1) neighbor lookup
        if vector_idx < len(self.adjacency_list):
            self.adjacency_list[vector_idx].append(query_idx)
    
    fn get_connected_queries(self, vector_idx: Int) -> List[Int]:
        """Get queries connected to a vector (O(1) lookup)."""
        if vector_idx < len(self.adjacency_list):
            return self.adjacency_list[vector_idx]
        return List[Int]()
    
    fn build_from_training_queries(
        inout self,
        vectors: List[List[Float32]], 
        training_queries: List[List[Float32]],
        k_connections: Int = 50
    ):
        """
        Build bipartite graph with SIMD-optimized similarity computation.
        
        This is designed to translate directly to GPU kernels:
        - Batch similarity computation 
        - Parallel top-k selection
        - Memory coalescing patterns
        """
        print(f"🏗️ Building bipartite graph: {len(training_queries)} queries, {len(vectors)} vectors")
        
        let n_queries = len(training_queries)
        let n_vectors = len(vectors)
        
        var construction_start = time.perf_counter()
        
        # Process queries in batches for SIMD efficiency
        let batch_size = 16  # Optimize for CPU cache
        
        for batch_start in range(0, n_queries, batch_size):
            let batch_end = min(batch_start + batch_size, n_queries)
            
            if batch_start % 100 == 0:
                print(f"   Processing batch {batch_start//batch_size + 1}/{(n_queries + batch_size - 1)//batch_size}")
            
            # Process batch of queries
            for q in range(batch_start, batch_end):
                let query = training_queries[q]
                
                # SIMD-optimized batch similarity computation
                var similarities = self._compute_similarities_simd(query, vectors)
                
                # Find top-k with optimized selection
                var top_k_indices = self._find_top_k_optimized(similarities, k_connections)
                
                # Add edges for top-k vectors
                for i in range(len(top_k_indices)):
                    let vector_idx = top_k_indices[i]
                    let similarity = similarities[vector_idx]
                    self.add_edge(vector_idx, q, similarity)
        
        var construction_end = time.perf_counter()
        let construction_time = (construction_end - construction_start) * 1000.0
        
        print(f"✅ Bipartite graph complete: {construction_time:.2f}ms")
        print(f"   Edges: {self.edge_count}, Density: {Float64(self.edge_count)/Float64(n_vectors * n_queries):.4f}")
    
    fn _compute_similarities_simd(self, query: List[Float32], vectors: List[List[Float32]]) -> List[Float32]:
        """SIMD-optimized similarity computation (CPU) - GPU-ready pattern."""
        var similarities = List[Float32]()
        let dimension = len(query)
        let n_vectors = len(vectors)
        
        # Pre-compute query norm (SIMD-friendly)
        var query_norm_sq: Float32 = 0.0
        for i in range(0, dimension, SIMD_WIDTH):
            let remaining = min(SIMD_WIDTH, dimension - i)
            var simd_sum: SIMD[DType.float32, SIMD_WIDTH] = 0.0
            
            for j in range(remaining):
                if i + j < dimension:
                    simd_sum[j] = query[i + j] * query[i + j]
            
            query_norm_sq += simd_sum.reduce_add()
        
        let query_norm = (query_norm_sq ** 0.5)
        
        # Compute similarities with SIMD acceleration
        for v in range(n_vectors):
            let vector = vectors[v]
            
            var dot_product: Float32 = 0.0
            var vector_norm_sq: Float32 = 0.0
            
            # SIMD-optimized dot product and norm computation
            for i in range(0, dimension, SIMD_WIDTH):
                let remaining = min(SIMD_WIDTH, dimension - i)
                var simd_dot: SIMD[DType.float32, SIMD_WIDTH] = 0.0
                var simd_norm: SIMD[DType.float32, SIMD_WIDTH] = 0.0
                
                for j in range(remaining):
                    if i + j < dimension:
                        let q_val = query[i + j]
                        let v_val = vector[i + j]
                        simd_dot[j] = q_val * v_val
                        simd_norm[j] = v_val * v_val
                
                dot_product += simd_dot.reduce_add()
                vector_norm_sq += simd_norm.reduce_add()
            
            let vector_norm = (vector_norm_sq ** 0.5)
            
            # Compute cosine similarity
            var similarity: Float32 = 0.0
            if query_norm > 0.0 and vector_norm > 0.0:
                similarity = dot_product / (query_norm * vector_norm)
            
            similarities.append(similarity)
        
        return similarities
    
    fn _find_top_k_optimized(self, similarities: List[Float32], k: Int) -> List[Int]:
        """Optimized top-k selection - designed for GPU translation."""
        let n = len(similarities)
        var indices = List[Int]()
        
        # Create index-value pairs for sorting
        var pairs = List[Tuple[Float32, Int]]()
        for i in range(n):
            pairs.append((similarities[i], i))
        
        # Partial sort - only need top-k elements
        # This pattern translates well to GPU radix sort or parallel selection
        self._partial_sort(pairs, k)
        
        # Extract top-k indices
        for i in range(min(k, len(pairs))):
            indices.append(pairs[i][1])
        
        return indices
    
    fn _partial_sort(self, inout pairs: List[Tuple[Float32, Int]], k: Int):
        """Partial sort for top-k selection."""
        let n = len(pairs)
        let sort_limit = min(k, n)
        
        # Simple insertion sort for small k (efficient for CPU)
        # This will be replaced with GPU-optimized sorting
        for i in range(1, sort_limit):
            let key = pairs[i]
            var j = i - 1
            
            while j >= 0 and pairs[j][0] < key[0]:  # Descending order
                pairs[j + 1] = pairs[j]
                j -= 1
            
            pairs[j + 1] = key
        
        # Ensure we only keep top-k elements
        if len(pairs) > k:
            # Truncate to top-k
            var top_k = List[Tuple[Float32, Int]]()
            for i in range(k):
                top_k.append(pairs[i])
            pairs = top_k


struct HybridRoarGraph:
    """
    Hybrid RoarGraph: CPU-optimized with GPU-ready architecture.
    
    Design principles:
    - Perfect algorithm implementation on CPU
    - Memory layouts that translate to GPU
    - Batch processing patterns for GPU efficiency
    - Automatic device selection and fallback
    """
    
    var vectors: List[List[Float32]]        # CPU-friendly vector storage
    var bipartite_graph: BipartiteGraph      # GPU-ready graph structure
    var training_queries: List[List[Float32]]  # Training query cache
    var device_context: DeviceContext        # Device abstraction
    var is_trained: Bool
    var dimension: Int
    
    fn __init__(inout self, device_preference: String = "cpu", dimension: Int = 384):
        """Initialize hybrid index with device preference."""
        self.device_context = DeviceContext(device_preference)
        self.vectors = List[List[Float32]]()
        self.training_queries = List[List[Float32]]()
        self.bipartite_graph = BipartiteGraph(100000, 1000)  # Default capacity
        self.is_trained = False
        self.dimension = dimension
        
        print(f"🚀 Hybrid RoarGraph initialized: {dimension}D, device: {self.device_context.device_type}")
    
    fn add_vector(inout self, vector_data: List[Float32]) raises:
        """Add vector with validation and GPU-ready storage."""
        if len(vector_data) != self.dimension:
            raise Error(f"Vector dimension {len(vector_data)} != expected {self.dimension}")
        
        # Store vector (CPU-friendly format, GPU-ready for later transfer)
        self.vectors.append(vector_data)
        self.is_trained = False  # Invalidate training
    
    fn train_with_queries(inout self, training_query_vectors: List[List[Float32]]):
        """
        Train RoarGraph with query-guided optimization.
        
        CPU implementation with patterns that translate to GPU:
        - Batch processing for GPU efficiency
        - Memory-coalesced data access
        - Parallel-friendly algorithms
        """
        print(f"🎓 Training RoarGraph: {len(training_query_vectors)} queries, {len(self.vectors)} vectors")
        
        if len(self.vectors) == 0:
            print("⚠️ No vectors to train on")
            return
        
        # Cache training queries for later use
        self.training_queries = training_query_vectors
        
        # Validate training queries
        for i in range(len(training_query_vectors)):
            if len(training_query_vectors[i]) != self.dimension:
                print(f"⚠️ Query {i} dimension mismatch: {len(training_query_vectors[i])} != {self.dimension}")
                return
        
        # Build bipartite graph with SIMD optimization
        var training_start = time.perf_counter()
        
        # Recreate graph with correct dimensions
        self.bipartite_graph = BipartiteGraph(len(self.vectors), len(training_query_vectors))
        
        # Build graph using training queries
        self.bipartite_graph.build_from_training_queries(
            self.vectors,
            self.training_queries,
            k_connections=min(50, len(self.vectors))  # Adaptive connection count
        )
        
        var training_end = time.perf_counter()
        let training_time = (training_end - training_start) * 1000.0
        
        self.is_trained = True
        print(f"✅ Training complete: {training_time:.2f}ms")
        print(f"   Graph density: {Float64(self.bipartite_graph.edge_count)/Float64(len(self.vectors) * len(self.training_queries)):.4f}")
    
    fn search(self, query_vector: List[Float32], k: Int = 10) -> List[Tuple[Int, Float32]]:
        """
        Search using hybrid CPU/GPU approach.
        
        On CPU: SIMD-optimized RoarGraph traversal
        On GPU: Massively parallel graph traversal (future)
        """
        if len(query_vector) != self.dimension:
            print(f"⚠️ Query dimension mismatch: {len(query_vector)} != {self.dimension}")
            return List[Tuple[Int, Float32]]()
        
        if not self.is_trained:
            print("🔍 Using brute force search (not trained)")
            return self._brute_force_search(query_vector, k)
        
        # RoarGraph search with CPU optimization
        return self._roargraph_search_cpu(query_vector, k)
    
    fn _roargraph_search_cpu(self, query_vector: List[Float32], k: Int) -> List[Tuple[Int, Float32]]:
        """CPU-optimized RoarGraph search with GPU-ready patterns."""
        
        # Step 1: Find most similar training queries (SIMD-optimized)
        var query_similarities = self.bipartite_graph._compute_similarities_simd(query_vector, self.training_queries)
        
        # Step 2: Collect candidate vectors from top training queries
        let top_training_queries = min(10, len(self.training_queries))
        var candidate_vectors = Set[Int]()  # Use Set to avoid duplicates
        
        # Get top training queries using optimized selection
        var top_query_indices = self.bipartite_graph._find_top_k_optimized(query_similarities, top_training_queries)
        
        # Collect candidates from bipartite graph
        for i in range(len(top_query_indices)):
            let query_idx = top_query_indices[i]
            
            # Find vectors connected to this training query
            for e in range(self.bipartite_graph.edge_count):
                if self.bipartite_graph.right_nodes[e] == query_idx:
                    let vector_idx = self.bipartite_graph.left_nodes[e]
                    candidate_vectors.add(vector_idx)
        
        # Step 3: Compute exact similarities to candidates (SIMD-optimized)
        var candidate_list = List[Int]()
        for vector_idx in candidate_vectors:
            candidate_list.append(vector_idx)
        
        var candidate_similarities = List[Float32]()
        for i in range(len(candidate_list)):
            let vector_idx = candidate_list[i]
            let similarity = self._compute_exact_similarity(query_vector, self.vectors[vector_idx])
            candidate_similarities.append(similarity)
        
        # Step 4: Select top-k results
        var similarity_pairs = List[Tuple[Float32, Int]]()
        for i in range(len(candidate_list)):
            similarity_pairs.append((candidate_similarities[i], candidate_list[i]))
        
        # Sort and return top-k
        self.bipartite_graph._partial_sort(similarity_pairs, k)
        
        var results = List[Tuple[Int, Float32]]()
        for i in range(min(k, len(similarity_pairs))):
            let vector_idx = similarity_pairs[i][1]
            let similarity = similarity_pairs[i][0]
            results.append((vector_idx, similarity))
        
        return results
    
    fn _brute_force_search(self, query_vector: List[Float32], k: Int) -> List[Tuple[Int, Float32]]:
        """SIMD-optimized brute force search fallback."""
        var similarities = self.bipartite_graph._compute_similarities_simd(query_vector, self.vectors)
        var top_k_indices = self.bipartite_graph._find_top_k_optimized(similarities, k)
        
        var results = List[Tuple[Int, Float32]]()
        for i in range(len(top_k_indices)):
            let vector_idx = top_k_indices[i]
            let similarity = similarities[vector_idx]
            results.append((vector_idx, similarity))
        
        return results
    
    fn _compute_exact_similarity(self, query: List[Float32], vector: List[Float32]) -> Float32:
        """SIMD-optimized exact similarity computation."""
        let dimension = len(query)
        
        var dot_product: Float32 = 0.0
        var query_norm_sq: Float32 = 0.0
        var vector_norm_sq: Float32 = 0.0
        
        # SIMD-optimized computation
        for i in range(0, dimension, SIMD_WIDTH):
            let remaining = min(SIMD_WIDTH, dimension - i)
            var simd_dot: SIMD[DType.float32, SIMD_WIDTH] = 0.0
            var simd_query_norm: SIMD[DType.float32, SIMD_WIDTH] = 0.0
            var simd_vector_norm: SIMD[DType.float32, SIMD_WIDTH] = 0.0
            
            for j in range(remaining):
                if i + j < dimension:
                    let q_val = query[i + j]
                    let v_val = vector[i + j]
                    simd_dot[j] = q_val * v_val
                    simd_query_norm[j] = q_val * q_val
                    simd_vector_norm[j] = v_val * v_val
            
            dot_product += simd_dot.reduce_add()
            query_norm_sq += simd_query_norm.reduce_add()
            vector_norm_sq += simd_vector_norm.reduce_add()
        
        let query_norm = (query_norm_sq ** 0.5)
        let vector_norm = (vector_norm_sq ** 0.5)
        
        if query_norm > 0.0 and vector_norm > 0.0:
            return dot_product / (query_norm * vector_norm)
        else:
            return 0.0
    
    fn get_performance_stats(self) -> Tuple[Int, Int, Bool, String]:
        """Get performance statistics for benchmarking."""
        let vector_count = len(self.vectors)
        let edge_count = self.bipartite_graph.edge_count if self.is_trained else 0
        return (vector_count, edge_count, self.is_trained, self.device_context.device_type)


# Implement missing Set type for candidate collection
struct Set[T: KeyElement]:
    """Simple set implementation for candidate vector collection."""
    var data: List[T]
    
    fn __init__(inout self):
        self.data = List[T]()
    
    fn add(inout self, value: T):
        """Add value if not already present."""
        for i in range(len(self.data)):
            if self.data[i] == value:
                return  # Already exists
        self.data.append(value)
    
    fn __iter__(self) -> _SetIterator[T]:
        """Return iterator for the set."""
        return _SetIterator(self.data)


struct _SetIterator[T: KeyElement]:
    """Iterator for Set type."""
    var data: List[T]
    var index: Int
    
    fn __init__(inout self, data: List[T]):
        self.data = data
        self.index = 0
    
    fn __next__(inout self) -> T:
        """Get next item."""
        let value = self.data[self.index]
        self.index += 1
        return value
    
    fn __len__(self) -> Int:
        """Get remaining items."""
        return len(self.data) - self.index


fn benchmark_hybrid_roargraph(vector_count: Int, dimension: Int = 384, query_count: Int = 100) -> Tuple[Float64, Float64, Float64]:
    """
    Benchmark the hybrid RoarGraph implementation.
    
    Returns: (construction_time_ms, training_time_ms, avg_search_time_ms)
    """
    print(f"🏁 Benchmarking Hybrid RoarGraph: {vector_count} vectors, {dimension}D")
    print(f"   Device: CPU (SIMD-optimized), SIMD width: {SIMD_WIDTH}")
    
    # Initialize index
    var index = HybridRoarGraph("cpu", dimension)
    
    # Generate test data
    print("📊 Generating test data...")
    var test_vectors = List[List[Float32]]()
    var training_queries = List[List[Float32]]()
    var search_queries = List[List[Float32]]()
    
    # Generate realistic test vectors
    for v in range(vector_count):
        var vector = List[Float32]()
        for d in range(dimension):
            # Create normalized-like vectors (common in embeddings)
            let value = Float32((v * 31 + d * 17) % 1000) / 1000.0 - 0.5
            vector.append(value)
        test_vectors.append(vector)
    
    # Generate training and search queries
    for q in range(query_count):
        var query = List[Float32]()
        for d in range(dimension):
            let value = Float32((q * 23 + d * 41) % 1000) / 1000.0 - 0.5
            query.append(value)
        training_queries.append(query)
        search_queries.append(query)
    
    # Benchmark construction
    print("🏗️ Benchmarking construction...")
    var construction_start = time.perf_counter()
    
    for v in range(vector_count):
        try:
            index.add_vector(test_vectors[v])
        except e:
            print(f"Error adding vector {v}: {e}")
            break
    
    var construction_end = time.perf_counter()
    let construction_time = (construction_end - construction_start) * 1000.0
    
    # Benchmark training
    print("🎓 Benchmarking training...")
    var training_start = time.perf_counter()
    index.train_with_queries(training_queries)
    var training_end = time.perf_counter()
    let training_time = (training_end - training_start) * 1000.0
    
    # Benchmark search
    print("🔍 Benchmarking search...")
    let search_test_count = min(10, len(search_queries))
    var search_start = time.perf_counter()
    
    for q in range(search_test_count):
        let results = index.search(search_queries[q], k=10)
        
        # Verify results
        if len(results) > 0:
            let top_result = results[0]
            if q == 0:  # Print first result for verification
                print(f"   Sample result: vector {top_result[0]}, similarity {top_result[1]:.4f}")
    
    var search_end = time.perf_counter()
    let avg_search_time = (search_end - search_start) * 1000.0 / Float64(search_test_count)
    
    # Print results
    let stats = index.get_performance_stats()
    print(f"📈 Benchmark Results:")
    print(f"   Construction: {construction_time:.2f}ms ({Float64(vector_count * 1000) / construction_time:.0f} vec/s)")
    print(f"   Training: {training_time:.2f}ms")
    print(f"   Search: {avg_search_time:.3f}ms per query")
    print(f"   Final stats: {stats[0]} vectors, {stats[1]} edges, trained: {stats[2]}")
    print(f"   Device: {stats[3]}")
    
    return (construction_time, training_time, avg_search_time)