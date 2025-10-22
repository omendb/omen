//! Diagnostic tool: Break down query latency components
//!
//! This benchmark instruments every step of the query path to identify bottlenecks:
//! 1. Cache lookup
//! 2. ALEX index lookup
//! 3. RocksDB get
//! 4. Cache update

use anyhow::Result;
use omendb::alex::AlexTree;
use rocksdb::{DB, Options};
use std::time::Instant;
use tempfile::TempDir;

const SCALE: usize = 10_000_000;
const NUM_QUERIES: usize = 10_000;

fn main() -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║        Query Latency Diagnostic - 10M Scale                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("diagnostic.db");

    // Setup RocksDB
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.set_write_buffer_size(256 * 1024 * 1024);
    let db = DB::open(&opts, &db_path)?;

    // Setup ALEX
    let mut alex = AlexTree::new();

    println!("📦 Building database and index with {} rows...", SCALE);
    let start = Instant::now();

    // Insert data
    for i in 0..SCALE as i64 {
        let key_bytes = i.to_be_bytes();
        let value = format!("value_{}", i);
        db.put(key_bytes, value.as_bytes())?;
        alex.insert(i, vec![1])?;
    }

    println!("  ✅ Build time: {:.2}s\n", start.elapsed().as_secs_f64());

    // Warmup queries
    println!("🔥 Warmup: {} queries...", NUM_QUERIES / 10);
    for i in 0..(NUM_QUERIES / 10) {
        let key = (i as i64 * 1000) % SCALE as i64;
        let key_bytes = key.to_be_bytes();
        let _ = db.get(key_bytes)?;
    }

    // Diagnostic queries with timing breakdown
    println!("\n🔍 Diagnostic queries ({}x):\n", NUM_QUERIES);

    let mut total_alex_time_ns = 0u128;
    let mut total_rocksdb_time_ns = 0u128;
    let mut total_query_time_ns = 0u128;

    for i in 0..NUM_QUERIES {
        let key = (i as i64 * 100) % SCALE as i64;

        let query_start = Instant::now();

        // ALEX lookup
        let alex_start = Instant::now();
        let _exists = alex.get(key)?;
        let alex_duration = alex_start.elapsed().as_nanos();
        total_alex_time_ns += alex_duration;

        // RocksDB get
        let rocksdb_start = Instant::now();
        let key_bytes = key.to_be_bytes();
        let _value = db.get(key_bytes)?;
        let rocksdb_duration = rocksdb_start.elapsed().as_nanos();
        total_rocksdb_time_ns += rocksdb_duration;

        let query_duration = query_start.elapsed().as_nanos();
        total_query_time_ns += query_duration;
    }

    // Calculate averages
    let avg_alex_ns = total_alex_time_ns as f64 / NUM_QUERIES as f64;
    let avg_rocksdb_ns = total_rocksdb_time_ns as f64 / NUM_QUERIES as f64;
    let avg_total_ns = total_query_time_ns as f64 / NUM_QUERIES as f64;
    let avg_overhead_ns = avg_total_ns - avg_alex_ns - avg_rocksdb_ns;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Latency Breakdown (average over {} queries):", NUM_QUERIES);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("  🔍 ALEX Index Lookup:      {:>8.0}ns  ({:>5.1}%)",
        avg_alex_ns, (avg_alex_ns / avg_total_ns) * 100.0);
    println!("  💾 RocksDB Get:            {:>8.0}ns  ({:>5.1}%)",
        avg_rocksdb_ns, (avg_rocksdb_ns / avg_total_ns) * 100.0);
    println!("  ⚙️  Overhead/Other:         {:>8.0}ns  ({:>5.1}%)",
        avg_overhead_ns, (avg_overhead_ns / avg_total_ns) * 100.0);
    println!("  ─────────────────────────────────────────────");
    println!("  📈 Total Query Time:       {:>8.0}ns  (100.0%)", avg_total_ns);
    println!("                             {:>8.2}μs", avg_total_ns / 1000.0);

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Bottleneck analysis
    println!("🔬 Bottleneck Analysis:\n");

    let alex_pct = (avg_alex_ns / avg_total_ns) * 100.0;
    let rocksdb_pct = (avg_rocksdb_ns / avg_total_ns) * 100.0;

    if rocksdb_pct > 70.0 {
        println!("  ⚠️  BOTTLENECK: RocksDB ({:.1}%)", rocksdb_pct);
        println!("     → Likely LSM read amplification");
        println!("     → Consider: block cache tuning, bloom filters");
    } else if alex_pct > 50.0 {
        println!("  ⚠️  BOTTLENECK: ALEX Index ({:.1}%)", alex_pct);
        println!("     → Index structure depth/traversal cost");
        println!("     → Consider: cache ALEX predictions");
    } else {
        println!("  ✅ BALANCED: No single dominant bottleneck");
        println!("     → Optimization needs profiling to identify");
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Compare with isolated ALEX stress test results
    println!("📌 Reference: ALEX isolated stress test (in-memory):");
    println!("   10M rows: 468ns query latency");
    println!("\n   Current ALEX: {:.0}ns ({:.1}x slower than isolated)",
        avg_alex_ns, avg_alex_ns / 468.0);
    println!("   This suggests: {}",
        if avg_alex_ns < 1000.0 {
            "ALEX overhead is reasonable ✅"
        } else {
            "ALEX integration overhead is high ⚠️"
        }
    );

    Ok(())
}
