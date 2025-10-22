# YC W25 Application Roadmap

**Deadline:** November 10, 2025 (5 weeks from now)
**Strategy:** Embedded PostgreSQL for time-series + AI workloads with learned index optimization
**Goal:** Get accepted to YC W25 with strong technical differentiation

---

## Reality Check: What YC Actually Wants

### YC Traction Benchmarks (2024-2025 data)

**What successful applicants had:**
- Top tier (5-10%): $150K-500K ARR
- Mid tier (60%): $3-5K MRR
- Average accepted: ~$2-5K MRR ("couple thousand/month")
- **Key insight:** Many accepted companies had $0 revenue but strong technical moat + user interest

**YC acceptance rate:** <1% (~300 out of 27,000 applications)

### What You NEED for database/infrastructure companies:

1. **Technical moat** - 10-100x improvement (DuckDB, QuestDB model)
2. **Clear problem** - Specific pain point (not "databases could be better")
3. **Large market** - $1B+ TAM
4. **Early traction** - GitHub stars, active users, OR revenue (pick 2 of 3)
5. **Founder credibility** - Domain expertise, technical ability

### What You DON'T Need:

- ❌ Significant revenue ($0-5K MRR is fine)
- ❌ Team (solo founders get accepted, though harder)
- ❌ Polished product (MVP is enough)

---

## Revised Monetization Strategy

**Research:** Turso (embedded SQLite, YC W23) pricing in 2025:
- Free: 500M rows read/month, 10M writes/month, 5GB storage
- Developer: $5/month - 2.5B rows read/month
- Scaler: $29-99/month - More resources
- Enterprise: Custom

**Our pricing (revised to match market):**

```
FREE (Embedded Database)
├── Unlimited local storage
├── PostgreSQL wire protocol
├── pgvector (vectors)
├── Learned index optimization
└── Community support

STARTER: $9/month
├── Cloud sync (1 project, 10GB)
├── Point-in-time recovery (7 days)
├── 3 devices max
└── Email support

PRO: $29/month
├── Cloud sync (unlimited projects, 100GB)
├── Point-in-time recovery (30 days)
├── Unlimited devices
├── Read replicas
└── Priority support

ENTERPRISE: $299-999/month
├── Dedicated infrastructure
├── 99.99% SLA
├── Custom features
└── Dedicated support engineer
```

**For YC application:**
- Target: $1-5K MRR by Demo Day (10-50 paying customers at $9-29/month)
- Backup: If no revenue, need 1K+ GitHub stars + 100+ active users

---

## 5-Week Sprint Plan

### Week 1 (Oct 2-8): VALIDATE OR KILL

**Goal:** Prove 10-50x speedup on time-series workloads

**Tasks:**
1. **Day 1-2:** Run comprehensive benchmarks
   ```bash
   # Time-series benchmark (1M inserts, sequential keys)
   cargo run --release --bin bench_timeseries -- \
     --rows 1000000 --compare sqlite,questdb

   # Vector search benchmark (1M vectors, 128-dim)
   cargo run --release --bin bench_vectors -- \
     --vectors 1000000 --dim 128 --compare pgvector,pinecone

   # Analytical benchmark (TPC-H)
   cargo run --release --bin bench_tpch -- \
     --scale-factor 1 --compare postgres,duckdb
   ```

2. **Day 3:** Analyze results
   - ARE we 10-50x faster on time-series? (need YES)
   - ARE we competitive on vectors? (need YES)
   - ARE we faster on analytics? (nice to have)

3. **Day 4-5:** Write technical blog post
   - "Learned Indexes: 50x Faster Time-Series in Embedded Databases"
   - Include reproducible benchmarks
   - Publish on personal blog, prepare for HN

4. **Day 6-7:** Fix critical issues found in benchmarking
   - Performance issues
   - Stability issues
   - Edge cases

**Decision Point (End of Week 1):**
- ✅ IF 10-50x faster on time-series: PROCEED with YC
- ⚠️ IF 2-5x faster: Still viable, but weaker pitch
- ❌ IF <2x faster: DON'T apply this batch, need more work

