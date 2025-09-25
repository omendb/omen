# OmenDB Benchmarks

This directory contains performance benchmarks for OmenDB, the embedded vector database optimized for AI applications.

## 🚀 Quick Start

Run the consolidated benchmark suite:

```bash
pixi run python benchmarks/comprehensive.py
```

For quick benchmarks only:

```bash
pixi run python benchmarks/comprehensive.py --quick
```

## 📊 Consolidated Benchmark Suite

The main benchmark (`comprehensive.py`) covers:

### 1. Performance Overview
- Small (1K vectors, 128D)
- Medium (10K vectors, 128D)  
- Large (50K vectors, 128D)
- High-dimensional (10K vectors, 768D)

### 2. Dimension Scaling Analysis
- Tests dimensions: 32, 64, 128, 256, 384, 512, 768, 1024, 1536
- Measures single vs batch performance
- Analyzes SIMD alignment effects

### 3. Batch Optimization
- Batch sizes: 1, 10, 50, 100, 500, 1000
- Measures speedup vs single insertion
- Identifies optimal batch sizes

### 4. Framework Integration
- NumPy arrays (zero-copy)
- Python lists
- PyTorch tensors (if available)
- TensorFlow tensors (if available)

### 5. Memory Efficiency
- Memory overhead analysis
- Scaling with dataset size
- Memory usage patterns

## 🔬 Specialized Benchmarks

### SIMD Optimization Analysis
```bash
pixi run python benchmarks/simd_batch_comparison.py
```
Validates SIMD batch optimization assumptions and potential improvements.

### Dimension Boundary Analysis
```bash
pixi run python benchmarks/dimension_boundary_analysis.py
```
Deep dive into performance behavior at specific dimension boundaries.

### Real-World Embedding Test
```bash
pixi run python benchmarks/real_world_embedding_test.py
```
Tests with real embedding models (OpenAI, Cohere, etc.).

## 📈 Results

Results are saved to the `results/` directory:
- `results/latest.json` - Most recent benchmark results
- `results/LATEST_SUMMARY.md` - Human-readable summary
- `results/benchmark_YYYYMMDD_HHMMSS.json` - Timestamped results

## 🎯 Performance Targets

Current targets for v0.1.0:
- **Insertion**: 200K+ vectors/second (achieved: 210K on Fedora x86)
- **Query**: <1ms latency for 1M vectors
- **Memory**: <2x overhead vs raw vectors
- **Batch speedup**: 8-16x vs single insertion (current: 1.1x, needs work)

## 🔍 Comprehensive Competitor Benchmarks

For comprehensive competitor analysis:

```bash
# Full competitor benchmarks (requires dependencies)
pixi run python benchmarks/comprehensive_benchmark.py --config standard
```

Available configurations:
- `--config quick`: 2-3 minutes, basic validation
- `--config standard`: 10-15 minutes, comprehensive analysis
- `--config enterprise`: 30-60 minutes, production-scale testing

### 1. Install Dependencies

```bash
# Using Make (recommended)
make benchmark-deps

# Or using pixi directly
pixi install
```

### 2. Run Your First Benchmark

```bash
# Quick validation (2-3 minutes)
make benchmark-quick

# Or using pixi
pixi run benchmark-quick
```

### 3. View Results

```bash
# Analyze results
make benchmark-results

# Results are saved in benchmarks/results/
ls benchmarks/results/
```

## 📊 Benchmark Configurations

### Quick Benchmark
- **Scales**: 100, 1K vectors
- **Dimensions**: 128D, 384D
- **Time**: ~2-3 minutes
- **Memory**: ~2GB RAM
- **Use case**: Development validation, CI/CD

```bash
make benchmark-quick
pixi run benchmark-quick
```

### Standard Benchmark
- **Scales**: 1K, 5K, 10K vectors
- **Dimensions**: 128D, 384D, 768D, 1536D
- **Time**: ~10-15 minutes
- **Memory**: ~4GB RAM
- **Use case**: Comprehensive performance analysis

```bash
make benchmark-standard
pixi run benchmark-standard
```

### Enterprise Benchmark
- **Scales**: 10K, 50K, 100K, 500K vectors
- **Dimensions**: 384D, 768D, 1536D, 3072D
- **Time**: ~30-60 minutes
- **Memory**: ~8GB RAM
- **Use case**: Production-scale validation

```bash
make benchmark-enterprise
pixi run benchmark-enterprise
```

## 🔬 Advanced Features

### Real-World Embeddings
Tests realistic embedding patterns from production ML models:

- **Sentence-BERT**: 384D embeddings
- **BERT Base**: 768D embeddings  
- **OpenAI Ada-002**: 1536D embeddings
- **Large Language Models**: 3072D embeddings
- **CLIP Vision**: 768D embeddings
- **Word2Vec**: 300D embeddings

### Stress Testing
Comprehensive robustness validation:

- **Large-scale stress**: Memory pressure testing
- **Concurrent access**: Multi-threaded query patterns
- **Edge cases**: Zero vectors, extreme values, error handling
- **Memory pressure**: Multiple database instances

### Competitor Analysis
Direct performance comparison against:

- **ChromaDB**: Embedded vector database
- **Faiss**: Facebook AI Similarity Search
- **Qdrant**: High-performance vector database
- **Pinecone**: Managed vector database (API required)

