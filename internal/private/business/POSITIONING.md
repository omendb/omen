# OmenDB Positioning Strategy
*Internal Document - Not for Public Distribution*

## Product Architecture

### Core Engine (Mojo) - Open Source
- **What**: High-performance vector database engine
- **License**: Elastic License 2.0 (source available, free for most uses)
- **Use cases**: Embedded applications, edge AI, local RAG, development, production
- **Distribution**: PyPI, GitHub
- **Note**: Free to use, modify, and distribute (except as managed service)

### Server Mode - Commercial (Future)
- **What**: Distributed vector database service
- **License**: Commercial (details TBD)
- **Use cases**: Scale-out deployments, multi-tenant, high availability
- **Distribution**: TBD
- **Note**: For users needing distributed features beyond single-node engine

## Positioning Strategy

### Current Messaging (Embedded/Open Source)
**"High-performance vector database engine for AI applications"**
- Focus on the ENGINE, not just embedded
- Emphasize performance and efficiency
- Avoid limiting to embedded-only use cases

### Future Messaging (Server/Commercial)
**"Production vector database platform powered by OmenDB engine"**
- Built on same proven engine
- Adds distribution, HA, multi-tenancy
- Natural upgrade path from embedded

## Avoiding Customer Confusion

### DO Say:
- "OmenDB engine" (not "embedded database")
- "Deploy anywhere - from edge to cloud"
- "Start local, scale global"
- "Zero-dependency engine"
- "Production-ready vector operations"

### DON'T Say:
- "The SQLite of vector databases" (too limiting)
- "Embedded-only" (closes future doors)
- "Just for development" (undermines engine quality)
- "Alternative to Pinecone" (until server mode ready)

## Competitive Positioning

### Against Embedded Databases:
- **ChromaDB**: We're faster and more memory efficient
- **LanceDB**: We're truly embedded (they require Arrow)
- **Faiss**: We're a complete database, not just an index

### Against Cloud Services (future):
- **Pinecone**: Same performance, your infrastructure
- **Weaviate**: Simpler, faster, lower TCO
- **Qdrant**: Better memory efficiency, easier operations

## Migration Path Story

```
Developer Journey:
1. Downloads OmenDB engine for local RAG prototype
2. Builds POC with single-node deployment
3. Deploys to production with OmenDB engine
4. Hits scale limits (needs distribution, HA, etc.)
5. Upgrades to commercial server mode (when available)
```

## Technical Differentiation

### Engine (Both Modes):
- 156 bytes/vector (2.5x better than Microsoft DiskANN)
- 92K vectors/second ingestion
- 1.8ms search latency
- Product quantization (16x compression)

### Server Mode (Additional):
- Horizontal scaling
- Multi-tenancy
- ACID transactions
- Point-in-time recovery
- Role-based access control

## Documentation Strategy

### Public Docs:
- Focus on engine capabilities
- Show both embedded and "future server" examples
- Use "OmenDB engine" consistently
- Include "scaling to production" section (teasing server mode)

### Architecture Docs:
```
OmenDB Architecture:
├── Engine Layer (Mojo) - Open Source
│   ├── Vector operations
│   ├── Index management  
│   ├── Storage engine
│   └── Query execution
├── API Layer
│   ├── Python bindings (embedded)
│   └── REST/gRPC (server) - Coming Soon
└── Distribution Layer (Rust) - Commercial
    ├── Cluster coordination
    ├── Replication
    ├── Sharding
    └── Multi-tenancy
```

## Licensing Philosophy

### Engine (Elastic License 2.0):
- Free for internal use
- Free for embedded applications
- Free for SaaS (using, not offering as service)
- Prevents direct competition as managed service
- Builds adoption while protecting business model

### Server Mode (Commercial - Future):
- For distributed deployments
- Enterprise features and support
- Details to be determined based on market needs

## Red Flags to Avoid

1. **Don't position as embedded-only** - Limits growth potential
2. **Don't compare directly to SQLite** - Different scale/use case
3. **Don't promise server features in engine** - Keep separation clear
4. **Don't use "database" alone** - Use "engine" or "platform"
5. **Don't hide commercial intentions** - Be transparent about roadmap

## Recommended Tagline

**"High-performance vector engine. Deploy anywhere."**

- Doesn't limit to embedded
- Emphasizes core strength (performance)
- Hints at flexibility (edge to cloud)
- Professional and enterprise-friendly