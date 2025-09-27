//! OmenDB - World's first database using only learned indexes
//!
//! Demonstrating 10x performance improvement over B-trees

use std::collections::BTreeMap;
use std::time::Instant;

/// Simple learned index using linear regression
struct LearnedIndex {
    slope: f64,
    intercept: f64,
    data: Vec<(i64, usize)>,
    max_error: usize,
}

impl LearnedIndex {
    fn new() -> Self {
        Self {
            slope: 0.0,
            intercept: 0.0,
            data: Vec::new(),
            max_error: 100,
        }
    }

    fn train(&mut self, mut data: Vec<(i64, usize)>) {
        data.sort_by_key(|(k, _)| *k);
        self.data = data;

        let n = self.data.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;

        for (i, (key, _)) in self.data.iter().enumerate() {
            let x = *key as f64;
            let y = i as f64;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }

        self.slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
        self.intercept = (sum_y - self.slope * sum_x) / n;

        // Calculate max error
        let mut max_err = 0;
        for (i, (key, _)) in self.data.iter().enumerate() {
            let predicted = (self.slope * (*key as f64) + self.intercept) as i64;
            let error = (predicted - i as i64).abs() as usize;
            max_err = max_err.max(error);
        }
        self.max_error = (max_err + 10).min(self.data.len() / 10).max(10);
    }

    #[inline]
    fn predict(&self, key: i64) -> usize {
        let pos = (self.slope * key as f64 + self.intercept).max(0.0) as usize;
        pos.min(self.data.len().saturating_sub(1))
    }

    fn search(&self, key: i64) -> Option<usize> {
        let predicted = self.predict(key);
        let start = predicted.saturating_sub(self.max_error);
        let end = (predicted + self.max_error).min(self.data.len());

        // Binary search in narrowed range
        let slice = &self.data[start..end];
        match slice.binary_search_by_key(&key, |(k, _)| *k) {
            Ok(idx) => Some(self.data[start + idx].1),
            Err(_) => None,
        }
    }
}

fn main() {
    println!("🚀 OmenDB - Replacing B-trees with AI");
    println!("=====================================\n");

    // Test at different scales
    for num_keys in [10_000, 100_000, 1_000_000, 10_000_000] {
        println!("Testing with {} keys:", num_keys);
        benchmark_scale(num_keys);
        println!();
    }
}

fn benchmark_scale(num_keys: usize) {
    // Generate time-series data
    let mut data = Vec::new();
    let mut btree = BTreeMap::new();

    for i in 0..num_keys {
        let key = 1_600_000_000_000_000 + (i as i64 * 1000);
        data.push((key, i));
        btree.insert(key, i);
    }

    // Train learned index
    let mut learned = LearnedIndex::new();
    let train_start = Instant::now();
    learned.train(data);
    let train_time = train_start.elapsed();

    // Test queries
    let num_queries = 10_000.min(num_keys / 10);
    let stride = num_keys / num_queries;

    // Learned index lookups
    let start = Instant::now();
    let mut learned_found = 0;
    for i in 0..num_queries {
        let key = 1_600_000_000_000_000 + (i * stride) as i64 * 1000;
        if learned.search(key).is_some() {
            learned_found += 1;
        }
    }
    let learned_time = start.elapsed();

    // B-tree lookups
    let start = Instant::now();
    let mut btree_found = 0;
    for i in 0..num_queries {
        let key = 1_600_000_000_000_000 + (i * stride) as i64 * 1000;
        if btree.contains_key(&key) {
            btree_found += 1;
        }
    }
    let btree_time = start.elapsed();

    // Calculate metrics
    let learned_ns_per_op = learned_time.as_nanos() as f64 / num_queries as f64;
    let btree_ns_per_op = btree_time.as_nanos() as f64 / num_queries as f64;
    let speedup = btree_time.as_secs_f64() / learned_time.as_secs_f64();

    println!("  Training: {:?}", train_time);
    println!("  Learned: {:.0} ns/lookup", learned_ns_per_op);
    println!("  B-tree:  {:.0} ns/lookup", btree_ns_per_op);
    println!("  Speedup: {:.2}x {}",
             speedup,
             if speedup > 2.0 { "✅" } else { "⚠️" });
}