---

### Week 2 (Oct 9-15): BUILD MVP

**Goal:** Production-ready embedded database with time-series + vector support

**Tasks:**

**If benchmarks showed we need pgvector (likely):**
1. **Day 1-3:** Add pgvector integration (30 hours)
   - Integrate pgvector Rust library
   - Add vector index support to learned index
   - Test vector similarity search

2. **Day 4-5:** Polish PostgreSQL wire protocol (15 hours)
   - Add prepared statements (critical for compatibility)
   - Fix any psql compatibility issues
   - Test with popular drivers (psycopg2, asyncpg, pg, etc.)

3. **Day 6-7:** Create examples (15 hours)
   - Time-series example (IoT sensor data)
   - AI/RAG example (LangChain + OmenDB)
   - Edge computing example (offline-first app)

**Deliverable:** Working MVP that developers can actually use

---

### Week 3 (Oct 16-22): LAUNCH & TRACTION

**Goal:** 500-2,000 GitHub stars, 50-100 active users

**Tasks:**

1. **Day 1-2:** Polish documentation
   - Quick start guide (5 minutes to running)
   - API documentation
   - Example applications
   - Performance benchmarks (with charts)

2. **Day 3:** Launch sequence
   - **Morning:** Publish technical blog post on personal blog
   - **10am PT:** Post to Hacker News: "Show HN: OmenDB – PostgreSQL-compatible database with learned indexes (50x faster time-series)"
   - **Afternoon:** Share on Twitter/X with benchmarks
   - **Evening:** Post to r/programming, r/rust, r/PostgreSQL

3. **Day 4-7:** Respond to feedback
   - Answer every HN comment
   - Fix reported bugs IMMEDIATELY
   - Talk to interested users (DM, email, Discord)
   - Create Discord/Slack community if demand exists

**Success Metrics:**
- 500-2,000 GitHub stars (realistic for good HN launch)
- 50-100 people actually try it
- 5-10 detailed user conversations
- 1-2 blog posts/tweets from users

**Backup Plan:**
- If HN launch flops: Launch on Product Hunt next day
- Post to AI/ML communities (r/MachineLearning, r/LocalLlama)
- Direct outreach to 50 developers building time-series/AI apps

---

### Week 4 (Oct 23-29): CUSTOMER VALIDATION

**Goal:** 10-20 customer interviews, 1-5 paying customers (optional but strong)

**Tasks:**

1. **Day 1-3:** Customer discovery interviews (10-20 people)
   - Who: Early users from Week 3
   - Questions:
     - What problem were you trying to solve?
     - What did you try before OmenDB?
     - What would make you pay for this?
     - What's missing?
   - Document quotes, pain points, feature requests

2. **Day 4-5:** Launch cloud sync service (optional but valuable)
   - Simple MVP: S3 backup + restore
   - Stripe integration ($9/month tier)
   - Landing page with pricing
   - Goal: Get 1-5 paying customers ($9-45 MRR)

3. **Day 6-7:** Build YC pitch materials
   - One-liner: "PostgreSQL-compatible embedded database, 50x faster for time-series + AI workloads"
   - Demo video (2 minutes):
     - Show time-series benchmark (50x faster than SQLite)
     - Show RAG example (LangChain + OmenDB)
     - Show sync service
   - Pitch deck (optional, but helpful)

**Deliverable:** Clear evidence of market need (customer quotes, testimonials)

---

### Week 5 (Oct 30 - Nov 5): YC APPLICATION

**Goal:** Submit compelling YC W25 application by Nov 10

**Tasks:**

1. **Day 1-2:** Write YC application
   - Use YC application template
   - Focus on technical moat + traction
   - Include metrics, benchmarks, user quotes

2. **Day 3-4:** Record demo video
   - Keep it under 2 minutes
   - Show benchmark comparison (most important)
   - Show working product
   - Show user testimonial (if available)

3. **Day 5:** Review and submit
   - Have 2-3 people review application
   - Submit by Nov 5 (5 days before deadline)
   - Don't wait until last minute

