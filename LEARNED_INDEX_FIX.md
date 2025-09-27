# Why Our Learned Indexes Failed (And How to Fix Them)

## The Problem with Our Implementation

Our tests showed learned indexes 8-14x SLOWER because we made critical mistakes:

1. **Wrong Training Target**: We trained on existence (true/false) not positions
2. **No Error Bounds**: Real learned indexes use bounded search after prediction
3. **No Proper Updates**: Missing gapped arrays or delta buffers
4. **Wrong Workload**: Testing random keys instead of sequential/clustered

## What Actually Works (From Research)

### LearnedKV (2024) - 4.32x Speedup
- **Two-tier design**: LSM for writes, Learned Index for reads
- **Non-blocking conversion**: Build index during GC
- **Greedy-PLR+ model**: Piecewise linear with bounded error
- **Page-aware**: Optimize for SSD page boundaries

### BLI (2025) - 2.21x Speedup
- **Globally sorted, locally unsorted**: Buckets instead of arrays
- **Hint functions**: Suggest optimal slot placement
- **SPMC concurrency**: One writer, many readers
- **RCU for structure mods**: Lock-free updates

### ALEX (2020) - Microsoft's Solution
- **Gapped arrays**: Leave space for inserts (like B+ tree)
- **Adaptive nodes**: Switch between models based on data
- **Delta buffers**: Accumulate updates, merge periodically
- **Exponential search**: Better than binary for small errors

## The Fixed Implementation

```python
class ProperLearnedIndex:
    def __init__(self, error_bound=100):
        self.model = None
        self.data = []  # Sorted array
        self.error_bound = error_bound
        self.delta_buffer = []  # For updates

    def train(self, keys):
        """Train on POSITIONS not existence"""
        self.data = sorted(keys)
        X = np.array(self.data).reshape(-1, 1)
        y = np.arange(len(self.data))  # POSITIONS!
        self.model = LinearRegression().fit(X, y)

    def search(self, key):
        """Predict position then bounded search"""
        # 1. Predict position
        pred = int(self.model.predict([[key]])[0])

        # 2. Bound the prediction
        start = max(0, pred - self.error_bound)
        end = min(len(self.data), pred + self.error_bound)

        # 3. Exponential search (better than binary)
        return self._exponential_search(key, start, end)

    def _exponential_search(self, key, start, end):
        """Faster than binary for small ranges"""
        # Start from predicted position
        i = start
        bound = 1

        # Exponentially increase search range
        while i < end and self.data[i] < key:
            i += bound
            bound *= 2

        # Binary search in final range
        left = max(start, i // 2)
        right = min(i, end - 1)
        return binary_search(self.data, key, left, right)
```

## Why Learned Indexes Can Work

### When They Win
1. **Sequential data** (timestamps, IDs)
2. **Clustered patterns** (hot/cold, zipfian)
3. **Read-heavy workloads** (90%+ reads)
4. **Known distributions** (can model CDF)

### When They Lose
1. **Random keys** (no pattern to learn)
2. **Write-heavy** (constant retraining)
3. **Small datasets** (B-tree overhead minimal)
4. **Adversarial patterns** (designed to break model)

## Implementation Fixes Needed

### 1. Fix Training Target
```rust
// WRONG - training on hints
let training_data: Vec<(i64, bool)> = keys.map(|k| (k, true));

// RIGHT - training on positions
let training_data: Vec<(i64, usize)> = keys
    .enumerate()
    .map(|(pos, key)| (key, pos));
```

### 2. Add Error Bounds
```rust
// Always search within bounds after prediction
let predicted = model.predict(key);
let start = max(0, predicted - ERROR_BOUND);
let end = min(data.len(), predicted + ERROR_BOUND);
let result = exponential_search(&data[start..end], key);
```

### 3. Implement Gapped Arrays
```rust
// Leave space for inserts (ALEX approach)
struct GappedArray<T> {
    data: Vec<Option<T>>,
    capacity_left: Vec<usize>,  // Space before each element
    capacity_right: Vec<usize>, // Space after each element
}
```

### 4. Add Delta Buffer
```rust
// Accumulate updates (LearnedKV approach)
struct DeltaBuffer {
    updates: BTreeMap<Key, Value>,
    max_size: usize,
}

// Query checks delta first, then learned index
fn get(&self, key: Key) -> Option<Value> {
    self.delta_buffer.get(key)
        .or_else(|| self.learned_index.get(key))
}
```

## Proper Benchmarking

### Use SOSD Framework
```bash
# Clone standard benchmark
git clone https://github.com/learnedsystems/SOSD
cd SOSD && ./scripts/download.sh

# Test with real datasets
./build/benchmark \
    --keys data/books_200M \
    --queries data/books_200M_queries \
    --index learned
```

### Test Right Workloads
1. **Sequential**: Timestamps, auto-increment IDs
2. **Zipfian**: 80% queries on 20% keys
3. **Normal**: Gaussian distribution
4. **Books/Web**: Real-world from SOSD

## Next Steps

1. Fix our `learneddb` implementation with proper position training
2. Add error bounds and exponential search
3. Implement gapped arrays for updates
4. Test on sequential/clustered data (not random)
5. Use SOSD benchmarks for fair comparison

The research shows learned indexes CAN work - we just implemented them wrong.