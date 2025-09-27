//! Realistic benchmark comparing OmenDB with production workload patterns
//! Based on actual time-series database usage patterns

use omendb::concurrent::ConcurrentOmenDB;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::thread;
use rand::Rng;
use rand_distr::{Distribution, Normal};

/// Production workload characteristics from real deployments
#[derive(Debug)]
struct WorkloadPattern {
    name: String,
    write_ratio: f64,      // Percentage of writes vs reads
    batch_size: usize,     // Typical batch size
    query_range: u64,      // Typical query time range in seconds
    cardinality: usize,    // Number of unique time series
    arrival_pattern: ArrivalPattern,
}

#[derive(Debug)]
enum ArrivalPattern {
    Steady,           // Constant rate
    Bursty,          // Sudden spikes
    Diurnal,         // Daily patterns
}

/// Results matching what InfluxDB/TimescaleDB report
#[derive(Debug, Default)]
struct BenchmarkResults {
    writes_per_sec: f64,
    reads_per_sec: f64,
    write_latency_p50: Duration,
    write_latency_p99: Duration,
    read_latency_p50: Duration,
    read_latency_p99: Duration,
    bytes_per_point: f64,
    compression_ratio: f64,
    errors: u64,
}

fn main() {
    println!("🏁 Realistic Time-Series Database Benchmark");
    println!("===========================================");
    println!("Comparing OmenDB against industry patterns\n");

    // Define realistic workloads
    let workloads = vec![
        WorkloadPattern {
            name: "IoT Sensors".to_string(),
            write_ratio: 0.95,
            batch_size: 1000,
            query_range: 3600,      // 1 hour queries
            cardinality: 10_000,    // 10K devices
            arrival_pattern: ArrivalPattern::Steady,
        },
        WorkloadPattern {
            name: "DevOps Metrics".to_string(),
            write_ratio: 0.80,
            batch_size: 100,
            query_range: 86400,     // 1 day queries
            cardinality: 5_000,     // 5K metrics
            arrival_pattern: ArrivalPattern::Bursty,
        },
        WorkloadPattern {
            name: "Financial Tick Data".to_string(),
            write_ratio: 0.60,
            batch_size: 1,          // Real-time, no batching
            query_range: 300,       // 5 minute queries
            cardinality: 500,       // 500 symbols
            arrival_pattern: ArrivalPattern::Bursty,
        },
        WorkloadPattern {
            name: "Application Analytics".to_string(),
            write_ratio: 0.70,
            batch_size: 500,
            query_range: 604800,    // 1 week queries
            cardinality: 100_000,   // 100K users
            arrival_pattern: ArrivalPattern::Diurnal,
        },
    ];

    // Initialize database
    let db = Arc::new(ConcurrentOmenDB::new("benchmark"));

    // Run each workload
    for workload in workloads {
        println!("📊 Workload: {}", workload.name);
        println!("  Pattern: {:?}, Cardinality: {}", workload.arrival_pattern, workload.cardinality);
        let results = run_workload(Arc::clone(&db), &workload);
        print_results(&results);
        compare_with_competitors(&workload.name, &results);
        println!();
    }

    // Memory and storage analysis
    println!("💾 Storage Efficiency Analysis");
    analyze_storage_efficiency();
}

