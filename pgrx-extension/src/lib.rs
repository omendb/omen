use pgrx::prelude::*;
use omendb::{LinearIndex, RMIIndex, LearnedIndex};

pg_module_magic!();

/// Safe benchmark function that won't crash
#[pg_extern]
fn learned_index_benchmark(num_keys: i32) -> String {
    // Validate input
    if num_keys <= 0 || num_keys > 1_000_000 {
        return "Error: num_keys must be between 1 and 1,000,000".to_string();
    }

    use std::collections::BTreeMap;
    use std::time::Instant;

    // Generate test data safely
    let mut data = Vec::new();
    let mut btree = BTreeMap::new();

    for i in 0..num_keys {
        let key = i as i64 * 2;
        let value = i as i64;
        data.push((key, value));
        btree.insert(key, value);
    }

    // Train learned indexes with error handling
    let linear_index = match LinearIndex::train(data.clone()) {
        Ok(index) => index,
        Err(e) => return format!("Failed to train linear index: {:?}", e)
    };

    let rmi_index = match RMIIndex::train(data) {
        Ok(index) => index,
        Err(e) => return format!("Failed to train RMI index: {:?}", e)
    };

    // Generate test queries
    let num_queries = 1000.min(num_keys);
    let test_keys: Vec<i64> = (0..num_queries)
        .map(|i| ((i % num_keys) as i64) * 2)
        .collect();

    // Benchmark Linear Index
    let start = Instant::now();
    let mut linear_found = 0;
    for &key in &test_keys {
        if linear_index.get(&key).is_some() {
            linear_found += 1;
        }
    }
    let linear_time = start.elapsed();

    // Benchmark RMI
    let start = Instant::now();
    let mut rmi_found = 0;
    for &key in &test_keys {
        if rmi_index.get(&key).is_some() {
            rmi_found += 1;
        }
    }
    let rmi_time = start.elapsed();

    // Benchmark B-tree
    let start = Instant::now();
    let mut btree_found = 0;
    for &key in &test_keys {
        if btree.get(&key).is_some() {
            btree_found += 1;
        }
    }
    let btree_time = start.elapsed();

    // Calculate performance safely
    let linear_qps = if linear_time.as_secs_f64() > 0.0 {
        num_queries as f64 / linear_time.as_secs_f64()
    } else {
        0.0
    };

    let rmi_qps = if rmi_time.as_secs_f64() > 0.0 {
        num_queries as f64 / rmi_time.as_secs_f64()
    } else {
        0.0
    };

    let btree_qps = if btree_time.as_secs_f64() > 0.0 {
        num_queries as f64 / btree_time.as_secs_f64()
    } else {
        0.0
    };

    format!(
        "Learned Index Benchmark Results:\n\
         Dataset: {} keys, {} queries\n\
         \n\
         Linear Index: {:.0} queries/sec ({} found)\n\
         RMI Index:    {:.0} queries/sec ({} found)\n\
         BTreeMap:     {:.0} queries/sec ({} found)\n\
         \n\
         Linear Speedup: {:.2}x\n\
         RMI Speedup:    {:.2}x\n\
         \n\
         Note: This is a demonstration of learned index performance.\n\
         Real CREATE INDEX integration coming soon.",
        num_keys, num_queries,
        linear_qps, linear_found,
        rmi_qps, rmi_found,
        btree_qps, btree_found,
        if btree_qps > 0.0 { linear_qps / btree_qps } else { 0.0 },
        if btree_qps > 0.0 { rmi_qps / btree_qps } else { 0.0 }
    )
}

/// Simple test function to verify extension loads
#[pg_extern]
fn learned_index_version() -> String {
    "OmenDB Learned Index Extension v0.1.0 - Experimental".to_string()
}

/// Explain how learned indexes work
#[pg_extern]
fn learned_index_info() -> String {
    "Learned indexes use machine learning to predict data location instead of tree traversal.\n\
     \n\
     Traditional B-tree: O(log n) with multiple disk seeks\n\
     Learned index: O(1) with 1-2 CPU instructions\n\
     \n\
     Result: 2-10x faster lookups for ordered data.\n\
     \n\
     Try: SELECT learned_index_benchmark(10000);\n\
     \n\
     More info: https://github.com/omendb/omendb".to_string()
}

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {}
    pub fn postgresql_conf_options() -> Vec<&'static str> { vec![] }
}