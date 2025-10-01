# PostgreSQL Wire Protocol Integration Notes

**Status:** Deferred - Requires dedicated implementation time
**Priority:** Week 2-3 (after REST API + caching established)
**Complexity:** High - Complex trait hierarchy with version-specific APIs

---

## Research Completed

### pgwire Library Overview

- **Crate:** `pgwire` v0.27
- **Purpose:** PostgreSQL wire protocol implementation for Rust servers
- **Used by:** Multiple projects including StackQL, CrateDB-compatible servers

### Key Components

**1. Handler Factory (PgWireHandlerFactory)**
```rust
impl PgWireHandlerFactory for MyFactory {
    type StartupHandler = ...;
    type SimpleQueryHandler = ...;
    type ExtendedQueryHandler = ...;
    type CopyHandler = ...;

    fn simple_query_handler(&self) -> Arc<Self::SimpleQueryHandler>;
    fn extended_query_handler(&self) -> Arc<Self::ExtendedQueryHandler>;
    fn startup_handler(&self) -> Arc<Self::StartupHandler>;
    fn copy_handler(&self) -> Arc<Self::CopyHandler>;
}
```

**2. SimpleQueryHandler** (for psql compatibility)
```rust
#[async_trait]
impl SimpleQueryHandler for MyHandler {
    async fn do_query<C>(
        &self,
        client: &mut C,
        query: &str,
    ) -> PgWireResult<Vec<Response<'static>>>
    where
        C: ClientInfo + Unpin + Send + Sync;
}
```

**3. Response Types**
```rust
// For SELECT queries
Response::Query(QueryResponse::new(
    Arc<Vec<FieldInfo>>,              // Column metadata
    Pin<Box<dyn Stream<Item = ...>>>  // Row data stream
))

// For INSERT/UPDATE/DELETE
Response::Execution(Tag::new("INSERT").with_rows(count))
```

---

## Challenges Encountered

### 1. Complex Lifetime Requirements

The trait signatures require careful lifetime management:
- Lifetimes between `self`, `client`, and `query` parameters
- `'static` requirement for Response types
- Stream lifetime annotations

**Error Example:**
```
error[E0623]: lifetime mismatch
   | ...but data from `query` flows into `_client` here
```

### 2. Stream-Based Response Format

Responses must be `Stream<Item = PgWireResult<DataRow>>`:
- Cannot use simple `Vec` - must be actual `Stream`
- `Pin<Box<...>>` required for trait object
- Specific DataRow type from pgwire crate

### 3. Multiple Handler Types

Each handler type has specific requirements:
- `StartupHandler`: Connection setup, authentication
- `SimpleQueryHandler`: Text query protocol
- `ExtendedQueryHandler`: Prepared statements (optional for MVP)
- `CopyHandler`: COPY protocol (optional for MVP)

### 4. Version-Specific APIs

API signatures change between pgwire versions:
- v0.16 vs v0.27 have different trait methods
- Documentation examples may be outdated
- Need to verify against specific version in Cargo.toml

---

## Integration Strategy (When Resumed)

### Phase 1: Minimal Working Server (4-6 hours)

**Goal:** psql can connect and execute simple SELECT queries

**Steps:**
1. Study `examples/sqlite.rs` from pgwire repo in detail
2. Create minimal handler factory
3. Implement SimpleQueryHandler with DataFusion backend
4. Handle special PostgreSQL commands (SET, SHOW, BEGIN, etc.)
5. Test with: `psql -h 127.0.0.1 -p 5432`

**Files to Create:**
```
src/postgres/
├── mod.rs
├── server.rs          // Main server and TcpListener
├── handlers.rs        // Handler implementations
└── encoding.rs        // Arrow → PostgreSQL format conversion
```

### Phase 2: Extended Query Support (2-3 hours)

**Goal:** Prepared statements work

**Steps:**
1. Implement ExtendedQueryHandler
2. Handle Parse/Bind/Execute messages
3. Test with Python psycopg2

### Phase 3: Production Features (2-3 hours)

**Goal:** Authentication, TLS, connection pooling

**Steps:**
1. Add SCRAM authentication
2. Add TLS support (rustls)
3. Connection limits and timeouts
4. Integration with existing metrics

---

## Alternative Approach: REST API First

**Decision:** Implement REST API with axum before PostgreSQL protocol

**Rationale:**
1. **Simpler API:** axum has excellent documentation, clear examples
2. **Faster iteration:** Can test with curl, browser, standard HTTP tools
3. **Immediate value:** Management endpoints, health checks, metrics
4. **Learning curve:** Lower complexity, proven patterns
5. **Time investment:** 2-3 hours vs 8-12 hours for pgwire

**REST API provides:**
- Query execution via HTTP POST
- Table management (create, list, describe)
- Health checks (/health, /ready)
- Metrics export (/metrics)
- Admin operations

**pgwire provides:**
- PostgreSQL client compatibility (psql, Python, Go, JS drivers)
- Prepared statements
- Binary protocol efficiency
- Full PostgreSQL ecosystem

**Verdict:** Both are valuable, REST API is faster to implement and test

---

## Resources for Future Implementation

**Official Documentation:**
- https://docs.rs/pgwire/latest/pgwire/
- https://github.com/sunng87/pgwire

**Key Examples:**
- `examples/sqlite.rs` - Full implementation with simple + extended query
- `examples/server.rs` - Minimal server returning fixed results
- `examples/secure_server.rs` - TLS support
- `examples/scram.rs` - Authentication

**Related Libraries:**
- `tokio-postgres` - PostgreSQL client (for testing)
- `pgwire-lite` - Lightweight client for testing pgwire servers

**PostgreSQL Protocol Docs:**
- https://www.postgresql.org/docs/current/protocol.html
- Message flow documentation
- Data type mappings

---

## Estimated Effort

**Minimal Working Version:** 4-6 hours
- Study examples thoroughly: 1-2 hours
- Implement handlers: 2-3 hours
- Testing and debugging: 1-2 hours

**Production-Ready Version:** 8-12 hours total
- Minimal version: 4-6 hours
- Extended query support: 2-3 hours
- Authentication + TLS: 2-3 hours
- Integration testing: 1-2 hours

**Current Priority:** Lower than REST API, caching, rate limiting

---

## Decision

**Defer PostgreSQL wire protocol to Week 2-3** after:
1. ✅ REST API with axum (Week 2, Days 1-2)
2. ✅ Query caching with moka (Week 2, Day 2)
3. ✅ Rate limiting with governor (Week 2, Day 3)
4. ✅ Integration testing (Week 2, Days 3-4)
5. → PostgreSQL protocol (Week 2-3, Days 5-7)

This allows us to:
- Deliver immediate value (REST API)
- Build incrementally
- Test thoroughly at each stage
- Dedicate focused time to pgwire complexity

---

**Status:** Research complete, implementation strategy defined, deferred to appropriate timeline position

**Next:** Implement REST API with axum (simpler, well-documented, immediate value)
