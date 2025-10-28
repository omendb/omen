//! Stress test multi-level ALEX at 100M scale
//!
//! Ultimate validation that our architecture scales to enterprise levels.
//! Tests build performance, query latency, and memory usage at 100M rows.

use anyhow::Result;
use omen::alex::MultiLevelAlexTree;
use rand::prelude::*;
use rusqlite::{Connection, params};
use std::time::Instant;

fn main() -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          100M Scale Stress Test - Multi-Level ALEX          ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Test at increasing scales to show progression
    let scales = vec![
        ("10M", 10_000_000),
        ("25M", 25_000_000),
        ("50M", 50_000_000),
        ("75M", 75_000_000),
        ("100M", 100_000_000),
    ];

    for (label, size) in scales {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📊 Testing at {} scale", label);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        if let Err(e) = test_scale(size, label) {
            eprintln!("❌ Error at {} scale: {}", label, e);
            break;
        }
    }

    Ok(())
}

fn test_scale(size: usize, label: &str) -> Result<()> {
    // Generate test data
    println!("🔄 Generating {} test keys...", size);
    let start = Instant::now();
    let mut data = Vec::with_capacity(size);
    let mut rng = thread_rng();

    // Use deterministic seed for reproducibility
    let mut seeded_rng = StdRng::seed_from_u64(12345);

    for _ in 0..size {
        data.push(seeded_rng.gen::<i64>());
    }

    let gen_time = start.elapsed();
    println!("  Generation time: {:.2}s", gen_time.as_secs_f64());

    // Sort for bulk loading
    println!("  Sorting keys...");
    let start = Instant::now();
    data.sort_unstable();
    let sort_time = start.elapsed();
    println!("  Sort time: {:.2}s", sort_time.as_secs_f64());

    // Prepare data with values
    let data_with_values: Vec<(i64, Vec<u8>)> = data
        .iter()
        .map(|&k| (k, vec![0u8; 8]))
        .collect();

    // Build multi-level ALEX
    println!("\n📦 Building multi-level ALEX tree...");
    let start = Instant::now();
    let tree = match MultiLevelAlexTree::bulk_build(data_with_values.clone()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("  ❌ Failed to build tree: {}", e);
            return Err(e);
        }
    };
    let alex_build_time = start.elapsed();

    println!("  ✅ Build time: {:.2}s", alex_build_time.as_secs_f64());
    println!("  Height: {}", tree.height());
    println!("  Leaves: {}", tree.num_leaves());
    println!("  Keys/leaf: {:.1}", size as f64 / tree.num_leaves() as f64);

    // Memory estimation
    let leaf_memory = tree.num_leaves() * 88; // Approximate bytes per leaf
    let total_memory = leaf_memory + (tree.num_leaves() * 8); // Plus routing overhead
    println!("  Memory estimate: {:.2} MB", total_memory as f64 / (1024.0 * 1024.0));

    // Test queries (sample 10K random keys)
    println!("\n🔍 Testing queries (10K samples)...");
    let sample_size = 10_000.min(size);
    let query_keys: Vec<i64> = data.choose_multiple(&mut rng, sample_size)
        .copied()
        .collect();

    let start = Instant::now();
    let mut found = 0;
    for &key in &query_keys {
        if tree.get(key)?.is_some() {
            found += 1;
        }
    }
    let query_time = start.elapsed();
    let avg_query = query_time.as_nanos() as f64 / query_keys.len() as f64;

    println!("  Total time: {:.2}ms", query_time.as_millis());
    println!("  Avg per query: {:.1}ns", avg_query);
    println!("  Found: {}/{}", found, query_keys.len());
    println!("  Miss rate: {:.2}%", (1.0 - found as f64 / query_keys.len() as f64) * 100.0);

    // Latency percentiles (run 1000 queries for percentiles)
    println!("\n📊 Query latency percentiles (1000 samples):");
    let mut latencies = Vec::new();
    let percentile_keys: Vec<i64> = data.choose_multiple(&mut rng, 1000)
        .copied()
        .collect();

    for &key in &percentile_keys {
        let start = Instant::now();
        let _ = tree.get(key)?;
        latencies.push(start.elapsed().as_nanos());
    }

    latencies.sort_unstable();
    let p50 = latencies[latencies.len() * 50 / 100];
    let p90 = latencies[latencies.len() * 90 / 100];
    let p95 = latencies[latencies.len() * 95 / 100];
    let p99 = latencies[latencies.len() * 99 / 100];

    println!("  P50: {}ns", p50);
    println!("  P90: {}ns", p90);
    println!("  P95: {}ns", p95);
    println!("  P99: {}ns", p99);

    // Test inserts (small batch to verify functionality)
    println!("\n📝 Testing inserts (100 random keys)...");
    let mut tree_mut = MultiLevelAlexTree::bulk_build(data_with_values.clone())?;
    let insert_keys: Vec<i64> = (0..100).map(|_| rng.gen()).collect();

    let start = Instant::now();
    for &key in &insert_keys {
        tree_mut.insert(key, vec![0u8; 8])?;
    }
    let insert_time = start.elapsed();

    println!("  Insert time: {:.2}ms", insert_time.as_millis());
    println!("  Avg per insert: {:.2}μs",
             insert_time.as_nanos() as f64 / insert_keys.len() as f64 / 1000.0);

    // Compare with SQLite (only up to 50M for reasonable time)
    if size <= 50_000_000 {
        println!("\n🔀 Comparing with SQLite...");

        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE bench (
                key INTEGER PRIMARY KEY,
                value BLOB
            )",
            [],
        )?;

        println!("  Building SQLite database...");
        let start = Instant::now();
        let tx = conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare("INSERT INTO bench (key, value) VALUES (?1, ?2)")?;
            for (key, value) in &data_with_values[..size.min(10_000_000)] {
                stmt.execute(params![key, value])?;
            }
        }
        tx.commit()?;
        let sqlite_build_time = start.elapsed();
        println!("  SQLite build: {:.2}s", sqlite_build_time.as_secs_f64());

        // Create index
        let start = Instant::now();
        conn.execute("CREATE INDEX idx_key ON bench(key)", [])?;
        let index_time = start.elapsed();
        println!("  SQLite index: {:.2}s", index_time.as_secs_f64());

        // Test SQLite queries
        let mut stmt = conn.prepare("SELECT value FROM bench WHERE key = ?1")?;
        let sample_keys: Vec<i64> = data[..size.min(10_000_000)]
            .choose_multiple(&mut rng, 1000)
            .copied()
            .collect();

        let start = Instant::now();
        for &key in &sample_keys {
            let _ = stmt.query_row(params![key], |_row| Ok(()));
        }
        let sqlite_query_time = start.elapsed();
        let sqlite_avg = sqlite_query_time.as_nanos() as f64 / sample_keys.len() as f64;

        println!("  SQLite avg query: {:.1}ns", sqlite_avg);
        println!("  ALEX speedup: {:.2}x", sqlite_avg / avg_query);
    }

    // Summary
    println!("\n✅ {} Summary:", label);
    println!("  Build performance: {:.1} M keys/sec",
             size as f64 / alex_build_time.as_secs_f64() / 1_000_000.0);
    println!("  Query throughput: {:.1} M queries/sec",
             1_000_000_000.0 / avg_query / 1_000_000.0);
    println!("  Space efficiency: {:.2} bytes/key",
             total_memory as f64 / size as f64);

    Ok(())
}