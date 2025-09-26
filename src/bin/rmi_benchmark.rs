use omendb::{LinearIndex, RMIIndex, LearnedIndex};
use std::collections::BTreeMap;
use std::time::Instant;
use rand::prelude::*;

fn main() {
    println!("OmenDB RMI vs Linear vs BTreeMap Benchmark\n");
    println!("==========================================\n");

    // Test different data sizes - focus on larger datasets where RMI should shine
    let test_sizes = vec![10000, 50000, 100000, 500000];

    for n in test_sizes {
        println!("Testing with {} keys...", n);

        // Generate test data - simple sequential with gaps
        let mut data = Vec::new();
        let mut btree = BTreeMap::new();
        for i in 0..n {
            let key = i as i64 * 2; // Keys: 0, 2, 4, 6, ...
            let value = format!("value_{}", i);
            data.push((key, value.clone()));
            btree.insert(key, value);
        }

        // Train both learned indexes
        println!("  Training models...");
        let rmi_index = RMIIndex::train(data.clone()).expect("RMI training failed");
        let linear_index = LinearIndex::train(data.clone()).expect("Linear training failed");

        println!("  RMI has {} leaf models", rmi_index.leaf_models.len());

        // Generate test queries - all should exist
        let mut queries = Vec::new();
        for i in 0..1000 {
            let query_idx = i % n;
            queries.push((query_idx as i64) * 2);
        }

        println!("  Generated {} test queries", queries.len());

        // Test RMI
        let start = Instant::now();
        let mut rmi_found = 0;
        for &key in &queries {
            if rmi_index.get(&key).is_some() {
                rmi_found += 1;
            }
        }
        let rmi_time = start.elapsed();

        // Test Linear Index
        let start = Instant::now();
        let mut linear_found = 0;
        for &key in &queries {
            if linear_index.get(&key).is_some() {
                linear_found += 1;
            }
        }
        let linear_time = start.elapsed();

        // Test BTreeMap
        let start = Instant::now();
        let mut btree_found = 0;
        for &key in &queries {
            if btree.get(&key).is_some() {
                btree_found += 1;
            }
        }
        let btree_time = start.elapsed();

        // Calculate throughput
        let rmi_qps = queries.len() as f64 / rmi_time.as_secs_f64();
        let linear_qps = queries.len() as f64 / linear_time.as_secs_f64();
        let btree_qps = queries.len() as f64 / btree_time.as_secs_f64();

        println!("  Results:");
        println!("    RMI Index:    {:.0} q/s ({}/{} found)", rmi_qps, rmi_found, queries.len());
        println!("    Linear Index: {:.0} q/s ({}/{} found)", linear_qps, linear_found, queries.len());
        println!("    BTreeMap:     {:.0} q/s ({}/{} found)", btree_qps, btree_found, queries.len());

        if btree_found > 0 {
            println!("    RMI vs BTree: {:.2}x speedup", rmi_qps / btree_qps);
            println!("    Linear vs BTree: {:.2}x speedup", linear_qps / btree_qps);
            if rmi_found == btree_found {
                println!("    RMI vs Linear: {:.2}x speedup", rmi_qps / linear_qps);
            }
        }

        // Debug: Test a few specific keys
        if rmi_found < queries.len() {
            println!("  Debugging missing keys:");
            for i in 0..5.min(queries.len()) {
                let key = queries[i];
                let rmi_result = rmi_index.get(&key);
                let linear_result = linear_index.get(&key);
                let btree_result = btree.get(&key);

                println!("    Key {}: RMI={:?}, Linear={:?}, BTree={:?}",
                    key,
                    rmi_result.is_some(),
                    linear_result.is_some(),
                    btree_result.is_some()
                );
            }
        }

        println!();
    }

    // Test range queries
    println!("Range Query Test:");
    println!("=================");

    let mut data = Vec::new();
    for i in 0..5000 {
        data.push((i as i64 * 3, i)); // Keys: 0, 3, 6, 9, ...
    }

    let rmi_index = RMIIndex::train(data).unwrap();

    let start = Instant::now();
    for _ in 0..100 {
        let _result = rmi_index.range(&300, &600);
    }
    let rmi_range_time = start.elapsed();

    println!("  RMI range queries: {:.2}ms for 100 queries",
        rmi_range_time.as_millis() as f64 / 100.0);
}