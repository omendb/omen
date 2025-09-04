# ZenDB Development Guide

## Quick Start for New Claude Code Sessions

### Repository Status  
- **Language**: Rust (confirmed optimal choice, universal C FFI bindings)
- **License**: Elastic License 2.0 (open core model)
- **Architecture**: "Database that grows with you" - embedded → distributed scaling
- **Stage**: Project foundation complete, Phase 1 implementation ready
- **Key Differentiators**: Time-travel, real-time subscriptions, schema-as-code

### Immediate Development Priorities (Phase 1: Core + Killer Features)

**Foundation (Months 1-6)**
1. **Page Manager** (`src/storage/page_manager.rs`) - Memory-mapped I/O, 16KB pages
2. **B+Tree Implementation** (`src/storage/btree.rs`) - Primary storage structure
3. **LSM Buffer** (`src/storage/lsm_buffer.rs`) - Write burst absorption 
4. **Hybrid Engine** (`src/storage/hybrid_engine.rs`) - Adaptive B+Tree/LSM switching
5. **MVCC Core** (`src/transaction/mvcc.rs`) - Hybrid Logical Clock timestamps

**Killer Features (Months 6-12)**
6. **Time-Travel Queries** (`src/transaction/time_travel.rs`) - AS OF TIMESTAMP support
7. **Real-Time Subscriptions** (`src/query/subscriptions.rs`) - Live query updates
8. **Schema Migrations** (`src/schema/migrations.rs`) - Schema-as-code tooling
9. **Vector Search** (`src/storage/vector.rs`) - HNSW index integration
10. **PostgreSQL Protocol** (`src/network/postgres.rs`) - Wire protocol compatibility

### Research-Validated Architecture Decisions

**Storage Engine**: Hybrid B+Tree + LSM Buffer  
- **Justification**: Bf-Tree research (2024) shows 2.5× better scan performance
- **Implementation**: B+Tree for reads, LSM buffer absorbs write bursts
- **Page Size**: 16KB (SSD-optimized)

**MVCC**: Hybrid Logical Clocks (HLC)
- **Justification**: Google Spanner achieves 2ms write latency, 30% less coordination  
- **Implementation**: Physical + logical timestamps for ordering
- **Concurrency**: Lock-free readers with optimistic writers

**Wire Protocol**: PostgreSQL Compatibility
- **Justification**: Instant ecosystem integration (ORMs, tools, drivers)
- **Implementation**: Start with basic query/response, add features incrementally

**Time-Travel Queries**: Natural MVCC Extension
- **Justification**: Unique differentiation, huge developer appeal for debugging
- **Implementation**: Extend HLC timestamps, version-aware B+Tree traversal
- **Syntax**: `SELECT * FROM users AS OF TIMESTAMP '2025-01-01 10:30:00'`

### Performance Targets (Research-Based)

| Metric | Target | Current SOTA Baseline |
|--------|--------|--------------------|
| Hybrid throughput | >3× SQLite | Bf-Tree: 2.5× RocksDB |
| Write latency | <2ms | Google Spanner: 2ms |
| Point reads | <50μs | Industry standard |
| Concurrent users | 10k+ | 100× SQLite improvement |

### Development Commands

```bash
# Build and test
cargo build --release
cargo test
cargo bench

# Run in embedded mode
cargo run -- --embedded --data-path ./test.db

# Run basic checks
cargo clippy
cargo fmt
```

### Testing Strategy

**Property-Based Testing** (highest priority)
- Use `proptest` crate for storage engine correctness
- Test B+Tree invariants under concurrent operations
- Validate crash recovery scenarios

**Benchmarking**
- Compare against SQLite for embedded performance
- Measure against PostgreSQL for distributed performance
- Focus on concurrent read/write workloads

### Language Bindings Strategy

**Universal C FFI Core**
```rust
// Core C interface for maximum compatibility
#[no_mangle]
pub extern "C" fn zen_query(
    db: *mut ZenDB,
    sql: *const c_char,
    result: *mut *mut c_char
) -> i32;
```

**Priority Language Bindings**
1. **Python** (PyO3) - Primary Zenith framework integration
2. **Node.js** (NAPI-RS) - Web development market
3. **Go** (CGO) - Cloud/infrastructure applications
4. **C#/.NET** (P/Invoke) - Enterprise market
5. **Java** (JNI) - Enterprise/Android development

**Query Patterns: Conventional + Enhanced**
- Standard SQL works exactly as expected
- Time-travel extensions with intuitive syntax
- Vector queries using familiar SQL patterns
- Real-time subscriptions with framework integration

### Key Implementation Notes

**Memory Safety**: All storage operations must be memory-safe
- Use Arc/Mutex for shared state
- Validate all unsafe operations thoroughly
- Implement comprehensive error handling

**Performance**: Prioritize zero-copy operations
- Memory-mapped I/O for pages
- Avoid unnecessary serialization/deserialization
- Use SIMD where applicable

**Correctness**: ACID guarantees are non-negotiable
- Implement WAL before optimization
- Test crash recovery extensively  
- Validate isolation levels

**Reliability Focus**: NO accuracy-risky AI features
- ❌ Natural language query interface (affects precision)
- ❌ AI schema changes (affects reliability)  
- ❌ Automatic query rewriting (affects predictability)
- ✅ Query optimization hints (suggestions only)
- ✅ Performance insights (diagnostic only)

### Architecture Validation

✅ **Research-Confirmed Feasibility**
- Hybrid storage engines are proven (Bf-Tree, LAB-DB)
- HLC-based MVCC is production-ready (Spanner, CockroachDB)
- PostgreSQL protocol compatibility is well-documented

✅ **Competitive Gaps Identified**
- Auto-tuning hybrid engines (current solutions need expert tuning)
- Multi-modal transactions (relational + vector + time-series)
- Framework-optimized database (purpose-built for Zenith)

✅ **Market Timing**
- Research trends align with ZenDB vision
- No existing solution combines all these features
- Embedded→distributed scaling is unique value proposition

### Business Model & Target Market

**Open Core (Elastic License 2.0)**
- **Open Source**: Core engine, embedded mode, basic time-travel, PostgreSQL protocol
- **Commercial**: Extended time-travel, distributed clustering, enterprise security, managed service

**Target Market Priority**
1. **Phase 1**: Indie developers and startups (embedded → small distributed)
2. **Phase 2**: Mid-market SaaS companies (managed service adoption)
3. **Phase 3**: Enterprise (large-scale deployments, compliance)

**Competitive Position**: Only database combining embedded→distributed + time-travel + real-time + schema-as-code

### Next Session Action Items

**Immediate Implementation Priorities:**
1. **Page Manager**: Memory-mapped I/O, 16KB pages, allocation/deallocation
2. **Property-Based Testing**: Set up proptest framework for correctness validation
3. **B+Tree + MVCC**: Implement search/insert with HLC timestamp support  
4. **Time-Travel Extension**: AS OF TIMESTAMP query support
5. **Benchmark Harness**: Performance comparison vs SQLite/PostgreSQL

**Success Criteria:**
- **Month 6**: 3× SQLite performance, time-travel queries working
- **Month 12**: Real-time subscriptions, 1K GitHub stars, 100 production deployments
- **Month 18**: Distributed mode, vector search, first commercial customers

**The foundation is research-validated and market-ready. Focus on implementation quality and the 3 killer features that no competitor offers.**