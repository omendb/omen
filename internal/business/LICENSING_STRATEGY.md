# OmenDB Licensing & Repository Strategy

**Date**: September 29, 2025
**Decision needed**: Licensing strategy that protects against cloud providers while enabling adoption

---

## Current State

| Repo | Current License | Status | Purpose |
|------|----------------|--------|---------|
| `pg-learned` | MIT | ✅ Open source | Demo PostgreSQL extension |
| `core` | Proprietary | 🔒 Closed | Main database implementation |
| `website` | None | 🌐 Marketing | Public marketing site |

---

## The Cloud Provider Problem

**Risk**: AWS/Google/Azure could:
1. Take your open source code
2. Offer managed version (RDS for OmenDB)
3. Undercut your pricing with their scale
4. You get zero revenue while they profit

**Examples**:
- **Elasticsearch** → AWS OpenSearch (fork after license change)
- **Redis** → AWS MemoryDB (forced Redis to change license in 2024)
- **MongoDB** → AWS DocumentDB (compatible API, forced SSPL license)
- **Terraform** → AWS/others using it (forced OpenTofu fork after BSL)

**Result**: Original creators lost revenue, forced to change licenses, community splits

---

## License Options (Protecting Against Cloud Providers)

### Option 1: AGPL v3 (Strong Protection) ⭐

**What it does**:
- If someone offers your software as a service (SaaS/cloud), they MUST open source their modifications
- AWS/Google can't just wrap it and sell managed version without contributing back
- Loophole: They could clean-room reimplement (expensive, unlikely)

**Who uses this**:
- MongoDB (before switching to SSPL)
- Grafana
- MinIO (object storage)
- SuiteCRM

**Pros**:
- ✅ Strong protection against cloud providers
- ✅ Still OSI-approved open source
- ✅ Forces contributions back
- ✅ Enterprises okay with it (have legal teams)
- ✅ Can still offer commercial license for those who don't want AGPL

**Cons**:
- ⚠️ Some enterprises scared of AGPL (less common now)
- ⚠️ If AWS really wants to, they'll find workarounds
- ⚠️ Slightly slower adoption than MIT/Apache

**Dual licensing model**:
```
AGPL v3: Free for everyone, BUT if you offer as cloud service you must open source
Commercial: Pay us for license if you don't want AGPL obligations
```

---

### Option 2: BSL (Business Source License) (Good Protection)

**What it does**:
- Source code is available (not "open source" but "source available")
- Free for non-production use
- Production use allowed up to certain scale
- After X years (usually 4), converts to open source (Apache 2.0)
- Explicitly blocks cloud providers

**Who uses this**:
- CockroachDB (switched to BSL)
- MariaDB MaxScale
- Sentry
- Couchbase

**Example clause**:
```
Additional Use Grant: You may use the Licensed Work in production
if you do not offer the Licensed Work to third parties as a
hosted or managed service.

Change Date: 4 years from release
Change License: Apache License 2.0
```

**Pros**:
- ✅ Clear protection: "You can't offer this as a service"
- ✅ Converts to true open source after 4 years
- ✅ Balances protection vs adoption
- ✅ Companies understand it (precedent exists)

**Cons**:
- ⚠️ Not OSI-approved "open source" (some people care)
- ⚠️ More complex license (need lawyer to review)
- ⚠️ Slightly slower adoption than true open source

---

### Option 3: SSPL (Server Side Public License) (Maximum Protection)

**What it does**:
- Like AGPL but more aggressive
- If you offer as a service, you must open source ENTIRE STACK
- "If you run this as SaaS, open source your infrastructure, management tools, everything"
- Basically: AWS can't do it without open sourcing AWS itself

**Who uses this**:
- MongoDB (switched to SSPL in 2018)
- Graylog
- Elasticsearch (before re-licensing)

