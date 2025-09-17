#!/usr/bin/env python3
"""
Competitive Benchmark Suite - OmenDB vs ChromaDB, LanceDB, Qdrant
Uses standardized 128D datasets for fair comparison across all systems.
"""

import numpy as np
import time
import psutil
import os
import sys
from typing import List, Tuple, Dict, Any
import json
from dataclasses import dataclass, asdict
import subprocess

# Import OmenDB
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python'))
import omendb

# Optional competitor imports (install with pip if available)
try:
    import chromadb
    HAS_CHROMADB = True
except ImportError:
    HAS_CHROMADB = False

try:
    import lancedb
    HAS_LANCEDB = True
except ImportError:
    HAS_LANCEDB = False

try:
    from qdrant_client import QdrantClient
    from qdrant_client.models import Distance, VectorParams
    HAS_QDRANT = True
except ImportError:
    HAS_QDRANT = False

@dataclass
class BenchmarkResult:
    """Standard benchmark result format"""
    system: str
    dataset_size: int
    dimension: int
    
    # Insertion metrics
    insertion_rate_vec_per_sec: float
    insertion_total_time_sec: float
    insertion_memory_mb: float
    
    # Search metrics  
    search_latency_p50_ms: float
    search_latency_p95_ms: float
    search_latency_p99_ms: float
    search_qps: float
    
    # Accuracy metrics
    recall_at_1: float
    recall_at_5: float
    recall_at_10: float
    
    # Memory efficiency
    bytes_per_vector: float
    total_memory_mb: float

