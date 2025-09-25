# OmenDB Current Status

## 🎯 FOCUS: PostgreSQL Extension with Learned Indexes

**Pivot Date**: September 25, 2025
**Target**: 10x faster than B-trees via PostgreSQL extension
**Deadline**: Oct 7 - Go/No-Go decision
**YC Application**: November 10, 2025 (45 days)

---

**Last Updated**: September 25, 2025 (15:30 PST)

## Current State

### ✅ Completed
- Strategic pivot to learned databases
- Documentation consolidated (5 core files)
- Research papers organized (external/papers/)
- Monetization strategy defined
- Architecture focused on PG extension only

### 🚧 In Progress
- [ ] Simple linear RMI implementation
- [ ] PostgreSQL extension setup (pgrx)
- [ ] Benchmark vs BTreeMap

### 🎯 Next 48 Hours
1. Implement linear model on sorted array
2. Achieve 5x performance or pivot
3. Start PostgreSQL wrapper if successful

## Key Decisions Made

### Architecture
- **PostgreSQL Extension ONLY** (no embedded/server modes)
- **Linear models first** (neural networks later)
- **Delta buffer** for updates (ALEX approach)

### Monetization
- **Free tier**: Basic learned indexes
- **Enterprise**: $50-200K/year (monitoring, auto-retrain)
- **Cloud**: Future SaaS after traction

### Performance Targets
- **Minimum viable**: 5x faster than B-tree
- **YC demo**: 10x faster
- **Extension overhead**: ~20% (acceptable)

## Success Metrics

### Oct 7 Checkpoint (Go/No-Go)
- [ ] 10x performance demonstrated
- [ ] PostgreSQL CREATE INDEX working
- [ ] Benchmark results documented

### Nov 1 Target (YC Submit)
- [ ] Demo video showing 10x
- [ ] 100+ GitHub stars
- [ ] Application submitted early

### Nov 10 Deadline (YC Final)
- [ ] Polished application
- [ ] Working prototype public
- [ ] Community momentum

## Current Blockers

### Technical
- Need to prove 10x performance (no code yet)
- PostgreSQL integration complexity unknown
- Update handling strategy unclear

### Business
- No ML co-founder identified
- No customer validation yet
- Limited runway (self-funded)

## Daily Log

### Sept 25, 2025
- ✅ Pivoted from vector DB to learned indexes
- ✅ Consolidated 15+ docs to 5 core files
- ✅ Defined PostgreSQL extension focus
- ✅ Created research paper repository
- ⏳ Started RMI implementation (pending)

### Sept 26, 2025
- [ ] Complete linear model prototype
- [ ] Run first benchmarks
- [ ] Make go/no-go on continuing

## Resource Allocation

### Time (Daily)
- 50% coding (RMI implementation)
- 20% benchmarking
- 20% PostgreSQL integration
- 10% documentation

### Mental Energy
- Morning: Core algorithm (hardest)
- Afternoon: Integration work
- Evening: Research papers

## The One Metric That Matters

**Lookup latency: Must be <40ns (10x faster than 200ns B-tree)**

If we can't hit this, everything else is irrelevant.

---

*"Ship PostgreSQL extension with 10x demo by Oct 7 or pivot."*