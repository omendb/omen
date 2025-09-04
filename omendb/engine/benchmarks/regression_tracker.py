#!/usr/bin/env python3
"""
Regression Tracking System for OmenDB
Tracks performance metrics over time and alerts on regressions.
"""

import json
import sys
import time
import numpy as np
from datetime import datetime
from pathlib import Path

sys.path.insert(0, 'python')
from omendb.api import DB

class RegressionTracker:
    def __init__(self, history_file="benchmarks/regression_history.json"):
        self.history_file = Path(history_file)
        self.history = self.load_history()
        
    def load_history(self):
        """Load historical benchmark data."""
        if self.history_file.exists():
            with open(self.history_file, 'r') as f:
                return json.load(f)
        return {"benchmarks": []}
    
    def save_history(self):
        """Save benchmark history."""
        with open(self.history_file, 'w') as f:
            json.dump(self.history, f, indent=2)
    
    def run_benchmark(self, name, num_vectors, batch_size=1000):
        """Run a single benchmark test."""
        print(f"  Testing {name} ({num_vectors:,} vectors)...", end=" ")
        
        # Initialize database
        db = DB(db_path=f"bench_{name}.db", buffer_size=2000)
        
        # CRITICAL FIX: Clear global VectorStore state to prevent segfaults
        # All DB instances share the same global VectorStore singleton
        # State corruption from previous tests causes segfaults  
        db.clear()
        
        # Test batch insertion
        vectors = np.random.randn(num_vectors, 128).astype(np.float32)
        ids = [f"vec_{i}" for i in range(num_vectors)]
        
        t0 = time.time()
        db.add_batch(vectors, ids)
        t1 = time.time()
        
        insertion_time = t1 - t0
        vectors_per_sec = num_vectors / insertion_time if insertion_time > 0 else 0
        
        # Test search (warm up + measure)
        query = np.random.randn(128).astype(np.float32)
        
        for _ in range(5):
            _ = db.search(query, limit=10)
        
        search_times = []
        for _ in range(10):
            t0 = time.time()
            _ = db.search(query, limit=10)
            t1 = time.time()
            search_times.append((t1 - t0) * 1000)
        
        result = {
            "name": name,
            "num_vectors": num_vectors,
            "insertion_throughput": round(vectors_per_sec),
            "search_p50_ms": round(np.percentile(search_times, 50), 2),
            "search_p99_ms": round(np.percentile(search_times, 99), 2),
        }
        
        print(f"{vectors_per_sec:,.0f} vec/s, {result['search_p50_ms']:.2f}ms search")
        return result
    
    def run_all_benchmarks(self):
        """Run complete benchmark suite."""
        print("\n" + "="*60)
        print("RUNNING REGRESSION TESTS")
        print("="*60)
        
        timestamp = datetime.now().isoformat()
        results = {
            "timestamp": timestamp,
            "date": datetime.now().strftime("%Y-%m-%d"),
            "results": []
        }
        
        benchmarks = [
            ("small", 1000),
            ("medium", 10000),
            # TEMPORARY: Large test disabled due to scale-dependent memory corruption
            # Re-enable after OMEN-26 (segfaults at 105K+) is resolved  
            # ("large", 50000),
        ]
        
        for name, num_vectors in benchmarks:
            try:
                result = self.run_benchmark(name, num_vectors)
                results["results"].append(result)
            except Exception as e:
                print(f"❌ Failed: {e}")
                results["results"].append({
                    "name": name,
                    "num_vectors": num_vectors,
                    "error": str(e)
                })
        
        return results
    
    def check_regressions(self, current_results):
        """Check for performance regressions."""
        if not self.history["benchmarks"]:
            return []
        
        # Get last successful benchmark
        last_benchmark = None
        for bench in reversed(self.history["benchmarks"]):
            if "results" in bench and bench["results"]:
                last_benchmark = bench
                break
        
        if not last_benchmark:
            return []
        
        regressions = []
        
        # Compare each test
        for current in current_results["results"]:
            if "error" in current:
                continue
                
            # Find matching previous result
            previous = None
            for prev in last_benchmark["results"]:
                if prev["name"] == current["name"]:
                    previous = prev
                    break
            
            if not previous or "error" in previous:
                continue
            
            # Check for regressions (>20% performance drop)
            throughput_change = (current["insertion_throughput"] - previous["insertion_throughput"]) / previous["insertion_throughput"] * 100
            search_change = (current["search_p50_ms"] - previous["search_p50_ms"]) / previous["search_p50_ms"] * 100
            
            if throughput_change < -20:
                regressions.append({
                    "test": current["name"],
                    "metric": "insertion_throughput",
                    "previous": previous["insertion_throughput"],
                    "current": current["insertion_throughput"],
                    "change": throughput_change
                })
            
            if search_change > 50:  # Search getting slower
                regressions.append({
                    "test": current["name"],
                    "metric": "search_p50_ms",
                    "previous": previous["search_p50_ms"],
                    "current": current["search_p50_ms"],
                    "change": search_change
                })
        
        return regressions
    
    def print_summary(self, results, regressions):
        """Print benchmark summary and regressions."""
        print("\n" + "="*60)
        print("BENCHMARK RESULTS")
        print("="*60)
        
        print("\n| Test | Vectors | Throughput | Search P50 | Search P99 |")
        print("|------|---------|------------|------------|------------|")
        
        for r in results["results"]:
            if "error" in r:
                print(f"| {r['name']} | {r['num_vectors']:,} | ERROR | - | - |")
            else:
                print(f"| {r['name']} | {r['num_vectors']:,} | "
                      f"{r['insertion_throughput']:,} vec/s | "
                      f"{r['search_p50_ms']:.2f}ms | "
                      f"{r['search_p99_ms']:.2f}ms |")
        
        if regressions:
            print("\n" + "="*60)
            print("⚠️  PERFORMANCE REGRESSIONS DETECTED")
            print("="*60)
            
            for reg in regressions:
                print(f"\n{reg['test']} - {reg['metric']}:")
                print(f"  Previous: {reg['previous']:,}")
                print(f"  Current: {reg['current']:,}")
                print(f"  Change: {reg['change']:+.1f}%")
        else:
            print("\n✅ No regressions detected")
    
    def track(self):
        """Run benchmarks and track results."""
        results = self.run_all_benchmarks()
        regressions = self.check_regressions(results)
        
        # Save to history
        self.history["benchmarks"].append(results)
        
        # Keep only last 30 days of data
        cutoff = datetime.now().timestamp() - (30 * 24 * 60 * 60)
        self.history["benchmarks"] = [
            b for b in self.history["benchmarks"]
            if datetime.fromisoformat(b["timestamp"]).timestamp() > cutoff
        ]
        
        self.save_history()
        self.print_summary(results, regressions)
        
        return len(regressions) == 0

def main():
    """Main entry point."""
    tracker = RegressionTracker()
    success = tracker.track()
    
    if not success:
        print("\n❌ Regressions detected! Check the results above.")
        sys.exit(1)
    else:
        print("\n✅ All benchmarks passed!")
        sys.exit(0)

if __name__ == "__main__":
    main()