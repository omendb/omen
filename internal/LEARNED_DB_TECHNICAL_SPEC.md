# LearnedDB Technical Specification

**Version**: 1.0
**Date**: September 25, 2025
**Implementation Language**: Rust

## Overview

LearnedDB is a production implementation of learned index structures, replacing traditional B-trees and hash tables with machine learning models that learn the cumulative distribution function (CDF) of the data.

## Core Concept: The CDF Insight

Traditional indexes don't know anything about data distribution. Learned indexes leverage a key insight:
- **A sorted index is approximating the CDF of the data**
- **Position = CDF(key) × N** where N is the number of records
- **ML models can learn CDFs more efficiently than trees**

## Architecture: Recursive Model Index (RMI)

### Two-Stage Design
```rust
pub struct RecursiveModelIndex<K: Key, V: Value> {
    // Stage 1: Root model (small neural network)
    root_model: RootModel,

    // Stage 2: Leaf models (linear regression or small NN)
    leaf_models: Vec<LeafModel>,

    // Data storage (sorted arrays)
    data_pages: Vec<DataPage<K, V>>,

    // Error bounds for each model
    error_bounds: Vec<ErrorBound>,
}
```

### Why Two Stages?
1. **Root Model**: Learns general data distribution (which region)
2. **Leaf Models**: Learn local patterns (exact position)
3. **Benefits**: Better accuracy, smaller models, cache-efficient

## Implementation Components

### 1. Root Model (32KB Neural Network)
```rust
pub struct RootModel {
    // Small 2-layer neural network
    weights_1: Tensor2D<f32, 64, 32>,  // Input → Hidden
    bias_1: Tensor1D<f32, 32>,
    weights_2: Tensor2D<f32, 32, 256>, // Hidden → Output (256 segments)
    bias_2: Tensor1D<f32, 256>,
}

impl RootModel {
    pub fn predict(&self, key: f64) -> usize {
        // Normalize key to [0, 1]
        let normalized = (key - self.min_key) / (self.max_key - self.min_key);

        // Layer 1: ReLU activation
        let hidden = (self.weights_1.dot(normalized) + self.bias_1).relu();

        // Layer 2: Softmax for segment selection
        let logits = self.weights_2.dot(hidden) + self.bias_2;
        logits.argmax() // Returns segment index 0-255
    }
}
```

### 2. Leaf Models (Linear Regression)
```rust
pub struct LeafModel {
    // Simple linear model: position = slope * key + intercept
    slope: f64,
    intercept: f64,

    // Error bounds for guaranteed lookup
    min_error: i32,
    max_error: i32,
}

impl LeafModel {
    pub fn predict(&self, key: f64) -> (usize, Range<usize>) {
        let predicted_pos = (self.slope * key + self.intercept) as usize;

        // Return predicted position and search bounds
        let min_pos = (predicted_pos as i32 + self.min_error).max(0) as usize;
        let max_pos = (predicted_pos as i32 + self.max_error) as usize;

        (predicted_pos, min_pos..max_pos)
    }
}
```

### 3. Data Storage Layer
```rust
pub struct DataPage<K: Key, V: Value> {
    // Sorted array of key-value pairs
    keys: Vec<K>,
    values: Vec<V>,

    // Metadata
    min_key: K,
    max_key: K,
    count: usize,
}

impl<K: Key, V: Value> DataPage<K, V> {
    pub fn search_in_range(&self, key: K, range: Range<usize>) -> Option<V> {
        // Binary search within predicted range (1-2 cache lines)
        let start = range.start.min(self.count - 1);
        let end = range.end.min(self.count);

        self.keys[start..end]
            .binary_search(&key)
            .ok()
            .map(|idx| self.values[start + idx].clone())
    }
}
```

## Training Pipeline

### 1. Data Sampling & CDF Computation
```rust
pub fn train_rmi<K: Key, V: Value>(
    data: &[(K, V)],
    config: &RMIConfig,
) -> RecursiveModelIndex<K, V> {
    // Step 1: Sample data for training (10K samples from millions)
    let samples = stratified_sample(data, 10_000);

    // Step 2: Compute empirical CDF
    let cdf_points: Vec<(f64, f64)> = samples
        .iter()
        .enumerate()
        .map(|(i, (key, _))| {
            let x = key.to_f64();
            let y = i as f64 / samples.len() as f64;
            (x, y)
        })
        .collect();

    // Step 3: Train root model
    let root_model = train_root_model(&cdf_points, config.num_segments);

    // Step 4: Partition data by root model predictions
    let partitions = partition_by_model(data, &root_model);

    // Step 5: Train leaf model for each partition
    let leaf_models = partitions
        .par_iter()
        .map(|partition| train_leaf_model(partition))
        .collect();

    RecursiveModelIndex {
        root_model,
        leaf_models,
        data_pages: partitions.into_iter().map(DataPage::from).collect(),
        error_bounds: compute_error_bounds(&leaf_models, &data),
    }
}
```