**Nov 10:** Application deadline (buffer in case of issues)

---

## YC Application: What to Write

### One-Liner (Company Description)

**Template:** "PostgreSQL-compatible embedded database, 50x faster for time-series and AI workloads"

**Why this works:**
- Clear positioning (PostgreSQL-compatible = huge market)
- Specific advantage (50x faster, not vague "better")
- Target workloads (time-series + AI = $5B+ TAM)

### Problem Statement

**Bad:** "Databases are slow and expensive"

**Good:**
> "Developers building time-series applications (IoT, observability, analytics) use SQLite or PostgreSQL. But SQLite struggles with time-series inserts (100K/sec max) and PostgreSQL requires expensive server infrastructure. For AI applications, adding pgvector to SQLite isn't possible, and managed Postgres costs $50-500/month."

### Solution

**Bad:** "We use learned indexes to make databases faster"

**Good:**
> "OmenDB is PostgreSQL-compatible but embedded (single binary, no server). Learned indexes optimize sequential keys, giving us 50x faster time-series inserts (5M/sec vs SQLite's 100K/sec). Built-in pgvector support for AI/RAG workloads. Pure Rust, works offline, optional cloud sync."

### Traction (Week 3-4 results)

**Benchmark goals:**
- ✅ 1,000-2,000 GitHub stars
- ✅ 100-200 active users
- ✅ $1-5K MRR (optional)
- ✅ 10-20 customer interviews

**Example traction statement:**
> "Launched 3 weeks ago. 1,200 GitHub stars, 150 active users. $2K MRR from 25 customers paying $9-29/month for cloud sync. Talked to 20 users - main use cases are IoT data (40%), AI/RAG apps (35%), edge computing (25%). Growing 50% week-over-week."

**Backup (if no revenue):**
> "Launched 3 weeks ago. 1,200 GitHub stars, 150 active users trying it. Talked to 20 users - main pain points are SQLite too slow for time-series (10x improvement needed) and pgvector requires managed Postgres ($200/month). 15 users said they'd pay $9-29/month for cloud sync."

### Market Size

**Time-series databases:** $1.45B (2024) → $4.42B (2033), 15.2% CAGR
**Vector databases:** $4B by 2028
**Total Addressable Market (TAM):** $5B+ (time-series + vectors)

### Why You?

**Be honest about solo founder status:**
> "Solo founder, 10 years software engineering experience. Built [previous projects]. Expert in Rust, databases, and distributed systems. Learned about RMI (Recursive Model Index) from MIT research, validated 50x speedup on time-series workloads."

**If you have relevant experience, highlight it:**
- Ex-Google/Meta/Amazon engineer?
- Built databases before?
- Published papers/research?
- Contributed to PostgreSQL/SQLite/DuckDB?

### Why Now?

**Three converging trends:**
1. **AI explosion (2024-2025):** Every app needs vector search. LangChain has 80K+ GitHub stars, LlamaIndex 30K+. Developers need embedded PostgreSQL + pgvector.
2. **Edge computing growth:** Apps moving to edge (Cloudflare Workers, Fly.io, etc.) need embedded databases.
3. **Cost pressure:** Managed Postgres costs $200-500/month. Developers want local-first, pay only for sync.

---

## Success Criteria

### Must-Have (Week 3-4):
- ✅ Benchmarks prove 10-50x speedup on time-series
- ✅ 500-2,000 GitHub stars
- ✅ 50-100 active users
- ✅ 10-20 customer interviews with quotes

### Nice-to-Have:
- ✅ $1-5K MRR (shows monetization works)
- ✅ User testimonials ("This is 10x faster than SQLite")
- ✅ Blog posts/tweets from users
- ✅ Technical co-founder or advisor

### Deal-Breakers (Don't Apply If):
- ❌ Benchmarks show <5x speedup (not differentiated enough)
- ❌ Can't get 500+ GitHub stars (no product-market fit signal)
- ❌ Zero active users after launch (nobody cares)

---

## Risks & Mitigation

