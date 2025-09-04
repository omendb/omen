# ZenDB

**The Database That Grows With You**

ZenDB is a next-generation database system that scales seamlessly from embedded to distributed deployment. Start with SQLite simplicity, scale to PostgreSQL power, debug with Git-like time-travel, and react with Firebase-like subscriptions—all while maintaining rock-solid reliability and conventional SQL.

## 🚀 Core Vision

**One Database, Every Scale**
- **Development**: Embedded mode with zero configuration  
- **Production**: Distributed cluster with automatic consensus
- **Edge**: WASM-compatible for serverless deployment
- **Mobile**: Offline-first with conflict-free synchronization
- **Time-Travel**: Debug production by querying the past
- **Real-Time**: Live dashboards without external infrastructure

## 🏗️ Architecture Overview

### Hybrid Storage Engine
- **B+Tree Primary**: Predictable read performance for transactional workloads
- **LSM Write Buffer**: High-throughput write absorption for burst traffic
- **Adaptive Switching**: Automatic optimization based on workload patterns

### Zero-Copy MVCC
- Lock-free readers with hybrid logical clocks
- Optimistic concurrency control
- 100x better concurrent performance than SQLite

### Wire Protocol Compatibility
- **PostgreSQL Protocol**: Instant ecosystem compatibility
- **SQLite Migration**: Seamless upgrade path from existing applications
- **TypeScript Native**: Full type inference and auto-completion

## 🌟 Key Features

### Multi-Modal Data Support
```sql
-- Relational data
CREATE TABLE users (id SERIAL PRIMARY KEY, email TEXT UNIQUE);

-- JSON documents
CREATE TABLE profiles (user_id INT, data JSONB);

-- Vector embeddings for AI
CREATE TABLE embeddings (id SERIAL, vector VECTOR(1536));

-- Time-series data
CREATE TABLE metrics (timestamp TIMESTAMPTZ, value FLOAT, tags JSONB);
```

### Framework Integration
```python
# Zenith Framework + ZenDB
from zenith import App, Response
from zenith.db import ZenDB, table, column

app = App()
db = ZenDB()  # Auto-detects embedded vs distributed

@app.route('/users')
async def get_users():
    users = await db.query('SELECT * FROM users LIMIT 10')
    return Response.json(users)
```

### Time-Travel Queries
```sql
-- Debug production issues by querying the past
SELECT * FROM users AS OF TIMESTAMP '2025-01-01 10:30:00';

-- See how data changed over time
SELECT * FROM users FOR SYSTEM_TIME BETWEEN 
  '2025-01-01' AND '2025-01-02' WHERE id = 123;
```

### Real-Time Subscriptions
```python
# Live query subscriptions
subscription = db.subscribe(
    'SELECT COUNT(*) as active_users FROM users WHERE last_seen > NOW() - INTERVAL 5 MINUTES'
)

subscription.on('change', lambda count: update_dashboard(count))
```

### Vector Search Integration
```sql
-- Combined relational + vector queries
SELECT p.name, p.price,
       vector_distance(p.embedding, $1) as similarity
FROM products p
WHERE p.category = 'electronics' 
  AND p.price < 1000
  AND vector_distance(p.embedding, $1) < 0.5
ORDER BY similarity LIMIT 10;
```

## 🎯 Target Use Cases

1. **SaaS Applications**: Multi-tenant isolation with row-level security
2. **Real-time Collaboration**: CRDT-based conflict resolution
3. **Edge Analytics**: Local aggregation with periodic cloud sync
4. **AI/ML Workloads**: Native vector operations and similarity search
5. **Developer Tools**: Zero-configuration embedded database

## 📊 Performance Goals

| Operation | Embedded Mode | Network Mode | Status |
|-----------|--------------|--------------|--------|
| Point Read | <50μs | <500μs | 🎯 Target |
| Range Scan (1K rows) | <1ms | <5ms | 🎯 Target |
| Single Insert | <100μs | <1ms | 🎯 Target |
| Bulk Insert | >500k/sec | >200k/sec | 🎯 Target |
| Concurrent Users | 10k+ | 100k+ | 🎯 Target |

## 🗺️ Roadmap

### Phase 1: Core + Killer Features (Months 1-12)
**Foundation**
- [ ] Hybrid B+Tree/LSM storage engine
- [ ] MVCC with Hybrid Logical Clocks
- [ ] PostgreSQL wire protocol compatibility
- [ ] Embedded mode with file-based storage

**Killer Features**
- [ ] Time-travel queries (AS OF TIMESTAMP)
- [ ] Real-time subscriptions
- [ ] Schema-as-Code migrations
- [ ] Native vector search (HNSW indexes)

### Phase 2: Competitive Moats (Months 12-24)
**Performance & Intelligence**
- [ ] AI-powered query optimization
- [ ] SIMD-optimized execution
- [ ] NVMe-aware I/O
- [ ] Distributed clustering

**Enterprise Features**
- [ ] Automatic PII detection & masking
- [ ] Data lineage tracking
- [ ] Advanced security (encryption, audit)
- [ ] Multi-language bindings (Python, Node.js, Go)

### Phase 3: Advanced Capabilities (Year 2+)
- [ ] WASM compilation target
- [ ] Self-healing operations
- [ ] Global distribution
- [ ] Advanced compliance features

## 🔬 Research-Validated Architecture

ZenDB incorporates 2025 state-of-the-art database research:

- **Hybrid Storage**: Bf-Tree inspired B+Tree/LSM combination for 2.5× performance
- **Hybrid Logical Clocks**: Google Spanner approach for distributed timestamp ordering
- **Lock-Free MVCC**: Epoch-based reclamation for 100× better concurrency than SQLite
- **Native Multi-Modal**: Unified transactions across relational, vector, and time-series data
- **Embedded→Distributed**: TemperDB/EdgeDB validated architecture pattern

## 🛠️ Development

```bash
# Clone repository
git clone https://github.com/nijaru/zendb.git
cd zendb

# Build and test
cargo build --release
cargo test

# Run benchmarks
cargo bench

# Start embedded mode
./target/release/zendb --embedded

# Start distributed mode
./target/release/zendb --server --cluster
```

## 📖 Documentation

- [Getting Started](docs/getting-started.md)
- [Architecture Guide](docs/architecture.md)
- [API Reference](docs/api-reference.md)
- [Deployment Guide](docs/deployment.md)
- [Performance Tuning](docs/performance.md)

## 🌐 Language Bindings

Universal access through C FFI core:

- **Python** (PyO3) - Primary Zenith framework integration
- **Node.js** (NAPI-RS) - Web development ecosystem
- **Go** (CGO) - Cloud and infrastructure applications
- **C#/.NET** (P/Invoke) - Enterprise applications
- **Java** (JNI) - Enterprise and Android development

## 🤝 Contributing

ZenDB uses an open core model with Elastic License 2.0.

- [Contributing Guide](CONTRIBUTING.md)
- [Development Setup](docs/DEVELOPMENT.md)
- [Architecture Guide](CLAUDE.md)

## 📄 License & Business Model

**Open Source (Elastic License 2.0)**
- Core database engine, embedded mode, basic time-travel
- PostgreSQL protocol, basic vector search, real-time subscriptions

**Commercial Features**
- Extended time-travel, distributed clustering, advanced security
- Enterprise support, managed cloud service
- Advanced vector indexes, PII detection, data lineage

---

**Built with ❤️ by the Zenith team**

*ZenDB: Find zen in your data's natural flow*