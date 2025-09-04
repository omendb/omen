# ZenDB Cloud-Native Architecture

## Overview
ZenDB is a disaggregated, serverless SQL database built for cloud-native deployment while maintaining embedded database simplicity.

## Core Architecture Principles

### 1. Disaggregated Storage and Compute
```
┌─────────────────────────────────────────────┐
│             Client Applications              │
└─────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────┐
│          Edge Proxy Layer (Global)          │
│        (Cloudflare Workers / Lambda)        │
└─────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────┐
│         Compute Layer (Stateless)           │
│     ┌──────────────┐  ┌──────────────┐     │
│     │ Query Engine │  │ Query Engine │     │
│     │   (WASM)     │  │   (WASM)     │     │
│     └──────────────┘  └──────────────┘     │
│            │                  │             │
│     ┌──────▼──────────────────▼──────┐     │
│     │   Local Page Cache (B+Tree)    │     │
│     │     (Our Current Engine)       │     │
│     └─────────────────────────────────┘     │
└─────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────┐
│         Storage Layer (Infinite)            │
│     ┌──────────────┐  ┌──────────────┐     │
│     │  S3/R2/GCS   │  │  WAL Stream  │     │
│     │   (Pages)    │  │   (Kinesis)  │     │
│     └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────┘
```

### 2. Page-Oriented Cloud Storage

Our existing PageManager becomes the caching layer:

```rust
pub trait CloudStorage: Send + Sync {
    async fn read_page(&self, page_id: PageId) -> Result<Page>;
    async fn write_page(&self, page: &Page) -> Result<()>;
    async fn list_pages(&self, prefix: &str) -> Result<Vec<PageId>>;
}

pub struct HybridPageManager {
    local_cache: BTreePageManager,  // Our current implementation
    cloud_store: Box<dyn CloudStorage>,
    write_buffer: Arc<RwLock<WriteBuffer>>,
}

// Reads check cache first, then cloud
// Writes go to WAL stream + async to cloud storage
```

### 3. Branching and Version Control

Every database is a git-like repository:

```rust
pub struct DatabaseBranch {
    branch_id: Uuid,
    parent_branch: Option<Uuid>,
    base_version: u64,  // LSN/timestamp
    page_overrides: HashMap<PageId, PageVersion>,
}

// Branches share unchanged pages (copy-on-write)
// Merging branches = merging page sets
```

### 4. Serverless Execution Model

```rust
// Compute nodes are ephemeral WebAssembly instances
#[wasm_bindgen]
pub struct QueryExecutor {
    catalog: RemoteCatalog,
    page_cache: LocalCache,
    session: SessionState,
}

// Cold start: ~50ms (WASM instantiation + catalog load)
// Warm performance: ~1ms per query
// Auto-scales: 0 to 1000s of instances
```

### 5. Smart Client SDK

```typescript
// TypeScript SDK with offline-first capability
import { ZenDB } from '@zendb/client';

const db = new ZenDB({
    endpoint: 'zendb://myapp',
    offlineCache: '10MB',
    syncMode: 'eventual'
});

// Queries execute locally when possible
const users = await db.query('SELECT * FROM users');

// Mutations sync through edge proxy
await db.execute('INSERT INTO users VALUES (?)');

// Subscribe to changes (WebSocket/SSE)
db.subscribe('SELECT * FROM orders WHERE status = ?', ['pending'])
  .on('change', order => updateUI(order));
```

## Development Phases

### Phase 1: Cloud Storage Backend (Current Sprint)
1. Implement S3/R2 backend for PageManager
2. Add async write-through caching
3. Test with MinIO locally

### Phase 2: Row Storage & SQL (Next 2 Weeks)
1. Design row format with schema versioning
2. Implement table catalog in special B+Tree
3. Basic SQL executor (CREATE TABLE, INSERT, SELECT)
4. PostgreSQL wire protocol

### Phase 3: Serverless Runtime (Month 2)
1. Compile to WASM target
2. Cloudflare Worker deployment
3. Connection pooling at edge
4. Query result caching

### Phase 4: Branching & Time-Travel (Month 3)
1. Copy-on-write page management
2. Branch metadata system
3. Time-travel using page versions
4. Branch merging logic

### Phase 5: Global Distribution (Month 4-6)
1. Multi-region replication
2. Geo-partitioning
3. Read replicas at edge
4. Conflict resolution (CRDTs)

## Why This Architecture Wins

### 1. **True Serverless** 
- No servers to manage
- Scales to zero (no cost when idle)
- Instant scaling (WASM instances)
- Per-query billing possible

### 2. **Planet-Scale by Default**
- Storage in S3/R2 (11 9s durability)
- Compute at edge (300+ locations)
- No single point of failure
- Infinite storage capacity

### 3. **Developer Experience**
- Instant database creation
- Zero-config branching
- Time-travel debugging
- Offline-first SDKs

### 4. **Cost Efficiency**
- Storage: $0.015/GB/month (S3)
- Compute: $0 when idle
- No overprovisioning
- Automatic tier optimization

### 5. **Unique Features**
- Git-like branching (nobody else has this)
- Time-travel at no extra cost
- Edge-native (sub-10ms globally)
- Embedded fallback (works offline)

## Comparison with Competition

| Feature | ZenDB | Neon | PlanetScale | Cloudflare D1 |
|---------|--------|------|------------|---------------|
| Serverless | ✅ True | ⚠️ Auto-suspend | ❌ Provisioned | ✅ True |
| Branching | ✅ Instant | ✅ Copy-on-write | ✅ Schema only | ❌ No |
| Edge Native | ✅ Global | ❌ Regional | ❌ Regional | ✅ Global |
| Time Travel | ✅ Built-in | ⚠️ Limited | ❌ No | ❌ No |
| PostgreSQL Compatible | ✅ Wire protocol | ✅ Full | ⚠️ MySQL | ⚠️ SQLite |
| Offline Capable | ✅ SDK cache | ❌ No | ❌ No | ⚠️ Read-only |

## Implementation Strategy

### Start Simple, Think Cloud-Native

1. **Keep current B+Tree engine** as caching layer
2. **Add cloud storage backend** incrementally
3. **Build SQL layer** with future distribution in mind
4. **Deploy to edge** as soon as we have basic SQL
5. **Add features** based on user feedback

### Critical Decisions Made Now

1. **Page-based storage** enables disaggregation
2. **MVCC timestamps** enable branching/time-travel
3. **Rust + WASM** enables edge deployment
4. **PostgreSQL protocol** ensures compatibility
5. **Event-sourced WAL** enables replication

## Success Metrics

### Technical
- Cold start < 100ms
- P50 query latency < 10ms globally
- Storage cost < $0.02/GB/month
- 99.99% availability SLA

### Business
- Self-serve sign-up to first query < 30 seconds
- Developer adoption without sales team
- $0 to start, scales with usage
- 80% gross margins at scale