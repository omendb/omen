# ZenDB Architecture Migration Plan

## Decision: Archive ZenDB, Preserve Patterns in OmenDB

### Preservation Strategy
```bash
# 1. Extract valuable patterns
mkdir -p internal/knowledge/zendb_patterns/
cp -r zendb/src/storage/ internal/knowledge/zendb_patterns/mvcc/
cp -r zendb/src/sql/ internal/knowledge/zendb_patterns/sql/  
cp zendb/tests/ internal/knowledge/zendb_patterns/test_patterns/
cp zendb/DESIGN.md internal/knowledge/zendb_patterns/

# 2. Archive the codebase
mkdir -p archive/
git mv zendb archive/zendb
git commit -m "archive: preserve ZenDB for future reference, migrating patterns to OmenDB"
```

### Key Patterns to Migrate to Mojo

#### 1. MVCC Transaction Layer
```rust
// ZenDB pattern (Rust)
pub struct TransactionManager {
    active_transactions: HashMap<TxId, Transaction>,
    commit_timestamp: AtomicU64,
}

// OmenDB equivalent (Mojo) 
struct TransactionManager:
    var active_transactions: Dict[Int, Transaction]  # Will use custom Dict
    var commit_timestamp: Atomic[Int]
```

#### 2. Metadata Storage Patterns
```rust
// ZenDB: Column store for metadata
pub struct MetadataStore {
    columns: HashMap<String, Column>,
    row_index: Vec<RowId>,
}

// OmenDB: Integrate with HNSW nodes
struct MetadataStore:
    var columns: SparseMap[String, Column]  # Custom implementation
    var node_metadata: UnsafePointer[Metadata]  # Parallel to HNSW nodes
```

#### 3. Query Planning Logic
```rust
// ZenDB: Hybrid query optimizer
impl QueryPlanner {
    fn optimize_hybrid_query(&self, query: HybridQuery) -> ExecutionPlan
}

// OmenDB: HNSW + metadata query optimization
struct HybridQueryPlanner:
    fn optimize_query(self, query: HybridQuery) -> ExecutionPlan:
        # Decide vector-first vs metadata-first based on selectivity
```

### Integration Architecture

#### Multimodal OmenDB Structure
```mojo
struct MultimodalIndex:
    var vector_index: HNSWIndex           # Primary HNSW+ index
    var metadata_store: MetadataStore     # Structured data (from ZenDB patterns)
    var text_index: FullTextIndex        # BM25 search
    var transaction_manager: TransactionManager  # ACID (from ZenDB)

fn hybrid_search(
    self,
    vector_query: Optional[UnsafePointer[Float32]],
    text_query: Optional[String], 
    metadata_filter: Optional[MetadataFilter],
    k: Int = 10
) -> List[HybridResult]:
    # Query planning logic from ZenDB
    # HNSW traversal with integrated filtering
    # Return unified results
```

### Migration Benefits
1. **Unified codebase** - Single Mojo implementation vs Rust+Mojo
2. **Performance consistency** - All operations benefit from Mojo SIMD
3. **Simpler deployment** - One binary, not two databases  
4. **Better Python integration** - Native Mojo, no FFI overhead

### Timeline
- **Week 1**: Archive ZenDB, extract patterns to `/internal/knowledge/zendb_patterns/`
- **Week 2**: Begin HNSW+ implementation with metadata hooks
- **Week 3**: Implement metadata storage using ZenDB patterns
- **Week 4**: Add transaction layer and hybrid search

---
*ZenDB → OmenDB multimodal migration strategy*