#!/usr/bin/env python3
"""
Automated Regression Testing for OmenDB
======================================

Monitors performance and correctness across commits to prevent regressions.
Runs comprehensive test suite and performance benchmarks automatically.
"""

import sys
import os
import time
import json
import subprocess
from datetime import datetime
from typing import Dict, List, Tuple, Optional

sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
sys.path.insert(0, '/Users/nick/github/omendb/omendb/benchmarks')

import omendb
import omendb.native as native
import numpy as np
from standardized_datasets import StandardizedDatasets

class RegressionMonitor:
    """Automated regression testing and performance monitoring."""
    
    def __init__(self, results_file: str = "regression_results.json"):
        self.results_file = results_file
        self.current_results = {}
        self.baseline_performance = {
            "batch_1k": 70000,  # vec/s baseline
            "batch_5k": 68000,  # vec/s baseline  
            "search_latency": 1.0,  # ms baseline
            "test_pass_rate": 100.0,  # % baseline
            "recall_at_10": 0.95,  # Industry standard baseline
            "recall_at_100": 0.98   # Industry standard baseline
        }
    
    def run_performance_test(self) -> Dict:
        """Run standardized performance benchmark."""
        results = {}
        
        # Test different batch sizes
        batch_sizes = [100, 1000, 5000, 10000]
        
        for size in batch_sizes:
            native._reset()  # Clean state
            db = omendb.DB()
            vectors = np.random.rand(size, 128).astype(np.float32)
            
            start = time.perf_counter()
            ids = db.add_batch(vectors)
            elapsed = time.perf_counter() - start
            
            throughput = size / elapsed
            results[f"batch_{size}"] = {
                "throughput": throughput,
                "elapsed": elapsed,
                "vectors": len(ids)
            }
        
        # Test search latency
        native._reset()
        db = omendb.DB()
        setup_vectors = np.random.rand(10000, 128).astype(np.float32)
        db.add_batch(setup_vectors)
        
        query = np.random.rand(128).astype(np.float32)
        search_times = []
        
        for _ in range(10):
            start = time.perf_counter()
            search_results = db.search(query.tolist(), limit=10)
            elapsed = (time.perf_counter() - start) * 1000
            search_times.append(elapsed)
        
        results["search_latency"] = {
            "avg_ms": sum(search_times) / len(search_times),
            "min_ms": min(search_times),
            "max_ms": max(search_times)
        }
        
        return results
    
    def run_standardized_benchmark(self) -> Dict:
        """Run industry-standard dataset benchmark with Recall@10/100 metrics."""
        try:
            print("🔬 Running standardized dataset benchmark...")
            datasets = StandardizedDatasets()
            
            # Run SIFT-128 benchmark (industry standard)
            benchmark_results = datasets.run_standardized_benchmark(
                omendb.DB, buffer_size=25000
            )
            
            # Also test with smaller dataset for Recall@100
            print("\n📊 Testing Recall@100 with extended search...")
            native._reset()
            db = omendb.DB(buffer_size=25000)
            
            # Generate smaller test set for Recall@100 testing
            np.random.seed(42)
            n_vectors = 1000
            n_queries = 100
            dimension = 128
            
            # Create clustered vectors for realistic recall
            vectors = np.random.rand(n_vectors, dimension).astype(np.float32) * 100
            query_vectors = vectors[:n_queries] + np.random.normal(0, 0.1, (n_queries, dimension))
            
            # Add vectors
            vector_list = [vectors[i].tolist() for i in range(n_vectors)]
            ids = [f"vec_{i}" for i in range(n_vectors)]
            db.add_batch(ids, vector_list)
            db.flush()
            
            # Test Recall@100 (search for 100 neighbors)
            correct_100 = 0
            for i in range(n_queries):
                query = query_vectors[i].tolist()
                results = db.search(query, limit=100)
                
                # Check if original vector is in top 100 results
                target_id = f"vec_{i}"
                found_in_100 = any(result.id == target_id for result in results)
                if found_in_100:
                    correct_100 += 1
            
            recall_100 = correct_100 / n_queries
            
            return {
                "dataset": benchmark_results["dataset"],
                "recall_at_10": benchmark_results["recall_at_10"],
                "recall_at_100": recall_100,
                "insert_throughput": benchmark_results["insert_throughput_vec_per_sec"],
                "avg_search_latency_ms": benchmark_results["avg_search_latency_ms"],
                "queries_per_second": benchmark_results["queries_per_second"],
                "train_vectors": benchmark_results["train_size"],
                "test_vectors": benchmark_results["test_size"]
            }
            
        except Exception as e:
            return {
                "status": "ERROR",
                "error": str(e),
                "recall_at_10": 0.0,
                "recall_at_100": 0.0
            }
    
    def run_correctness_test(self) -> Dict:
        """Run comprehensive correctness test suite."""
        try:
            # Run the comprehensive test suite
            cmd = ["python", "test_comprehensive_correctness.py"]
            result = subprocess.run(
                cmd, 
                capture_output=True, 
                text=True,
                cwd="/Users/nick/github/omendb/omendb"
            )
            
            # Parse results
            output = result.stdout
            if "ALL TESTS PASSED" in output:
                pass_rate = 100.0
                # Extract test count
                if "/" in output:
                    passed_line = [line for line in output.split('\n') if '/' in line and 'passed' in line][-1]
                    passed, total = passed_line.split('/')[0].split()[-1], passed_line.split('/')[1].split()[0]
                    pass_rate = (int(passed) / int(total)) * 100
                
                return {
                    "status": "PASS",
                    "pass_rate": pass_rate,
                    "output_lines": len(output.split('\n'))
                }
            else:
                return {
                    "status": "FAIL", 
                    "pass_rate": 0.0,
                    "error_output": output
                }
        except Exception as e:
            return {
                "status": "ERROR",
                "pass_rate": 0.0, 
                "error": str(e)
            }
    
    def check_regression(self, current: Dict, baseline: Dict) -> List[str]:
        """Check for performance regressions."""
        regressions = []
        
        # Check batch performance
        for size in [1000, 5000]:
            key = f"batch_{size}"
            if key in current and key.replace("_", "_") in baseline:
                current_perf = current[key]["throughput"] 
                baseline_perf = baseline.get(key.replace("batch_", "batch_"), 0)
                
                regression_threshold = 0.9  # 10% regression threshold
                if current_perf < baseline_perf * regression_threshold:
                    regression_pct = ((baseline_perf - current_perf) / baseline_perf) * 100
                    regressions.append(
                        f"{key}: {regression_pct:.1f}% regression "
                        f"({current_perf:.0f} vs {baseline_perf:.0f} vec/s)"
                    )
        
        # Check search latency
        if "search_latency" in current:
            current_latency = current["search_latency"]["avg_ms"]
            baseline_latency = baseline.get("search_latency", 1.0)
            
            if current_latency > baseline_latency * 1.5:  # 50% latency regression
                regression_pct = ((current_latency - baseline_latency) / baseline_latency) * 100
                regressions.append(
                    f"search_latency: {regression_pct:.1f}% regression "
                    f"({current_latency:.2f}ms vs {baseline_latency:.2f}ms)"
                )
        
        return regressions
    
    def check_standardized_regression(self, current: Dict, baseline: Dict) -> List[str]:
        """Check for regressions in standardized benchmark metrics."""
        regressions = []
        
        if "standardized" not in current:
            return regressions
            
        current_std = current["standardized"]
        
        # Check Recall@10
        if "recall_at_10" in current_std:
            current_recall = current_std["recall_at_10"]
            baseline_recall = baseline.get("recall_at_10", 0.95)
            
            if current_recall < baseline_recall * 0.95:  # 5% recall degradation threshold
                degradation_pct = ((baseline_recall - current_recall) / baseline_recall) * 100
                regressions.append(
                    f"recall_at_10: {degradation_pct:.1f}% degradation "
                    f"({current_recall:.3f} vs {baseline_recall:.3f})"
                )
        
        # Check Recall@100  
        if "recall_at_100" in current_std:
            current_recall = current_std["recall_at_100"]
            baseline_recall = baseline.get("recall_at_100", 0.98)
            
            if current_recall < baseline_recall * 0.95:  # 5% recall degradation threshold
                degradation_pct = ((baseline_recall - current_recall) / baseline_recall) * 100
                regressions.append(
                    f"recall_at_100: {degradation_pct:.1f}% degradation "
                    f"({current_recall:.3f} vs {baseline_recall:.3f})"
                )
        
        return regressions
    
    def run_full_regression_test(self) -> Dict:
        """Run complete regression test suite."""
        print("🔍 Running Regression Test Suite")
        print("=" * 50)
        
        # Get git info
        try:
            git_hash = subprocess.check_output(["git", "rev-parse", "HEAD"]).decode().strip()
            git_branch = subprocess.check_output(["git", "rev-parse", "--abbrev-ref", "HEAD"]).decode().strip()
        except:
            git_hash = "unknown"
            git_branch = "unknown"
        
        # Run tests
        print("📊 Running performance benchmarks...")
        perf_results = self.run_performance_test()
        
        print("🔬 Running standardized dataset benchmarks...")
        standardized_results = self.run_standardized_benchmark()
        
        print("✅ Running correctness tests...")
        correctness_results = self.run_correctness_test()
        
        # Check for regressions
        regressions = self.check_regression(perf_results, self.baseline_performance)
        standardized_regressions = self.check_standardized_regression(
            {"standardized": standardized_results}, self.baseline_performance
        )
        all_regressions = regressions + standardized_regressions
        
        # Compile results
        results = {
            "timestamp": datetime.now().isoformat(),
            "git_hash": git_hash,
            "git_branch": git_branch,
            "performance": perf_results,
            "standardized": standardized_results,
            "correctness": correctness_results,
            "regressions": all_regressions,
            "status": "PASS" if not all_regressions and correctness_results.get("status") == "PASS" else "FAIL"
        }
        
        # Save results
        self.save_results(results)
        
        # Print summary
        self.print_summary(results)
        
        return results
    
    def save_results(self, results: Dict):
        """Save results to JSON file."""
        try:
            if os.path.exists(self.results_file):
                with open(self.results_file, 'r') as f:
                    all_results = json.load(f)
            else:
                all_results = []
            
            all_results.append(results)
            
            # Keep only last 100 results
            all_results = all_results[-100:]
            
            with open(self.results_file, 'w') as f:
                json.dump(all_results, f, indent=2)
                
        except Exception as e:
            print(f"⚠️  Could not save results: {e}")
    
    def print_summary(self, results: Dict):
        """Print test summary."""
        print("\n" + "=" * 50)
        print("📊 REGRESSION TEST SUMMARY")
        print("=" * 50)
        
        status_emoji = "✅" if results["status"] == "PASS" else "❌"
        print(f"{status_emoji} Overall Status: {results['status']}")
        print(f"🕒 Timestamp: {results['timestamp']}")
        print(f"📝 Git: {results['git_branch']} ({results['git_hash'][:8]})")
        
        print("\n📊 Performance Results:")
        perf = results["performance"]
        for key, data in perf.items():
            if key.startswith("batch_"):
                size = key.split("_")[1]
                print(f"  {size:>4} vectors: {data['throughput']:8,.0f} vec/s")
            elif key == "search_latency":
                print(f"  Search latency: {data['avg_ms']:.2f}ms avg")
        
        if "standardized" in results and results["standardized"].get("status") != "ERROR":
            std = results["standardized"]
            print("\n🔬 Standardized Benchmark (SIFT-128):")
            print(f"  Dataset: {std.get('dataset', 'N/A')}")
            print(f"  Recall@10: {std.get('recall_at_10', 0):.3f}")
            print(f"  Recall@100: {std.get('recall_at_100', 0):.3f}") 
            print(f"  Insert: {std.get('insert_throughput', 0):8,.0f} vec/s")
            print(f"  Search: {std.get('avg_search_latency_ms', 0):.2f}ms, {std.get('queries_per_second', 0):.0f} QPS")
        elif "standardized" in results:
            print(f"\n⚠️ Standardized benchmark error: {results['standardized'].get('error', 'Unknown')}")
        
        print(f"\n✅ Correctness: {results['correctness']['status']} "
              f"({results['correctness']['pass_rate']:.1f}% pass rate)")
        
        if results["regressions"]:
            print(f"\n⚠️  {len(results['regressions'])} Regressions Detected:")
            for regression in results["regressions"]:
                print(f"  - {regression}")
        else:
            print("\n🎉 No regressions detected!")


if __name__ == "__main__":
    monitor = RegressionMonitor()
    results = monitor.run_full_regression_test()
    
    # Exit with error code if tests failed
    sys.exit(0 if results["status"] == "PASS" else 1)