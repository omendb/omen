// Benchmark cache effectiveness
//!
//! Tests Table performance with and without cache to validate:
//! - Cache reduces query latency (target: 80x improvement on cache hits)
//! - Cache hit rate for typical workloads (target: 80-90%)
//! - Overall speedup with cache enabled (target: 2-3x at 10M scale)

use anyhow::Result;
use arrow::datatypes::{DataType, Field, Schema};
use omendb::catalog::Catalog;
use omendb::row::Row;
use omendb::value::Value;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

const DATA_SIZE: usize = 100_000;
const QUERY_COUNT: usize = 10_000;
const CACHE_SIZE: usize = 10_000; // 10% of data

fn main() -> Result<()> {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  Cache Effectiveness Benchmark");
    println!("═══════════════════════════════════════════════════════════════\n");
    println!("Dataset: {} rows", format_number(DATA_SIZE));
    println!("Queries: {} lookups", format_number(QUERY_COUNT));
    println!("Cache size: {} entries ({}% of data)\n",
        format_number(CACHE_SIZE),
        (CACHE_SIZE as f64 / DATA_SIZE as f64 * 100.0) as usize
    );

    // Test 1: No cache (baseline)
    println!("📊 Test 1: Table WITHOUT cache (baseline)");
    println!("───────────────────────────────────────────────────────────────");
    let no_cache_us = benchmark_without_cache()?;
    println!();

    // Test 2: With cache
    println!("📊 Test 2: Table WITH cache ({} entries)", format_number(CACHE_SIZE));
    println!("───────────────────────────────────────────────────────────────");
    let (cache_us, hit_rate) = benchmark_with_cache()?;
    println!();

    // Results
    println!("═══════════════════════════════════════════════════════════════");
    println!("  RESULTS");
    println!("═══════════════════════════════════════════════════════════════");
    println!();
    println!("Average query latency:");
    println!("  Without cache: {:.3} μs", no_cache_us);
    println!("  With cache:    {:.3} μs", cache_us);
    println!();

    let speedup = no_cache_us / cache_us;
    let verdict = if speedup >= 2.0 {
        "✅ EXCELLENT"
    } else if speedup >= 1.5 {
        "✅ GOOD"
    } else if speedup >= 1.2 {
        "⚠️  MODEST"
    } else {
        "❌ INSUFFICIENT"
    };

    println!("🎯 Speedup: {:.2}x {}", speedup, verdict);
    println!("📈 Cache hit rate: {:.1}%", hit_rate);
    println!();

    // Analysis
    if hit_rate >= 80.0 && speedup >= 2.0 {
        println!("✅ SUCCESS: Cache achieves 2-3x target with 80%+ hit rate");
    } else if hit_rate < 80.0 {
        println!("⚠️  Cache hit rate below 80% target");
        println!("   Consider: Increase cache size or adjust workload");
    } else if speedup < 2.0 {
        println!("⚠️  Speedup below 2x target");
        println!("   Investigate: Overhead sources, cache implementation");
    }
    println!();

    Ok(())
}

fn benchmark_without_cache() -> Result<f64> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    // Create schema
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Int64, false),
        Field::new("text", DataType::Utf8, false),
    ]));

    // Create table without cache
    let table_dir = temp_dir.path().join("test_table");
    let mut table = omendb::table::Table::new(
        "test".to_string(),
        schema,
        "id".to_string(),
        table_dir,
    )?;

    // Insert data
    println!("  Inserting {} rows...", format_number(DATA_SIZE));
    let insert_start = Instant::now();
    for i in 0..DATA_SIZE {
        let row = Row::new(vec![
            Value::Int64(i as i64),
            Value::Int64(i as i64 * 100),
            Value::Text(format!("row_{}", i)),
        ]);
        table.insert(row)?;
    }
    let insert_ms = insert_start.elapsed().as_secs_f64() * 1000.0;
    println!("  Insert time: {:.2} ms ({} rows/sec)",
        insert_ms,
        format_number((DATA_SIZE as f64 / (insert_ms / 1000.0)) as usize)
    );

    // Query with Zipfian distribution (80% of queries hit 20% of data - realistic workload)
    // With repeated queries to allow cache to be effective
    println!("  Running {} queries (Zipfian distribution)...", format_number(QUERY_COUNT));
    let query_start = Instant::now();

    // Pre-generate query keys with Zipfian distribution
    let mut query_keys = Vec::with_capacity(QUERY_COUNT);
    for i in 0..QUERY_COUNT {
        let key = if (i / 10) % 5 == 0 {
            // 20% of time: random cold data
            ((i / 10) * 7919) % DATA_SIZE
        } else {
            // 80% of time: hot data (first 10% of rows)
            (i / 10) % (DATA_SIZE / 10)
        };
        query_keys.push(key);
    }

    // Execute queries (keys will be repeated, allowing cache hits)
    for &key in &query_keys {
        let _result = table.get(&Value::Int64(key as i64))?;
    }

    let query_us = query_start.elapsed().as_micros() as f64 / QUERY_COUNT as f64;
    println!("  Query time: {:.3} μs avg", query_us);

    Ok(query_us)
}

fn benchmark_with_cache() -> Result<(f64, f64)> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    // Create schema
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Int64, false),
        Field::new("text", DataType::Utf8, false),
    ]));

    // Create table WITH cache
    let table_dir = temp_dir.path().join("test_table");
    let mut table = omendb::table::Table::new_with_cache(
        "test".to_string(),
        schema,
        "id".to_string(),
        table_dir,
        CACHE_SIZE,
    )?;

    // Insert data
    println!("  Inserting {} rows...", format_number(DATA_SIZE));
    let insert_start = Instant::now();
    for i in 0..DATA_SIZE {
        let row = Row::new(vec![
            Value::Int64(i as i64),
            Value::Int64(i as i64 * 100),
            Value::Text(format!("row_{}", i)),
        ]);
        table.insert(row)?;
    }
    let insert_ms = insert_start.elapsed().as_secs_f64() * 1000.0;
    println!("  Insert time: {:.2} ms ({} rows/sec)",
        insert_ms,
        format_number((DATA_SIZE as f64 / (insert_ms / 1000.0)) as usize)
    );

    // Query with Zipfian distribution (same as above)
    println!("  Running {} queries (Zipfian distribution)...", format_number(QUERY_COUNT));
    let query_start = Instant::now();

    // Pre-generate query keys with Zipfian distribution (same as baseline)
    let mut query_keys = Vec::with_capacity(QUERY_COUNT);
    for i in 0..QUERY_COUNT {
        let key = if (i / 10) % 5 == 0 {
            // 20% of time: random cold data
            ((i / 10) * 7919) % DATA_SIZE
        } else {
            // 80% of time: hot data (first 10% of rows)
            (i / 10) % (DATA_SIZE / 10)
        };
        query_keys.push(key);
    }

    // Execute queries (keys will be repeated, allowing cache hits)
    for &key in &query_keys {
        let _result = table.get(&Value::Int64(key as i64))?;
    }

    let query_us = query_start.elapsed().as_micros() as f64 / QUERY_COUNT as f64;
    println!("  Query time: {:.3} μs avg", query_us);

    // Get cache statistics
    let stats = table.cache_stats().unwrap();
    println!("  Cache hits: {}", format_number(stats.hits as usize));
    println!("  Cache misses: {}", format_number(stats.misses as usize));
    println!("  Cache hit rate: {:.1}%", stats.hit_rate);

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
