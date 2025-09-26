# Monorepo Organization - OmenDB

## Structure Overview

```
omendb/core/
├── src/                     # Core learned index library (Rust)
│   ├── lib.rs              # Main library exports
│   ├── linear.rs           # LinearIndex implementation
│   ├── rmi.rs              # RMI (Recursive Model Index)
│   └── error.rs            # Error handling
├── apps/                   # Applications and services
│   └── website/            # Marketing website (Astro)
│       ├── src/pages/      # Website pages
│       ├── src/layouts/    # Page layouts
│       └── src/components/ # Reusable components
├── docs/                   # Organized documentation
│   ├── internal/           # AI agent and development docs
│   ├── extension/          # PostgreSQL extension docs
│   ├── database/           # Standalone database docs
│   └── website/            # Website strategy and content docs
├── learneddb/              # Standalone database (Rust + RocksDB)
├── pgrx-extension/         # PostgreSQL extension (Rust + pgrx)
├── benchmarks/             # Performance benchmarks
├── external/               # Research papers and references
└── .github/workflows/      # CI/CD automation
```

## Benefits of This Organization

### 1. Clear Separation of Concerns
- **Core library** (`src/`) - Pure learned index implementations
- **Applications** (`apps/`) - User-facing services and websites
- **Documentation** (`docs/`) - Organized by audience and purpose
- **Packages** (`learneddb/`, `pgrx-extension/`) - Deployable components

### 2. AI Agent Efficiency
- All related docs in consistent structure under `docs/`
- Internal docs separate from user-facing docs
- Easy navigation for Claude Code and other AI tools

### 3. Developer Experience
- Monorepo allows atomic changes across components
- Shared tooling and CI/CD workflows
- Clear development vs production boundaries

### 4. Deployment Flexibility
- Website deploys independently via GitHub Pages
- Database packages can be published separately
- Extension follows PostgreSQL ecosystem patterns

## Documentation Strategy

### Internal Docs (`docs/internal/`)
**Audience**: AI agents, developers, maintainers

- `ARCHITECTURE.md` - Complete system design
- `BUSINESS.md` - Strategy and market analysis
- `STATUS.md` - Current performance metrics
- `ROADMAP.md` - Development timeline
- `RESEARCH.md` - Academic research and findings

### Extension Docs (`docs/extension/`)
**Audience**: PostgreSQL users, DBAs

- Installation guides
- SQL function reference
- Performance tuning
- Troubleshooting

### Database Docs (`docs/database/`)
**Audience**: Application developers

- API reference
- Integration examples
- Performance characteristics
- Migration guides

### Website Docs (`docs/website/`)
**Audience**: Marketing, content strategy

- `WEBSITE_STRATEGY.md` - Overall approach
- Content guidelines
- SEO strategy
- Analytics setup

## Git Workflow

### Branch Strategy
- `main` - Production ready code
- Feature branches for development
- Automated deployment on push to main

### Commit Organization
- Atomic commits per logical component
- Clear commit messages with scope
- Example: `feat(website): add interactive demo page`

### AI Agent Guidelines
- Always read `docs/internal/` first for context
- Update relevant docs when making changes
- Keep documentation current with code changes

## Deployment Pipelines

### Website (`apps/website/`)
- Builds on push to main (if website files changed)
- Deploys to GitHub Pages automatically
- Custom domain: omendb.io

### Core Library (`src/`)
- Tests run on all PRs
- Publishable to crates.io
- Semantic versioning

### Database (`learneddb/`)
- Integration tests with RocksDB
- Docker images for deployment
- Performance regression testing

### Extension (`pgrx-extension/`)
- PostgreSQL version matrix testing
- Package for multiple distros
- Integration with PostgreSQL ecosystem

## Future Expansions

### When to Add New Directories

**`packages/`** - When we have 3+ reusable libraries
**`tools/`** - When we have custom build/dev tools
**`examples/`** - When we have 5+ usage examples
**`infrastructure/`** - When we have deployment configs

### Scaling Guidelines

- Keep flat structure until clear need for nesting
- Optimize for AI agent navigation
- Document organizational decisions
- Maintain consistent patterns across components

## Success Metrics

### Developer Productivity
- Time to understand codebase (new contributors)
- Time to make changes across components
- AI agent task completion rate

### Deployment Reliability
- Build success rate across all components
- Deployment frequency
- Mean time to recovery

### Documentation Quality
- Coverage of all user-facing features
- Accuracy of installation guides
- User feedback on clarity

---

*This organization optimizes for rapid development while maintaining clarity for both human developers and AI agents.*