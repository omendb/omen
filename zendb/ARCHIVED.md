# ⚠️ ARCHIVED PROJECT

## ZenDB - Archived Feb 5, 2025

### Status: ARCHIVED - Do not use for new development

### Why Archived
- Pivoted to multimodal OmenDB with HNSW+ in Mojo
- ZenDB patterns (MVCC, SQL) will be ported to OmenDB
- Focusing resources on single multimodal database

### Valuable Patterns Preserved
The following patterns from ZenDB are being migrated to OmenDB:
- MVCC transaction management
- SQL query planning
- Hybrid storage architecture
- Test patterns (61/70 passing tests)

### For AI Agents
**DO NOT use this code for new development.**

Instead:
1. Focus on `/omendb/` directory
2. Reference `/internal/architecture/MULTIMODAL.md`
3. Port valuable patterns to Mojo as needed

### Original Description
ZenDB was a Rust-based hybrid database combining:
- ACID transactions with MVCC
- SQL query interface
- Vector search capabilities
- Time-travel queries

---
*This codebase is preserved for reference only. All new development happens in OmenDB.*