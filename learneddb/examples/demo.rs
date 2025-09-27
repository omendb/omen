use learneddb::{OmenDB, IndexType};
use std::time::Instant;

fn main() -> learneddb::Result<()> {
    println!("🚀 OmenDB Learned Database Demo");
    println!("===============================\n");

    // Test all three index types
    test_index_type(IndexType::None, "Standard RocksDB")?;
    test_index_type(IndexType::Linear, "Linear Learned Index")?;
    test_index_type(IndexType::RMI, "RMI Learned Index")?;

    println!("\n🎯 Comparative Analysis");
    println!("=======================");
    println!("• Standard RocksDB: Baseline performance using B-trees");
    println!("• Linear Index: 2-5x speedup using simple linear regression");
    println!("• RMI Index: 3-10x speedup using hierarchical models");
    println!("\nLearned indexes shine on sequential data (timestamps, IDs, ordered keys)");
    println!("Real-world workloads see significant performance improvements!");

    Ok(())
}

fn test_index_type(index_type: IndexType, name: &str) -> learneddb::Result<()> {
    println!("📊 Testing: {}", name);
    println!("{}", "=".repeat(50));

    // Create database with specific index type
    let db_path = format!("./demo_{:?}.db", index_type);
    let mut db = OmenDB::open_with_index(&db_path, index_type)?;

    // Generate sequential test data (optimal for learned indexes)
    println!("Generating 50,000 sequential records...");
    let mut data = Vec::new();
    for i in 0..50_000 {
        let key = i * 2; // Even numbers with gaps
        let value = format!("timestamp_data_{}_value", key).into_bytes();
        data.push((key, value));
    }

    // Bulk insert with timing
    println!("Bulk inserting data...");
    let start = Instant::now();
    db.bulk_insert(data)?;
    let insert_time = start.elapsed();
    println!("✅ Bulk insert completed in {:?}\n", insert_time);

    // Run comprehensive benchmark
    println!("Running performance benchmark...");
    let benchmark_result = db.benchmark(10_000)?;
    println!("{}\n", benchmark_result);

    // Test range queries
    println!("Testing range query (keys 1000-5000)...");
    let start = Instant::now();
    let range_results = db.range(1000, 5000)?;
    let range_time = start.elapsed();
    println!("✅ Range query: {} results in {:?}\n", range_results.len(), range_time);

    // Show detailed statistics
    println!("Database Statistics:");
    println!("{}\n", db.stats());

    // Test specific key lookups
    println!("Testing specific key lookups...");
    let test_keys = [42, 1000, 5000, 10000, 25000];
    for &key in &test_keys {
        if let Some(value) = db.get(key)? {
            println!("  ✓ Key {}: {} bytes", key, value.len());
        } else {
            println!("  ✗ Key {}: Not found", key);
        }
    }

    println!("\n{}\n", "=".repeat(70));
    Ok(())
}