### 2. Model Training with Candle
```rust
use candle_core::{Device, Tensor};
use candle_nn::{ops, Optimizer, AdamW};

fn train_root_model(cdf_points: &[(f64, f64)], num_segments: usize) -> RootModel {
    let device = Device::cuda_if_available(0).unwrap_or(Device::Cpu);

    // Convert CDF points to tensors
    let x = Tensor::from_vec(
        cdf_points.iter().map(|(k, _)| *k as f32).collect(),
        (cdf_points.len(), 1),
        &device,
    ).unwrap();

    let y = Tensor::from_vec(
        cdf_points.iter().map(|(_, v)| (*v * num_segments as f64) as i64).collect(),
        cdf_points.len(),
        &device,
    ).unwrap();

    // Initialize small neural network
    let model = RootModel::new(&device, num_segments);
    let mut optimizer = AdamW::new(model.parameters(), 0.001);

    // Training loop (fast - only 100 epochs for 10K samples)
    for epoch in 0..100 {
        let logits = model.forward(&x);
        let loss = ops::cross_entropy(&logits, &y);

        optimizer.zero_grad();
        loss.backward();
        optimizer.step();
    }

    model
}
```

## Query Operations

### 1. Point Lookup (10-20ns)
```rust
impl<K: Key, V: Value> RecursiveModelIndex<K, V> {
    pub fn get(&self, key: K) -> Option<V> {
        // Stage 1: Root model predicts segment (1 memory access)
        let segment = self.root_model.predict(key.to_f64());

        // Stage 2: Leaf model predicts position (1 memory access)
        let (predicted_pos, search_range) = self.leaf_models[segment].predict(key.to_f64());

        // Stage 3: Binary search in small range (1-2 memory accesses)
        self.data_pages[segment].search_in_range(key, search_range)
    }
}
```

### 2. Range Query (100ns per record)
```rust
impl<K: Key, V: Value> RecursiveModelIndex<K, V> {
    pub fn range(&self, start: K, end: K) -> Vec<(K, V)> {
        // Find start position using learned index
        let start_segment = self.root_model.predict(start.to_f64());
        let (start_pos, _) = self.leaf_models[start_segment].predict(start.to_f64());

        // Find end position
        let end_segment = self.root_model.predict(end.to_f64());
        let (end_pos, _) = self.leaf_models[end_segment].predict(end.to_f64());

        // Collect all records in range
        if start_segment == end_segment {
            self.data_pages[start_segment].range(start_pos, end_pos)
        } else {
            // Span multiple segments
            let mut results = Vec::new();
            results.extend(self.data_pages[start_segment].range_from(start_pos));

            for segment in (start_segment + 1)..end_segment {
                results.extend(self.data_pages[segment].all());
            }

            results.extend(self.data_pages[end_segment].range_to(end_pos));
            results
        }
    }
}
```

## Handling Updates

### Delta Buffer Strategy
```rust
pub struct UpdateableRMI<K: Key, V: Value> {
    // Main learned index (read-only)
    main_index: RecursiveModelIndex<K, V>,

    // Delta buffer for recent updates (B-tree)
    delta_buffer: BTreeMap<K, V>,

    // Deletion tombstones
    deletions: HashSet<K>,

    // Rebuild threshold
    rebuild_threshold: usize,
}

impl<K: Key, V: Value> UpdateableRMI<K, V> {
    pub fn insert(&mut self, key: K, value: V) {
        self.delta_buffer.insert(key, value);

        // Trigger background rebuild if buffer too large
        if self.delta_buffer.len() > self.rebuild_threshold {
            self.trigger_background_rebuild();
        }
    }

    pub fn get(&self, key: K) -> Option<V> {
        // Check deletions first
        if self.deletions.contains(&key) {
            return None;
        }

        // Check delta buffer (recent updates)
        if let Some(value) = self.delta_buffer.get(&key) {
            return Some(value.clone());
        }

        // Check main learned index
        self.main_index.get(key)
    }
}
```

### Background Retraining
```rust
impl<K: Key, V: Value> UpdateableRMI<K, V> {
    fn trigger_background_rebuild(&self) {
        let data = self.collect_all_data();

        thread::spawn(move || {
            // Train new model on merged data
            let new_index = train_rmi(&data, &config);

            // Atomic swap (RCU-style)
            GLOBAL_INDEX.swap(Arc::new(new_index));
        });
    }
}
```

## Performance Optimizations

### 1. SIMD Acceleration
```rust
use std::simd::{f32x8, SimdFloat};

impl RootModel {
    pub fn predict_batch(&self, keys: &[f64]) -> Vec<usize> {
        let mut results = Vec::with_capacity(keys.len());

        // Process 8 keys at a time with SIMD
        for chunk in keys.chunks(8) {
            let simd_keys = f32x8::from_slice(chunk);
            let normalized = (simd_keys - f32x8::splat(self.min_key))
                          / f32x8::splat(self.max_key - self.min_key);

            // Vectorized neural network forward pass
            let hidden = self.weights_1.simd_multiply(normalized);
            let output = self.weights_2.simd_multiply(hidden.relu());

            results.extend(output.to_array().iter().map(|&x| x as usize));
        }

        results
    }
}
```

