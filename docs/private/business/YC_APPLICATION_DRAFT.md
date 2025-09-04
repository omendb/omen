# Y Combinator Application - OmenDB (Fall 2025)

## Company

**Company name**: OmenDB

**Company url**: https://omendb.com (domain secured, site launching with OSS release)

**Demo video**: [Will record showing 0.001ms startup vs 100ms+ competitors]

**Describe what your company does in 50 characters or less**:
Instant vector database - 0.001ms startup, fast search

## Contact

**Founder**: Nick Russo
**Email**: [your email]
**Phone**: [your phone]

## Founders

**Nick Russo** (Solo Founder)
- 15 years programming (10 professional)
- Self-taught engineer, 14 years Linux
- Previous: Software engineer at early-stage startups
- Built complete v0.1.0 in Mojo achieving "impossible" 0.001ms startup

**Please tell us about the time you most successfully hacked some (non-computer) system to your advantage**:
At my last startup, we needed expensive GPU servers for ML training but had no budget. I discovered university research clusters often had idle time. Built relationships with grad students, offered to help optimize their code in exchange for cluster access during off-hours. Got $50K worth of compute for free, shipped our MVP 3 months early.

## Progress

**How long have you been working on this?**
6 months full-time. Started researching vector databases for a RAG project, discovered they all took 100ms+ to start up. Spent 2 months researching why (index loading, memory allocation). Found Mojo language could solve this. 4 months building.

**How far along are you?**
- v0.1.0 complete with benchmarks
- 0.001ms startup achieved (100-1000x faster than competitors)
- 4,420 vec/s @128D (matches Faiss at standard dimensions)
- Ready for open source launch next week
- Rust server implementation complete

**What is your monthly growth rate?**
Pre-launch. Plan: Launch OSS → 100 developers month 1 → 1,000 month 3 → monetize cloud tier month 6

**How many users do you have?**
Pre-launch. Have working v0.1.0, comprehensive benchmarks, technical validation complete.

## Idea

**What are you building and for who?**
OmenDB is an embedded vector database with instant startup (0.001ms) that works like SQLite but for AI applications. 

For developers building AI apps who need vector search but hate the complexity of current solutions. Every ChatGPT wrapper, RAG app, and AI agent needs vector search, but Pinecone requires servers and Faiss takes forever to load.

We give them instant vector search that works offline, starts immediately, and scales to the cloud when needed.

**Describe what you've built in one or two sentences**:
Vector database that starts in 0.001ms (vs 100ms+ for competitors) and matches Faiss performance. Works embedded like SQLite, scales to cloud like Pinecone.

**Why did you pick this idea to work on?**
Building a RAG app, tried loading a Faiss index - took 300ms just to start. Realized every AI developer faces this. Researched why (loading serialized graphs into memory). Found Mojo could keep data structures in memory-mappable format. No one else doing this.

**What domain expertise do you have?**
- 14 years Linux/systems programming
- Studied database storage engine papers (LSM trees, B-trees)
- Built distributed systems at startups
- Deep-dived into HNSW algorithm, vector search papers
- First production Mojo database (cutting edge)

**How do you know people want this?**
Every AI app needs vector search:
- Pinecone worth $2.6B but developers complain about cost/complexity
- "Faiss is fast but startup time kills our latency" - common HN complaint
- ChromaDB/Weaviate popular but still 50-100ms startup
- SQLite for vectors doesn't exist yet

## Market

**How big is your market?**
Vector database market: $1.5B (2024) → $11.5B (2030)
TAM: Every AI application needs vector search
SAM: Developers building RAG, agents, semantic search ($3B)
SOM: Embedded-first developers wanting simplicity ($300M)

**How will you make money?**
Three tiers:
1. Free: Embedded database (like SQLite) - customer acquisition
2. Platform: $99-999/month managed cloud - self-serve SaaS
3. Enterprise: $5-50K/month - on-premise, SLAs, support

**Who are your competitors?**
- Pinecone: Cloud-only, expensive, no embedded option
- Faiss: Library not database, 100ms+ startup, complex
- ChromaDB: 50ms startup, less performant
- Weaviate: Heavy infrastructure, slow startup

**What's your advantage?**
Physics advantage: 0.001ms startup is 100x faster. Competitors can't match without complete rewrite. Like SQLite vs Postgres - different use cases, we own "embedded AI database" category.

## Equity

**Have you raised funding?**
No.

**Are you incorporated?**
Not yet. Will be Delaware C-Corp.

## Others

**What convinced you to apply to YC?**
Built the hard part (0.001ms startup), need help with distribution. YC network perfect for B2B dev tools. Want to be the SQLite of vector databases - YC can accelerate that by years.

**How did you hear about YC?**
Following since 2010. PG essays shaped how I think about startups.

## Curious

**Please tell us something surprising or amusing that one of you has discovered**:
Vector databases are basically just storing shopping lists in 128-dimensional space. When you search, you're finding the most similar shopping list. Pinecone charges $600/month for a grocery store proximity detector. We do it in 0.001ms for free.

---

## Additional Notes for Application

### Key Points to Emphasize:
1. **Solo but capable**: Built entire v0.1.0, will find co-founder through YC
2. **Technical moat**: First mover with Mojo, physics advantage
3. **Clear monetization**: Free tier for growth, paid for scale
4. **Immediate value**: Developers can use it today

### Demo Video Script (60 seconds):
1. Show competitor startup times (Faiss: 127ms, ChromaDB: 52ms)
2. Show OmenDB: 0.001ms (visual shock)
3. Quick code demo - 3 lines to working vector search
4. Performance benchmark graph
5. "Every AI app needs vector search. We make it instant."

### Questions They'll Ask:
- **"Why Mojo?"** → Only language that can achieve this performance with memory safety
- **"Why not contribute to Faiss?"** → Different architecture, like asking why not contribute to Postgres instead of building SQLite
- **"How do you beat VC-funded competitors?"** → We're 100x faster at the physics level. They literally cannot match our startup time.