# OmenDB Development Roadmap

## Core Strategy: Pure Rust Implementation

Build a production-grade learned database in Rust, focusing on real performance gains through better algorithms and architecture rather than language changes.

## Timeline

### ✅ Phase 0: Foundation (COMPLETED - September 2025)
- [x] Prove learned indexes can be faster (1.4x achieved)
- [x] Implement hot/cold architecture
- [x] Add MVCC transaction support
- [x] Create modular engine design
- [x] Make architecture decision (Pure Rust)

### 🚧 Phase 1: MVP (October 2025)
**Goal: Ship working database with 2-3x PostgreSQL performance**

- [ ] Complete modular engine implementation
  - [ ] Index engines (Linear, RMI, B-tree)
  - [ ] Storage engines (RocksDB, In-memory)
  - [ ] Compute engines (SIMD, Scalar)
- [ ] Create PyO3 Python package
  - [ ] Direct Python bindings (no FFI complexity)
  - [ ] NumPy/Pandas integration
  - [ ] Publish to PyPI
- [ ] Deploy demo to omendb.com
  - [ ] Interactive benchmarks
  - [ ] Live performance comparison
  - [ ] API documentation

### 📈 Phase 2: Optimization (November 2025)
**Goal: Achieve 5-10x PostgreSQL performance with pure Rust**

- [ ] SIMD optimizations
  - [ ] Use `packed_simd2` crate
  - [ ] Vectorized distance calculations
  - [ ] Batch operations
- [ ] Parallelization
  - [ ] Rayon for data parallelism
  - [ ] Concurrent readers
  - [ ] Parallel index training
- [ ] Cache optimizations
  - [ ] Cache-aligned structures
  - [ ] Prefetching strategies
  - [ ] NUMA awareness

### 🏢 Phase 3: Production (December 2025 - Q1 2026)
**Goal: First production deployments**

- [ ] Enterprise features
  - [ ] Authentication/authorization
  - [ ] Audit logging
  - [ ] Backup/restore
  - [ ] Monitoring/metrics
- [ ] Cloud deployment
  - [ ] Docker containers
  - [ ] Kubernetes operators
  - [ ] Cloud-native storage
- [ ] Customer pilots
  - [ ] 3-5 design partners
  - [ ] Performance tuning
  - [ ] Feature requests

### 🌍 Phase 4: Scale (Q2-Q3 2026)
**Goal: Distributed database with 100+ customers**

- [ ] Distributed architecture
  - [ ] Sharding
  - [ ] Replication
  - [ ] Consensus (Raft)
  - [ ] Cross-region support
- [ ] Advanced features
  - [ ] Vector similarity search
  - [ ] Time-series optimizations
  - [ ] Graph capabilities
  - [ ] SQL compatibility layer
- [ ] Business growth
  - [ ] 100+ customers
  - [ ] $1M ARR target
  - [ ] Series A fundraising

### 🔮 Phase 5: Future Technologies (Q4 2026+)
**Goal: Evaluate next-generation technologies when mature**

#### Mojo Re-evaluation Criteria
- ✅ Mojo reaches 1.0 stability
- ✅ Production deployments exist
- ✅ MLIR compilation is stable
- ✅ Clear 2x+ performance advantage
- ✅ We have resources for dual-stack

#### MAX Platform Evaluation
- For inference workloads only
- GPU-accelerated similarity search
- Not for core database operations

#### Decision Points
- **Q4 2026**: First evaluation of Mojo 1.0 (if released)
- **Q1 2027**: MAX platform for GPU workloads
- **Q2 2027**: Consider specialized accelerators

## Performance Targets

### Current State (September 2025)
```
Metric                  Achievement
------                  -----------
Point Lookup            41M ops/sec (1.39x baseline)
Range Query             26M results/sec (1.46x baseline)
Bulk Insert             15K records/sec
Transaction Throughput  10K txn/sec
```

### Target State (December 2025)
```
Metric                  Target (Pure Rust)
------                  ------------------
Point Lookup            200M ops/sec (5x PostgreSQL)
Range Query             100M results/sec (10x PostgreSQL)
Bulk Insert             100K records/sec
Transaction Throughput  50K txn/sec
```

### Stretch Goals (With Future Tech, 2027+)
```
Metric                  Potential
------                  ---------
Point Lookup            500M ops/sec (with Mojo MLIR)
Range Query             250M results/sec (with GPU)
Vector Search           1M vec/sec (with MAX)
```

## Technology Stack

### Core (Committed)
- **Language**: Rust
- **Storage**: RocksDB
- **SIMD**: packed_simd2
- **Parallelism**: Rayon
- **Async**: Tokio
- **API**: PyO3 (Python), Tonic (gRPC)

### Future (Evaluation Only)
- **Mojo**: Re-evaluate Q4 2026 if 1.0 released
- **MAX**: For GPU inference workloads only
- **MLIR**: When proven in production
- **WebGPU**: Browser-based acceleration

## Key Principles

1. **Ship Fast**: Use proven technology (Rust)
2. **Real Performance**: Focus on algorithms, not languages
3. **Customer First**: Build what users need
4. **Modular Design**: Keep architecture flexible
5. **Measure Everything**: Data-driven decisions

## Success Metrics

### 2025 Q4
- [ ] 1,000+ GitHub stars
- [ ] 100+ beta users
- [ ] 3 production pilots

### 2026 Q2
- [ ] 10,000+ PyPI downloads/month
- [ ] 50+ paying customers
- [ ] $500K ARR

### 2026 Q4
- [ ] 100+ customers
- [ ] $1M+ ARR
- [ ] Series A raised

## Risk Mitigation

### Technology Risks
- **Risk**: Rust performance insufficient
- **Mitigation**: Modular design allows component swapping
- **Backup**: Already proven 1.4x improvement, 5x achievable

### Market Risks
- **Risk**: Enterprises slow to adopt
- **Mitigation**: Start with startups and scale-ups
- **Backup**: Open source for adoption, enterprise for revenue

### Competition Risks
- **Risk**: Postgres adds learned indexes
- **Mitigation**: Focus on complete solution, not just indexes
- **Backup**: Best implementation wins, we're ahead

## Conclusion

OmenDB will succeed by:
1. **Shipping a working product** in pure Rust
2. **Achieving real performance gains** through better algorithms
3. **Growing with customers** rather than chasing technology
4. **Keeping options open** for future optimizations

Mojo and MLIR are exciting future technologies, but Rust gives us everything we need to build a world-class database today. We'll re-evaluate new technologies when they're production-ready and we have specific needs they solve.

---
*Last Updated: September 26, 2025*
*Next Review: October 31, 2025*