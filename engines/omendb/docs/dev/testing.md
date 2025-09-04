# Testing Strategy

**Comprehensive testing approach for OmenDB development**

## 🎯 Testing Philosophy

**Core Principles:**
- **Performance First**: Every change must maintain or improve performance
- **Cross-Platform**: Test on macOS, Linux, Windows
- **Real-World**: Use actual embedding patterns and data
- **Regression Prevention**: Catch performance and functionality regressions

## 📋 Test Categories

### 1. Unit Tests
**Location**: `test/unit/`  
**Purpose**: Test individual components in isolation

```bash
# Run unit tests
pixi run python -m pytest test/unit/ -v
```

**Coverage Areas:**
- Vector operations
- Similarity calculations
- Data validation
- Error handling
- Storage operations

### 2. Integration Tests
**Location**: `test/integration/`  
**Purpose**: Test component interactions

```bash
# Run integration tests
pixi run python -m pytest test/integration/ -v
```

**Coverage Areas:**
- Python-Mojo interface
- Database persistence
- Query processing pipeline
- Memory management
- Cross-platform compatibility

### 3. Performance Tests
**Location**: `benchmarks/`  
**Purpose**: Validate performance characteristics

```bash
# Run performance benchmarks
pixi run python benchmarks/comprehensive_benchmark.py
pixi run python benchmarks/omendb_benchmark.py
pixi run python benchmarks/real_world_embedding_test.py
```

**Coverage Areas:**
- Query latency
- Insert throughput
- Memory usage
- Scaling characteristics
- Competitive comparisons

### 4. Native Module Tests
**Location**: `test/native_module/`  
**Purpose**: Test Mojo native implementation

```bash
# Test native module compilation
pixi run mojo -I omendb omendb/native.mojo

# Test RoarGraph integration
pixi run mojo test/native_module/test_roargraph.mojo
```

## 🚀 Performance Regression Testing

### Continuous Benchmarking
**Frequency**: Every commit  
**Metrics**: Query time, insert rate, memory usage

```python
# Performance thresholds (fail if exceeded)
MAX_QUERY_TIME_MS = 1.0        # 1ms max query time
MIN_INSERT_RATE = 1000         # 1K vectors/sec minimum
MAX_MEMORY_MB_PER_1K = 10      # 10MB max for 1K vectors

# Regression detection
if current_performance > baseline * 1.2:  # 20% slowdown
    raise PerformanceRegression("Performance degraded significantly")
```

### Benchmark Validation
- **Compare vs competitors**: ChromaDB, Faiss baseline performance
- **Cross-platform**: Same benchmarks on macOS, Linux, Windows
- **Scale testing**: 1K, 10K, 100K vector performance
- **Memory profiling**: Track memory usage under load

## 🔧 Development Testing Workflow

### Pre-Commit Testing
```bash
# Format code
pixi run mojo format omendb/ test/

# Run unit tests
pixi run python -m pytest test/unit/ -x

# Basic performance check
pixi run python -c "from python.omendb import DB; db = DB(); db.add('test', [1,2,3]); print('✅ Basic functionality working')"
```

### Feature Development Testing
```bash
# Full test suite
pixi run python -m pytest test/ -v

# Performance validation
pixi run python benchmarks/omendb_benchmark.py

# Memory leak check
pixi run python test/memory_leak_test.py
```

### Release Testing
```bash
# Complete test suite
pixi run python -m pytest test/ --tb=short

# Full benchmarks
pixi run python benchmarks/comprehensive_benchmark.py

# Cross-platform validation (CI/CD)
# - macOS (local)
# - Linux (GitHub Actions)
# - Windows (GitHub Actions)
```

## 📊 Test Data Strategy

### Synthetic Data
**Purpose**: Controlled, reproducible testing
```python
# Generate consistent test vectors
np.random.seed(42)
vectors = np.random.normal(0, 1, (1000, 384)).astype(np.float32)
```

### Real-World Embeddings
**Purpose**: Validate with actual ML model outputs
```python
# Simulate different embedding patterns
openai_embeddings = generate_realistic_embeddings(1000, 1536, "openai")
sentence_bert = generate_realistic_embeddings(1000, 384, "sentence_bert")
word2vec = generate_realistic_embeddings(1000, 300, "word2vec")
```

