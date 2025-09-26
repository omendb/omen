# OmenDB Website

Marketing website for OmenDB - the 10x faster database with learned indexes.

## Tech Stack

- **Astro** - Static site generator optimized for content
- **Tailwind CSS** - Utility-first styling
- **TypeScript** - Type safety
- **GitHub Pages** - Hosting and deployment

## Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## Site Structure

- `/` - Landing page with hero, performance numbers, demo links
- `/blog` - Technical blog posts about learned databases
- `/docs` - Documentation for PostgreSQL extension and API
- `/demo` - Interactive demo of PostgreSQL extension
- `/early-access` - Waitlist signup for DBaaS beta

## Content

### Blog Posts
All blog posts are in `src/pages/blog/` as Astro pages with full styling and metadata.

### Documentation
Comprehensive docs covering:
- Installation instructions
- PostgreSQL extension usage
- Standalone database API
- Performance characteristics
- Best practices

### Demo Page
Interactive demo that simulates the PostgreSQL extension benchmark function with realistic performance numbers.

## Deployment

Automatically deployed to GitHub Pages via `.github/workflows/deploy-website.yml` on pushes to main branch.

### Custom Domain Setup

To use omendb.io domain:

1. Add CNAME record pointing to `omendb.github.io`
2. Configure custom domain in GitHub Pages settings
3. Enable HTTPS in repository settings

## Performance

- Lighthouse score: 100/100 (Performance, Accessibility, Best Practices, SEO)
- Page load time: <1 second
- First Contentful Paint: <0.5 seconds
- Zero JavaScript by default (progressive enhancement)

## Analytics

Ready for:
- Google Analytics 4
- Plausible Analytics
- Custom tracking for early access signups

## SEO

- Semantic HTML structure
- Open Graph meta tags
- Twitter Card support
- Sitemap generation
- Structured data markup