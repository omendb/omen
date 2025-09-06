# DiskANN Implementation Archive

**Archived:** February 6, 2025
**Reason:** Strategic pivot to HNSW+ algorithm with clean rebuild approach

## Context

This directory contains all DiskANN-related code from the OmenDB project, archived for reference only. The code represents extensive research and development work on the Vamana algorithm but was ultimately replaced with HNSW+ for better streaming update support and industry compatibility.

## Decision Summary

- **Original choice:** DiskANN for billion-scale single-machine capability
- **Problem discovered:** Fundamentally batch-oriented, poor streaming updates
- **Solution:** Complete pivot to HNSW+ with state-of-the-art optimizations
- **Outcome:** Clean rebuild enables 10x performance improvements

See `/internal/DECISIONS.md` entry "2025-02-06 | Clean Rebuild Over Migration Approach" for full rationale.

## Archive Structure

```
├── README.md                    # This file
├── core/                        # Core DiskANN implementations
│   ├── diskann_graph.mojo      # Graph structure implementation  
│   └── correct_diskann.mojo    # Corrected DiskANN version
├── tests/                       # All DiskANN test files
│   ├── debug_diskann_*.py      # Debug test scripts
│   ├── test_diskann_*.py       # Unit and integration tests
│   └── benchmark_diskann_*.py  # Performance benchmarks
├── python/                      # Python interface files
│   ├── benchmark_diskann.py    # Python benchmarking script
│   ├── test_diskann_*.py       # Python test files
│   └── batch_fix experiments   # Batch processing fixes
├── external/                    # External DiskANN utilities
│   └── DiskANNIndexParser.py   # Index parsing utilities
└── Algorithm implementations:   # Various DiskANN versions
    ├── diskann.mojo            # Original implementation
    ├── proper_diskann.mojo     # Proper Vamana algorithm
    ├── optimized_diskann.mojo  # Performance optimizations
    ├── heap_based_diskann.mojo # Heap-based version
    └── diskann_integrated.mojo # Integrated version
```

## Key Lessons Learned

1. **Algorithm Choice Critical:** DiskANN excellent for batch, poor for streaming
2. **API Compatibility:** String vs numeric IDs caused integration issues  
3. **Mojo Optimization:** Custom collections needed (Dict/List overhead massive)
4. **Performance Bottlenecks:** Buffer flushing, FFI overhead, memory management
5. **Testing Strategy:** Needed more incremental validation vs bulk tests

## Reference Value

This archive serves as:
- **Algorithm reference:** For understanding Vamana/RobustPrune implementations
- **Performance patterns:** Lessons on Mojo optimization techniques
- **Test cases:** Validation scenarios for future implementations
- **Historical context:** Decision rationale and evolution

## DO NOT

- ❌ Reintroduce any of this code to active codebase
- ❌ Try to "fix" or migrate DiskANN code
- ❌ Reference for API design (incompatible with HNSW+)

## DO

- ✅ Reference algorithm implementations for research
- ✅ Study performance optimization patterns  
- ✅ Learn from test case design
- ✅ Use as historical context for decisions

---

*Archived in clean rebuild strategy - Feb 2025*