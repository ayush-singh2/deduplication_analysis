# GitHub Repository Setup Guide

## 🎯 Repository Information

- **Repository Name**: `dedupl`
- **Organization**: `humanrightsconnected`
- **URL**: <https://github.com/humanrightsconnected/dedupl>
- **Clone URL**: <https://github.com/humanrightsconnected/dedupl.git>

## 🚀 Initial Setup Steps

### 1. Push to New Repository

If you haven't already pushed to the new repository:

```bash
# Add the new remote (if not already added)
git remote add origin https://github.com/humanrightsconnected/dedupl.git

# Push main branch
git push -u origin main

# Push any other branches
git push origin --all

# Push tags
git push origin --tags
```

### 2. GitHub Repository Settings

#### Repository Configuration

1. **Description**: "Media file deduplication toolkit using perceptual fingerprinting"
2. **Website**: <https://github.com/humanrightsconnected/dedupl>
3. **Topics**: `deduplication`, `media`, `audio`, `video`, `image`, `fingerprinting`, `python`

#### Branch Protection Rules

Navigate to Settings → Branches → Add protection rule for `main`:

- ✅ Require a pull request before merging
- ✅ Require status checks to pass before merging
- ✅ Require branches to be up to date before merging
- ✅ Include administrators

#### Required Status Checks

Add these checks (will appear after first CI run):

- `lint / Lint Code`
- `test / Test Python 3.11 on ubuntu-latest`
- `build / Build Distribution`

### 3. Enable GitHub Features

#### Issues & Projects

- ✅ Enable Issues for bug reports and feature requests
- ✅ Enable Projects for project management
- ✅ Enable Wiki for extended documentation

#### GitHub Actions

- ✅ Enable Actions (should be automatic with `.github/workflows/ci.yml`)
- ✅ Enable Dependabot for security updates

#### Secrets (if needed for future features)

Navigate to Settings → Secrets and variables → Actions:

- Add any API keys or tokens for external services
- Consider adding PyPI tokens for automated releases

### 4. Repository Labels

Create these labels for better issue management:

```bash
# Bug tracking
bug           # Red       - Something isn't working
critical      # Dark red  - Critical bugs

# Feature development  
enhancement   # Blue      - New feature or request
feature       # Green     - Feature implementation

# Maintenance
documentation # Light blue - Documentation improvements
refactor      # Purple    - Code refactoring
performance   # Orange    - Performance improvements

# Testing
testing       # Yellow    - Testing related
ci/cd         # Gray      - CI/CD improvements

# Priority
priority-high   # Red     - High priority
priority-medium # Orange  - Medium priority  
priority-low    # Yellow  - Low priority

# Areas
area-audio    # Pink     - Audio deduplication
area-image    # Cyan     - Image deduplication
area-video    # Magenta  - Video deduplication
area-core     # Brown    - Core functionality
```

### 5. Create Initial Issues

Consider creating these initial issues to track ongoing work:

1. **Complete video deduplication module** (#1)
   - Label: `enhancement`, `area-video`
   - Migrate legacy video script to new structure

2. **Add integration tests** (#2)
   - Label: `testing`, `enhancement`
   - End-to-end testing with real media files

3. **Performance benchmarking** (#3)
   - Label: `performance`, `testing`
   - Establish baseline performance metrics

4. **GUI interface** (#4)
   - Label: `enhancement`, `feature`
   - Desktop application for non-technical users

## 📋 README Badge Setup

Add these badges to your README.md:

```markdown
# dedupl

[![CI](https://github.com/humanrightsconnected/dedupl/workflows/CI/badge.svg)](https://github.com/humanrightsconnected/dedupl/actions)
[![codecov](https://codecov.io/gh/humanrightsconnected/dedupl/branch/main/graph/badge.svg)](https://codecov.io/gh/humanrightsconnected/dedupl)
[![Python 3.8+](https://img.shields.io/badge/python-3.8+-blue.svg)](https://www.python.org/downloads/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Code style: black](https://img.shields.io/badge/code%20style-black-000000.svg)](https://github.com/psf/black)
```

## 🔄 Release Strategy

### Semantic Versioning

Follow semantic versioning (semver):

- `MAJOR.MINOR.PATCH` (e.g., `1.0.0`)
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Process

1. Update version in `pyproject.toml`
2. Update `CHANGELOG.md` with release notes
3. Create git tag: `git tag v1.0.0`
4. Push tag: `git push origin v1.0.0`
5. GitHub Actions can automate PyPI publishing

### GitHub Releases

Create releases for major versions:

- Use tag names like `v1.0.0`
- Include changelog in release notes
- Attach distribution files if needed

## 🛡️ Security Setup

### Security Policy

Create `.github/SECURITY.md`:

```markdown
# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |

## Reporting a Vulnerability

Please report security vulnerabilities by emailing [security contact].
Do not create public issues for security vulnerabilities.
```

### Dependabot Configuration

Create `.github/dependabot.yml`:

```yaml
version: 2
updates:
  - package-ecosystem: "pip"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 5
```

## 📊 Analytics & Monitoring

### GitHub Insights

Monitor repository health with:

- **Code frequency**: Track development activity
- **Contributors**: Monitor team contributions
- **Traffic**: See who's visiting and cloning
- **Dependencies**: Monitor dependency health

### CodeCov Integration

Already configured in CI workflow. Set up account at:

- <https://codecov.io/gh/humanrightsconnected/dedupl>

## 🤝 Community Setup

### Contributing Guidelines

Create `CONTRIBUTING.md` with:

- Code of conduct
- Development setup instructions
- Pull request process
- Coding standards

### Issue Templates

Create `.github/ISSUE_TEMPLATE/`:

- `bug_report.md` - Bug report template
- `feature_request.md` - Feature request template
- `question.md` - Question template

### Pull Request Template

Create `.github/pull_request_template.md`:

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature  
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Tests pass locally
- [ ] Added tests for new functionality
- [ ] Updated documentation

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Comments added to complex code
- [ ] No new warnings introduced
```

## ✅ Verification Checklist

After setup, verify:

- [ ] Repository is public/private as intended
- [ ] CI workflow runs successfully
- [ ] Tests pass on all platforms
- [ ] Code coverage reports correctly
- [ ] Branch protection rules work
- [ ] Issues and PRs can be created
- [ ] All links in documentation work
- [ ] Repository appears in organization

Your `dedupl` repository is now professionally configured and ready for collaborative development! 🎉
