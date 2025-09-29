# Strategic Decisions Summary

**Date**: September 29, 2025
**Status**: Recommendations ready for decision

---

## 1. License Strategy: Elastic License v2 ⭐

### Recommendation: **Elastic License v2.0** (ELv2)

**Why not AGPL**:
- AGPL scares enterprise legal teams
- "Must open source modifications" is confusing
- Google bans AGPL internally
- Slows adoption unnecessarily

**Why not Apache/MIT**:
- Zero protection from AWS/cloud providers
- They'll take your code and offer "RDS for OmenDB"
- You get $0 while they profit

**Why not BSL**:
- 4-year auto-conversion to Apache 2.0
- Protection expires (unacceptable)

**Why ELv2 wins**:
- ✅ **Crystal clear**: "You can't offer this as a managed service"
- ✅ **Permanent protection** (no expiration)
- ✅ **Less enterprise friction** than AGPL
- ✅ **Proven**: Elastic used it successfully against AWS
- ✅ **Simple commercial licensing**: "Pay to offer as service"

### License Structure

```
omendb/omendb/              → Elastic License v2.0
  Core database, public

omendb/omendb-cloud/        → Proprietary
  Enterprise features, private repo forever

omendb/pg-learned/          → MIT (keep as is)
  Demo extension

omendb/website/             → MIT
  Marketing
```

### Revenue Model

**Free (ELv2)**:
- Self-hosted OmenDB
- All core features
- Can't offer as managed service to others
- No support

**Commercial License ($5K-50K/year)**:
- Can offer as managed service
- Commercial relationship
- Email support

**OmenDB Cloud ($500-10K/month)**:
- We operate it
- High availability
- Support + SLAs
- Our primary revenue

**Enterprise ($50K-500K/year)**:
- Distributed clustering (closed source)
- Advanced security (closed source)
- Multi-region (closed source)
- Custom features + support

---

## 2. Market Focus: ML Training Metrics (Wedge) ⭐

### Recommendation: **Start with ML training metrics, expand to general time-series**

**Why not general time-series immediately**:
- Too broad, too much competition
- Need $3-5M to build distributed architecture
- 18 months to production
- High risk

**Why ML training metrics**:
- ✅ **11x proven advantage** on this exact workload
- ✅ **Clear pain point**: "My training dashboard is slow"
- ✅ **Market timing**: AI boom, everyone needs this
- ✅ **Fast to revenue**: 3-6 months to paid pilots
- ✅ **Capital efficient**: Bootstrap or raise $500K-1M
- ✅ **Expansion path**: Wedge into general time-series later

### 6-Month Plan

**Month 1-2: Build MVP**
- PostgreSQL wire protocol
- Basic SQL (SELECT, INSERT, WHERE, time ranges)
- Python SDK (ML integration)
- Docker one-command deploy

**Month 3-4: Get 5 Pilots**
- Reach out to AI startups (YC companies)
- Free beta for feedback
- Integrate with PyTorch, TensorFlow
- Fix bugs, add must-haves

**Month 5-6: Launch + Revenue**
- 5 paying customers ($500-2K/month)
- $2.5-10K MRR
- Clear product-market fit
- Case studies

**Then decide**:
- Keep bootstrapping → 20 customers → $50K MRR
- Raise $1-2M → expand to general time-series
- Get acquired → proven ML metrics tech

### Customer Profile

**Target**:
- AI startups training LLMs
- ML platform companies
- Research labs
- They ALL have slow metrics dashboards

**Competition**:
- Prometheus + Grafana (slow, complex)
- Weights & Biases ($200M valuation, but slow)
- TensorBoard (clunky)
- InfluxDB (not optimized for ML)

**Our advantage**: 11x faster, built for bursty training patterns

---

## 3. Funding Strategy: Build First, Then Raise

### Recommendation: **Build for 3 months, get traction, then raise with leverage**

**Why not YC now**:
- Only 6 weeks until deadline
- No time for customer validation
- Zero pilot customers
- Weak application (tech but no traction)
- 7% equity for $500K (bad terms)

