//! Performance profiling tool for OmenDB
//! Identifies CPU hotspots, memory allocations, and lock contention

use omendb::concurrent::ConcurrentOmenDB;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;
use rand::Rng;

#[derive(Debug, Clone)]
struct ProfilingMetrics {
    operation: String,
    duration_ns: u128,
    memory_before: usize,
    memory_after: usize,
    thread_id: usize,
}

fn main() {
    println!("🔬 OmenDB Performance Profiling");
    println!("================================");
    println!("Enable profiling with: cargo build --release && valgrind --tool=callgrind target/release/profile");
    println!();

    // Initialize database
    let db = Arc::new(ConcurrentOmenDB::new("profile_test"));
    let mut metrics = Vec::new();

    // Test scenarios
    println!("Running profiling scenarios...\n");

    // Scenario 1: Sequential Inserts
    println!("📊 Scenario 1: Sequential Insert Performance");
    let start = Instant::now();
    for i in 0..100_000 {
        let op_start = Instant::now();
        let mem_before = get_memory_usage();

        db.insert(1000000 + i, i as f64 * 0.5, 1).unwrap();

        let mem_after = get_memory_usage();

        if i % 10000 == 0 {
            metrics.push(ProfilingMetrics {
                operation: "sequential_insert".to_string(),
                duration_ns: op_start.elapsed().as_nanos(),
                memory_before: mem_before,
                memory_after: mem_after,
                thread_id: 0,
            });
            println!("  Inserted {} records, memory: {} MB", i, mem_after / 1024 / 1024);
        }
    }
    let seq_duration = start.elapsed();
    println!("  ✓ Sequential: 100K inserts in {:.2}s ({:.0} ops/sec)\n",
             seq_duration.as_secs_f64(),
             100_000.0 / seq_duration.as_secs_f64());

    // Scenario 2: Random Reads
    println!("📊 Scenario 2: Random Read Performance");
    let mut rng = rand::thread_rng();
    let start = Instant::now();
    let mut hits = 0;
    let mut misses = 0;

    for _ in 0..50_000 {
        let key = 1000000 + rng.gen_range(0..100_000);
        let op_start = Instant::now();

        match db.search(key) {
            Ok(Some(_)) => hits += 1,
            _ => misses += 1,
        }

        if hits % 5000 == 0 {
            metrics.push(ProfilingMetrics {
                operation: "random_read".to_string(),
                duration_ns: op_start.elapsed().as_nanos(),
                memory_before: get_memory_usage(),
                memory_after: get_memory_usage(),
                thread_id: 0,
            });
        }
    }
    let read_duration = start.elapsed();
    println!("  ✓ Random: 50K reads in {:.2}s ({:.0} ops/sec)",
             read_duration.as_secs_f64(),
             50_000.0 / read_duration.as_secs_f64());
    println!("  Cache hit rate: {:.1}%\n", hits as f64 / (hits + misses) as f64 * 100.0);

    // Scenario 3: Concurrent Access
    println!("📊 Scenario 3: Concurrent Access (Lock Contention)");
    let start = Instant::now();
    let mut handles = vec![];

    for thread_id in 0..8 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            let mut thread_metrics = Vec::new();
            for i in 0..10_000 {
                let op_start = Instant::now();

                if i % 2 == 0 {
                    db_clone.insert(2000000 + thread_id * 10000 + i, i as f64, thread_id as i64).ok();
                } else {
                    db_clone.search(2000000 + thread_id * 10000 + i - 1).ok();
                }

                if i % 1000 == 0 {
                    thread_metrics.push(ProfilingMetrics {
                        operation: "concurrent_mixed".to_string(),
                        duration_ns: op_start.elapsed().as_nanos(),
                        memory_before: 0,
                        memory_after: 0,
                        thread_id,
                    });
                }
            }
            thread_metrics
        });
        handles.push(handle);
    }

    for handle in handles {
        if let Ok(thread_metrics) = handle.join() {
            metrics.extend(thread_metrics);
        }
    }
    let concurrent_duration = start.elapsed();
    println!("  ✓ Concurrent: 80K ops in {:.2}s ({:.0} ops/sec)\n",
             concurrent_duration.as_secs_f64(),
             80_000.0 / concurrent_duration.as_secs_f64());

    // Scenario 4: Memory Pressure
    println!("📊 Scenario 4: Memory Pressure Test");
    let start = Instant::now();
    let initial_memory = get_memory_usage();

    // Insert large batch
    for i in 0..500_000 {
        db.insert(3000000 + i, i as f64 * 1.5, 2).ok();

        if i % 100_000 == 0 && i > 0 {
            let current_memory = get_memory_usage();
            let memory_growth = current_memory - initial_memory;
            println!("  Records: {}, Memory growth: {} MB",
                    i, memory_growth / 1024 / 1024);
        }
    }

    let final_memory = get_memory_usage();
    let memory_per_record = (final_memory - initial_memory) / 500_000;
    println!("  ✓ Memory per record: {} bytes\n", memory_per_record);

    // Scenario 5: Index Rebuild Performance
    println!("📊 Scenario 5: Index Rebuild Performance");
    let start = Instant::now();

    // Force index rebuild by inserting non-sequential data
    let mut rng = rand::thread_rng();
    for _ in 0..50_000 {
        let key = 4000000 + rng.gen_range(0..1_000_000);
        db.insert(key, rng.gen_range(0.0..1000.0), 3).ok();
    }

    let rebuild_duration = start.elapsed();
    println!("  ✓ Index operations on 50K random inserts: {:.2}s\n",
             rebuild_duration.as_secs_f64());

    // Analyze metrics
    analyze_metrics(&metrics);

    // Generate flame graph data (if running under perf)
    println!("📈 Profiling Tips:");
    println!("  1. CPU Profile: perf record -g ./target/release/profile && perf report");
    println!("  2. Memory Profile: valgrind --tool=massif ./target/release/profile");
    println!("  3. Cache Profile: valgrind --tool=callgrind ./target/release/profile");
    println!("  4. Flame Graph: cargo flamegraph --bin profile");
}

