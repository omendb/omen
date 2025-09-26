use pgrx::prelude::*;
use omendb::{LinearIndex, RMIIndex, LearnedIndex};
use std::collections::HashMap;
use std::sync::{Mutex, LazyLock};

// Initialize the extension
pg_module_magic!();

// Global storage for learned indexes
// In production, this would be more sophisticated with proper lifecycle management
static LEARNED_INDEXES: LazyLock<Mutex<HashMap<String, LinearIndex<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[pg_extern]
fn create_learned_index(index_name: &str, table_name: &str, column_name: &str) -> String {
    // This is a simplified version - in reality we'd integrate with PostgreSQL's
    // index creation system and read actual table data

    let mock_data = vec![
        (1i64, "value1".to_string()),
        (2i64, "value2".to_string()),
        (3i64, "value3".to_string()),
        (10i64, "value10".to_string()),
        (20i64, "value20".to_string()),
    ];

    match LinearIndex::train(mock_data) {
        Ok(index) => {
            let mut indexes = LEARNED_INDEXES.lock().unwrap();
            indexes.insert(index_name.to_string(), index);
            format!("Learned index '{}' created for {}.{}", index_name, table_name, column_name)
        },
        Err(e) => {
            format!("Failed to create learned index: {:?}", e)
        }
    }
}

#[pg_extern]
fn lookup_learned_index(index_name: &str, key: i64) -> Option<String> {
    let indexes = LEARNED_INDEXES.lock().unwrap();

    match indexes.get(index_name) {
        Some(index) => index.get(&key),
        None => None
    }
}

#[pg_extern]
fn benchmark_learned_vs_btree(num_keys: i32) -> String {
    use std::collections::BTreeMap;
    use std::time::Instant;

    // Generate test data
    let mut data = Vec::new();
    let mut btree = BTreeMap::new();

    for i in 0..num_keys {
        let key = i as i64 * 2; // Even keys
        let value = format!("value_{}", i);
        data.push((key, value.clone()));
        btree.insert(key, value);
    }

    // Train learned index
    let learned_index = match LinearIndex::train(data) {
        Ok(index) => index,
        Err(e) => return format!("Failed to train learned index: {:?}", e)
    };

    let num_queries = 1000;
    let test_keys: Vec<i64> = (0..num_queries).map(|i| (i % num_keys) as i64 * 2).collect();

    // Benchmark learned index
    let start = Instant::now();
    for &key in &test_keys {
        let _ = learned_index.get(&key);
    }
    let learned_time = start.elapsed();

    // Benchmark B-tree
    let start = Instant::now();
    for &key in &test_keys {
        let _ = btree.get(&key);
    }
    let btree_time = start.elapsed();

    let speedup = btree_time.as_nanos() as f64 / learned_time.as_nanos() as f64;

    format!(
        "Benchmark Results ({} keys, {} queries):\n\
         Learned Index: {:?}\n\
         BTreeMap:      {:?}\n\
         Speedup:       {:.2}x",
        num_keys, num_queries, learned_time, btree_time, speedup
    )
}

#[pg_extern]
fn benchmark_rmi_postgres(num_keys: i32) -> String {
    use std::collections::BTreeMap;
    use std::time::Instant;

    // Generate test data
    let mut data = Vec::new();
    let mut btree = BTreeMap::new();

    for i in 0..num_keys {
        let key = i as i64 * 2; // Even keys
        let value = format!("value_{}", i);
        data.push((key, value.clone()));
        btree.insert(key, value);
    }

    // Train RMI and Linear indexes
    let rmi_index = match RMIIndex::train(data.clone()) {
        Ok(index) => index,
        Err(e) => return format!("Failed to train RMI index: {:?}", e)
    };

    let linear_index = match LinearIndex::train(data) {
        Ok(index) => index,
        Err(e) => return format!("Failed to train Linear index: {:?}", e)
    };

    let num_queries = 1000;
    let test_keys: Vec<i64> = (0..num_queries).map(|i| (i % num_keys) as i64 * 2).collect();

    // Benchmark RMI
    let start = Instant::now();
    for &key in &test_keys {
        let _ = rmi_index.get(&key);
    }
    let rmi_time = start.elapsed();

    // Benchmark Linear
    let start = Instant::now();
    for &key in &test_keys {
        let _ = linear_index.get(&key);
    }
    let linear_time = start.elapsed();

    // Benchmark B-tree
    let start = Instant::now();
    for &key in &test_keys {
        let _ = btree.get(&key);
    }
    let btree_time = start.elapsed();

    let rmi_qps = num_queries as f64 / rmi_time.as_secs_f64();
    let linear_qps = num_queries as f64 / linear_time.as_secs_f64();
    let btree_qps = num_queries as f64 / btree_time.as_secs_f64();

    format!(
        "PostgreSQL RMI Benchmark ({} keys, {} queries):\n\
         RMI Index:    {:.0} q/s\n\
         Linear Index: {:.0} q/s\n\
         BTreeMap:     {:.0} q/s\n\
         RMI vs BTree: {:.2}x speedup\n\
         Linear vs BTree: {:.2}x speedup",
        num_keys, num_queries, rmi_qps, linear_qps, btree_qps,
        rmi_qps / btree_qps, linear_qps / btree_qps
    )
}

/// SQL function to demonstrate learned index performance
#[pg_extern]
fn hello_omendb() -> &'static str {
    "Hello from OmenDB Learned Index Extension!"
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_create_learned_index() {
        let result = crate::create_learned_index("test_idx", "test_table", "id");
        assert!(result.contains("created"));
    }

    #[pg_test]
    fn test_benchmark() {
        let result = crate::benchmark_learned_vs_btree(100);
        assert!(result.contains("Speedup"));
    }

    #[pg_test]
    fn test_rmi_benchmark() {
        let result = crate::benchmark_rmi_postgres(100);
        assert!(result.contains("speedup"));
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}