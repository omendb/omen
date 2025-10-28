//! Real benchmark: Learned Index vs B-tree
//! Tests with realistic time-series workloads to validate core value proposition

use omendb::index::RecursiveModelIndex;
use rand::distributions::Distribution;
use rand::Rng;
use std::collections::BTreeMap;
use std::time::Instant;

#[derive(Debug)]
struct BenchmarkResult {
    workload_name: String,
    data_size: usize,

    // Build times
    btree_build_ms: f64,
    learned_build_ms: f64,

    // Search times (avg over 10K queries)
    btree_search_us: f64,
    learned_search_us: f64,

    // Memory usage (approximate)
    btree_memory_mb: f64,
    learned_memory_mb: f64,

    // Speedup
    speedup: f64,
}

fn main() {
    println!("🔬 Learned Index vs B-tree Benchmark");
    println!("====================================\n");

    let mut results = Vec::new();

    // Test 1: Sequential time-series (IoT sensors)
    println!("📊 Test 1: Sequential Time-Series (IoT Sensors)");
    results.push(benchmark_sequential(1_000_000));

    // Test 2: Bursty writes (Training metrics)
    println!("\n📊 Test 2: Bursty Writes (AI Training Metrics)");
    results.push(benchmark_bursty(1_000_000));

    // Test 3: Multiple series interleaved (Multi-tenant)
    println!("\n📊 Test 3: Interleaved Series (Multi-tenant)");
    results.push(benchmark_interleaved(1_000_000));

    // Test 4: Zipfian distribution (Skewed access)
    println!("\n📊 Test 4: Zipfian Distribution (Skewed Access)");
    results.push(benchmark_zipfian(1_000_000));

    // Test 5: Uniform random (Worst case for learned)
    println!("\n📊 Test 5: Uniform Random (Worst Case)");
    results.push(benchmark_random(1_000_000));

    // Summary
    print_summary(&results);
}

fn benchmark_sequential(size: usize) -> BenchmarkResult {
    // Generate sequential timestamps (IoT sensor data pattern)
    let data: Vec<i64> = (0..size as i64).collect();
    run_benchmark("Sequential (IoT)", data)
}

fn benchmark_bursty(size: usize) -> BenchmarkResult {
    // Generate bursty data (training metrics pattern)
    let mut data = Vec::with_capacity(size);
    let mut rng = rand::thread_rng();
    let mut timestamp = 0i64;

    while data.len() < size {
        // Burst of sequential writes
        let burst_size = rng.gen_range(100..1000);
        for _ in 0..burst_size.min(size - data.len()) {
            data.push(timestamp);
            timestamp += 1;
        }
        // Gap (idle period)
        timestamp += rng.gen_range(10..100);
    }

    run_benchmark("Bursty (Training)", data)
}

fn benchmark_interleaved(size: usize) -> BenchmarkResult {
    // Generate interleaved series (multi-tenant pattern)
    let mut data = Vec::with_capacity(size);
    let num_series = 10;
    let mut timestamps: Vec<i64> = vec![0; num_series];
    let mut rng = rand::thread_rng();

    for _ in 0..size {
        let series = rng.gen_range(0..num_series);
        data.push(timestamps[series]);
        timestamps[series] += rng.gen_range(1..10);
    }

    data.sort_unstable();
    run_benchmark("Interleaved (Multi-tenant)", data)
}

fn benchmark_zipfian(size: usize) -> BenchmarkResult {
    // Generate Zipfian distribution (skewed access pattern)
    let mut data = Vec::with_capacity(size);
    let mut rng = rand::thread_rng();

    // Simple Zipfian: 80% of queries hit 20% of data
    let hot_range = (size as f64 * 0.2) as i64;

    for _ in 0..size {
        let val = if rng.gen::<f64>() < 0.8 {
            // Hot data (80% probability)
            rng.gen_range(0..hot_range)
        } else {
            // Cold data (20% probability)
            rng.gen_range(hot_range..size as i64)
        };
        data.push(val);
    }

    data.sort_unstable();
    run_benchmark("Zipfian (Skewed)", data)
}

fn benchmark_random(size: usize) -> BenchmarkResult {
    // Generate uniform random (worst case for learned indexes)
    let mut data: Vec<i64> = (0..size)
        .map(|_| rand::thread_rng().gen_range(0..size as i64 * 10))
        .collect();

    data.sort_unstable();
    data.dedup();

    run_benchmark("Random (Worst case)", data)
}

