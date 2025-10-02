# VC/YC Funding Strategy for OmenDB

**Date:** October 2, 2025
**Purpose:** Detailed monetization, funding paths, and algorithm-first strategy analysis

---

## Executive Summary: Two Viable Paths

After deep research into successful database companies, there are **TWO proven paths**:

**Path A: Better Algorithms → VC Funding**
- Examples: DuckDB, ClickHouse, QuestDB
- Strategy: Build 10-100x faster database through superior algorithms
- Timeline: 6-12 months to traction, then raise Series A
- Risk: High technical risk, but HUGE upside if it works
- **This IS a viable path - don't dismiss it**

**Path B: Niche Market → Bootstrapped → VC**
- Examples: Supabase, Neon, PlanetScale
- Strategy: Solve specific problem exceptionally well, then expand
- Timeline: 3-6 months to first revenue, 12-18 months to raise
- Risk: Lower technical risk, but slower growth

**Key Insight:** You're right that better algorithms CAN win. DuckDB and ClickHouse prove this. The question is: **which algorithms, and how do we validate them quickly?**

---

## Part 1: Detailed Monetization Models

### Model 1: Open Core (Most Common for Databases)

**How it works:**
- Core database: Open source (Apache 2.0 or MIT)
- Enterprise features: Proprietary, paid only

**Revenue Tiers:**

```
FREE (Open Source)
├── Core database engine
├── PostgreSQL wire protocol
├── Vector search (pgvector)
├── Embedded deployment
├── Community support (Discord, GitHub)
└── Up to 10GB data

STARTER: $29-99/month
├── Everything in Free
├── Cloud sync service (backup + multi-device)
├── Point-in-time recovery (7 days)
├── Email support
├── Up to 100GB data
└── 5 devices

PRO: $299-999/month
├── Everything in Starter
├── Advanced replication
├── Read replicas
├── Point-in-time recovery (30 days)
├── Priority support (24/7)
├── Unlimited devices
└── Up to 1TB data

ENTERPRISE: Custom ($5K-50K/month)
├── Everything in Pro
├── Multi-region deployment
├── Custom SLA (99.99% uptime)
├── Dedicated support engineer
├── On-premises deployment
├── Custom features/integrations
└── Unlimited data
```

**What's Proprietary (Paid Only):**
- Multi-region replication
- Advanced monitoring dashboard
- Compliance features (SOC2, HIPAA, GDPR tooling)
- Role-based access control (RBAC)
- Audit logging
- Advanced backup/recovery (point-in-time to any second)

**Example Revenue Model:**
- 10,000 free users
- 300 Starter ($49/mo avg) = $14,700/month
- 50 Pro ($499/mo avg) = $24,950/month
- 10 Enterprise ($10K/mo avg) = $100,000/month
- **Total: $140K MRR = $1.68M ARR**

**Why VCs Like This:**
- Proven model (MongoDB, Elastic, CockroachDB, etc.)
- Open source = free marketing (GitHub stars)
- Clear upgrade path (free → paid)
- Sticky customers (hard to switch databases)

### Model 2: Cloud-First SaaS (Fastest Revenue)

**How it works:**
- Fully managed cloud service
- Usage-based pricing (compute + storage)
- Open source, but cloud is the main product

**Revenue Tiers:**

```
FREE
├── 500MB storage
├── 1GB bandwidth
├── Community support
└── Public projects

HOBBY: $25/month
├── 8GB storage
├── 100GB bandwidth
├── Email support
└── Unlimited projects

PRO: $25/month base + usage
├── $0.10 per GB storage/month
├── $0.09 per GB bandwidth
├── Dedicated compute resources
├── Priority support
└── Advanced features (replicas, backups)

ENTERPRISE: Custom
├── Volume discounts
├── SLA (99.99%)
├── Dedicated infrastructure
├── Private deployments
└── Custom contracts
```

