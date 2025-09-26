# OmenDB Monorepo Structure

## Proposed Organization
```
omendb/
├── core/               # Current Rust learned index library
├── learneddb/          # Standalone database (new)
├── extension/          # PostgreSQL extension (stabilized)
├── website/            # Blog, docs, marketing site
│   ├── blog/
│   │   └── posts/
│   ├── docs/
│   └── static/
├── benchmarks/         # Shared benchmarking suite
├── internal/           # Internal docs (STATUS, ROADMAP, etc.)
└── .github/
    ├── workflows/      # CI/CD
    └── ISSUE_TEMPLATE/ # Bug reports, features
```

## Benefits of Monorepo
1. **Single Claude Code context** - AI agent can manage everything
2. **Atomic commits** - Blog posts ship with code changes
3. **Shared benchmarks** - Consistent performance testing
4. **Unified CI/CD** - One test suite for everything
5. **Cross-linking** - Docs reference exact code versions

## Migration Plan
1. Keep everything in `omendb/core/` for now
2. Create subdirectories as needed
3. Move files gradually as they stabilize
4. No breaking changes to existing code

## Website Strategy
- Static site generator (Zola or mdBook)
- Markdown for all content
- Deploy to GitHub Pages or Vercel
- Blog posts in `website/blog/posts/`
- Docs auto-generated from code comments

This keeps everything manageable from a single Claude Code instance.