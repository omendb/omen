# Critical Learned Index Papers

## Essential Reading (Download These First)

### 1. The Case for Learned Index Structures (2018)
**Google Research - Kraska et al.**
- https://arxiv.org/pdf/1712.01208.pdf
- Original RMI paper, introduced concept
- **Key insight**: B-trees approximate CDF, ML can learn CDFs better

### 2. ALEX: An Updatable Adaptive Learned Index (2020)
**Microsoft Research**
- https://arxiv.org/pdf/1905.08898.pdf
- **Solves update problem** with adaptive nodes
- Gapped arrays for inserts

### 3. RadixSpline: A Single-Pass Learned Index (2020)
**TU Munich & MIT**
- https://arxiv.org/pdf/2004.14541.pdf
- **Simpler than RMI**, single-pass construction
- Good fallback if RMI too complex

### 4. SOSD: A Benchmark for Learned Indexes (2021)
**MIT & Intel**
- https://arxiv.org/pdf/1911.13014.pdf
- Comprehensive benchmarks
- **Use their evaluation framework**

## Implementation References

### 5. PGM-Index: Piecewise Geometric Model Index (2020)
- https://arxiv.org/pdf/1910.06169.pdf
- Alternative to RMI, provable guarantees
- Compressed space, optimal query time

### 6. CDFShop: Exploring and Optimizing Learned Index Structures (2020)
- https://dl.acm.org/doi/pdf/10.1145/3318464.3384706
- Tools for tuning RMI parameters
- **Optimization strategies**

### 7. LIPP: Updatable Learned Index (2021)
- https://arxiv.org/pdf/2104.05520.pdf
- Handles updates without retraining
- Good for write-heavy workloads

## Advanced Topics (Read Later)

### 8. Learned Cardinality Estimation (2019)
- https://arxiv.org/pdf/1809.00677.pdf
- Use ML for query optimization
- Future feature after basic index works

### 9. FITing-Tree: A Data-aware Index Structure (2019)
- https://arxiv.org/pdf/1801.10207.pdf
- Hybrid learned/traditional approach
- Fallback strategy if pure learned fails

### 10. XIndex: Scalable Learned Index for Multicore (2020)
- https://arxiv.org/pdf/1901.05321.pdf
- Concurrent learned indexes
- Important for production

## Quick Reference

### Update Strategies
1. **Delta Buffer** (ALEX): Recent updates in buffer, merge periodically
2. **Gapped Arrays** (ALEX): Leave space for inserts
3. **Model Retraining** (RMI): Rebuild periodically
4. **Hybrid** (FITing-Tree): Learned + B-tree for updates

### Model Types
1. **Linear**: Fast, simple, good for uniform data
2. **Piecewise Linear** (PGM): Better accuracy, more complex
3. **Neural Networks** (RMI): Best accuracy, slower
4. **Radix + Spline** (RadixSpline): Good compromise

### Error Bounds
- All learned indexes need error bounds
- Binary search within error range
- Typical: ±100-1000 positions
- Trade-off: Model size vs error bound

## Code Repositories

### Official Implementations
- **RMI**: https://github.com/learnedsystems/RMI (Rust)
- **SOSD**: https://github.com/learnedsystems/SOSD (C++)
- **RadixSpline**: https://github.com/learnedsystems/RadixSpline (C++)
- **ALEX**: https://github.com/microsoft/ALEX (C++)
- **PGM-Index**: https://github.com/gvinciguerra/PGM-index (C++)

## Download Commands

```bash
# Download essential papers
cd external/papers

# Core papers (read first)
wget https://arxiv.org/pdf/1712.01208.pdf -O 01-learned-index-structures.pdf
wget https://arxiv.org/pdf/1905.08898.pdf -O 02-alex-updatable.pdf
wget https://arxiv.org/pdf/2004.14541.pdf -O 03-radixspline.pdf
wget https://arxiv.org/pdf/1911.13014.pdf -O 04-sosd-benchmark.pdf

# Implementation details
wget https://arxiv.org/pdf/1910.06169.pdf -O 05-pgm-index.pdf
wget https://dl.acm.org/doi/pdf/10.1145/3318464.3384706 -O 06-cdfshop.pdf

# Clone reference implementations
cd ../learned-systems
git clone https://github.com/learnedsystems/RMI.git
git clone https://github.com/learnedsystems/SOSD.git
git clone https://github.com/learnedsystems/RadixSpline.git
```

---

*Start with papers 1-3. If RMI too complex, use RadixSpline. ALEX solves updates.*