fn run_benchmark(name: &str, data: Vec<i64>) -> BenchmarkResult {
    let size = data.len();
    println!("  Data points: {}", size);

    // Build B-tree
    let start = Instant::now();
    let mut btree = BTreeMap::new();
    for (i, &key) in data.iter().enumerate() {
        btree.insert(key, i);
    }
    let btree_build_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("  B-tree build: {:.2}ms", btree_build_ms);

    // Build learned index
    let start = Instant::now();
    let mut learned = RecursiveModelIndex::new(size);
    for &key in &data {
        learned.add_key(key);
    }
    let learned_build_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("  Learned build: {:.2}ms", learned_build_ms);

    // Benchmark searches
    let num_queries = 10_000;
    let query_keys: Vec<i64> = (0..num_queries)
        .map(|_| data[rand::thread_rng().gen_range(0..data.len())])
        .collect();

    // B-tree search
    let start = Instant::now();
    for &key in &query_keys {
        let _ = btree.get(&key);
    }
    let btree_search_us = start.elapsed().as_secs_f64() * 1_000_000.0 / num_queries as f64;
    println!("  B-tree search: {:.3}μs/query", btree_search_us);

    // Learned index search
    let start = Instant::now();
    for &key in &query_keys {
        let _ = learned.search(key);
    }
    let learned_search_us = start.elapsed().as_secs_f64() * 1_000_000.0 / num_queries as f64;
    println!("  Learned search: {:.3}μs/query", learned_search_us);

    // Memory estimates (rough)
    let btree_memory_mb = (size * 24) as f64 / 1_048_576.0; // Node overhead
    let learned_memory_mb = (size * 8) as f64 / 1_048_576.0; // Model weights

    let speedup = btree_search_us / learned_search_us;

    println!("  Speedup: {:.2}x", speedup);

    if speedup > 1.0 {
        println!("  ✅ Learned index FASTER");
    } else {
        println!("  ⚠️  B-tree faster (learned not optimal here)");
    }

    BenchmarkResult {
        workload_name: name.to_string(),
        data_size: size,
        btree_build_ms,
        learned_build_ms,
        btree_search_us,
        learned_search_us,
        btree_memory_mb,
        learned_memory_mb,
        speedup,
    }
}

fn print_summary(results: &[BenchmarkResult]) {
    println!("\n\n📋 SUMMARY");
    println!("==========\n");

    println!(
        "{:<30} {:>10} {:>15} {:>15} {:>10}",
        "Workload", "Size", "B-tree (μs)", "Learned (μs)", "Speedup"
    );
    println!("{}", "-".repeat(82));

    for result in results {
        println!(
            "{:<30} {:>10} {:>15.3} {:>15.3} {:>9.2}x",
            result.workload_name,
            format!("{}", result.data_size),
            result.btree_search_us,
            result.learned_search_us,
            result.speedup
        );
    }

    println!("\n🎯 KEY FINDINGS:\n");

    let avg_speedup: f64 = results.iter().map(|r| r.speedup).sum::<f64>() / results.len() as f64;
    let best = results
        .iter()
        .max_by(|a, b| a.speedup.partial_cmp(&b.speedup).unwrap())
        .unwrap();
    let worst = results
        .iter()
        .min_by(|a, b| a.speedup.partial_cmp(&b.speedup).unwrap())
        .unwrap();

    println!("• Average speedup: {:.2}x", avg_speedup);
    println!(
        "• Best workload: {} ({:.2}x faster)",
        best.workload_name, best.speedup
    );
    println!(
        "• Worst workload: {} ({:.2}x)",
        worst.workload_name, worst.speedup
    );

    let winning_workloads: Vec<_> = results.iter().filter(|r| r.speedup > 1.5).collect();

    if winning_workloads.is_empty() {
        println!("\n⚠️  WARNING: Learned indexes not showing clear advantage");
        println!("   Consider: Model complexity, training data, or hybrid approach");
    } else {
        println!("\n✅ Learned indexes WIN on:");
        for result in winning_workloads {
            println!(
                "   • {} ({:.2}x faster)",
                result.workload_name, result.speedup
            );
        }
    }

    println!("\n💡 RECOMMENDATION:\n");
    if avg_speedup > 2.0 {
        println!("Strong performance advantage! Focus on these workloads for YC demo.");
    } else if avg_speedup > 1.3 {
        println!("Moderate advantage. Highlight specific winning workloads (not general claims).");
    } else {
        println!("Advantage not proven. Consider:");
        println!("  1. Hybrid approach (learned + B-tree fallback)");
        println!("  2. Focus on memory efficiency instead of speed");
        println!("  3. Target specific distributions where learned excels");
    }
}
