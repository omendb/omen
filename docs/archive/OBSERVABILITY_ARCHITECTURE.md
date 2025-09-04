# OmenDB Observability Architecture
*Strategic Decision: Server Layer, Not Engine Core*

## Architectural Philosophy

Following the Redis model: **Clean engine, observable server**

```
┌─────────────────────────────────────────┐
│          Server Layer (Rust)            │ ← Full observability here
│  - Prometheus metrics                   │
│  - OpenTelemetry traces                 │
│  - Health endpoints                     │
│  - Performance dashboards               │
└─────────────────────────────────────────┘
                    ↑
            Lightweight Stats API
                    ↑
┌─────────────────────────────────────────┐
│         Engine Core (Mojo)              │ ← Zero overhead
│  - Optional stats collection            │
│  - Operation counters                   │
│  - Basic memory tracking                │
│  - No external dependencies             │
└─────────────────────────────────────────┘
```

## Why This Architecture?

### Embedded Mode (Primary Use Case)
- **Zero overhead** when observability not needed
- No dependencies on metrics libraries
- Clean, fast, simple - like SQLite
- Users bring their own monitoring

### Server Mode (Enterprise Use Case)  
- Rich observability at server layer
- Prometheus/Grafana integration
- Distributed tracing support
- Zero impact on embedded users

## What the Engine Needs to Provide

### 1. Lightweight Stats Structure
```mojo
struct EngineStats:
    # Counters (cheap to maintain)
    var total_vectors: Int
    var total_searches: Int
    var total_inserts: Int
    var total_errors: Int
    
    # Memory (already tracking)
    var memory_bytes: Int
    var index_memory_bytes: Int
    
    # Optional latency histograms
    var search_latencies: Optional[Histogram]
    var insert_latencies: Optional[Histogram]
```

### 2. Stats Collection API
```mojo
struct VectorStore:
    var collect_stats: Bool = False  # OFF by default
    var stats: Optional[EngineStats]
    
    fn enable_stats(mut self):
        """Enable stats collection (opt-in)."""
        self.collect_stats = True
        self.stats = Optional[EngineStats](EngineStats())
    
    fn get_stats(self) -> Optional[EngineStats]:
        """Return stats if collection enabled."""
        return self.stats
    
    fn reset_stats(mut self):
        """Reset counters for server-mode intervals."""
        if self.stats:
            self.stats.value().reset_counters()
```

### 3. Operation Hooks (Zero-Cost When Disabled)
```mojo
fn search(...) -> List[SearchResult]:
    var start_ns = 0
    if self.collect_stats:
        start_ns = time_ns()
    
    # ... actual search ...
    
    if self.collect_stats and self.stats:
        var stats = self.stats.value()
        stats.total_searches += 1
        if start_ns > 0:
            stats.search_latencies.add(time_ns() - start_ns)
    
    return results
```

## Server Layer Responsibilities

### Rust Server Observability
```rust
// The Rust server layer adds rich observability
struct OmenServer {
    engine: OmenEngine,
    metrics: PrometheusMetrics,
    tracer: OpenTelemetryTracer,
}

impl OmenServer {
    async fn search(&self, query: Vector) -> Result<Vec<SearchResult>> {
        let span = self.tracer.span("search");
        let timer = self.metrics.search_duration.start_timer();
        
        // Call engine (which has minimal overhead)
        let results = self.engine.search(query)?;
        
        // Server layer handles all metrics
        timer.observe_duration();
        self.metrics.searches_total.inc();
        span.record_result(&results);
        
        Ok(results)
    }
}
```

## Implementation Priority

### Phase 1: Engine Stats API (v0.1.0)
- [x] Basic memory stats (already done)
- [ ] Operation counters (cheap)
- [ ] Stats enable/disable flag
- [ ] Clean stats struct

### Phase 2: Server Implementation (v0.2.0)
- [ ] Rust server with Prometheus
- [ ] Health check endpoints
- [ ] Grafana dashboards
- [ ] OpenTelemetry integration

### Phase 3: Advanced (v0.3.0)
- [ ] Custom trace points
- [ ] Performance profiling hooks
- [ ] Query plan analysis
- [ ] Resource limits

## Key Design Decisions

1. **Stats OFF by default** - Zero overhead for embedded mode
2. **No external deps in engine** - Keep Mojo code pure
3. **Server owns complexity** - All integrations at Rust layer
4. **Minimal API surface** - Just enough hooks for server

## Comparison with Competitors

| Database | Engine Overhead | Server Observability |
|----------|----------------|---------------------|
| OmenDB   | None (opt-in)  | Full (Rust layer)   |
| SQLite   | None           | External only       |
| DuckDB   | Minimal        | External mostly     |
| Redis    | Minimal        | Extensive           |
| Postgres | Moderate       | Extensive           |
| Qdrant   | Built-in       | Built-in            |

We're following the Redis model: clean engine, observable server.

## Next Steps

1. **Immediate**: Fix 35K vector limit (core functionality)
2. **Then**: Add basic stats API to engine
3. **Later**: Build Rust server with full observability
4. **Future**: Enterprise features at server layer

This architecture ensures embedded users get maximum performance while enterprise users get full observability - without compromise.