**Real-World Example (Neon's Pricing):**
- Free: 10 branches, 3GB storage
- Launch: $19/mo + $0.16/compute hour
- Scale: $69/mo + usage
- Enterprise: Custom

**Why This Works:**
- **Immediate revenue** (customers pay from day 1)
- No self-hosting support burden
- Predictable infrastructure costs
- Easy to upsell (storage, compute, bandwidth)

**Revenue Example (Year 1):**
- Month 3: 100 free, 10 paid ($250 MRR)
- Month 6: 500 free, 100 paid ($5K MRR)
- Month 12: 2000 free, 500 paid ($25K MRR)
- **Year 1: $15K-25K MRR**

### Model 3: Hybrid (Open Source + Managed Service)

**How it works:**
- Core database: Fully open source
- Managed service: Paid (handles complexity)
- Both self-hosted and cloud options

**Revenue Streams:**

1. **Managed Database Service** ($29-999/month)
   - Like Supabase, but for embedded databases
   - Auto-scaling, backups, monitoring
   - One-click deployment

2. **Sync/Replication Service** ($29-299/month)
   - For local-first applications
   - Edge sync protocol
   - Conflict resolution

3. **Enterprise Support** ($2K-20K/month)
   - SLA (99.9% or 99.99%)
   - Dedicated engineer
   - Custom features
   - Priority fixes

4. **Training & Consulting** ($5K-50K/project)
   - Migration services
   - Performance optimization
   - Architecture review

**Revenue Mix (Mature Company):**
- 60% from managed service
- 25% from enterprise support
- 10% from consulting
- 5% from training

**Example: Supabase (Hybrid Model Success)**
- 2024 Revenue: $31M
- 2025 Projected: $65M (109% YoY growth)
- Team: 127 people
- Funding: $395M raised
- Model: Open source core + managed service + enterprise

**Why VCs LOVE This:**
- Open source = massive user base
- Managed service = recurring revenue
- Enterprise = high LTV customers
- Multiple revenue streams = less risk

### Model 4: Commercial Open Source (COSS)

**Research Finding:** Commercial open source companies average **7x greater valuations at IPO** and **14x at M&A** vs closed-source peers.

**Success Metrics (from Linux Foundation 2025 Report):**
- Median IPO valuation: $1.3B (vs $171M closed-source)
- Median M&A valuation: $482M (vs $34M closed-source)
- Total COSS funding 2024: $26.4B
- Funding velocity: 20-34% faster for COSS vs proprietary

**Key Insight:** Open source + commercial model OUTPERFORMS proprietary software.

**COSS Monetization Patterns:**

1. **Open Core** (60% of COSS companies)
   - Examples: MongoDB, Elastic, Redis
   - Free: Core features
   - Paid: Enterprise features

2. **Managed Service** (30% of COSS companies)
   - Examples: Supabase, Neon, PlanetScale
   - Free: Self-hosted
   - Paid: Managed cloud

3. **Support + SLA** (10% of COSS companies)
   - Examples: Red Hat, SUSE
   - Free: Software
   - Paid: Support contracts

**Which Model for OmenDB?**

**Recommendation: Hybrid (Model 3)**
- Open source core (PostgreSQL + vectors + embedded)
- Managed sync service (recurring revenue)
- Enterprise support (high margin)
- Consulting (bootstrap revenue)

**Why:**
- Fastest path to revenue (sync service)
- Open source = marketing + community
- Multiple revenue streams = less risk
- Clear upgrade path for all customer sizes

---

## Part 2: VC/YC Funding Path

### What VCs Look For in Database Startups

Based on research of YC-funded database companies (Lantern, Fortress, PeerDB, Spiral):

**1. Clear Market Problem**
- NOT: "We have a faster database"
- BUT: "Developers waste 10 hours/week managing database sync"

**2. 10x Better, Not 10% Better**
- Performance: 10x faster (not 2x)
- Cost: 90% cheaper (not 20%)
- Complexity: 10x simpler (not slightly easier)

**3. Unfair Advantage**
- Novel algorithm (learned indexes?)
- Deep domain expertise (former Timescale engineer?)
- Network effects (community?)

**4. Early Traction**
- GitHub stars: 1K-5K+ (proves interest)
- Active users: 100-1000+ (proves usage)
- Revenue (optional but strong): $5K-20K MRR

**5. Large TAM (Total Addressable Market)**
- NOT: "Rust developers need embedded DB" ($100M TAM)
- BUT: "AI applications need vector storage" ($4B TAM by 2028)

### YC Application Strategy

**YC Batch Stats (2024):**
- Applications: ~20,000
- Interviews: ~3% (600 companies)
- Accepted: ~1.5% (300 companies)
- **Odds: 1 in 67**

**What YC Wants to See:**

1. **One Sentence Description**
   - Bad: "PostgreSQL-compatible database with learned indexes"
   - Good: "Embedded PostgreSQL for AI apps - runs on laptop, includes vectors"

2. **Problem (Clear & Painful)**
   - Bad: "Databases are slow"
   - Good: "Developers building AI apps need PostgreSQL + vectors, but Supabase requires cloud and costs $200/month. We run locally, cost $0."

3. **Solution (Simple & Obvious)**
   - Bad: "Hybrid HTAP with recursive model indexes"
   - Good: "Single binary, PostgreSQL compatible, includes pgvector, works offline"

4. **Traction (Concrete Numbers)**
   - Bad: "Growing fast"
   - Good: "1,200 GitHub stars, 300 active users, $2K MRR from 20 paying customers (launched 2 months ago)"

5. **Why You? (Unfair Advantage)**
   - Bad: "We're passionate about databases"
   - Good: "Ex-Timescale engineer, built pgvector at scale, 10 years database expertise"

6. **Why Now? (Market Timing)**
   - Bad: "Databases are important"
   - Good: "AI boom created 100,000 developers building RAG apps in last 6 months. They all need vector storage."

**Example YC-Style Pitch:**

> **OmenDB: PostgreSQL + Vectors, Zero Cloud Required**
>
> **Problem:** Developers building AI/RAG apps want PostgreSQL + pgvector, but all solutions (Supabase, Neon, Timescale) require cloud hosting ($50-500/month). For local development, edge deployments, and privacy-focused apps, this doesn't work.
>
> **Solution:** Single binary (30MB) with PostgreSQL wire protocol + pgvector built-in. Works offline, costs $0 for core features. Optional sync service for backups/multi-device.
>
> **Traction:** Launched 2 months ago. 1,200 GitHub stars, 300 active users, 20 paying sync customers ($2K MRR). Growing 40% month-over-month.
>
> **Why us:** Ex-Timescale engineer, built pgvector at 10TB scale. Rust expert (5 years). Shipped 3 database projects (2 acquired).
>
> **Why now:** AI app development exploded in 2024. LangChain has 80K+ stars. LlamaIndex 30K+ stars. Every one of those developers needs vector storage. Market is $4B by 2028 (Gartner).
>
> **Ask:** $500K to hire 2 engineers, build enterprise features, and scale to $100K MRR in 12 months.

### Funding Timeline

**Pre-Seed/YC Path:**

**Month 0-3: Build & Launch**
- Build MVP (embedded PostgreSQL + vectors)
- Launch on Hacker News, Product Hunt
- Target: 500-1000 GitHub stars, 50-100 active users

**Month 3-6: Iterate & Grow**
- Talk to users, find product-market fit
- Add most-requested features
- Launch paid sync service
- Target: 2K GitHub stars, 300 users, $2K-5K MRR

**Month 6-9: Apply to YC**
- Strong traction metrics
- Apply to YC (or AngelList, On Deck, etc.)
- Target: $5K-10K MRR, 5K stars, 1000 users

**Month 9-12: YC Batch (if accepted)**
- $500K investment ($125K + $375K on standard terms)
- 3 months of intense growth
- Target: $25K-50K MRR by Demo Day

**Month 12-18: Series A**
- Raise $3-10M Series A
- Valuation: $20-50M (based on $300K-600K ARR)
- Use funds to hire team, build enterprise features

### Alternative: Angel/Seed (No YC)

If you don't get into YC or want to raise directly:

**Angel Round: $250K-500K**
- 10-20 angels @ $25K-50K each
- Valuation: $2-5M (10-20% dilution)
- Use: Extend runway, hire 1-2 people

**Seed Round: $1-3M**
- Lead investor + syndicate
- Valuation: $5-15M (15-25% dilution)
- Use: Build team, scale marketing

**How to Find Investors:**

1. **Twitter/X:** Build in public, share metrics
2. **AngelList:** Create profile, apply to syndicates
3. **Intro network:** Warm intros from other founders
4. **Database community:** MongoDB, Timescale, Supabase angels
5. **YC Alumni:** Even if you don't get in, they invest

**Investor Targets (Database-Friendly VCs):**
- **Battery Ventures** (invested in: Cockroach, QuestDB)
- **8VC** (invested in: PeerDB)
- **Insight Partners** (invested in: Timescale, Vanta)
- **Redpoint Ventures** (invested in: HashiCorp, Stripe)
- **Felicis Ventures** (invested in: Notion, Plaid)

---

## Part 3: The "Better Algorithms" Strategy

### You're Right - This CAN Work

Examples that prove algorithm-first strategy works:

**1. DuckDB (Columnar + Vectorization)**
- **Innovation:** Columnar storage + vectorized execution + optimized for analytics
- **Result:** 10-100x faster than SQLite on analytical queries
- **Traction:** 15K GitHub stars, used by major companies
- **Funding:** Bootstrapped initially, now backed by Motherduck ($52.5M raised)
- **Key:** Solved a CLEAR problem (analytics on laptop) with measurably better performance

**2. ClickHouse (Columnar + Distributed)**
- **Innovation:** Columnar storage + distributed architecture + specialized for real-time analytics
- **Result:** 100-1000x faster than PostgreSQL on analytical queries
- **Traction:** 30K+ GitHub stars, $250M+ funding
- **Key:** Not just faster, but **insanely** faster (100x, not 2x)

**3. QuestDB (Time-Series Optimized)**
- **Innovation:** Custom storage engine optimized for time-series + fast SQL
- **Result:** 10x faster ingestion than InfluxDB, 4x faster queries
- **Traction:** 13K GitHub stars, $15M Series A
- **Key:** Specialized algorithm for ONE workload (time-series)

### Why Better Algorithms Won (Pattern Analysis)

All three share these traits:

1. **10-100x improvement, not 2x**
   - DuckDB: 100x faster than SQLite for analytics
   - ClickHouse: 1000x faster than MySQL for OLAP
   - QuestDB: 10x faster than InfluxDB

2. **Measurable in 30 seconds**
   - User can run benchmark immediately
   - Difference is OBVIOUS (not subtle)
   - "Holy shit" moment within first use

3. **Solved specific workload exceptionally well**
   - DuckDB: Analytics on laptop
   - ClickHouse: Real-time analytics at scale
   - QuestDB: Time-series ingestion + queries

4. **Existing alternatives were terrible**
   - DuckDB: SQLite too slow for analytics
   - ClickHouse: MySQL/PostgreSQL terrible for OLAP
   - QuestDB: InfluxDB slow, InfluxQL confusing

### Could OmenDB Win with Better Algorithms?

**YES, if you can prove:**

1. **10-100x speedup on specific workload**
   - NOT: "2x faster than PostgreSQL generally"
   - BUT: "100x faster than pgvector on 1M vector search"
   - OR: "10x faster than SQLite on time-series inserts with learned indexes"

2. **Measurable in benchmarks**
   - Reproducible benchmarks
   - Run on standard laptop
   - Results obvious within 1 minute

3. **Addresses real pain point**
   - NOT: "Databases could be faster"
   - BUT: "Vector search on 1M embeddings takes 5 seconds in pgvector, 50ms in OmenDB"

### Potential Algorithm Strategies for OmenDB

**Option 1: Best Vector Search Performance**

**Strategy:** Beat pgvector/Pinecone on speed + recall

**How:**
- Implement state-of-art vector index (DiskANN, HNSW++)
- Optimize for Rust (zero-copy, SIMD)
- Prove 10-100x faster than pgvector on 1M+ vectors

**Benchmarks to Win:**
- SIFT1M (1M 128-dim vectors) - standard benchmark
- Target: <10ms p99 latency, 95%+ recall
- Beat: pgvector (50-100ms), Pinecone (20-50ms)

**Market:** $4B vector DB market by 2028

**Example Pitch:**
> "pgvector is slow. 1M vectors = 5 second queries. We're 100x faster (50ms) by using StreamingDiskANN + Rust optimization. Same PostgreSQL interface, drop-in replacement."

**VC Appeal:** Clear 100x improvement, measurable, huge market

**Option 2: Fastest Embedded Time-Series DB**

**Strategy:** Beat SQLite on time-series workloads using learned indexes

**How:**
- Learned index optimized for sorted sequential writes
- Prove 10x faster inserts than SQLite
- Prove 10x faster range queries

**Benchmarks to Win:**
- 1M time-series inserts/sec (vs SQLite 100K/sec)
- Range query (1 day of data from 1 year): <10ms vs 100ms

**Market:** $1.45B → $4.42B (2024-2033)

**Example Pitch:**
> "IoT devices generate time-series data. SQLite can't keep up - only 100K inserts/sec. Learned indexes get us to 1M inserts/sec (10x faster). Same SQLite API."

**VC Appeal:** Clear technical advantage, IoT market huge

**Option 3: Hybrid OLTP/OLAP with Learned Placement**

**Strategy:** Use ML to auto-optimize hot/cold data placement

**How:**
- Learned model predicts which data will be accessed
- Hot data: In-memory, row-oriented
- Cold data: On-disk, columnar
- Automatic, no manual tuning

**Benchmarks to Win:**
- OLTP: Match PostgreSQL (<10ms writes)
- OLAP: 10x faster than PostgreSQL (columnar reads)
- Auto-tuning: No manual config needed

**Market:** $22.8B ETL market

**Example Pitch:**
> "Real-time analytics requires ETL (Kafka → warehouse). We eliminate ETL with learned hot/cold placement. OLTP and OLAP in one database, no configuration."

**VC Appeal:** Solves $22.8B problem, novel ML approach

### Critical Question: Can You Actually Deliver 10-100x?

**Learned Index Reality Check:**

Your current results:
- ✅ 2,862x speedup at 10K rows
- ✅ 22,554x speedup at 100K rows
- ⚠️ BUT: Only on point queries
- ⚠️ BUT: Not validated on updates, deletes
- ⚠️ BUT: Not tested at 10M, 100M, 1B rows

**Path to Validation (4-6 weeks):**

Week 1-2: Large-Scale Testing
- Test learned index at 10M, 100M, 1B rows
- Measure: Insert throughput, point query, range query
- Measure: Update/delete performance
- Document: Where it's fast, where it's slow

Week 3-4: Comparison Benchmarks
- Run TPC-H, TPC-C (standard benchmarks)
- Compare vs: PostgreSQL, SQLite, DuckDB
- Measure: 10x faster? 100x faster? 2x faster?

Week 5-6: Publish Results
- Blog post: "Learned Indexes: 100x Faster for Time-Series"
- GitHub: Reproducible benchmarks
- Hacker News: "We built the fastest embedded time-series DB"

**Outcome Scenarios:**

**Scenario A: Learned Index is 10-100x Faster (Best Case)**
- Positioning: "Fastest database for [specific workload]"
- VC Pitch: "Better algorithms = 100x speedup"
- Timeline: Raise $1-3M based on tech advantage
- Risk: Medium (must scale to production)

**Scenario B: Learned Index is 2-3x Faster (Realistic)**
- Positioning: Niche use case (e.g., "IoT time-series on edge")
- VC Pitch: "Better algorithms + niche market"
- Timeline: Bootstrap to revenue first, then raise
- Risk: Low-Medium (smaller but defensible)

**Scenario C: Learned Index is Same Speed (Worst Case)**
- Positioning: Pivot to features (PostgreSQL + vectors + embedded)
- VC Pitch: "Feature combination, not algorithms"
- Timeline: Bootstrap or small angel round
- Risk: High (no technical moat)

---

## Part 4: Recommended Path (Hybrid Approach)

**My recommendation: Don't choose ONE strategy. Do BOTH in parallel.**

### Month 1-2: Validation Phase

**Parallel Work Stream A: Validate Algorithms (Technical Risk)**
- Test learned indexes at 10M, 100M, 1B rows
- Run TPC-H, TPC-C benchmarks
- Compare vs PostgreSQL, SQLite, DuckDB, pgvector
- Goal: Prove 10-100x speedup on SPECIFIC workload

**Parallel Work Stream B: Validate Market (Market Risk)**
- Add pgvector integration (4 weeks)
- Launch "Embedded PostgreSQL + Vectors" MVP
- Get 100-500 GitHub stars
- Talk to 20-50 developers building AI apps
- Goal: Prove people actually want this

**Decision Point (End of Month 2):**

IF learned indexes prove 10-100x faster:
→ Lead with algorithms, position as "Fastest DB for X"
→ Raise VC funding based on technical advantage

IF learned indexes prove 2-5x faster:
→ Lead with features, position as "PostgreSQL + Vectors + Embedded"
→ Bootstrap to revenue, then raise

### Month 3-6: Growth Phase

**Path A: Algorithm-First (IF validation shows 10-100x)**
- Publish benchmarks showing 10-100x speedup
- Launch on Hacker News: "We built the fastest [X] database"
- Target: 2K-5K GitHub stars (technical community)
- Apply to YC with "better algorithms" story

**Path B: Feature-First (IF validation shows <10x)**
- Launch pgvector integration
- Build AI/RAG tutorials and examples
- Target: 1K-2K GitHub stars + early revenue
- Apply to YC with "niche market" story

### Month 6-12: Revenue or Raise

**Path A: Raise Series A ($3-10M)**
- Traction: 5K+ stars, 1000+ users
- Position: "We're 100x faster than [competitor] for [workload]"
- Use funds: Build team, scale infrastructure

**Path B: Bootstrap to $50K-100K MRR**
- Traction: 2K+ stars, 300+ paying customers
- Position: "Fastest growing embedded PostgreSQL + vectors"
- Then raise Seed ($1-3M) to accelerate

### Why This Hybrid Approach Works

1. **De-risks both technical and market risk**
   - If algorithms don't pan out: you have market validation
   - If market is small: you have technical differentiation

2. **Gives you TWO pitches for investors**
   - Tech pitch: "100x faster through better algorithms"
   - Market pitch: "Fastest growing database in AI space"

3. **Follows successful examples**
   - DuckDB: Started with algorithms, added market fit
   - Supabase: Started with market, added tech differentiation

4. **Maximizes chances of funding**
   - VCs want either: (1) 10x better tech OR (2) fast-growing users
   - You'll have at least one, hopefully both

---

## Part 5: Final Recommendation

### What I Would Do (If I Were You)

**Week 1-2: Brutal Honesty Test**

Run these benchmarks RIGHT NOW:
1. TPC-H (analytical benchmark) - OmenDB vs PostgreSQL vs DuckDB
2. Vector search (1M vectors) - OmenDB vs pgvector vs Pinecone
3. Time-series (1M inserts) - OmenDB vs SQLite vs QuestDB

**If any benchmark shows 10-100x speedup:**
→ Lead with that. "World's fastest [X]"
→ This is your YC pitch
→ This is your technical moat

**If all benchmarks show <5x speedup:**
→ Learned indexes are a feature, not the main pitch
→ Lead with "Embedded PostgreSQL + Vectors"
→ This is your bootstrap-to-revenue story

**Week 3-4: Pick Your Story**

**Story A: "Fastest Database for [Specific Workload]"**
- IF: Benchmarks show 10-100x speedup
- Pitch: Better algorithms beat established players
- Funding: $1-3M Seed based on tech
- Risk: High (must scale), High reward (big exit)

**Story B: "PostgreSQL + Vectors + Embedded"**
- IF: Benchmarks show <5x speedup
- Pitch: Perfect product for AI/edge applications
- Funding: Bootstrap → $500K-1M Angel → $3-5M Seed
- Risk: Low-Medium, Medium reward (steady growth)

**Story C: "Both" (Recommended)**
- Pitch: "Fastest embedded PostgreSQL for AI workloads"
- Combines: Technical differentiation + market niche
- Funding: Either path works (tech-first or market-first)
- Risk: Medium, High reward

**My Prediction:**

You'll find learned indexes are 10-50x faster for TIME-SERIES workloads (sorted, sequential keys), but only 2-5x faster for general workloads. This gives you:

**Positioning:** "Fastest embedded database for time-series and AI workloads"

**This is PERFECT for VC pitch:**
- Technical moat (learned indexes for time-series)
- Large market ($1.45B time-series + $4B vectors = $5B+ TAM)
- Clear differentiation (vs SQLite, vs pgvector)

---

## Bottom Line

**Three Key Insights:**

1. **You CAN win with better algorithms** - DuckDB, ClickHouse, QuestDB prove this. BUT only if you have 10-100x improvement, not 2x.

2. **Open source + commercial model makes MORE money** - Linux Foundation data shows 7-14x better exits than proprietary. Don't be afraid of open source.

3. **Validate FAST, decide FAST** - Run benchmarks this week. If you have 10-100x speedup: raise VC money. If not: bootstrap to revenue.

**Next Action:**
1. Run TPC-H, TPC-C, vector search benchmarks THIS WEEK
2. If 10-100x faster: write technical blog post, apply to YC
3. If <5x faster: build pgvector integration, launch on Hacker News

Either way: you have a viable path to funding.
