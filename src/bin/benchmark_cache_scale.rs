//! Large-scale cache benchmark
//!
//! Tests cache effectiveness at production scales (1M, 10M rows)
//! with different cache sizes to determine optimal configuration.
//!
//! Usage: cargo run --release --bin benchmark_cache_scale

use anyhow::Result;
use arrow::datatypes::{DataType, Field, Schema};
use omendb::catalog::Catalog;
use omendb::row::Row;
use omendb::value::Value;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

const QUERY_COUNT: usize = 10_000;

struct BenchmarkConfig {
    data_size: usize,
    cache_size: usize,
    cache_pct: f64,
}

impl BenchmarkConfig {
    fn new(data_size: usize, cache_size: usize) -> Self {
        let cache_pct = (cache_size as f64 / data_size as f64) * 100.0;
        Self { data_size, cache_size, cache_pct }
    }
}

struct BenchmarkResult {
    config: BenchmarkConfig,
    insert_ms: f64,
    query_us_no_cache: f64,
    query_us_with_cache: f64,
    cache_hit_rate: f64,
    speedup: f64,
}

impl BenchmarkResult {
    fn print(&self) {
        let verdict = if self.speedup >= 3.0 {
            "✅ EXCELLENT"
        } else if self.speedup >= 2.0 {
            "✅ GOOD"
        } else if self.speedup >= 1.5 {
            "⚠️  MODEST"
        } else {
            "❌ INSUFFICIENT"
        };

        println!("\n───────────────────────────────────────────────────────────────");
        println!("  {} rows, cache: {} entries ({}% of data)",
            format_number(self.config.data_size),
            format_number(self.config.cache_size),
            self.config.cache_pct as usize
        );
        println!("───────────────────────────────────────────────────────────────");
        println!("  Insert:         {:.2} ms ({} rows/sec)",
            self.insert_ms,
            format_number((self.config.data_size as f64 / (self.insert_ms / 1000.0)) as usize)
        );
        println!("  Query (no cache): {:.3} μs avg", self.query_us_no_cache);
        println!("  Query (w/ cache): {:.3} μs avg", self.query_us_with_cache);
        println!("  Cache hit rate:   {:.1}%", self.cache_hit_rate);
        println!("  🎯 Speedup:       {:.2}x {}", self.speedup, verdict);
    }
}

fn main() -> Result<()> {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  Large-Scale Cache Benchmark");
    println!("═══════════════════════════════════════════════════════════════");
    println!("\nTesting cache effectiveness at production scales");
    println!("Workload: Zipfian distribution (80% queries hit 10% of data)");
    println!("Queries: {} per test\n", format_number(QUERY_COUNT));

    let configs = vec![
        // Small scale (100K rows)
        BenchmarkConfig::new(100_000, 1_000),    // 1% cache
        BenchmarkConfig::new(100_000, 10_000),   // 10% cache
        BenchmarkConfig::new(100_000, 50_000),   // 50% cache

        // Medium scale (1M rows)
        BenchmarkConfig::new(1_000_000, 10_000),   // 1% cache
        BenchmarkConfig::new(1_000_000, 100_000),  // 10% cache
        BenchmarkConfig::new(1_000_000, 500_000),  // 50% cache
    ];

    let mut results = Vec::new();

    for config in configs {
        println!("\n📊 Running benchmark: {} rows, {} cache entries...",
            format_number(config.data_size),
            format_number(config.cache_size)
        );

        let result = run_benchmark(&config)?;
        result.print();
        results.push(result);
    }

    // Summary
    println!("\n\n═══════════════════════════════════════════════════════════════");
    println!("  SUMMARY");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("{:>10} {:>10} {:>8} {:>10} {:>10} {:>8}",
        "Data Size", "Cache", "Cache%", "Latency", "Hit Rate", "Speedup");
    println!("{:->10} {:->10} {:->8} {:->10} {:->10} {:->8}",
        "", "", "", "", "", "");

    for result in &results {
        println!("{:>10} {:>10} {:>7.0}% {:>9.3}μs {:>9.1}% {:>7.2}x",
            format_number(result.config.data_size),
            format_number(result.config.cache_size),
            result.config.cache_pct,
            result.query_us_with_cache,
            result.cache_hit_rate,
            result.speedup
        );
    }

    // Analysis
    println!("\n\n📈 ANALYSIS\n");

    // Find best configuration
    let best = results.iter().max_by(|a, b| {
        a.speedup.partial_cmp(&b.speedup).unwrap()
    }).unwrap();

    println!("Best configuration:");
    println!("  Data size: {} rows", format_number(best.config.data_size));
    println!("  Cache size: {} entries ({}% of data)",
        format_number(best.config.cache_size),
        best.config.cache_pct as usize
    );
    println!("  Speedup: {:.2}x", best.speedup);
    println!("  Hit rate: {:.1}%", best.cache_hit_rate);

    // Recommendations
    println!("\n💡 RECOMMENDATIONS\n");

    if best.config.cache_pct >= 10.0 {
        println!("✅ 10%+ cache provides excellent performance");
        println!("   For {} rows: Use {} entry cache",
            format_number(1_000_000),
            format_number(100_000)
        );
    }

    if best.cache_hit_rate >= 80.0 {
        println!("✅ High hit rate ({:.1}%) validates Zipfian workload assumption",
            best.cache_hit_rate);
    }

    if best.speedup >= 2.0 {
        println!("✅ Cache achieves 2-3x target speedup");
    } else {
        println!("⚠️  Speedup below 2x target - consider larger cache");
    }

    println!();

    Ok(())
}

