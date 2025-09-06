#!/usr/bin/env python3
"""
Benchmark comparison tool for OmenDB vs competitors.
Uses standard ANN-benchmarks datasets for fair comparison.
"""

import time
import json
import psutil
import numpy as np
from pathlib import Path
from typing import Dict, List, Tuple
import argparse

# Import databases - these need to be installed
def import_databases():
    """Import available databases for benchmarking."""
    import sys
    from pathlib import Path
    
    databases = {}
    
    # Add OmenDB Python bindings to path
    omendb_path = Path(__file__).parent.parent / "omendb" / "engine" / "python"
    if omendb_path.exists():
        sys.path.insert(0, str(omendb_path))
        try:
            import omendb
            databases['omendb'] = omendb
        except ImportError as e:
            print(f"Warning: OmenDB import failed: {e}")
    else:
        print(f"Warning: OmenDB path not found: {omendb_path}")
    
    try:
        import chromadb
        databases['chroma'] = chromadb
    except ImportError:
        print("Warning: ChromaDB not installed")
    
    try:
        import lancedb
        databases['lancedb'] = lancedb
    except ImportError:
        print("Warning: LanceDB not installed")
    
    return databases


def load_dataset(dataset_name: str) -> Tuple[np.ndarray, np.ndarray]:
    """Load standard dataset from ANN-benchmarks."""
    # These would normally come from ann-benchmarks datasets
    # For now, generate synthetic data
    
    if dataset_name == "sift-128":
        # SIFT1M: 128 dimensions, 1M vectors
        n_vectors = 100000  # Use 100K for quick testing
        dim = 128
    elif dataset_name == "gist-960":
        # GIST: 960 dimensions
        n_vectors = 100000
        dim = 960
    elif dataset_name == "glove-100":
        # GloVe: 100 dimensions
        n_vectors = 100000
        dim = 100
    else:
        n_vectors = 10000
        dim = 384  # Default OpenAI dimension
    
    print(f"Generating {dataset_name} dataset: {n_vectors} vectors, {dim} dimensions")
    vectors = np.random.randn(n_vectors, dim).astype(np.float32)
    queries = np.random.randn(1000, dim).astype(np.float32)
    
    return vectors, queries


def benchmark_omendb(vectors: np.ndarray, queries: np.ndarray) -> Dict:
    """Benchmark OmenDB performance."""
    import sys
    from pathlib import Path
    
    # Add OmenDB Python bindings to path
    omendb_path = Path(__file__).parent.parent / "omendb" / "engine" / "python"
    sys.path.insert(0, str(omendb_path))
    
    from omendb import DB
    
    # Create temporary database file
    db_path = Path("/tmp/omendb_benchmark.db")
    if db_path.exists():
        db_path.unlink()
    
    db = DB(str(db_path))
    
    # Measure build time
    start = time.time()
    for i, vec in enumerate(vectors):
        db.add(f"vec_{i}", vec.tolist())
    build_time = time.time() - start
    
    # Measure memory
    process = psutil.Process()
    memory_mb = process.memory_info().rss / 1024 / 1024
    
    # Measure query time
    search_times = []
    for q in queries[:100]:  # Test with 100 queries
        start = time.time()
        results = db.search(q.tolist(), limit=10)
        search_times.append(time.time() - start)
    query_time = sum(search_times)
    
    return {
        'build_rate': len(vectors) / build_time,
        'qps': 100 / query_time,
        'memory_mb': memory_mb,
        'memory_per_vector': memory_mb * 1024 * 1024 / len(vectors),
        'build_time': build_time,
        'query_time': query_time,
        'avg_query_ms': (query_time / 100) * 1000
    }


def benchmark_chroma(vectors: np.ndarray, queries: np.ndarray) -> Dict:
    """Benchmark ChromaDB performance."""
    import chromadb
    import tempfile
    
    # Use temporary directory for ChromaDB
    with tempfile.TemporaryDirectory() as tmpdir:
        client = chromadb.PersistentClient(path=tmpdir)
        
        # Reset and create collection
        client.reset()
        collection = client.create_collection("benchmark")
        
        # Measure build time
        ids = [f"vec_{i}" for i in range(len(vectors))]
        start = time.time()
        # Chroma requires batching for large datasets
        batch_size = 5000
        for i in range(0, len(vectors), batch_size):
            batch_ids = ids[i:i+batch_size]
            batch_vecs = vectors[i:i+batch_size].tolist()
            collection.add(
                ids=batch_ids,
                embeddings=batch_vecs
            )
        build_time = time.time() - start
        
        # Measure memory
        process = psutil.Process()
        memory_mb = process.memory_info().rss / 1024 / 1024
        
        # Measure query time
        search_times = []
        for q in queries[:100]:
            start = time.time()
            results = collection.query(
                query_embeddings=[q.tolist()],
                n_results=10
            )
            search_times.append(time.time() - start)
        query_time = sum(search_times)
        
        return {
            'build_rate': len(vectors) / build_time,
            'qps': 100 / query_time,
            'memory_mb': memory_mb,
            'memory_per_vector': memory_mb * 1024 * 1024 / len(vectors),
            'build_time': build_time,
            'query_time': query_time,
            'avg_query_ms': (query_time / 100) * 1000
        }