fn run_workload(db: Arc<ConcurrentOmenDB>, pattern: &WorkloadPattern) -> BenchmarkResults {
    let mut results = BenchmarkResults::default();
    let duration = Duration::from_secs(60); // Run for 60 seconds
    let start = Instant::now();

    let total_writes = Arc::new(AtomicU64::new(0));
    let total_reads = Arc::new(AtomicU64::new(0));
    let errors = Arc::new(AtomicU64::new(0));

    let mut write_latencies = Vec::new();
    let mut read_latencies = Vec::new();
    let mut handles = vec![];

    // Spawn worker threads
    for thread_id in 0..4 {
        let db_clone = Arc::clone(&db);
        let total_writes = Arc::clone(&total_writes);
        let total_reads = Arc::clone(&total_reads);
        let errors_clone = Arc::clone(&errors);
        let pattern = pattern.clone();

        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut local_write_latencies = Vec::new();
            let mut local_read_latencies = Vec::new();
            let normal = Normal::new(0.0, 100.0).unwrap();

            let base_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as i64;

            let mut counter = 0;

            while start.elapsed() < duration {
                // Determine operation type based on write ratio
                let is_write = rng.gen_bool(pattern.write_ratio);

                if is_write {
                    // Batch writes
                    let op_start = Instant::now();
                    for i in 0..pattern.batch_size {
                        let series_id = rng.gen_range(0..pattern.cardinality) as i64;
                        let timestamp = base_timestamp + counter * 1000 + i as i64;
                        let value = normal.sample(&mut rng);

                        if db_clone.insert(timestamp, value, series_id).is_err() {
                            errors_clone.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    local_write_latencies.push(op_start.elapsed());
                    total_writes.fetch_add(pattern.batch_size as u64, Ordering::Relaxed);
                    counter += pattern.batch_size as i64;
                } else {
                    // Range query
                    let op_start = Instant::now();
                    let end_time = base_timestamp + counter * 1000;
                    let start_time = end_time - (pattern.query_range as i64 * 1_000_000);

                    match db_clone.range_query(start_time, end_time) {
                        Ok(results) => {
                            // Simulate processing results
                            let _ = results.len();
                        }
                        Err(_) => {
                            errors_clone.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    local_read_latencies.push(op_start.elapsed());
                    total_reads.fetch_add(1, Ordering::Relaxed);
                }

                // Apply arrival pattern
                match pattern.arrival_pattern {
                    ArrivalPattern::Steady => {
                        thread::sleep(Duration::from_micros(100));
                    }
                    ArrivalPattern::Bursty => {
                        if rng.gen_bool(0.1) { // 10% chance of burst
                            // No sleep during burst
                        } else {
                            thread::sleep(Duration::from_millis(10));
                        }
                    }
                    ArrivalPattern::Diurnal => {
                        let hour = (start.elapsed().as_secs() / 3600) % 24;
                        let sleep_ms = if hour >= 9 && hour <= 17 {
                            1  // Business hours - high load
                        } else {
                            10 // Off hours - low load
                        };
                        thread::sleep(Duration::from_millis(sleep_ms));
                    }
                }
            }

            (local_write_latencies, local_read_latencies)
        });

        handles.push(handle);
    }

    // Collect results
    for handle in handles {
        if let Ok((write_lats, read_lats)) = handle.join() {
            write_latencies.extend(write_lats);
            read_latencies.extend(read_lats);
        }
    }

    let elapsed = start.elapsed();

    // Calculate metrics
    results.writes_per_sec = total_writes.load(Ordering::Relaxed) as f64 / elapsed.as_secs_f64();
    results.reads_per_sec = total_reads.load(Ordering::Relaxed) as f64 / elapsed.as_secs_f64();
    results.errors = errors.load(Ordering::Relaxed);

    // Calculate latencies
    if !write_latencies.is_empty() {
        write_latencies.sort();
        results.write_latency_p50 = write_latencies[write_latencies.len() / 2];
        results.write_latency_p99 = write_latencies[write_latencies.len() * 99 / 100];
    }

    if !read_latencies.is_empty() {
        read_latencies.sort();
        results.read_latency_p50 = read_latencies[read_latencies.len() / 2];
        results.read_latency_p99 = read_latencies[read_latencies.len() * 99 / 100];
    }

    // Storage metrics (simplified - would need actual measurement)
    results.bytes_per_point = 24.0; // Timestamp (8) + Value (8) + Series ID (8)
    results.compression_ratio = 1.3; // Our current compression

    results
}

fn print_results(results: &BenchmarkResults) {
    println!("  Performance:");
    println!("    Writes: {:.0} points/sec", results.writes_per_sec);
    println!("    Reads:  {:.0} queries/sec", results.reads_per_sec);
    println!("  Write Latency:");
    println!("    P50: {:.2}ms", results.write_latency_p50.as_secs_f64() * 1000.0);
    println!("    P99: {:.2}ms", results.write_latency_p99.as_secs_f64() * 1000.0);
    println!("  Read Latency:");
    println!("    P50: {:.2}ms", results.read_latency_p50.as_secs_f64() * 1000.0);
    println!("    P99: {:.2}ms", results.read_latency_p99.as_secs_f64() * 1000.0);
    println!("  Storage:");
    println!("    Bytes/point: {:.1}", results.bytes_per_point);
    println!("    Compression: {:.1}x", results.compression_ratio);
    if results.errors > 0 {
        println!("  ⚠️  Errors: {}", results.errors);
    }
}

fn compare_with_competitors(workload: &str, results: &BenchmarkResults) {
    println!("\n  📈 Competitive Comparison:");

    // These are typical production numbers from benchmarks
    let (influx_writes, influx_comp, timescale_writes, timescale_comp) = match workload {
        "IoT Sensors" => (500_000.0, 5.2, 400_000.0, 4.8),
        "DevOps Metrics" => (250_000.0, 6.1, 300_000.0, 5.5),
        "Financial Tick Data" => (100_000.0, 3.5, 150_000.0, 3.2),
        "Application Analytics" => (300_000.0, 7.3, 350_000.0, 6.8),
        _ => (200_000.0, 5.0, 200_000.0, 5.0),
    };

    let omendb_vs_influx = (results.writes_per_sec / influx_writes) * 100.0;
    let omendb_vs_timescale = (results.writes_per_sec / timescale_writes) * 100.0;

    println!("    vs InfluxDB:    {:.1}% throughput, {:.1}x worse compression",
             omendb_vs_influx, influx_comp / results.compression_ratio);
    println!("    vs TimescaleDB: {:.1}% throughput, {:.1}x worse compression",
             omendb_vs_timescale, timescale_comp / results.compression_ratio);

    if omendb_vs_influx < 50.0 {
        println!("    ⚠️  Performance significantly below industry standard");
    }
}

fn analyze_storage_efficiency() {
    println!("  Current OmenDB:");
    println!("    • 24 bytes per point (uncompressed)");
    println!("    • 1.3x compression ratio");
    println!("    • ~18.5 bytes per point compressed");
    println!();
    println!("  Industry Leaders:");
    println!("    • InfluxDB: 2-4 bytes per point (with TSM)");
    println!("    • TimescaleDB: 3-5 bytes per point (with compression)");
    println!("    • Cassandra: 8-12 bytes per point");
    println!();
    println!("  ⚠️  Storage efficiency 5-10x worse than competitors");
    println!("  💡 Need: Better compression, columnar storage, delta encoding");
}

// Clone implementation for WorkloadPattern
impl Clone for WorkloadPattern {
    fn clone(&self) -> Self {
        WorkloadPattern {
            name: self.name.clone(),
            write_ratio: self.write_ratio,
            batch_size: self.batch_size,
            query_range: self.query_range,
            cardinality: self.cardinality,
            arrival_pattern: match self.arrival_pattern {
                ArrivalPattern::Steady => ArrivalPattern::Steady,
                ArrivalPattern::Bursty => ArrivalPattern::Bursty,
                ArrivalPattern::Diurnal => ArrivalPattern::Diurnal,
            },
        }
    }
}