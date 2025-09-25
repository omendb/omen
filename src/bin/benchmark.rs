use omendb::{LinearIndex, LearnedIndex};
use std::collections::BTreeMap;
use std::time::Instant;
use rand::prelude::*;

fn main() {
    println!("OmenDB Learned Index vs BTreeMap Benchmark\n");
    println!("==========================================\n");

    // Test different data sizes
    let test_sizes = vec![100, 500, 1000, 5000, 10000, 50000, 100000];

    for n in test_sizes {
        println!("Testing with {} keys...", n);

        // Generate sorted data with some gaps
        let mut data = Vec::new();
        let mut btree = BTreeMap::new();
        for i in 0..n {
            let key = i as i64 * 2; // Keys: 0, 2, 4, 6, ...
            let value = format!("value_{}", i);
            data.push((key, value.clone()));
            btree.insert(key, value);
        }

        // Train learned index
        let index = LinearIndex::train(data.clone()).unwrap();

        // Generate test queries (80% existing, 20% non-existing)
        let mut queries = Vec::new();
        let mut rng = thread_rng();
        for _ in 0..10000 {
            if rng.gen_bool(0.8) {
                // Existing key
                let idx = rng.gen_range(0..n);
                queries.push(idx as i64 * 2);
            } else {
                // Non-existing key
                queries.push(rng.gen_range(0..n*2) as i64 * 2 + 1);
            }
        }

        // Benchmark learned index
        let start = Instant::now();
        let mut learned_found = 0;
        for &key in &queries {
            if index.get(&key).is_some() {
                learned_found += 1;
            }
        }
        let learned_time = start.elapsed();

        // Benchmark B-tree
        let start = Instant::now();
        let mut btree_found = 0;
        for &key in &queries {
            if btree.get(&key).is_some() {
                btree_found += 1;
            }
        }
        let btree_time = start.elapsed();

        // Calculate throughput
        let learned_qps = 10000.0 / learned_time.as_secs_f64();
        let btree_qps = 10000.0 / btree_time.as_secs_f64();
        let speedup = learned_qps / btree_qps;

        println!("  Learned Index: {:.0} queries/sec ({} found)", learned_qps, learned_found);
        println!("  BTreeMap:      {:.0} queries/sec ({} found)", btree_qps, btree_found);
        println!("  Speedup:       {:.2}x", speedup);

        if speedup >= 3.0 {
            println!("  ✅ TARGET ACHIEVED!");
        } else if speedup >= 2.0 {
            println!("  📈 Good progress (need 3x)");
        } else {
            println!("  ⚠️  Below target");
        }
        println!();
    }

    // Test range queries
    println!("\nRange Query Test (1000 keys):");
    println!("================================");

    let mut data = Vec::new();
    let mut btree = BTreeMap::new();
    for i in 0..1000 {
        let key = i as i64 * 2;
        let value = i;
        data.push((key, value));
        btree.insert(key, value);
    }

    let index = LinearIndex::train(data).unwrap();

    // Test different range sizes
    for range_size in [10, 50, 100, 500] {
        let start_key = 500;
        let end_key = start_key + range_size * 2;

        // Learned index range
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = index.range(&start_key, &end_key);
        }
        let learned_time = start.elapsed();

        // BTree range
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = btree.range(start_key..=end_key).map(|(_, v)| *v).collect::<Vec<_>>();
        }
        let btree_time = start.elapsed();

        let speedup = btree_time.as_nanos() as f64 / learned_time.as_nanos() as f64;

        println!("  Range size {}: {:.2}x speedup", range_size, speedup);
    }
}