def run_benchmark(dataset_name: str) -> Dict:
    """Run benchmark on all available databases."""
    print(f"\n{'='*60}")
    print(f"Running benchmark on dataset: {dataset_name}")
    print(f"{'='*60}")
    
    # Load dataset
    vectors, queries = load_dataset(dataset_name)
    
    # Import available databases
    databases = import_databases()
    
    results = {}
    
    # Benchmark each database
    if 'omendb' in databases:
        print("\nBenchmarking OmenDB...")
        try:
            results['omendb'] = benchmark_omendb(vectors, queries)
        except Exception as e:
            print(f"OmenDB benchmark failed: {e}")
    
    if 'chroma' in databases:
        print("\nBenchmarking ChromaDB...")
        try:
            results['chroma'] = benchmark_chroma(vectors, queries)
        except Exception as e:
            print(f"ChromaDB benchmark failed: {e}")
    
    return results


def print_results(results: Dict):
    """Print benchmark results in a nice table."""
    print(f"\n{'='*90}")
    print("BENCHMARK RESULTS")
    print(f"{'='*90}")
    
    if not results:
        print("No results to display")
        return
    
    # Header
    print(f"{'Database':<15} {'Build (vec/s)':<15} {'Query (QPS)':<15} "
          f"{'Avg Query (ms)':<15} {'Memory (MB)':<12} {'Bytes/Vector':<12}")
    print("-" * 90)
    
    # Results
    for db_name, metrics in results.items():
        print(f"{db_name:<15} "
              f"{metrics['build_rate']:<15.0f} "
              f"{metrics['qps']:<15.1f} "
              f"{metrics.get('avg_query_ms', 0):<15.2f} "
              f"{metrics['memory_mb']:<12.1f} "
              f"{metrics['memory_per_vector']:<12.0f}")
    
    print(f"{'='*90}")
    
    # Find winner for each metric
    if len(results) > 1:
        print("\nWINNERS:")
        metrics_to_compare = [
            ('build_rate', 'Fastest Build', True),
            ('qps', 'Fastest Query', True),
            ('avg_query_ms', 'Lowest Query Latency', False),
            ('memory_per_vector', 'Most Memory Efficient', False)
        ]
        
        for metric, label, higher_better in metrics_to_compare:
            values = {db: res.get(metric, 0) for db, res in results.items()}
            if higher_better:
                winner = max(values, key=values.get)
            else:
                winner = min(values, key=values.get)
            
            # Format value appropriately
            value = values[winner]
            if metric == 'avg_query_ms':
                formatted = f"{value:.2f}ms"
            elif metric == 'memory_per_vector':
                formatted = f"{value:.0f} bytes"
            else:
                formatted = f"{value:.1f}"
            
            print(f"  {label}: {winner} ({formatted})")


def save_results(results: Dict, filename: str):
    """Save results to JSON file."""
    output_dir = Path("benchmark_results")
    output_dir.mkdir(exist_ok=True)
    
    output_file = output_dir / filename
    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)
    
    print(f"\nResults saved to {output_file}")


def main():
    parser = argparse.ArgumentParser(
        description="Benchmark OmenDB against competitors"
    )
    parser.add_argument(
        '--dataset',
        choices=['sift-128', 'gist-960', 'glove-100', 'default'],
        default='default',
        help='Dataset to use for benchmarking'
    )
    parser.add_argument(
        '--output',
        default='benchmark_results.json',
        help='Output filename for results'
    )
    
    args = parser.parse_args()
    
    # Run benchmark
    results = run_benchmark(args.dataset)
    
    # Print results
    print_results(results)
    
    # Save results
    if results:
        save_results(results, args.output)


if __name__ == "__main__":
    main()