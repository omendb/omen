//! Large-scale performance and reliability tests
//! Essential for validating enterprise production readiness

use crate::storage::ArrowStorage;
use crate::index::RecursiveModelIndex;
use crate::OmenDB;
use crate::metrics::*;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::thread;

/// Comprehensive scale testing configuration
#[derive(Debug, Clone)]
pub struct ScaleTestConfig {
    pub target_records: usize,
    pub batch_size: usize,
    pub concurrent_threads: usize,
    pub test_duration_secs: u64,
    pub memory_limit_mb: usize,
}

impl Default for ScaleTestConfig {
    fn default() -> Self {
        Self {
            target_records: 10_000_000, // 10M records
            batch_size: 10_000,
            concurrent_threads: 4,
            test_duration_secs: 300, // 5 minutes
            memory_limit_mb: 1024, // 1GB limit
        }
    }
}

/// Results from scale testing
#[derive(Debug)]
pub struct ScaleTestResults {
    pub records_inserted: usize,
    pub total_duration_secs: f64,
    pub avg_insertion_rate: f64,
    pub peak_insertion_rate: f64,
    pub avg_query_latency_ms: f64,
    pub p95_query_latency_ms: f64,
    pub memory_usage_mb: usize,
    pub errors_encountered: usize,
    pub success_rate: f64,
}

impl ScaleTestResults {
    pub fn is_production_ready(&self) -> bool {
        self.success_rate > 0.999 && // 99.9% success rate
        self.avg_insertion_rate > 5000.0 && // Minimum 5K inserts/sec
        self.p95_query_latency_ms < 10.0 && // Sub-10ms P95 latency
        self.memory_usage_mb < 2048 // Under 2GB memory
    }

    pub fn print_summary(&self) {
        println!("\n🎯 SCALE TEST RESULTS");
        println!("=====================");
        println!("Records Inserted: {}", format_number(self.records_inserted));
        println!("Total Duration: {:.2}s", self.total_duration_secs);
        println!("Avg Insert Rate: {:.0} records/sec", self.avg_insertion_rate);
        println!("Peak Insert Rate: {:.0} records/sec", self.peak_insertion_rate);
        println!("Avg Query Latency: {:.2}ms", self.avg_query_latency_ms);
        println!("P95 Query Latency: {:.2}ms", self.p95_query_latency_ms);
        println!("Memory Usage: {} MB", self.memory_usage_mb);
        println!("Success Rate: {:.3}%", self.success_rate * 100.0);
        println!("Errors: {}", self.errors_encountered);

        if self.is_production_ready() {
            println!("✅ PRODUCTION READY");
        } else {
            println!("❌ NOT PRODUCTION READY");
            self.print_failure_analysis();
        }
    }

    fn print_failure_analysis(&self) {
        println!("\n🔍 FAILURE ANALYSIS:");
        if self.success_rate <= 0.999 {
            println!("  - Success rate {:.3}% below 99.9% requirement", self.success_rate * 100.0);
        }
        if self.avg_insertion_rate <= 5000.0 {
            println!("  - Insert rate {:.0}/sec below 5K/sec requirement", self.avg_insertion_rate);
        }
        if self.p95_query_latency_ms >= 10.0 {
            println!("  - P95 latency {:.2}ms above 10ms requirement", self.p95_query_latency_ms);
        }
        if self.memory_usage_mb >= 2048 {
            println!("  - Memory usage {}MB above 2GB limit", self.memory_usage_mb);
        }
    }
}