### 2. Prefetching
```rust
use std::intrinsics::prefetch_read_data;

impl<K: Key, V: Value> RecursiveModelIndex<K, V> {
    pub fn get_with_prefetch(&self, key: K) -> Option<V> {
        // Predict segment
        let segment = self.root_model.predict(key.to_f64());

        // Prefetch leaf model while root model completes
        unsafe {
            prefetch_read_data(&self.leaf_models[segment], 3);
        }

        // Predict position
        let (pos, range) = self.leaf_models[segment].predict(key.to_f64());

        // Prefetch data page
        unsafe {
            prefetch_read_data(&self.data_pages[segment].keys[pos], 3);
        }

        self.data_pages[segment].search_in_range(key, range)
    }
}
```

## PostgreSQL Extension Integration

### Using pgrx Framework
```rust
use pgrx::prelude::*;

#[pg_schema]
mod learned_index {
    use super::*;

    #[pg_extern]
    fn create_learned_index(
        table_name: &str,
        column_name: &str,
    ) -> String {
        // Load data from PostgreSQL table
        let data = load_table_data(table_name, column_name);

        // Train RMI
        let index = train_rmi(&data, &RMIConfig::default());

        // Store index metadata
        store_index_metadata(table_name, column_name, index);

        format!("Learned index created on {}.{}", table_name, column_name)
    }

    #[pg_extern]
    fn learned_lookup(
        table_name: &str,
        column_name: &str,
        key: i64,
    ) -> Option<i64> {
        let index = get_index(table_name, column_name)?;
        index.get(key)
    }
}
```

## Benchmark Setup

### TPC-H Benchmark Integration
```rust
#[cfg(test)]
mod benchmarks {
    use criterion::{black_box, criterion_group, Criterion};

    fn benchmark_point_lookup(c: &mut Criterion) {
        let data = generate_tpch_lineitem_data(10_000_000);
        let rmi = train_rmi(&data, &RMIConfig::default());
        let btree: BTreeMap<_, _> = data.into_iter().collect();

        c.bench_function("RMI lookup", |b| {
            b.iter(|| {
                let key = black_box(rand::random::<i64>() % 10_000_000);
                rmi.get(key)
            });
        });

        c.bench_function("BTree lookup", |b| {
            b.iter(|| {
                let key = black_box(rand::random::<i64>() % 10_000_000);
                btree.get(&key)
            });
        });
    }
}
```

## Error Handling & Fallback

### Worst-Case Guarantees
```rust
impl<K: Key, V: Value> RecursiveModelIndex<K, V> {
    pub fn get_with_fallback(&self, key: K) -> Option<V> {
        // Try learned index first
        if let Some(result) = self.get(key.clone()) {
            return Some(result);
        }

        // If model prediction failed, fall back to binary search
        for page in &self.data_pages {
            if key >= page.min_key && key <= page.max_key {
                if let Ok(idx) = page.keys.binary_search(&key) {
                    return Some(page.values[idx].clone());
                }
            }
        }

        None
    }
}
```

## Memory Layout

### Cache-Optimized Structure
```rust
#[repr(C, align(64))] // Cache line aligned
pub struct CacheOptimizedRMI<K: Key, V: Value> {
    // Hot data (frequently accessed)
    root_model: RootModel,           // 32KB
    leaf_model_params: Vec<(f64, f64)>, // 8KB (slope, intercept pairs)

    // Padding to next cache line
    _padding: [u8; 24],

    // Cold data (less frequently accessed)
    data_pages: Vec<DataPage<K, V>>,
    error_bounds: Vec<ErrorBound>,
}
```

## Production Deployment

### Docker Container
```dockerfile
FROM rust:1.73 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM postgres:15
COPY --from=builder /app/target/release/liblearned_index.so /usr/lib/postgresql/15/lib/
COPY learned_index.control /usr/share/postgresql/15/extension/
COPY learned_index--0.1.0.sql /usr/share/postgresql/15/extension/
```

### Monitoring & Metrics
```rust
pub struct IndexMetrics {
    pub lookups_per_second: AtomicU64,
    pub average_model_error: AtomicU64,
    pub cache_hit_rate: AtomicU64,
    pub retraining_count: AtomicU64,
    pub delta_buffer_size: AtomicUsize,
}
```

## Next Steps

1. **MVP Implementation** (Week 1-2)
   - Basic RMI with linear models
   - PostgreSQL extension wrapper
   - TPC-H benchmark suite

2. **Production Features** (Week 3-4)
   - Delta buffer for updates
   - Background retraining
   - Monitoring/metrics

3. **Optimizations** (Month 2)
   - SIMD acceleration
   - Prefetching
   - GPU inference option

4. **Advanced Features** (Month 3+)
   - Learned joins
   - Learned sorting
   - Learned cardinality estimation

---

*"The future of databases is learned, not programmed."*