### Risk 1: Benchmarks don't show 10-50x speedup
**Mitigation:** If <10x, pivot pitch to "PostgreSQL-compatible embedded database with vectors" (feature-first, not algorithm-first). Still viable but weaker.

### Risk 2: HN launch flops (<200 stars)
**Mitigation:**
- Launch on Product Hunt next day
- Direct outreach to 100 developers (IoT, AI, edge computing)
- Post to AI/ML communities
- Try again in 1 week with improved pitch

### Risk 3: Solo founder bias
**Mitigation:**
- Find technical advisor (ex-Timescale, ex-Postgres developer?)
- Highlight strong technical execution (benchmarks, clean code)
- Show you can execute fast (MVP in 5 weeks)

### Risk 4: Can't build sync service in time
**Mitigation:**
- Skip sync service for YC application
- Focus on embedded database + technical moat
- Revenue is optional for YC (technical advantage + users enough)

---

## Is This YC-Worthy? Honest Assessment

### ✅ YES, if:
1. Benchmarks prove 10-50x speedup (technical moat)
2. HN launch gets 500-2K stars (market validation)
3. Clear user pain point (customer interviews)
4. Large market ($5B+ TAM)

### ⚠️ MAYBE, if:
1. Benchmarks show 5-10x speedup (weaker moat, still viable)
2. HN launch gets 200-500 stars (decent interest)
3. Some revenue ($1-2K MRR)

### ❌ NO, if:
1. Benchmarks show <5x speedup (not differentiated)
2. Launch flops (<200 stars, <20 users)
3. Can't articulate clear problem (users don't care)

---

## My Honest Take

**You have a shot, but it's tight:**

**Pros:**
- ✅ Strong technical foundation (218 tests passing, 2,862x speedup at 100K rows)
- ✅ Proven tech stack (DataFusion, redb, pgwire)
- ✅ Clear market (time-series $1.45B + vectors $4B)
- ✅ 5 weeks is enough IF you execute fast

**Cons:**
- ⚠️ Solo founder (harder but not impossible)
- ⚠️ Unproven at scale (need to validate 10M+ rows)
- ⚠️ Competitive market (Turso, DuckDB, etc.)
- ⚠️ Tight timeline (5 weeks is aggressive)

**What will determine success:**

1. **Week 1 benchmarks:** If proven 10-50x faster, you have a strong pitch. If not, reconsider.

2. **Week 3 launch:** If HN launch gets 1K+ stars, you have market validation. If flops, you may not have PMF yet.

3. **Week 4 customer interviews:** If 10+ people say "I'd pay for this", you're viable. If nobody cares, pivot or wait.

**My recommendation:**

- **Run benchmarks this week**
- **If 10-50x faster:** Go all-in on YC application
- **If 5-10x faster:** Still apply, but temper expectations
- **If <5x faster:** Don't apply this batch, spend 3 months improving tech

**Bottom line:** You have a 10-30% chance of getting YC W25 IF you execute this plan perfectly. That's actually decent odds given <1% acceptance rate. Worth the shot.

---

## Next Steps (This Week)

**Day 1 (Today):**
- [ ] Set up benchmarking infrastructure
- [ ] Prepare time-series dataset (1M-10M sequential keys)
- [ ] Prepare vector dataset (SIFT1M, 1M 128-dim vectors)

**Day 2-3:**
- [ ] Run time-series benchmark (OmenDB vs SQLite vs QuestDB)
- [ ] Run vector benchmark (OmenDB vs pgvector)
- [ ] Run TPC-H benchmark (OmenDB vs PostgreSQL vs DuckDB)

**Day 4:**
- [ ] Analyze results
- [ ] DECIDE: Apply to YC or not? (based on benchmark results)
- [ ] If YES: Start writing technical blog post
- [ ] If NO: Focus on 3-month improvement plan instead

**Day 5-7:**
- [ ] Finish technical blog post
- [ ] Fix critical issues found in benchmarking
- [ ] Prepare for Week 2 (pgvector integration)

**Decision by Friday:** GO or NO-GO on YC W25 application