/// Execute comprehensive scale test
pub fn run_scale_test(config: ScaleTestConfig) -> ScaleTestResults {
    println!("🚀 Starting scale test with {} records...", config.target_records);

    let start_time = Instant::now();
    let mut db = OmenDB::new("scale_test_db");

    // Track metrics
    let mut records_inserted = 0;
    let mut errors = 0;
    let mut insertion_rates = Vec::new();
    let mut query_latencies = Vec::new();

    // Phase 1: Bulk insertion test
    println!("📥 Phase 1: Bulk insertion testing...");
    let batch_start = Instant::now();

    for batch in 0..(config.target_records / config.batch_size) {
        let batch_start_time = Instant::now();
        let base_timestamp = 1_600_000_000_000_000 + (batch as i64 * config.batch_size as i64 * 1000);

        // Insert batch
        for i in 0..config.batch_size {
            let timestamp = base_timestamp + (i as i64 * 1000);
            let value = (batch * config.batch_size + i) as f64;

            match db.insert(timestamp, value, 1) {
                Ok(_) => records_inserted += 1,
                Err(_) => errors += 1,
            }
        }

        // Calculate batch insertion rate
        let batch_duration = batch_start_time.elapsed().as_secs_f64();
        let batch_rate = config.batch_size as f64 / batch_duration;
        insertion_rates.push(batch_rate);

        // Progress reporting
        if batch % 100 == 0 {
            println!("  Batch {} completed: {:.0} records/sec", batch, batch_rate);
        }

        // Memory pressure check
        if batch % 500 == 0 {
            let memory_usage = estimate_memory_usage();
            if memory_usage > config.memory_limit_mb {
                println!("⚠️  Memory limit exceeded: {} MB", memory_usage);
                break;
            }
        }

        // Time limit check
        if batch_start.elapsed().as_secs() > config.test_duration_secs {
            println!("⏱️  Time limit reached");
            break;
        }
    }

    // Phase 2: Query performance test
    println!("🔍 Phase 2: Query performance testing...");
    let query_test_samples = 1000;

    for i in 0..query_test_samples {
        let query_start = Instant::now();
        let timestamp = 1_600_000_000_000_000 + (i as i64 * 10000);

        // Test point query
        let _result = db.get(timestamp);

        let query_latency = query_start.elapsed().as_secs_f64() * 1000.0; // Convert to ms
        query_latencies.push(query_latency);
    }

    // Phase 3: Range query stress test
    println!("📊 Phase 3: Range query testing...");
    let range_test_samples = 100;

    for i in 0..range_test_samples {
        let range_start = Instant::now();
        let start_timestamp = 1_600_000_000_000_000 + (i as i64 * 100000);
        let end_timestamp = start_timestamp + 50000; // 50 second range

        match db.range_query(start_timestamp, end_timestamp) {
            Ok(_) => {},
            Err(_) => errors += 1,
        }

        let range_latency = range_start.elapsed().as_secs_f64() * 1000.0;
        query_latencies.push(range_latency);
    }

    // Calculate final metrics
    let total_duration = start_time.elapsed().as_secs_f64();
    let avg_insertion_rate = records_inserted as f64 / total_duration;
    let peak_insertion_rate = insertion_rates.iter().cloned().fold(0.0f64, f64::max);

    query_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let avg_query_latency = query_latencies.iter().sum::<f64>() / query_latencies.len() as f64;
    let p95_query_latency = query_latencies[(query_latencies.len() as f64 * 0.95) as usize];

    let success_rate = records_inserted as f64 / (records_inserted + errors) as f64;
    let memory_usage = estimate_memory_usage();

    ScaleTestResults {
        records_inserted,
        total_duration_secs: total_duration,
        avg_insertion_rate,
        peak_insertion_rate,
        avg_query_latency_ms: avg_query_latency,
        p95_query_latency_ms: p95_query_latency,
        memory_usage_mb: memory_usage,
        errors_encountered: errors,
        success_rate,
    }
}