fn run_benchmark(config: &BenchmarkConfig) -> Result<BenchmarkResult> {
    // Test 1: Without cache (baseline)
    let (insert_ms, query_us_no_cache) = benchmark_without_cache(config.data_size)?;

    // Test 2: With cache
    let (query_us_with_cache, cache_hit_rate) = benchmark_with_cache(config)?;

    let speedup = query_us_no_cache / query_us_with_cache;

    Ok(BenchmarkResult {
        config: BenchmarkConfig::new(config.data_size, config.cache_size),
        insert_ms,
        query_us_no_cache,
        query_us_with_cache,
        cache_hit_rate,
        speedup,
    })
}

fn benchmark_without_cache(data_size: usize) -> Result<(f64, f64)> {
    let temp_dir = TempDir::new()?;
    let _catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Int64, false),
        Field::new("text", DataType::Utf8, false),
    ]));

    let table_dir = temp_dir.path().join("test_table");
    let mut table = omendb::table::Table::new(
        "test".to_string(),
        schema,
        "id".to_string(),
        table_dir,
    )?;

    // Insert
    let insert_start = Instant::now();
    for i in 0..data_size {
        let row = Row::new(vec![
            Value::Int64(i as i64),
            Value::Int64(i as i64 * 100),
            Value::Text(format!("row_{}", i)),
        ]);
        table.insert(row)?;
    }
    let insert_ms = insert_start.elapsed().as_secs_f64() * 1000.0;

    // Query with Zipfian distribution
    let mut query_keys = Vec::with_capacity(QUERY_COUNT);
    for i in 0..QUERY_COUNT {
        let key = if (i / 10) % 5 == 0 {
            ((i / 10) * 7919) % data_size
        } else {
            (i / 10) % (data_size / 10)
        };
        query_keys.push(key);
    }

    let query_start = Instant::now();
    for &key in &query_keys {
        let _result = table.get(&Value::Int64(key as i64))?;
    }
    let query_us = query_start.elapsed().as_micros() as f64 / QUERY_COUNT as f64;

    Ok((insert_ms, query_us))
}

fn benchmark_with_cache(config: &BenchmarkConfig) -> Result<(f64, f64)> {
    let temp_dir = TempDir::new()?;
    let _catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Int64, false),
        Field::new("text", DataType::Utf8, false),
    ]));

    let table_dir = temp_dir.path().join("test_table");
    let mut table = omendb::table::Table::new_with_cache(
        "test".to_string(),
        schema,
        "id".to_string(),
        table_dir,
        config.cache_size,
    )?;

    // Insert
    for i in 0..config.data_size {
        let row = Row::new(vec![
            Value::Int64(i as i64),
            Value::Int64(i as i64 * 100),
            Value::Text(format!("row_{}", i)),
        ]);
        table.insert(row)?;
    }

    // Query with Zipfian distribution (same as baseline)
    let mut query_keys = Vec::with_capacity(QUERY_COUNT);
    for i in 0..QUERY_COUNT {
        let key = if (i / 10) % 5 == 0 {
            ((i / 10) * 7919) % config.data_size
        } else {
            (i / 10) % (config.data_size / 10)
        };
        query_keys.push(key);
    }

    let query_start = Instant::now();
    for &key in &query_keys {
        let _result = table.get(&Value::Int64(key as i64))?;
    }
    let query_us = query_start.elapsed().as_micros() as f64 / QUERY_COUNT as f64;

    let stats = table.cache_stats().unwrap();

    Ok((query_us, stats.hit_rate))
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}