### Stress Testing Data
**Purpose**: Test limits and edge cases
```python
# Large scale
large_scale = generate_vectors(100000, 128)

# High dimensions
high_dim = generate_vectors(1000, 2048)

# Edge cases
empty_vectors = []
single_vector = [[1.0]]
zero_vectors = [[0.0] * 128]
```

## 🎯 Quality Gates

### Performance Gates
- **Query latency**: < 1ms for 10K vectors
- **Insert throughput**: > 1K vectors/second
- **Memory usage**: < 50MB for 10K vectors
- **Competitive**: Within 10x of Faiss performance

### Functionality Gates
- **API compatibility**: All existing APIs work
- **Data integrity**: Save/load preserves all data
- **Error handling**: Graceful failure for invalid inputs
- **Cross-platform**: Basic functionality on all platforms

### Code Quality Gates
- **Test coverage**: > 90% line coverage
- **Documentation**: All public APIs documented
- **Performance**: No regressions > 20%
- **Memory**: No memory leaks detected

## 🐛 Testing Edge Cases

### Data Edge Cases
```python
# Empty database
db = DB()
results = db.search([1, 2, 3])  # Should return empty

# Invalid dimensions
db.add("test", [1, 2, 3])
db.search([1, 2])  # Should raise ValidationError

# Large vectors
huge_vector = [1.0] * 10000
db.add("huge", huge_vector)  # Should handle gracefully
```

### Performance Edge Cases
```python
# Many small operations
for i in range(10000):
    db.add(f"doc_{i}", [random.random() for _ in range(3)])

# Few large operations  
batch_vectors = [[random.random() for _ in range(1536)] for _ in range(1000)]
for i, vec in enumerate(batch_vectors):
    db.add(f"batch_{i}", vec)

# High-frequency queries
for _ in range(1000):
    db.search([random.random() for _ in range(384)])
```

### Error Conditions
```python
# Disk full simulation
# Network interruption simulation
# Memory pressure testing
# Concurrent access testing
```

## 🔍 Test Analysis & Reporting

### Performance Reports
```python
# Generate performance report
results = {
    'query_latency': measure_query_performance(),
    'insert_throughput': measure_insert_performance(), 
    'memory_usage': measure_memory_usage(),
    'competitive_analysis': compare_with_competitors()
}

generate_performance_report(results)
```

### Coverage Reports
```bash
# Generate test coverage report
pixi run python -m pytest test/ --cov=omendb --cov-report=html
```

### Benchmark History
```python
# Track performance over time
benchmark_history = {
    'commit': git_commit_hash,
    'date': datetime.now(),
    'query_time_ms': 0.234,
    'insert_rate_vs': 2150,
    'memory_mb': 23.4
}

save_benchmark_history(benchmark_history)
```

## 🚨 Testing Anti-Patterns

### ❌ Avoid These Patterns

```python
# Don't test implementation details
assert db._internal_index.node_count == 1000  # ❌ Internal detail

# Don't use unrealistic data
test_vector = [1, 2, 3]  # ❌ Too simple

# Don't ignore performance
time.sleep(10)  # ❌ Hide performance issues

# Don't hardcode values
assert result.similarity == 0.9876543  # ❌ Brittle exact match
```

### ✅ Better Approaches

```python
# Test public behavior
results = db.search(vector, limit=10)
assert len(results) <= 10  # ✅ Public contract

# Use realistic data
test_vector = generate_realistic_embedding()  # ✅ Real patterns

# Measure performance
with timer() as get_time:
    db.search(vector)
assert get_time() < 1.0  # ✅ Performance requirement

# Use ranges for floats
assert 0.8 < result.similarity < 1.0  # ✅ Reasonable range
```

## 📈 Future Testing Improvements

### Planned Enhancements
- **Property-based testing**: Generate test cases automatically
- **Mutation testing**: Verify test quality
- **Distributed testing**: Multi-node test scenarios
- **Load testing**: Sustained performance under load
- **Chaos testing**: Failure injection and recovery

### Automation Improvements
- **Continuous benchmarking**: Automated performance tracking
- **Cross-platform CI**: Automated multi-platform testing
- **Performance alerts**: Automatic alerts for regressions
- **Test generation**: AI-generated test cases

This testing strategy ensures OmenDB maintains high quality and performance while enabling rapid development and confident releases.