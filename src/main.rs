//! OmenDB - World's first database using only learned indexes
//!
//! This is a 6-week sprint to YC S26 application

mod learned_index;
mod storage;
mod protocol;

use learned_index::{LearnedIndex, LinearLearnedIndex, LearnedIndexConfig};
use std::collections::BTreeMap;
use std::time::Instant;

fn main() {
    println!("🚀 OmenDB - Replacing B-trees with AI\n");

    // Quick demonstration of learned index performance
    demonstrate_learned_index();
}

fn demonstrate_learned_index() {
    println!("=== Learned Index Performance Demo ===\n");

    // Generate time-series data (sequential timestamps)
    let num_keys = 1_000_000;
    let mut data = Vec::new();
    let mut btree = BTreeMap::new();

    println!("📊 Generating {} time-series keys...", num_keys);
    for i in 0..num_keys {
        // Timestamps with microsecond precision
        let key = 1_600_000_000_000_000 + (i as i64 * 1000);
        let position = i;
        data.push((key, position));
        btree.insert(key, position);
    }

    // Train learned index
    println!("🧠 Training learned index...");
    let config = LearnedIndexConfig::default();
    let mut learned = LinearLearnedIndex::new(config);
    let train_start = Instant::now();
    learned.train(&data).unwrap();
    let train_time = train_start.elapsed();
    println!("   Training time: {:.2?}", train_time);
    println!("   Model stats: {}\n", learned.stats());

    // Benchmark lookups
    let num_lookups = 10_000;
    let test_keys: Vec<i64> = (0..num_lookups)
        .map(|i| 1_600_000_000_000_000 + (i as i64 * 100_000))
        .collect();

    // Learned index lookups
    println!("⚡ Testing {} lookups on learned index...", num_lookups);
    let start = Instant::now();
    let mut learned_found = 0;
    for &key in &test_keys {
        if learned.search(key).is_ok() {
            learned_found += 1;
        }
    }
    let learned_time = start.elapsed();

    // B-tree lookups
    println!("🌳 Testing {} lookups on B-tree...", num_lookups);
    let start = Instant::now();
    let mut btree_found = 0;
    for &key in &test_keys {
        if btree.contains_key(&key) {
            btree_found += 1;
        }
    }
    let btree_time = start.elapsed();

    // Results
    println!("\n=== RESULTS ===");
    println!("Learned Index:");
    println!("  Time: {:.2?}", learned_time);
    println!("  Found: {}/{}", learned_found, num_lookups);
    println!("  Rate: {:.0} lookups/sec", num_lookups as f64 / learned_time.as_secs_f64());

    println!("\nB-tree:");
    println!("  Time: {:.2?}", btree_time);
    println!("  Found: {}/{}", btree_found, num_lookups);
    println!("  Rate: {:.0} lookups/sec", num_lookups as f64 / btree_time.as_secs_f64());

    let speedup = btree_time.as_secs_f64() / learned_time.as_secs_f64();
    println!("\n🏆 Speedup: {:.2}x", speedup);

    if speedup > 2.0 {
        println!("✅ Learned indexes are faster!");
    } else {
        println!("⚠️  Need more optimization...");
    }

    // Range query demo
    println!("\n=== Range Query Demo ===");
    let range_start = 1_600_000_000_000_000;
    let range_end = 1_600_000_000_100_000;

    let start = Instant::now();
    let learned_range = learned.range(range_start, range_end).unwrap();
    let learned_range_time = start.elapsed();

    println!("Learned index range query:");
    println!("  Found {} keys in {:.2?}", learned_range.len(), learned_range_time);

    println!("\n📈 Next steps:");
    println!("  1. Implement hierarchical model for better accuracy");
    println!("  2. Add Arrow storage integration");
    println!("  3. PostgreSQL wire protocol");
    println!("  4. Launch on HackerNews!");
}