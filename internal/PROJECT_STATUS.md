# OmenDB Project Status
## September 26, 2025 - Final Strategy Locked

## 📍 **WHERE WE ARE**

### **Decision: Building OmenDB (Pure Learned Index Database)**
- **Product**: World's first database using ONLY learned indexes
- **Market**: Time-series databases ($8B market)
- **Tech Stack**: Full Rust (maximum performance)
- **Timeline**: 6 weeks to YC S26 application
- **Goal**: 3 pilot customers + working demo

### **Backup Plan: Documented**
- **Pivot Option**: Hybrid platform (DuckDB + learned optimization)
- **Decision Point**: End of Week 2 if pure approach fails
- **Pivot Doc**: `internal/PIVOT_PLAN.md` has full details

---

## 📋 **WHAT'S BUILT**

### **Research & Planning ✅**
- Comprehensive market analysis ($104.5B database market)
- Competition research (CockroachDB $5B, SingleStore $500M exit)
- Technical validation (LearnedKV 4.32x speedup papers)
- Business model defined (SaaS $500-10K/month)

### **Code Assets**
```
✅ pg-learned/          - PostgreSQL extension (marketing/credibility)
✅ proof_of_concept.py  - Initial learned index validation
✅ learneddb/src/       - Rust learned index prototype (needs porting)
❌ omendb-rust/        - Main implementation (TO BUILD)
```

### **Documentation ✅**
```
internal/
├── STRATEGY_FINAL.md      - Locked strategy for OmenDB
├── PIVOT_PLAN.md         - Detailed backup plan (hybrid)
├── BUSINESS_ANALYSIS.md  - Market research & validation
├── research/             - Academic papers & analysis
└── PROJECT_STATUS.md     - This file
```

---

## 🎯 **WEEK-BY-WEEK PLAN**

### **Week 1-2: Core Learned Index** (Sept 27 - Oct 10)
```rust
Focus: Get learned indexes working in Rust
- Port hierarchical model from Python research
- Achieve <100ns lookup on 10M keys
- Basic insert/search operations
- Benchmark vs B-tree

Success Metric: 10x faster lookups proven
Decision Point: Pivot if not achieving targets
```

### **Week 3-4: Storage & Queries** (Oct 11 - Oct 24)
```rust
Focus: Time-series optimized storage
- Arrow columnar integration
- Range queries on learned index
- Basic aggregations (sum, avg, min, max)
- Time-bucketing functions

Success Metric: 100M rows, <1s aggregation
```

### **Week 5: PostgreSQL Protocol** (Oct 25 - Oct 31)
```rust
Focus: Make it usable
- PostgreSQL wire protocol
- Connect from any SQL client
- Docker deployment ready
- Basic monitoring

Success Metric: Grafana can connect
```

### **Week 6: Launch & Customers** (Nov 1 - Nov 7)
```rust
Focus: Get traction
- Launch on HackerNews
- Direct outreach to time-series users
- Get 3 pilot commitments
- Submit YC application

Success Metric: 3 LOIs + 100 HN upvotes
```

---

## 💻 **NEXT IMMEDIATE TASKS**

### **Today/Tomorrow (Sept 26-27)**
1. [ ] Set up Rust project structure for omendb-rust
2. [ ] Port basic learned index from Python to Rust
3. [ ] Create benchmark harness (learned vs B-tree)

### **This Weekend (Sept 28-29)**
1. [ ] Get hierarchical model working
2. [ ] Achieve first 10x speedup proof
3. [ ] Decision: Continue or pivot

---

## 🚧 **KNOWN RISKS**

| Risk | Impact | Mitigation |
|------|--------|------------|
| Learned index doesn't scale | Fatal | Pivot to hybrid (Plan B ready) |
| Rust complexity slows us down | High | Use simpler architecture, consider Python |
| No customer interest | Fatal | PostgreSQL compatibility ensures easy trials |
| YC deadline pressure | High | Focus on demo over production quality |

---

## 📊 **SUCCESS CRITERIA**

### **Technical Success**
- [ ] 10x faster than B-tree on time-series
- [ ] <100ns lookup latency achieved
- [ ] 1M+ inserts/second capability
- [ ] PostgreSQL wire protocol working

### **Business Success**
- [ ] 3 pilot customers (LOIs signed)
- [ ] 100+ HackerNews upvotes
- [ ] Working demo at omendb.io
- [ ] YC application submitted

---

## 🔄 **PIVOT TRIGGERS**

**We pivot to hybrid approach if:**
- Week 2: Learned index performance < 5x improvement
- Week 3: Can't handle range queries efficiently
- Week 4: Zero customer interest
- Week 5: Technical complexity too high

**Pivot is pre-planned**: See `internal/PIVOT_PLAN.md`

---

## 📝 **NOTES & REMINDERS**

### **Technical Decisions**
- **Language**: Rust (not Python) for performance
- **Storage**: Arrow format for compatibility
- **Protocol**: PostgreSQL for instant adoption
- **GPU**: RAPIDS initially, custom CUDA later

### **Business Decisions**
- **Model**: SaaS only (not open source initially)
- **Pricing**: Usage-based ($500-10K/month)
- **Market**: Time-series first, expand later
- **Brand**: OmenDB (not LearnedDB)

### **What We're NOT Building**
- ❌ Full OLTP support
- ❌ Distributed system
- ❌ Complete SQL
- ❌ ACID guarantees

---

## 🚀 **LET'S BUILD**

**Focus for next 48 hours**: Get learned indexes working in Rust

**Mantra**: "Ship something that works, not everything that's possible"

---

*Updated September 26, 2025*
*Next review: September 28 (Week 1 checkpoint)*