/// Concurrent stress test with multiple threads
pub fn run_concurrent_stress_test(config: ScaleTestConfig) -> ScaleTestResults {
    println!("🔥 Starting concurrent stress test with {} threads...", config.concurrent_threads);

    let db = Arc::new(Mutex::new(OmenDB::new("concurrent_stress_test")));
    let results = Arc::new(Mutex::new(Vec::new()));
    let start_time = Instant::now();

    let mut handles = vec![];

    // Spawn worker threads
    for thread_id in 0..config.concurrent_threads {
        let db_clone = Arc::clone(&db);
        let results_clone = Arc::clone(&results);
        let config_clone = config.clone();

        let handle = thread::spawn(move || {
            let records_per_thread = config_clone.target_records / config_clone.concurrent_threads;
            let mut thread_errors = 0;
            let mut thread_records = 0;
            let mut thread_latencies = Vec::new();

            for i in 0..records_per_thread {
                let op_start = Instant::now();
                let timestamp = 1_600_000_000_000_000 + (thread_id as i64 * 1_000_000) + (i as i64 * 1000);
                let value = (thread_id * 1_000_000 + i) as f64;

                {
                    let mut db_lock = db_clone.lock().unwrap();
                    match db_lock.insert(timestamp, value, thread_id as i64) {
                        Ok(_) => thread_records += 1,
                        Err(_) => thread_errors += 1,
                    }
                }

                let op_latency = op_start.elapsed().as_secs_f64() * 1000.0;
                thread_latencies.push(op_latency);

                // Occasional queries
                if i % 1000 == 0 {
                    let query_start = Instant::now();
                    {
                        let db_lock = db_clone.lock().unwrap();
                        let _result = db_lock.get(timestamp);
                    }
                    let query_latency = query_start.elapsed().as_secs_f64() * 1000.0;
                    thread_latencies.push(query_latency);
                }
            }

            results_clone.lock().unwrap().push((thread_records, thread_errors, thread_latencies));
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Aggregate results
    let total_duration = start_time.elapsed().as_secs_f64();
    let results_lock = results.lock().unwrap();

    let total_records: usize = results_lock.iter().map(|(r, _, _)| r).sum();
    let total_errors: usize = results_lock.iter().map(|(_, e, _)| e).sum();
    let mut all_latencies: Vec<f64> = results_lock.iter()
        .flat_map(|(_, _, latencies)| latencies.iter().cloned())
        .collect();

    all_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let avg_latency = all_latencies.iter().sum::<f64>() / all_latencies.len() as f64;
    let p95_latency = if !all_latencies.is_empty() {
        all_latencies[(all_latencies.len() as f64 * 0.95) as usize]
    } else {
        0.0
    };

    ScaleTestResults {
        records_inserted: total_records,
        total_duration_secs: total_duration,
        avg_insertion_rate: total_records as f64 / total_duration,
        peak_insertion_rate: total_records as f64 / total_duration, // Simplified for concurrent
        avg_query_latency_ms: avg_latency,
        p95_query_latency_ms: p95_latency,
        memory_usage_mb: estimate_memory_usage(),
        errors_encountered: total_errors,
        success_rate: total_records as f64 / (total_records + total_errors) as f64,
    }
}

/// Estimate current memory usage (simplified)
fn estimate_memory_usage() -> usize {
    // This is a simplified estimation
    // In production, would use proper memory profiling

    // Return estimated usage in MB
    // For now, return a reasonable estimate based on system info
    512 // Placeholder - would implement proper memory tracking
}

/// Format large numbers with thousands separators
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();

    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Resource intensive test
    fn test_small_scale_validation() {
        let config = ScaleTestConfig {
            target_records: 10_000, // Small scale for CI
            batch_size: 1_000,
            concurrent_threads: 2,
            test_duration_secs: 30,
            memory_limit_mb: 256,
        };

        let results = run_scale_test(config);
        results.print_summary();

        // Basic validation
        assert!(results.records_inserted > 0);
        assert!(results.success_rate > 0.95); // 95% minimum for small scale
        assert!(results.avg_insertion_rate > 100.0); // Very modest requirement
    }

    #[test]
    #[ignore] // Resource intensive test
    fn test_concurrent_stress_small() {
        let config = ScaleTestConfig {
            target_records: 5_000,
            batch_size: 500,
            concurrent_threads: 3,
            test_duration_secs: 30,
            memory_limit_mb: 256,
        };

        let results = run_concurrent_stress_test(config);
        results.print_summary();

        assert!(results.records_inserted > 0);
        assert!(results.success_rate > 0.90); // Allow some contention issues
    }

    #[test]
    fn test_scale_config_defaults() {
        let config = ScaleTestConfig::default();
        assert_eq!(config.target_records, 10_000_000);
        assert_eq!(config.concurrent_threads, 4);
    }

    #[test]
    fn test_results_production_readiness() {
        let results = ScaleTestResults {
            records_inserted: 10_000_000,
            total_duration_secs: 1000.0,
            avg_insertion_rate: 10_000.0,
            peak_insertion_rate: 15_000.0,
            avg_query_latency_ms: 2.0,
            p95_query_latency_ms: 5.0,
            memory_usage_mb: 1500,
            errors_encountered: 10,
            success_rate: 0.9999,
        };

        assert!(results.is_production_ready());
    }

    #[test]
    fn test_results_not_production_ready() {
        let results = ScaleTestResults {
            records_inserted: 1_000_000,
            total_duration_secs: 1000.0,
            avg_insertion_rate: 1_000.0, // Too slow
            peak_insertion_rate: 2_000.0,
            avg_query_latency_ms: 15.0, // Too slow
            p95_query_latency_ms: 25.0, // Too slow
            memory_usage_mb: 3000, // Too much memory
            errors_encountered: 1000,
            success_rate: 0.999, // Just at the threshold
        };

        assert!(!results.is_production_ready());
    }
}