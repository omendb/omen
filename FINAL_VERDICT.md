# Final Verdict: Learned Indexes Don't Work

## Executive Summary

**Date**: September 26, 2025
**Verdict**: STOP the learned index approach
**Reality**: Learned indexes are 8-14x SLOWER than B-trees, not faster

## Test Results

### Proof of Concept (General Workloads)
- SQLite B-tree: Baseline
- Learned Index: **10x SLOWER**
- Python Dict: 36-47x FASTER

### Sequential/Time-Series Test
- Binary Search: Baseline
- Learned Index: **20-50x SLOWER**
- Even on perfectly sequential data

### Hot/Cold Architecture Test
- Binary Search: Baseline
- Hot/Cold Learned: **8-14x SLOWER**
- Even with 90% hot hit rate

## Why The Claims Were Wrong

### The "1.4x Speedup" Claim
Likely measurement errors:
1. Compared against wrong baseline (maybe unsorted data)
2. Measured build time not query time
3. Used synthetic micro-benchmark not real workload
4. Confused throughput units (ops/sec vs queries/sec)

### The "41M queries/sec" Claim
Completely unrealistic:
- State-of-the-art is 3-7M QPS (DragonflyDB, RocksDB)
- Our actual performance: ~80K QPS (500x slower than claimed)
- Binary search achieves 1M QPS easily
- Hash tables achieve 25M QPS

## Why Learned Indexes Fail in Practice

### Fundamental Issues
1. **Prediction Overhead**: Model inference costs more than binary search
2. **Cache Misses**: Random memory access patterns
3. **Error Correction**: Need to search anyway when prediction is wrong
4. **Memory Layout**: B-trees are cache-optimized, arrays aren't always

### When They Might Work (Theory)
Research suggests learned indexes only help when:
- Data is perfectly sequential
- No updates (static dataset)
- Can use GPU for batch predictions
- Range queries dominate (not point lookups)

### Why They Don't Work (Practice)
- Real data is never perfectly sequential
- Updates destroy learned patterns
- GPU overhead negates benefits
- Hash tables dominate point lookups

## Competitive Reality

### What Actually Works
1. **Hash Tables**: 25-40x faster than B-trees for point lookups
2. **B-trees**: Best all-around (good at everything)
3. **LSM Trees**: Best for write-heavy workloads
4. **Column Stores**: Best for analytics (DuckDB)

### State-of-the-Art Performance
- **DragonflyDB**: 3.8M QPS (uses shared-nothing architecture)
- **RocksDB**: 4.5-7M QPS (uses LSM trees)
- **Redis**: 150K QPS per core (uses hash tables)
- **DuckDB**: 100x faster on analytics (uses vectorization)

## Three Realistic Options

### Option 1: Pivot to Vector Search ✅
**Why This Makes Sense:**
- We have Mojo HNSW implementation (867 vec/s)
- Growing market (AI/ML applications)
- Clear performance target (20K vec/s)
- Achievable with optimization

**Next Steps:**
```bash
cd omendb/engine
pixi run python benchmarks/final_validation.py
# Fix bulk construction
# Implement proper parallelism
# Target: 20K vec/s
```

### Option 2: Build on DuckDB 🔄
**Why This Makes Sense:**
- DuckDB is already state-of-the-art
- We could add specialized indexes as extensions
- Focus on specific vertical (time-series, IoT)
- Leverage existing optimization

**Next Steps:**
```bash
# Build DuckDB extension
# Add specialized time-series operators
# Focus on specific use case
```

### Option 3: Stop and Do Something Else ❌
**Why This Makes Sense:**
- Core thesis disproven
- Competition already solved the problem
- Better opportunities elsewhere
- Don't throw good money after bad

## Lessons Learned

### What Went Wrong
1. **Built before proving**: Created architecture before validating concept
2. **Believed research papers**: Academic results don't translate to practice
3. **Wrong benchmarks**: Measured wrong metrics against wrong baselines
4. **Ignored reality**: Hash tables and B-trees are incredibly optimized

### What We Should Have Done
1. **Start with POC**: Test learned indexes vs B-trees first
2. **Use real benchmarks**: Compare against DuckDB, RocksDB, Redis
3. **Test real workloads**: Not synthetic patterns
4. **Validate with customers**: Do they even need this?

## Final Recommendation

### STOP the Learned Index Database

The evidence is overwhelming:
- Learned indexes are 8-14x SLOWER in practice
- Hash tables are 25-40x FASTER for point lookups
- B-trees are perfectly fine for range queries
- No customer has asked for learned indexes

### IF Continuing, Pivot to Vector Search

The only viable path:
- Focus on HNSW implementation
- Target 20K vec/s (currently 867)
- Compete with Qdrant/Pinecone
- Leverage Mojo for performance

### Timeline
- **By Oct 3**: Make final decision
- **If pivoting**: 8-12 weeks to MVP
- **If stopping**: Open source learnings

## The Hard Truth

We built a solution looking for a problem. Learned indexes sounded revolutionary but they're actually worse than 40-year-old B-trees. Sometimes the old solutions are old because they work.

The market doesn't need another key-value store. DragonflyDB, Redis, and RocksDB have that covered. The market might need a better vector database, but only if we can deliver 10x better performance or developer experience.

**Final word**: Don't continue something just because you started it. The data shows learned indexes don't work. Accept it and move on.

---

*Analysis completed: September 26, 2025*
*Decision required by: October 3, 2025*