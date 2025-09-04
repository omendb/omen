# OmenDB Web

Marketing website and documentation portal.

## Current Status (July 29, 2025)
- **Stack**: SolidJS + Vite + Tailwind
- **Performance**: <1s load time target
- **Messaging**: Instant startup + production HNSW performance
- **Phase**: Marketing site for public embedded + docs

## Repository Strategy

### **omendb-web/ Role** (This Repository - PUBLIC)
- **Marketing site**: Landing page, pricing, features
- **Documentation**: API reference, tutorials, quickstart guides
- **Performance demos**: Interactive benchmarks and comparisons
- **Community**: Blog, case studies, developer resources

### **Multi-Repository Architecture**
```
~/github/omendb/
├── omenDB/            # PUBLIC - Free embedded database
├── omendb-server/     # PRIVATE - Paid server edition  
├── omendb-admin/      # PRIVATE - Admin dashboard (planned)
└── omendb-web/        # PUBLIC - Marketing + docs (this repo)
```

## Quick Navigation
- **Public Repo**: @../omenDB/ - Latest benchmarks and features
- **Performance Data**: @../omenDB/benchmarks/CURRENT_BASELINES.md
- **Examples**: @../omenDB/examples/getting_started/

## Key Messaging Strategy
- **Primary**: "World's fastest starting vector database - 0.0002ms"
- **Performance**: "91K-210K vec/s batch operations (2-5x faster than competitors)"
- **Latency**: "480μs P99 (financial-grade performance)"
- **Value**: "50% Pinecone cost + instant startup advantage"

## Repository Structure
```
src/
├── components/      # UI components
├── pages/          # Home, Docs, Benchmarks
└── data/           # Performance metrics
```

## Development Workflow
```bash
npm run dev          # Start dev server
npm run build        # Production build
npm run typecheck    # Type checking
```

## Content Strategy
1. **Hero**: Instant startup advantage (unique differentiator)
2. **Benchmarks**: Interactive HNSW performance demos
3. **Pricing**: Free embedded → Platform → Enterprise tiers
4. **Docs**: 5-minute quickstart for embedded database
5. **Use Cases**: Edge deployment, IoT, rapid prototyping

## Performance Targets
- Page load: <1s
- Lighthouse: >90
- Core Web Vitals: All green

## What This Site Should NOT Contain

**❌ Admin functionality** - Goes in private omendb-admin/ repo
**❌ Customer management** - Private business logic
**❌ Billing interfaces** - Server-specific features
**❌ Internal metrics** - Private operational data

---
**Mission**: Convert developers with performance + simplicity
**Focus**: Instant startup unique advantage for embedded use cases
**Audience**: Developers evaluating vector databases, not existing customers