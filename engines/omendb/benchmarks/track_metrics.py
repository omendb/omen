#!/usr/bin/env python3
"""
OmenDB Metrics Tracker
Tracks performance metrics over time for historical analysis
"""

import json
import time
import subprocess
from datetime import datetime
from pathlib import Path
import numpy as np

class MetricsTracker:
    def __init__(self, metrics_file="metrics_history.json"):
        self.metrics_file = Path(metrics_file)
        self.history = self.load_history()
    
    def load_history(self):
        """Load existing metrics history"""
        if self.metrics_file.exists():
            with open(self.metrics_file, 'r') as f:
                return json.load(f)
        return []
    
    def save_history(self):
        """Save metrics history to file"""
        with open(self.metrics_file, 'w') as f:
            json.dump(self.history, f, indent=2)
    
    def run_benchmark(self, vector_count=10000, dimension=128):
        """Run benchmark and collect metrics"""
        import sys
        sys.path.insert(0, '../python')
        from omendb import OmenDB
        
        # Initialize database
        db = OmenDB(use_quantization=True)
        
        # Generate test vectors
        vectors = np.random.randn(vector_count, dimension).astype(np.float32)
        keys = [f"vec_{i}" for i in range(vector_count)]
        
        # Measure batch insert
        start = time.perf_counter()
        db.add_batch(keys, vectors)
        insert_time = time.perf_counter() - start
        insert_throughput = vector_count / insert_time
        
        # Measure search latency
        query = vectors[0]
        latencies = []
        for _ in range(100):
            start = time.perf_counter()
            results = db.search(query, k=10)
            latencies.append((time.perf_counter() - start) * 1000)  # ms
        
        search_p50 = np.percentile(latencies, 50)
        search_p95 = np.percentile(latencies, 95)
        
        # Get memory stats
        memory_stats = db.get_memory_stats()
        bytes_per_vector = memory_stats.get('total_bytes', 0) / vector_count
        
        # Get git commit
        try:
            commit = subprocess.check_output(
                ['git', 'rev-parse', '--short', 'HEAD'],
                text=True
            ).strip()
        except:
            commit = 'unknown'
        
        return {
            'timestamp': datetime.now().isoformat(),
            'commit': commit,
            'vector_count': vector_count,
            'dimension': dimension,
            'insert_throughput': round(insert_throughput, 0),
            'search_p50_ms': round(search_p50, 2),
            'search_p95_ms': round(search_p95, 2),
            'bytes_per_vector': round(bytes_per_vector, 0),
        }
    
    def track(self, vector_count=10000, dimension=128):
        """Run benchmark and save metrics"""
        print(f"Running benchmark with {vector_count} vectors...")
        metrics = self.run_benchmark(vector_count, dimension)
        
        self.history.append(metrics)
        self.save_history()
        
        print("\nMetrics collected:")
        for key, value in metrics.items():
            print(f"  {key}: {value}")
        
        # Compare with previous run
        if len(self.history) > 1:
            prev = self.history[-2]
            print("\nComparison with previous run:")
            
            # Throughput
            prev_throughput = prev.get('insert_throughput', 0)
            curr_throughput = metrics['insert_throughput']
            change = ((curr_throughput - prev_throughput) / prev_throughput * 100) if prev_throughput else 0
            print(f"  Throughput: {change:+.1f}%")
            
            # Memory
            prev_memory = prev.get('bytes_per_vector', 0)
            curr_memory = metrics['bytes_per_vector']
            change = ((curr_memory - prev_memory) / prev_memory * 100) if prev_memory else 0
            print(f"  Memory: {change:+.1f}%")
            
            # Latency
            prev_latency = prev.get('search_p50_ms', 0)
            curr_latency = metrics['search_p50_ms']
            change = ((curr_latency - prev_latency) / prev_latency * 100) if prev_latency else 0
            print(f"  Latency: {change:+.1f}%")
    
    def report(self, last_n=10):
        """Generate a report of recent metrics"""
        if not self.history:
            print("No metrics history found")
            return
        
        recent = self.history[-last_n:]
        
        print(f"\n{'='*80}")
        print(f"OmenDB Performance Trends (last {len(recent)} runs)")
        print(f"{'='*80}")
        print(f"{'Date':<20} {'Commit':<8} {'Throughput':<12} {'Memory':<10} {'P50 Latency':<12}")
        print(f"{'-'*80}")
        
        for m in recent:
            date = m['timestamp'][:10]
            commit = m['commit'][:7]
            throughput = f"{m['insert_throughput']:.0f} vec/s"
            memory = f"{m['bytes_per_vector']:.0f} B/vec"
            latency = f"{m['search_p50_ms']:.2f} ms"
            print(f"{date:<20} {commit:<8} {throughput:<12} {memory:<10} {latency:<12}")
        
        # Calculate trends
        if len(recent) > 1:
            print(f"\n{'='*80}")
            print("Trends:")
            
            # Throughput trend
            throughputs = [m['insert_throughput'] for m in recent]
            trend = (throughputs[-1] - throughputs[0]) / throughputs[0] * 100
            print(f"  Throughput: {trend:+.1f}% overall")
            
            # Memory trend
            memories = [m['bytes_per_vector'] for m in recent]
            trend = (memories[-1] - memories[0]) / memories[0] * 100
            print(f"  Memory: {trend:+.1f}% overall")
            
            # Best/worst
            best_throughput = max(throughputs)
            best_memory = min(memories)
            print(f"\n  Best throughput: {best_throughput:.0f} vec/s")
            print(f"  Best memory: {best_memory:.0f} bytes/vector")

if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="Track OmenDB performance metrics")
    parser.add_argument('--track', action='store_true', help='Run benchmark and track metrics')
    parser.add_argument('--report', action='store_true', help='Show metrics report')
    parser.add_argument('--vectors', type=int, default=10000, help='Number of vectors for benchmark')
    parser.add_argument('--dimension', type=int, default=128, help='Vector dimension')
    
    args = parser.parse_args()
    
    tracker = MetricsTracker()
    
    if args.track:
        tracker.track(args.vectors, args.dimension)
    elif args.report:
        tracker.report()
    else:
        # Default: run benchmark and show report
        tracker.track(args.vectors, args.dimension)
        tracker.report(last_n=5)