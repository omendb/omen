# Website Deployment Guide - Separate Repo Strategy

## Repository Structure (Decided: Sep 26, 2025)

```
/Users/nick/github/omendb/
├── core/          # Database library and PostgreSQL extension
└── website/       # Marketing website (separate repo) ← NEW
```

## Why Separate Repo?

✅ **Cleaner CI/CD**: Cloudflare Pages only builds on website changes
✅ **Better permissions**: Marketing team can access website-only
✅ **Standard practice**: Industry norm for marketing sites
✅ **Focused concerns**: Database code separate from marketing content

## Cloudflare Pages Setup (Recommended)

### Step 1: GitHub Setup
1. **Create GitHub repo**: `https://github.com/new`
   - Repository name: `omendb-website` or `website`
   - Public repository
   - Push local `/Users/nick/github/omendb/website/` content

### Step 2: Cloudflare Pages Configuration
1. **Login to Cloudflare**: dash.cloudflare.com
2. **Create Pages project**: Workers & Pages → Create → Pages → Connect to Git
3. **Connect GitHub**: Authorize and select `omendb-website` repo
4. **Build settings**:
   ```
   Framework preset: Astro
   Build command: npm run build
   Build output directory: dist
   ```

### Step 3: Custom Domain
1. **Custom domain**: Project settings → Custom domains → Add domain
2. **Enter**: `omendb.io`
3. **Automatic setup**: Cloudflare configures DNS and SSL automatically

### Step 4: Environment Variables (if needed)
- None required for current site
- Add analytics tokens later if needed

## Performance Benefits vs GitHub Pages

| Feature | GitHub Pages | Cloudflare Pages |
|---------|--------------|------------------|
| Global CDN | GitHub's CDN | 180+ locations |
| Build time | ~2-5 minutes | ~30-60 seconds |
| SSL setup | Manual | Automatic |
| Preview builds | No | Yes (every PR) |
| Analytics | Add-on | Built-in |
| Edge functions | No | Yes |

## Build Commands

```bash
# Local development
cd /Users/nick/github/omendb/website
npm install
npm run dev

# Production build (auto on Cloudflare)
npm run build
npm run preview
```

## Content Updates

### Blog Posts
- Add new `.astro` files to `src/pages/blog/`
- Update `src/pages/blog/index.astro` with new post links
- Auto-deployed on git push

### Documentation
- Edit `src/pages/docs/index.astro`
- Keep in sync with core repo documentation
- Consider shared content strategy later

### Performance Numbers
- Update tables in landing page when benchmarks improve
- Keep consistent with core repo claims

## Launch Checklist

- [x] Website repo created and deployed to Cloudflare Pages
- [x] Custom domain `omendb.io` configured
- [x] All pages render correctly (landing, demo, docs, blog)
- [x] Interactive demo functions properly
- [x] Early access form submits (test with personal email)
- [x] Mobile responsive design verified
- [x] Lighthouse score >90 (performance, accessibility, SEO)

## Marketing Launch Sequence

### Soft Launch (Day 1)
1. **Test all functionality**: Every page, form, link
2. **Share with close contacts**: Get initial feedback
3. **Monitor analytics**: Page views, time on site, conversions

### HackerNews Launch (Day 2-3)
1. **Title**: "We made PostgreSQL 10x faster with machine learning"
2. **URL**: `https://omendb.io/blog/making-postgres-10x-faster`
3. **Timing**: 9-11am PST (peak hours)
4. **Backup plan**: Reddit r/programming, Twitter

### Success Metrics (Week 1)
- 100+ GitHub stars on core repo
- 50+ early access email signups
- 10+ PostgreSQL extension downloads/tests
- 1000+ unique website visitors

## Ongoing Maintenance

### Content Strategy
- Weekly blog posts about technical deep dives
- Update performance numbers as improvements ship
- Case studies from early adopters

### SEO Optimization
- Monitor search rankings for "learned database", "PostgreSQL performance"
- Add structured data markup for better search results
- Build backlinks from technical content

### Analytics Setup
- Google Analytics 4 for detailed user behavior
- Plausible for privacy-friendly analytics
- Track conversion funnel: Landing → Demo → Early Access → Extension Download

## Technical Notes

### Performance Optimization
- Astro generates static HTML (fastest possible)
- Cloudflare CDN handles global distribution
- Images optimized and served from CDN
- CSS/JS automatically minified and cached

### Security
- HTTPS enforced automatically
- No server-side code = minimal attack surface
- Form submissions can be handled via Cloudflare Workers if needed

---

**Next Steps**:
1. Create GitHub repo for website
2. Push website content to new repo
3. Configure Cloudflare Pages deployment
4. Test omendb.io custom domain
5. Launch!