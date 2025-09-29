# OmenDB Development Context

## Strategic Direction (September 2025)

**Product**: Unified OLTP/OLAP database with learned index optimization
**Target**: $22.8B ETL market - companies needing real-time analytics
**Stack**: Rust (DataFusion + learned indexes + Arrow storage)
**Timeline**: 12 weeks to customer validation

## Technical Core

**Learned Indexes**:
- LearnedKV (2024): 4.32x speedup at 10M+ keys, 1KB+ values, Zipfian workloads
- LITune (Feb 2025): Deep RL for learned index tuning
- Our approach: Hybrid LSM + learned (not pure replacement)

**Market Position**:
- CockroachDB ($5B, ~$200M ARR) - market leader
- SingleStore ($1.3B, $110M ARR) - MySQL-focused
- TiDB ($270M raised, $13.1M ARR) - poor capital efficiency
- **Gap**: PostgreSQL-compatible HTAP with learned optimization

## Architecture

```
Unified Engine:
├── OLTP Layer: Row-oriented transactions (PostgreSQL wire protocol)
├── OLAP Layer: Columnar analytics (DataFusion + Arrow/Parquet)
├── Learned Optimizer: Hot/cold placement, query routing
└── Storage: Tiered (memory → SSD → object storage)
```

**Repository Structure**:
```
omendb/core/
├── omendb-rust/       # Main implementation
│   ├── learned_index/ # Hierarchical learned indexes
│   ├── storage/       # Arrow columnar storage
│   └── protocol/      # PostgreSQL wire protocol
├── benchmarks/        # Performance testing
└── internal/          # Strategy docs
```

## Competitive Advantages

1. **No ETL**: Real-time analytics on transactional data
2. **Learned Optimization**: Intelligent hot/cold placement
3. **PostgreSQL Compatible**: Drop-in replacement
4. **Elastic Scaling**: Separate OLTP/OLAP compute

## Technical Stack

**Core**: DataFusion (query engine), Arrow/Parquet (storage), PostgreSQL wire protocol
**Learned Components**: Hot/cold placement, query routing
**Language Support**: PostgreSQL drivers (all languages), Python, TypeScript, Go, Rust

## Performance Targets

**Year 1**:
- OLTP: 50K txn/sec, <10ms p99
- OLAP: 1M rows/sec scan, <1s queries
- Scale: 1TB databases, 100GB memory

**Year 2-3**: 500K txn/sec, 10M rows/sec scan, 10+ node clusters

## Development Environment

**Hardware**: 8+ cores, 32GB+ RAM, NVMe SSD, optional GPU (4090 for learned index acceleration)
**Stack**: Rust, PostgreSQL (compatibility testing), Kafka (real-time sync), Docker Compose, Kubernetes

### GPU Optimization (4090)
```bash
# Learned index acceleration
CUDA_DEVICE=0 python test_gpu_learned.py

# Arrow/DataFusion RAPIDS integration
pip install cudf cupy
```

## 12-Week Milestones

- **Week 1**: Validate learned indexes at scale (50M keys, 1KB values, Zipfian)
- **Week 3**: 5+ customer LOIs
- **Week 8**: Working MVP with real-time analytics
- **Week 12**: 3+ paying customers or funding

## Quick Start

```bash
# Test learned indexes
python test_proper_learned.py

# Start unified engine
cargo new unified-engine && cd unified-engine
```

---
*Updated: September 2025 | Timeline: 12 weeks to validation | Market: $22.8B ETL opportunity*