**Why build first**:
- 3 months to make it genuinely useful
- Get 5-10 real customers using it
- Prove product-market fit
- Then raise $2-5M seed with much better terms
- Or apply to YC with strong traction

### Funding Timeline

**Month 1-3: Build + Validate** (no funding needed)
- Make it work for ML metrics
- Get 5 pilots using it
- Fix bugs, iterate

**Month 4: Decision Point**
- If traction good → Raise $1-2M seed
- If traction amazing → Raise $3-5M
- If struggling → Pivot or bootstrap

**Options**:
1. **Angel/VC seed round**: Better terms than YC
2. **YC with traction**: Higher acceptance rate
3. **Bootstrap**: If pilots convert to paid

---

## 4. Open Source Strategy: Open Core

### Recommendation: **Stay proprietary for 3 months while building, then open source core with ELv2**

**Timeline**:

**Now - Month 3 (Closed)**:
- Build MVP
- Polish rough edges
- Get to working state
- Don't release half-baked code

**Month 4 (Open Source)**:
- Release core with ELv2
- Launch on HN/Reddit: "We made time-series 10x faster"
- Show reproducible benchmarks
- Build community

**Why wait 3 months**:
- Current code has rough edges
- No managed service ready yet (can't monetize)
- Want to launch with polished product
- Open source = marketing event, do it right

**What to open source**:
- Core learned index (RMI)
- Single-node database
- PostgreSQL protocol
- Benchmarks (prove 10x claim)

**What stays closed**:
- Distributed clustering
- Advanced security
- Auto-scaling
- Enterprise features

---

## 5. Repository Organization: Simplify

### Current (Messy)

```
omendb/core/omendb-rust/src/    ← too nested
omendb/core/website/            ← duplicate?
omendb/website/                 ← which one?
omendb/core/internal/           ← mixed with code
```

### Recommended (Clean)

```
omendb/
├── omendb/              # Main product (public when ready)
│   ├── src/
│   ├── tests/
│   ├── benchmarks/
│   ├── LICENSE-ELv2
│   └── README.md
│
├── omendb-cloud/        # Enterprise (private forever)
│   ├── distributed/
│   ├── security/
│   └── (never public)
│
├── pg-learned/          # Demo (public, MIT)
│   └── (marketing tool)
│
└── website/             # Marketing (public, MIT)
    └── omendb.io
```

**Changes needed**:
1. Flatten `core/omendb-rust/*` → `omendb/src/`
2. Delete duplicate website directory
3. Move `internal/` to separate private notes repo
4. Create `omendb-cloud/` for enterprise features

---

## Quick Decision Matrix

| Decision | Recommended | Why |
|----------|-------------|-----|
| **License** | Elastic v2 | Protects from AWS, less scary than AGPL |
| **Market** | ML metrics wedge | 11x proven, fast to revenue, expand later |
| **Funding** | Build 3mo → Raise | Get traction first, better terms |
| **Open Source** | Wait 3mo, then ELv2 | Polish first, launch properly |
| **Repo** | Simplify structure | Flatten to omendb/src/ |

---

## What to Do This Week

### Priority 1: Start Building Features
- PostgreSQL wire protocol (basic)
- Simple SQL parser (SELECT, INSERT, WHERE)
- Time-series query support
- Docker deployment

### Priority 2: Keep pg-learned as Marketing
- Already updated with 10x results ✅
- Keep MIT license (it's just a demo)
- Use for credibility building

### Priority 3: Clarify Strategy Docs
- Mark current status: 30% ready ✅
- Document licensing decision (this doc)
- Update CURRENT_STATUS.md as we progress

---

## Bottom Line

**Focus**: Build ML training metrics database (3 months)
**License**: Elastic License v2 when we open source
**Funding**: Build first, raise with traction
**Next**: PostgreSQL protocol + basic SQL

**Goal**: 5 paying customers at $500-2K/month by Month 6

Then expand or raise based on traction.

---

*This replaces all previous strategy documents as single source of truth*