fn get_memory_usage() -> usize {
    // Read from /proc/self/status on Linux
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<usize>() {
                            return kb * 1024; // Convert KB to bytes
                        }
                    }
                }
            }
        }
    }

    // Fallback for non-Linux
    0
}

fn analyze_metrics(metrics: &[ProfilingMetrics]) {
    println!("📊 Performance Analysis");
    println!("=======================");

    // Group by operation type
    let mut op_groups: std::collections::HashMap<String, Vec<u128>> = std::collections::HashMap::new();

    for metric in metrics {
        op_groups.entry(metric.operation.clone())
            .or_insert_with(Vec::new)
            .push(metric.duration_ns);
    }

    for (op, durations) in op_groups.iter() {
        if durations.is_empty() {
            continue;
        }

        let mut sorted = durations.clone();
        sorted.sort_unstable();

        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let p50 = sorted[sorted.len() / 2];
        let p95 = sorted[sorted.len() * 95 / 100];
        let p99 = sorted[sorted.len() * 99 / 100];

        println!("\n{} latencies (ns):", op);
        println!("  Min: {:>10}", min);
        println!("  P50: {:>10}", p50);
        println!("  P95: {:>10}", p95);
        println!("  P99: {:>10}", p99);
        println!("  Max: {:>10}", max);

        // Detect issues
        if p99 > p50 * 10 {
            println!("  ⚠️  High tail latency detected (P99 > 10x P50)");
        }
        if max > p99 * 100 {
            println!("  ⚠️  Extreme outliers detected (Max > 100x P99)");
        }
    }

    println!("\n🔍 Profiling Recommendations:");
    println!("  • High tail latency suggests lock contention or GC pauses");
    println!("  • Memory growth indicates potential leaks or inefficient allocation");
    println!("  • Use flamegraph to identify CPU hotspots");
    println!("  • Consider using jemalloc for better memory profiling");
}