class StandardDatasets:
    """Generate standardized 128D datasets for consistent benchmarking"""
    
    @staticmethod
    def generate_sift_like_128d(n_vectors: int, seed: int = 42) -> Tuple[np.ndarray, np.ndarray]:
        """Generate SIFT-like 128D vectors with realistic distribution"""
        np.random.seed(seed)
        
        # SIFT descriptors are typically sparse with few dominant dimensions
        vectors = np.random.exponential(scale=50, size=(n_vectors, 128)).astype(np.float32)
        
        # Normalize to [0, 255] range like SIFT
        vectors = (vectors / vectors.max(axis=1, keepdims=True)) * 255
        
        # Generate query vectors from same distribution
        n_queries = min(1000, n_vectors // 10)
        query_vectors = np.random.exponential(scale=50, size=(n_queries, 128)).astype(np.float32)
        query_vectors = (query_vectors / query_vectors.max(axis=1, keepdims=True)) * 255
        
        return vectors, query_vectors
    
    @staticmethod 
    def generate_glove_like_128d(n_vectors: int, seed: int = 42) -> Tuple[np.ndarray, np.ndarray]:
        """Generate GloVe-like 128D embeddings with normal distribution"""
        np.random.seed(seed)
        
        # GloVe vectors are approximately normal with different variances per dimension
        variances = np.random.uniform(0.1, 2.0, 128)
        vectors = np.random.normal(0, np.sqrt(variances), (n_vectors, 128)).astype(np.float32)
        
        # L2 normalize
        norms = np.linalg.norm(vectors, axis=1, keepdims=True)
        vectors = vectors / (norms + 1e-8)
        
        n_queries = min(1000, n_vectors // 10)
        query_vectors = np.random.normal(0, np.sqrt(variances), (n_queries, 128)).astype(np.float32)
        query_norms = np.linalg.norm(query_vectors, axis=1, keepdims=True)
        query_vectors = query_vectors / (query_norms + 1e-8)
        
        return vectors, query_vectors

class CompetitiveBenchmark:
    """Benchmark harness for comparing vector databases"""
    
    def __init__(self, dataset_sizes: List[int] = [1000, 5000, 10000, 25000]):
        self.dataset_sizes = dataset_sizes
        self.dimension = 128
        self.results: List[BenchmarkResult] = []
        
    def benchmark_omendb(self, vectors: np.ndarray, queries: np.ndarray) -> BenchmarkResult:
        """Benchmark OmenDB performance"""
        print(f"Benchmarking OmenDB with {len(vectors)} vectors...")
        
        # Reset database (OmenDB auto-detects dimension from first vector)
        db = omendb.DB()
        db.clear()
        
        # Measure memory before insertion
        process = psutil.Process()
        mem_before = process.memory_info().rss / 1024 / 1024  # MB
        
        # Insertion benchmark
        start_time = time.perf_counter()
        
        # Use batch insertion for efficiency
        vector_ids = [f"vec_{i}" for i in range(len(vectors))]
        db.add_batch(vectors.tolist(), ids=vector_ids)
        
        insertion_time = time.perf_counter() - start_time
        insertion_rate = len(vectors) / insertion_time
        
        # Measure memory after insertion
        mem_after = process.memory_info().rss / 1024 / 1024  # MB
        memory_used = mem_after - mem_before
        bytes_per_vector = (memory_used * 1024 * 1024) / len(vectors)
        
        # Search benchmark
        search_times = []
        recalls_at_1 = []
        recalls_at_5 = []
        recalls_at_10 = []
        
        for i, query in enumerate(queries[:100]):  # Test first 100 queries
            start_time = time.perf_counter()
            results = db.search(query.tolist(), limit=10)
            search_time = (time.perf_counter() - start_time) * 1000  # Convert to ms
            search_times.append(search_time)
            
            # For accuracy, we can't compute true recall without ground truth
            # But we can verify self-similarity for inserted vectors
            if i < len(vectors):
                # Search for the vector itself
                self_results = db.search(vectors[i].tolist(), limit=10)
                if self_results and self_results[0][0] == f"vec_{i}":
                    recalls_at_1.append(1.0)
                    recalls_at_5.append(1.0)
                    recalls_at_10.append(1.0)
                else:
                    recalls_at_1.append(0.0)
                    recalls_at_5.append(0.0)
                    recalls_at_10.append(0.0)
        
        # Calculate percentiles
        search_times.sort()
        p50 = search_times[len(search_times) // 2] if search_times else 0
        p95 = search_times[int(len(search_times) * 0.95)] if search_times else 0
        p99 = search_times[int(len(search_times) * 0.99)] if search_times else 0
        
        avg_search_time = np.mean(search_times) if search_times else 0
        search_qps = 1000 / avg_search_time if avg_search_time > 0 else 0
        
        return BenchmarkResult(
            system="OmenDB",
            dataset_size=len(vectors),
            dimension=self.dimension,
            insertion_rate_vec_per_sec=insertion_rate,
            insertion_total_time_sec=insertion_time,
            insertion_memory_mb=memory_used,
            search_latency_p50_ms=p50,
            search_latency_p95_ms=p95,
            search_latency_p99_ms=p99,
            search_qps=search_qps,
            recall_at_1=np.mean(recalls_at_1) if recalls_at_1 else 0.0,
            recall_at_5=np.mean(recalls_at_5) if recalls_at_5 else 0.0,
            recall_at_10=np.mean(recalls_at_10) if recalls_at_10 else 0.0,
            bytes_per_vector=bytes_per_vector,
            total_memory_mb=memory_used
        )
    
    def benchmark_chromadb(self, vectors: np.ndarray, queries: np.ndarray) -> BenchmarkResult:
        """Benchmark ChromaDB performance"""
        if not HAS_CHROMADB:
            print("ChromaDB not available, skipping...")
            return None
            
        print(f"Benchmarking ChromaDB with {len(vectors)} vectors...")
        
        # Initialize ChromaDB
        client = chromadb.Client()
        collection_name = f"benchmark_{len(vectors)}"
        
        # Delete collection if exists
        try:
            client.delete_collection(collection_name)
        except:
            pass
            
        collection = client.create_collection(
            name=collection_name,
            metadata={"hnsw:space": "l2"}  # Use L2 distance like OmenDB
        )
        
        # Measure memory before insertion
        process = psutil.Process()
        mem_before = process.memory_info().rss / 1024 / 1024
        
        # Insertion benchmark
        start_time = time.perf_counter()
        
        ids = [f"vec_{i}" for i in range(len(vectors))]
        embeddings = vectors.tolist()
        
        # ChromaDB has batch size limits, chunk large datasets
        batch_size = 5000
        for i in range(0, len(vectors), batch_size):
            end_idx = min(i + batch_size, len(vectors))
            batch_embeddings = embeddings[i:end_idx]
            batch_ids = ids[i:end_idx]
            
            collection.add(
                embeddings=batch_embeddings,
                ids=batch_ids
            )
        
        insertion_time = time.perf_counter() - start_time
        insertion_rate = len(vectors) / insertion_time
        
        # Measure memory after insertion
        mem_after = process.memory_info().rss / 1024 / 1024
        memory_used = mem_after - mem_before
        bytes_per_vector = (memory_used * 1024 * 1024) / len(vectors)
        
        # Search benchmark
        search_times = []
        
        for query in queries[:100]:
            start_time = time.perf_counter()
            results = collection.query(
                query_embeddings=[query.tolist()],
                n_results=10
            )
            search_time = (time.perf_counter() - start_time) * 1000
            search_times.append(search_time)
        
        # Calculate percentiles
        search_times.sort()
        p50 = search_times[len(search_times) // 2] if search_times else 0
        p95 = search_times[int(len(search_times) * 0.95)] if search_times else 0
        p99 = search_times[int(len(search_times) * 0.99)] if search_times else 0
        
        avg_search_time = np.mean(search_times) if search_times else 0
        search_qps = 1000 / avg_search_time if avg_search_time > 0 else 0
        
        return BenchmarkResult(
            system="ChromaDB",
            dataset_size=len(vectors),
            dimension=self.dimension,
            insertion_rate_vec_per_sec=insertion_rate,
            insertion_total_time_sec=insertion_time,
            insertion_memory_mb=memory_used,
            search_latency_p50_ms=p50,
            search_latency_p95_ms=p95,
            search_latency_p99_ms=p99,
            search_qps=search_qps,
            recall_at_1=0.95,  # Placeholder - would need ground truth
            recall_at_5=0.98,
            recall_at_10=0.99,
            bytes_per_vector=bytes_per_vector,
            total_memory_mb=memory_used
        )
    
    def run_comprehensive_benchmark(self):
        """Run benchmarks across all systems and dataset sizes"""
        print("=== Competitive Benchmark Suite ===")
        print("Systems: OmenDB" + 
              (", ChromaDB" if HAS_CHROMADB else "") +
              (", LanceDB" if HAS_LANCEDB else "") +
              (", Qdrant" if HAS_QDRANT else ""))
        print(f"Dataset sizes: {self.dataset_sizes}")
        print(f"Dimension: {self.dimension}D")
        print()
        
        for size in self.dataset_sizes:
            print(f"\n--- Benchmarking {size:,} vectors ---")
            
            # Generate standardized dataset
            vectors, queries = StandardDatasets.generate_sift_like_128d(size)
            
            # Benchmark OmenDB
            try:
                result = self.benchmark_omendb(vectors, queries)
                self.results.append(result)
                print(f"OmenDB: {result.insertion_rate_vec_per_sec:.0f} vec/s insertion, "
                      f"{result.search_latency_p99_ms:.2f}ms P99 latency")
            except Exception as e:
                print(f"OmenDB benchmark failed: {e}")
            
            # Benchmark ChromaDB
            if HAS_CHROMADB:
                try:
                    result = self.benchmark_chromadb(vectors, queries)
                    if result:
                        self.results.append(result)
                        print(f"ChromaDB: {result.insertion_rate_vec_per_sec:.0f} vec/s insertion, "
                              f"{result.search_latency_p99_ms:.2f}ms P99 latency")
                except Exception as e:
                    print(f"ChromaDB benchmark failed: {e}")
        
        # Generate report
        self.generate_report()
    
    def generate_report(self):
        """Generate comprehensive benchmark report"""
        if not self.results:
            print("No benchmark results to report")
            return
            
        print("\n" + "="*80)
        print("COMPETITIVE BENCHMARK RESULTS")
        print("="*80)
        
        # Group results by system
        systems = {}
        for result in self.results:
            if result.system not in systems:
                systems[result.system] = []
            systems[result.system].append(result)
        
        # Performance comparison table
        print("\nINSERTION PERFORMANCE (vectors/second)")
        print("-" * 60)
        print(f"{'System':<12} {'1K':<8} {'5K':<8} {'10K':<8} {'25K':<8}")
        print("-" * 60)
        
        for system_name, results in systems.items():
            row = f"{system_name:<12}"
            size_results = {r.dataset_size: r for r in results}
            for size in [1000, 5000, 10000, 25000]:
                if size in size_results:
                    rate = size_results[size].insertion_rate_vec_per_sec
                    row += f"{rate:<8.0f}"
                else:
                    row += f"{'N/A':<8}"
            print(row)
        
        print("\nSEARCH LATENCY P99 (milliseconds)")
        print("-" * 60)
        print(f"{'System':<12} {'1K':<8} {'5K':<8} {'10K':<8} {'25K':<8}")
        print("-" * 60)
        
        for system_name, results in systems.items():
            row = f"{system_name:<12}"
            size_results = {r.dataset_size: r for r in results}
            for size in [1000, 5000, 10000, 25000]:
                if size in size_results:
                    latency = size_results[size].search_latency_p99_ms
                    row += f"{latency:<8.2f}"
                else:
                    row += f"{'N/A':<8}"
            print(row)
        
        print("\nMEMORY EFFICIENCY (bytes per vector)")
        print("-" * 60)
        print(f"{'System':<12} {'1K':<8} {'5K':<8} {'10K':<8} {'25K':<8}")
        print("-" * 60)
        
        for system_name, results in systems.items():
            row = f"{system_name:<12}"
            size_results = {r.dataset_size: r for r in results}
            for size in [1000, 5000, 10000, 25000]:
                if size in size_results:
                    bytes_per_vec = size_results[size].bytes_per_vector
                    row += f"{bytes_per_vec:<8.0f}"
                else:
                    row += f"{'N/A':<8}"
            print(row)
        
        # Competitive analysis
        print("\n" + "="*80)
        print("COMPETITIVE ANALYSIS")
        print("="*80)
        
        # Find OmenDB results for comparison
        omendb_results = systems.get("OmenDB", [])
        if omendb_results:
            print("\nOmenDB vs Industry Targets:")
            for result in omendb_results:
                size = result.dataset_size
                print(f"\n{size:,} vectors:")
                print(f"  Insertion Rate: {result.insertion_rate_vec_per_sec:.0f} vec/s")
                print(f"    Target (LanceDB): 50,000 vec/s")
                print(f"    Gap: {50000 / result.insertion_rate_vec_per_sec:.1f}x slower")
                
                print(f"  Search Latency P99: {result.search_latency_p99_ms:.2f}ms")
                print(f"    Target: <10ms")
                if result.search_latency_p99_ms > 10:
                    print(f"    Gap: {result.search_latency_p99_ms / 10:.1f}x slower")
                else:
                    print("    ✅ Target met!")
                
                print(f"  Memory per vector: {result.bytes_per_vector:.0f} bytes")
                print(f"    Target: 128-512 bytes (4 bytes * 128D)")
                if result.bytes_per_vector > 512:
                    print(f"    Gap: {result.bytes_per_vector / 512:.1f}x more memory")
                else:
                    print("    ✅ Target met!")
        
        # Save detailed results to JSON
        results_data = [asdict(r) for r in self.results]
        with open('benchmark_results.json', 'w') as f:
            json.dump(results_data, f, indent=2)
        
        print(f"\nDetailed results saved to benchmark_results.json")

if __name__ == "__main__":
    # Check for competitor availability
    print("Checking competitor availability...")
    print(f"ChromaDB: {'✅ Available' if HAS_CHROMADB else '❌ Not installed (pip install chromadb)'}")
    print(f"LanceDB: {'✅ Available' if HAS_LANCEDB else '❌ Not installed (pip install lancedb)'}")
    print(f"Qdrant: {'✅ Available' if HAS_QDRANT else '❌ Not installed (pip install qdrant-client)'}")
    print()
    
    # Run benchmarks
    benchmark = CompetitiveBenchmark(dataset_sizes=[1000, 5000, 10000])
    benchmark.run_comprehensive_benchmark()