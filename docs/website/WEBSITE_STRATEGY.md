# Website Strategy - OmenDB

## Current Status: Need Professional Landing Site

### Demo Strategy
✅ **PostgreSQL Extension Demo**: Ready for independent showcase
- Stable, error-free benchmark function
- 2-3x performance demonstration
- No crashes, proper input validation
- Can be demoed separately from standalone DB

### Website Architecture Decision

**Phase 1: Landing/Marketing Site** (omendb.io)
- Host on GitHub Pages or Cloudflare Pages
- Static site generator: **Astro** (recommended)
- Domain: omendb.io (user owns this)

**Phase 2: DBaaS Dashboard** (app.omendb.io)
- Separate subdomain for customer portal
- User accounts, database management
- Only needed after market validation

## Technology Stack: Astro

**Why Astro?**
- Perfect for content-heavy sites (blog, docs)
- Excellent performance (100/100 Lighthouse scores)
- Built-in Markdown support for blog posts
- Works seamlessly with GitHub Pages/CF Pages
- Can add React components later if needed
- Zero JS by default, progressive enhancement

**Inspiration Sites:**
- Modular.com - Clean, technical, great performance demos
- Vercel.com - Simple hero, clear value prop, great docs
- Stripe.com - Professional, clear sections, excellent content

## Site Structure

```
omendb.io/
├── / (landing page)
├── /blog (technical posts)
├── /docs (documentation)
├── /demo (PostgreSQL extension demo)
└── /early-access (signup form)
```

### Landing Page Sections
1. **Hero**: "10x faster than PostgreSQL" + demo link
2. **Performance Numbers**: Real benchmark table
3. **How It Works**: 3-step explanation with visuals
4. **Try Now**: PostgreSQL extension + standalone DB
5. **Blog Preview**: Latest technical posts
6. **Early Access**: Email signup for DBaaS beta

### Demo Strategy
- Embedded PostgreSQL extension demo
- Show benchmark results in real-time
- Link to GitHub for technical users
- Clear "this is just the beginning" messaging

## Content Strategy

**Blog Posts** (website/blog/posts/)
- Technical deep dives
- Performance comparisons
- Use case studies
- Behind-the-scenes development

**Documentation** (website/docs/)
- PostgreSQL extension installation
- Standalone database setup
- API reference
- Architecture explanations

## Deployment

**GitHub Pages** (Recommended)
- Free hosting for omendb.io
- Automatic deployment on push
- Custom domain support
- Perfect for static sites

**Cloudflare Pages** (Alternative)
- Slightly better performance
- More advanced features
- Same workflow as GitHub Pages

## Monorepo Organization

**Current**: Good enough for now, but can be improved

**Future** (when needed):
```
core/
├── packages/
│   ├── omendb/ (core library)
│   ├── learneddb/ (standalone DB)
│   └── extension/ (PostgreSQL)
├── apps/
│   ├── website/ (landing site)
│   └── dashboard/ (future DBaaS)
├── docs/
│   ├── internal/ (AI/dev docs)
│   ├── extension/ (PostgreSQL docs)
│   ├── database/ (standalone docs)
│   └── website/ (website docs)
└── tools/ (scripts, configs)
```

## Next Steps

1. **Build Astro website** in apps/website/
2. **Configure GitHub Pages** deployment
3. **Point omendb.io** to GitHub Pages
4. **Test PostgreSQL extension demo** integration
5. **Launch and measure** (target: 500+ stars = continue)

## Success Metrics

**Week 1**:
- 100+ GitHub stars
- 50+ email signups
- 10+ PostgreSQL extension installs

**Month 1**:
- 500+ GitHub stars
- 200+ email signups
- 50+ PostgreSQL extension installs
- 5+ production use cases

**Month 3**:
- 1000+ GitHub stars
- 500+ email signups
- Demand validated → Build DBaaS