**Pros**:
- ✅ Maximum protection (cloud providers basically can't use it)
- ✅ Very clear anti-cloud-provider stance

**Cons**:
- ❌ NOT OSI-approved open source (controversial)
- ❌ Many companies won't touch SSPL (too restrictive)
- ❌ Debian/Fedora won't distribute SSPL software
- ❌ Creates controversy/bad PR

**Verdict**: Too aggressive, harms adoption more than helps

---

### Option 4: Apache 2.0 / MIT (No Protection)

**What it does**:
- True permissive open source
- Anyone can do anything (including AWS offering managed version)

**Pros**:
- ✅ Maximum adoption
- ✅ Simple, well-understood
- ✅ No friction

**Cons**:
- ❌ Zero protection against cloud providers
- ❌ Your innovation gets commoditized
- ❌ Must compete on service quality vs AWS scale

**When this works**:
- You're better at operating it than AWS (ClickHouse model)
- You have deep technical moat (AWS can't catch up easily)
- You're raising $50M+ to outspend AWS on features

---

## My Recommendation: AGPL v3 + Commercial License

Here's the strategy:

### Core Database: AGPL v3

```
omendb/core/
├── learned_index/         # AGPL v3
├── storage/              # AGPL v3
├── postgres_protocol/    # AGPL v3
└── wal/                  # AGPL v3
```

**Licensing**:
- AGPL v3 for everyone
- If you run as SaaS/cloud service, you must open source your modifications
- If you don't want AGPL obligations, buy commercial license

**Revenue model**:
1. **Self-hosted (AGPL)**: Free, must comply with AGPL
2. **OmenDB Cloud**: We operate it, you pay us
3. **Commercial license**: $50K-500K/year for enterprises who want to run without AGPL

---

### Demo Extension: MIT (Keep as is)

```
omendb/pg-learned/
└── (simple demo)         # MIT (already is)
```

**Why**:
- Marketing tool, not the real product
- Shows concept, builds credibility
- Doesn't give away core innovation (simplified models)

---

### Enterprise Features: Proprietary

```
omendb/enterprise/ (private repo)
├── distributed/          # Proprietary
├── cloud_native/         # Proprietary
├── advanced_security/    # Proprietary
└── support/              # Proprietary
```

**Never open sourced**:
- These are expensive to build ($2-3M)
- Real competitive moat
- What enterprises pay for

---

## License Comparison

| License | Cloud Protection | Adoption Impact | OSI-Approved | Recommended |
|---------|-----------------|-----------------|--------------|-------------|
| AGPL v3 | ⭐⭐⭐⭐ | ⭐⭐⭐ | ✅ Yes | ⭐ Best |
| BSL | ⭐⭐⭐⭐⭐ | ⭐⭐ | ❌ No | Good |
| SSPL | ⭐⭐⭐⭐⭐ | ⭐ | ❌ No | Too aggressive |
| Apache/MIT | ⭐ | ⭐⭐⭐⭐⭐ | ✅ Yes | Too risky |

---

## Repository Organization

### Current (Confusing)

```
omendb/
├── core/              # Main DB (currently monorepo)
│   ├── omendb-rust/   # Rust implementation
│   ├── internal/      # Strategy docs
│   ├── website/       # Wait, why is this here?
│   └── mvp/           # Old stuff?
├── pg-learned/        # Demo extension
└── website/           # Wait, another website?
```

**Problems**:
- Two website directories (core/website and website/)
- Monorepo structure unclear
- Internal docs mixed with code

### Recommended Structure

```
omendb/
├── omendb/            # Main product (AGPL v3)
│   ├── src/           # Core Rust code
│   ├── tests/
│   ├── benchmarks/
│   ├── docs/          # Public docs
│   ├── LICENSE        # AGPL v3
│   └── README.md
│
├── pg-learned/        # Demo extension (MIT) - Keep as is
│   ├── src/
│   ├── LICENSE        # MIT
│   └── README.md
│
├── omendb-cloud/      # Enterprise (Private repo)
│   ├── distributed/
│   ├── kubernetes/
│   ├── monitoring/
│   └── (never public)
│
└── website/           # Marketing (public, MIT/Apache)
    ├── blog/
    ├── docs/
    └── README.md
```

**Changes**:
1. Rename `core/` → `omendb/` (clear main product)
2. Move `core/omendb-rust/*` → `omendb/src/` (flatten)
3. Move `core/internal/` → separate private repo or local only
4. Delete duplicate website directory
5. Create `omendb-cloud/` private repo for enterprise features

---

## Migration Plan

### Week 1: Clean up current repos

```bash
# In omendb/core
git mv omendb-rust/src/* src/
git mv omendb-rust/Cargo.toml ./
git rm -rf internal/  # Move to private notes
git rm -rf website/   # Duplicate, use main website repo

# Add LICENSE files
cat > LICENSE <<EOF
GNU AFFERO GENERAL PUBLIC LICENSE
Version 3, 19 November 2007
...
EOF

cat > LICENSE.COMMERCIAL <<EOF
OmenDB Commercial License
Contact: nijaru7@gmail.com

For organizations that want to use OmenDB in production
without AGPL obligations.
EOF
```

### Week 2: Update documentation

```markdown
# README.md

## License

OmenDB is dual-licensed:

1. **AGPL v3** (free, open source)
   - Use freely for any purpose
   - If you offer OmenDB as a service (SaaS/cloud), you must open source your modifications

2. **Commercial License** (paid)
   - Run OmenDB as a service without open sourcing
   - Priority support and SLAs
   - Contact: nijaru7@gmail.com

## Why AGPL?

We chose AGPL to protect against cloud providers commoditizing our innovation
while keeping the software fully open source. You can:
- ✅ Use for free (even in production)
- ✅ Modify and distribute
- ✅ Build products with it
- ⚠️ If you offer as SaaS, open source your changes (or buy commercial license)

## Enterprise

For high-availability, distributed clusters, advanced security, and support:
- OmenDB Cloud: Managed service (coming soon)
- Enterprise License: Self-hosted with support
- Contact: nijaru7@gmail.com
```

---

## Specific Answers to Your Questions

### 1. "Should we review pg-learned or do anything with it?"

**Keep it as is (MIT license)** for these reasons:
- It's a marketing tool showing the concept
- Simple implementation (not giving away core innovation)
- Builds credibility without risk
- MIT license is fine because it's not the real product

Maybe update README to clarify:
```markdown
## Relationship to OmenDB

pg-learned is a demonstration extension showing learned index concepts.

For production use:
- OmenDB: Full database with advanced learned indexes (10x faster, AGPL v3)
- Visit: https://github.com/omendb/omendb
```

### 2. "Should we change the way our repos are organized?"

**Yes, simplify**:
```
Current:  omendb/core/omendb-rust/src/
Better:   omendb/omendb/src/

Current:  two website directories
Better:   one website repo
```

See migration plan above.

### 3. "What license should we use?"

**AGPL v3 + Commercial dual licensing**

Protects against cloud providers while staying true open source:
- AWS can't just wrap and sell it (must open source everything)
- Enterprises who want to avoid AGPL can pay you
- You control your managed service business

### 4. "You sure about Apache 2.0 vs something that protects us?"

**No, I was wrong earlier.** Apache 2.0 is too risky.

After thinking about cloud providers:
- ✅ **Use AGPL v3** (strong protection, still OSI open source)
- ⚠️ **Consider BSL** if you want even stronger protection
- ❌ **Don't use Apache/MIT** (AWS will eat you)

**AGPL is the sweet spot**: Protects you while being true open source.

---

## Real-World Example: How This Plays Out

**Scenario: AWS wants to offer "RDS for OmenDB"**

**With MIT/Apache**:
1. AWS takes your code (legally)
2. Offers managed OmenDB on AWS
3. Charges $X/month
4. You get $0

**With AGPL v3**:
1. AWS takes your code (still legal)
2. To offer as service, must open source ALL modifications
3. AWS infrastructure code, management tools, everything → open source
4. AWS won't do this (exposes their secret sauce)
5. Or AWS pays you for commercial license
6. Either way: you win

**With BSL**:
1. AWS takes your code
2. License explicitly says "no offering as service"
3. AWS can't do anything
4. But "not open source" hurts adoption

---

## Final Recommendation

### License Structure:
- **Core database** (`omendb/`): AGPL v3 + Commercial dual license
- **Demo extension** (`pg-learned/`): MIT (keep as is)
- **Enterprise features** (`omendb-cloud/`): Proprietary (never open)
- **Website** (`website/`): MIT or Apache 2.0

### Repository Organization:
- Flatten `core/` → `omendb/` (main product)
- Keep `pg-learned/` separate (demo)
- Create private `omendb-cloud/` for enterprise
- Clean up duplicate directories

### Messaging:
```
"OmenDB is open source (AGPL v3) with dual licensing.
Use for free, or buy commercial license if you need to run as SaaS.
Enterprise features available separately."
```

This gives you:
- ✅ Protection against AWS/Google/Azure
- ✅ True open source (AGPL is OSI-approved)
- ✅ Revenue from commercial licenses + managed service
- ✅ Clear story for adoption

**Start with AGPL v3 when you open source (in 3 months after MVP is polished).**

Sound good?
