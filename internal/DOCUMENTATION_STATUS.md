# Documentation Status

*Last Updated: January 2025*

## ✅ Completed Updates

### Root Documentation
- **README.md**: Updated with current project status, accurate test counts, known issues
- **CLAUDE.md**: Already current with AI context patterns
- **DEVELOPMENT.md**: Already contains proper workflows

### Project Documentation
- **omendb/engine/README.md**: Updated with current status, bottleneck warnings
- **zendb/README.md**: Updated to reflect actual implementation (87% tests passing)
- **omendb/server/README.md**: Added status warning about potential outdated code
- **omendb/web/README.md**: Added note about content needing updates

### Internal Documentation
- **internal/INDEX.md**: Completely rewritten with current status and proper organization
- **internal/archive/omendb-engine-investigations/**: Moved 13 redundant investigation docs

## 🧹 Cleanup Actions

### Files Archived
- Moved 13 investigation/debugging docs from `omendb/engine/benchmarks/` to `internal/archive/omendb-engine-investigations/`
- These were various stages of performance debugging and memory investigations

### Files Updated
- Removed outdated dates (September 2025 references)
- Removed old Linear issue references
- Updated test counts and status indicators
- Added proper status badges and warnings

## ⚠️ Areas Needing Attention

### Server Documentation
- `omendb/server/docs/internal/` contains ~30 design docs that may be outdated
- Need to review if server code is still aligned with engine

### Web Content
- Marketing copy needs updating to reflect actual capabilities
- Remove claims about features not yet implemented

### Internal Strategy Docs
- `internal/private/business/` contains multiple investor overview versions
- `internal/technical/` has specs that may be outdated

## 📋 Documentation Guidelines

### Keep Current
- Test counts and passing rates
- Known issues and bottlenecks
- Development commands and workflows

### Avoid
- Future dates and unrealistic timelines
- Feature claims for unimplemented functionality
- Duplicate investigation/debugging docs

### Best Practices
1. Archive old investigations rather than deleting
2. Update status indicators when tests change
3. Note when components may be outdated
4. Keep README files focused on current state

## Next Steps

1. Review and consolidate server internal docs
2. Update web marketing content to match reality
3. Clean up duplicate business/investor docs
4. Add performance benchmark results when available

---

*Documentation health: Good - major cleanup completed, READMEs reflect current state*