## 📈 Results Analysis

### Automated Reporting
- **JSON results**: Machine-readable performance data
- **Human-readable summaries**: Clear performance comparisons
- **Competitive analysis**: Win/loss ratios and rankings
- **Performance trends**: Scaling characteristics

### Example Results Structure
```
benchmarks/results/
├── benchmark_results_20250718_143022.json    # Raw performance data
├── benchmark_summary_20250718_143022.txt     # Human-readable summary
└── ...
```

## 🎯 Specific Competitor Benchmarks

### OmenDB vs ChromaDB
```bash
make benchmark-chromadb
pixi run benchmark-chromadb
```

### OmenDB vs Faiss
```bash
make benchmark-faiss
pixi run benchmark-faiss
```

### OmenDB vs Qdrant
```bash
make benchmark-qdrant
pixi run benchmark-qdrant
```

### OmenDB Only
```bash
make benchmark-omendb
pixi run benchmark-omendb
```

## 🛠️ Customization

### Custom Configurations
```bash
# Custom benchmark with specific options
python benchmarks/comprehensive_benchmark.py \
    --config=standard \
    --competitors omendb chromadb \
    --no-stress \
    --output custom_results
```

### Available Options
- `--config`: quick, standard, enterprise
- `--competitors`: omendb, chromadb, faiss, qdrant, pinecone
- `--no-stress`: Skip stress tests
- `--no-embeddings`: Skip real-world embedding tests
- `--output`: Custom output directory

## 🔍 System Requirements

### Minimum Requirements
- **Python 3.11+**
- **4GB RAM** (8GB recommended)
- **2GB disk space**
- **macOS or Linux** (Windows support coming)

### Check Your System
```bash
make benchmark-check
pixi run benchmark-check
```

## 📋 Troubleshooting

### Common Issues

#### Missing Dependencies
```bash
# Install all benchmark dependencies
make benchmark-deps
```

#### Memory Issues
```bash
# Use quick benchmark for limited memory
make benchmark-quick
```

#### Competitor Libraries
```bash
# Check which competitors are available
pixi run benchmark-check
```

### Performance Expectations

#### Quick Benchmark
- **1K vectors**: ~30 seconds
- **Memory usage**: ~1GB
- **Accuracy**: Full validation

#### Standard Benchmark
- **10K vectors**: ~2-3 minutes
- **Memory usage**: ~2-4GB
- **Accuracy**: Production-level validation

#### Enterprise Benchmark
- **100K vectors**: ~10-15 minutes
- **Memory usage**: ~4-8GB
- **Accuracy**: Enterprise-scale validation

## 📊 Understanding Results

### Performance Metrics
- **Throughput**: Vectors/second for insertions
- **Latency**: Milliseconds per query
- **Memory Usage**: MB consumed during operations
- **Accuracy**: Similarity search correctness

### Competitive Analysis
- **Win Rate**: Percentage of tests where OmenDB performs better
- **Performance Ranking**: 🥇🥈🥉 medals for different scenarios
- **Scaling Characteristics**: How performance changes with scale

### Result Interpretation
```text
📊 PERFORMANCE LEADERBOARD
🥇 OmenDB: 0.38ms query, 4,200 v/s insert
🥈 ChromaDB: 0.42ms query, 3,800 v/s insert
🥉 Faiss: 0.15ms query, 15,000 v/s insert

⚔️ COMPETITIVE ANALYSIS
🏆 vs ChromaDB: 67% win rate (6/9 tests)
🤝 vs Faiss: 33% win rate (3/9 tests)
```

## 🚀 Integration

### CI/CD Integration
```yaml
# GitHub Actions example
- name: Run Quick Benchmark
  run: make benchmark-quick
  
- name: Upload Results
  uses: actions/upload-artifact@v3
  with:
    name: benchmark-results
    path: benchmarks/results/
```

### Automated Monitoring
```bash
# Schedule regular benchmarks
crontab -e
0 2 * * * cd /path/to/omendb && make benchmark-standard
```

## 📚 Architecture

### Benchmark Structure
```
benchmarks/
├── comprehensive_benchmark.py    # Main benchmark system
├── results/                      # Output directory
├── PERFORMANCE_ANALYSIS.md      # Existing analysis
└── README.md                     # This file
```

### Key Components
1. **ComprehensiveBenchmark**: Main orchestrator
2. **Database Adapters**: Competitor implementations
3. **Test Configurations**: Scale and dimension matrices
4. **Result Analysis**: Performance comparison engine
5. **Automation**: Make and pixi integration

## 🎯 Next Steps

1. **Run your first benchmark**: `make benchmark-quick`
2. **Analyze results**: `make benchmark-results`
3. **Compare competitors**: `make benchmark-chromadb`
4. **Scale testing**: `make benchmark-enterprise`
5. **Integrate with CI**: Add to your automation pipeline

## 🤝 Contributing

### Adding New Competitors
1. Create new benchmark class in `comprehensive_benchmark.py`
2. Add to `_get_available_databases()` method
3. Add dependency to `pixi.toml`
4. Test with `make benchmark-check`

### Reporting Issues
- Performance discrepancies
- Missing competitor support
- System compatibility issues
- Result interpretation questions

---

**Ready to benchmark?** Start with `make benchmark-quick` and see how OmenDB performs against the competition!