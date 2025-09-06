# OmenDB Development Workflow

## Linear Issue Management

### Creating Issues
```bash
# Use Linear CLI or web interface
linear issue create \
  --title "Brief descriptive title" \
  --description "Detailed problem description with acceptance criteria" \
  --team OmenDB \
  --priority <urgent|high|medium|low>
```

### Issue Templates

#### Bug Report
```markdown
## Problem Statement
Brief description of the issue

## Steps to Reproduce
1. Step one
2. Step two
3. Expected vs actual behavior

## Impact
- Performance impact
- User-facing impact
- Development blocker status

## Investigation Results
- [ ] Root cause identified
- [ ] Solution approach confirmed
- [ ] Testing strategy defined

## Success Criteria
- [ ] Bug fixed and verified
- [ ] Regression test added
- [ ] Performance restored
```

#### Feature Request
```markdown
## Feature Description
What needs to be built and why

## Acceptance Criteria
- [ ] Specific, testable requirements
- [ ] Performance targets if applicable
- [ ] Documentation requirements

## Implementation Notes
- Architecture considerations
- Dependencies on other issues
- Timeline estimates

## Success Criteria
- [ ] Feature implemented
- [ ] Tests passing
- [ ] Documentation updated
```

### Priority Guidelines

**Urgent (P1)**:
- Blocks release
- Critical performance regression (>50%)
- Production-breaking bugs
- Security vulnerabilities

**High (P2)**:
- Major features for current release
- Significant performance improvements
- Important bug fixes

**Medium (P3)**:
- Minor features
- Optimizations
- Non-blocking bugs
- Technical debt

**Low (P4)**:
- Nice-to-have features
- Future considerations
- Documentation improvements

### Status Management

**Backlog**: Not started, prioritized  
**Todo**: Ready to start, dependencies resolved  
**In Progress**: Actively being worked on  
**Done**: Completed and verified  
**Canceled**: No longer needed

### Updating Issues

Always update Linear issues when:
- Starting work (Todo → In Progress)
- Making significant progress
- Discovering new information
- Changing approach or timeline
- Completing work (In Progress → Done)

**Template for updates:**
```markdown
## Progress Update (Date)

### ✅ Completed
- Item 1
- Item 2

### 🔄 In Progress  
- Item 3 (50% complete)
- Item 4 (blocked by X)

### 📋 Next Steps
- Item 5
- Item 6

### 🚨 Blockers/Issues
- Describe any blockers
- Link to dependent issues
```

## Performance Regression Tracking

### Automated Tracking
```bash
# Run regression tests
cd /path/to/omendb
PYTHONPATH=python:$PYTHONPATH python benchmarks/regression_tracker.py

# View history
cat benchmarks/regression_history.json | jq '.benchmarks[-1]'

# Schedule regular runs
crontab -e
# Add: 0 */6 * * * cd /path/to/omendb && ./benchmarks/track_metrics.sh >> benchmarks/metrics.log 2>&1
```

### Thresholds
- **Critical**: >50% performance drop
- **Warning**: >20% performance drop  
- **Search regression**: >50% latency increase

### Response Process
1. **Detection**: Automated system flags regression
2. **Triage**: Create urgent Linear issue within 24h
3. **Investigation**: Root cause analysis required
4. **Fix**: Restore performance or document tradeoff
5. **Prevention**: Add specific regression test

### Manual Testing
```bash
# Before significant changes
./benchmarks/track_metrics.sh

# After changes  
./benchmarks/track_metrics.sh

# Compare results manually if needed
```

## CI/CD Integration (Planned)

### GitHub Actions Workflow
```yaml
# .github/workflows/performance.yml
name: Performance Regression Tests
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  regression-test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Setup Mojo
      # ... setup steps
    - name: Run regression tests
      run: |
        PYTHONPATH=python:$PYTHONPATH python benchmarks/regression_tracker.py
    - name: Comment PR with results
      # ... comment results on PR
```

### PR Review Process
1. **Code Review**: Technical correctness
2. **Performance Check**: Automated regression detection
3. **Linear Update**: Update related issues
4. **Documentation**: Update docs if needed

## Build & Test Workflow

### Development Build
```bash
cd /path/to/omendb
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib
```

### Testing
```bash
# Quick smoke test
PYTHONPATH=python:$PYTHONPATH python benchmarks/quick_benchmark.py

# Regression test
PYTHONPATH=python:$PYTHONPATH python benchmarks/regression_tracker.py

# Scale test (when working)
PYTHONPATH=python:$PYTHONPATH python benchmarks/test_100k_scale.py
```

### Before Committing
- [ ] Code builds without errors
- [ ] Basic tests pass
- [ ] No performance regressions detected
- [ ] Linear issues updated
- [ ] Documentation updated if needed

## Documentation Updates

### When to Update Docs
- Performance metrics change significantly
- New features added
- Architecture changes
- Bug fixes that affect user experience
- API changes

### Documentation Hierarchy
```
docs/
├── STATUS.md           # Current state, metrics, known issues
├── TECH_SPEC.md        # Architecture and implementation
├── WORKFLOW.md         # This file - development processes
├── PERFORMANCE_INVESTIGATION.md  # Technical deep-dives
└── archive/            # Historical documents
```

### Update Process
1. **Identify affected docs**: Which files need updates?
2. **Update facts**: Change metrics, status, etc.
3. **Maintain consistency**: Ensure no contradictions
4. **Archive if major change**: Move old versions to archive/
5. **Update CLAUDE.md**: If significant milestone

### Quality Checks
- [ ] No duplicate information across files
- [ ] All metrics are current and accurate
- [ ] Cross-references work correctly
- [ ] Dates use ISO format (YYYY-MM-DD)
- [ ] Linear issues referenced where relevant

## Release Process

### Pre-Release Checklist
- [ ] All P1 issues resolved
- [ ] Performance regression tests passing
- [ ] Scale testing complete (blocked by OMEN-27)
- [ ] Documentation updated with real metrics
- [ ] Linear roadmap updated

### Release Steps
1. **Freeze code**: No new features
2. **Final testing**: Comprehensive test suite
3. **Documentation**: Update with final metrics
4. **Linear cleanup**: Close completed issues
5. **Release notes**: Document changes
6. **Post-release**: Monitor for issues

## Current Status & Issues

### Active Linear Issues (Sept 1, 2025)
- **OMEN-27**: 🔥 CRITICAL Performance regression (95% slower)
- **OMEN-21**: Scale testing blocked by OMEN-27
- **OMEN-26**: Segfaults at 105K vectors (may resolve with OMEN-27)
- **OMEN-7**: SIMD optimizations (in progress)

### Regression Tracking Status
✅ **System created**: Basic automated tracking  
❌ **CI integration**: Not connected to GitHub Actions  
❌ **Alerting**: Manual detection only  
⚠️ **Reliability**: Crashes at 50K+ vectors prevent full testing

### Next Workflow Improvements
1. Fix CI integration for automated PR checks
2. Add Linear API integration for automatic issue updates
3. Implement alerting for critical regressions
4. Create performance dashboard/visualization

---
*OmenDB-specific workflow procedures. See DOC_STANDARDS.md for general documentation practices.*