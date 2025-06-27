# Workflows Guide

Common development workflows and integration patterns with Patingin.

## Daily Development Workflows

### 1. Continuous Code Review

Integrate Patingin into your regular development cycle for immediate feedback.

```bash
# Start working on a feature
git checkout -b feature/user-authentication

# Make changes
vim lib/user.ex
vim lib/auth.ex

# Quick check as you work
patingin review

# Fix any issues interactively with Claude Code
patingin review --fix

# Continue development...
vim test/user_test.exs

# Final check before commit
patingin review --suggest

# Commit clean code
git add .
git commit -m "Add user authentication"
```

### 2. Pre-commit Validation

Catch issues before they enter version control.

```bash
# Stage your changes
git add .

# Check staged changes
patingin review --staged

# Fix critical issues interactively with Claude Code
patingin review --staged --severity critical --fix

# Commit only when clean
if patingin review --staged --severity critical --no-color; then
    git commit -m "Your commit message"
else
    echo "❌ Critical violations found. Please fix before committing."
fi
```

### 3. Work-in-Progress Review

Review current changes without committing.

```bash
# Check what you're currently working on
patingin review --uncommitted

# Focus on specific languages
patingin review --uncommitted --language elixir

# Get suggestions for improvement
patingin review --uncommitted --suggest
```

---

## Feature Development Workflows

### 4. Feature Branch Review

Comprehensive review before merging features.

```bash
# Review all changes in feature branch
patingin review --since origin/main

# Focus on critical issues
patingin review --since origin/main --severity critical

# Get detailed JSON for documentation
patingin review --since origin/main --json > feature-review.json

# Launch interactive Claude Code session to fix issues
patingin review --since origin/main --fix
```

### 5. Stacked Features

Manage dependent feature branches.

```bash
# Working on feature/auth-system
git checkout feature/auth-system
patingin review --since main

# Create dependent feature
git checkout -b feature/user-profile
# ... make changes ...

# Review only new changes (since parent feature)
patingin review --since feature/auth-system

# Review entire stack
patingin review --since main
```

### 6. Incremental Development

Review changes across multiple commits.

```bash
# Review last 3 commits
patingin review --since HEAD~3

# Review since specific commit
patingin review --since abc123

# Review since last tag
patingin review --since v1.2.0
```

---

## Team Collaboration Workflows

### 7. Pull Request Preparation

Ensure high-quality PRs before submission.

```bash
# Comprehensive PR review
patingin review --since origin/main --json > pr-violations.json

# Share violations with team
cat pr-violations.json | jq '.summary'

# Launch interactive Claude Code session to fix issues
patingin review --since origin/main --fix

# Manual review of remaining issues
patingin review --since origin/main --suggest

# Final verification
patingin review --since origin/main
```

### 8. Code Review Integration

Integrate with code review tools.

```bash
# Generate review comments
patingin review --since origin/main --json | \
  jq -r '.violations[] | "\(.file_path):\(.line_number): \(.rule_name) - \(.description)"'

# Focus on security for security review
patingin review --since origin/main --severity critical

# Language-specific reviews
patingin review --since origin/main --language elixir
```

### 9. Team Rule Management

Manage team-specific rules and conventions.

```bash
# Set up team rules
patingin rules add --project --elixir "Use team GenServer pattern"
patingin rules add --project --javascript "Use team logging framework"

# Share rules via git (commit .patingin.yml)
git add .patingin.yml
git commit -m "Add team code quality rules"

# Team members pull and benefit from shared rules
git pull
patingin rules  # Now shows team rules
```

---

## CI/CD Integration Workflows

### 10. GitHub Actions Integration

```yaml
# .github/workflows/code-quality.yml
name: Code Quality

on:
  pull_request:
    branches: [ main ]
  push:
    branches: [ main ]

jobs:
  patingin:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
      with:
        fetch-depth: 0  # Need full history for --since
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Install Patingin
      run: cargo install patingin
    
    - name: Run Patingin
      run: |
        # For PRs: check all changes
        if [ "${{ github.event_name }}" = "pull_request" ]; then
          patingin review --since origin/main --json > violations.json
        else
          # For pushes: check last commit
          patingin review --json > violations.json
        fi
    
    - name: Check Critical Violations
      run: |
        critical_count=$(cat violations.json | jq '.summary.critical_count')
        if [ "$critical_count" -gt 0 ]; then
          echo "❌ Found $critical_count critical violations"
          cat violations.json | jq '.violations[] | select(.severity == "critical")'
          exit 1
        fi
    
    - name: Upload Results
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: patingin-results
        path: violations.json
```

### 11. Pre-commit Hook Integration

```bash
#!/bin/sh
# .git/hooks/pre-commit

echo "🔍 Running Patingin pre-commit check..."

# Check staged changes for critical violations
if ! patingin review --staged --severity critical --no-color; then
    echo ""
    echo "❌ Critical violations found in staged changes."
    echo "💡 Fix them with: patingin review --staged --fix"
    echo "💡 Or commit with: git commit --no-verify"
    exit 1
fi

# Optional: Launch Claude Code to fix minor issues
echo "🤖 Launching Claude Code to fix violations..."
patingin review --staged --severity warning --fix

# Claude Code will handle staging changes as needed

echo "✅ Pre-commit check passed!"
exit 0
```

### 12. GitLab CI Integration

```yaml
# .gitlab-ci.yml
stages:
  - quality

patingin-check:
  stage: quality
  image: rust:latest
  before_script:
    - apt-get update && apt-get install -y git
    - cargo install patingin
  script:
    - git fetch origin $CI_DEFAULT_BRANCH
    - patingin review --since origin/$CI_DEFAULT_BRANCH --json > violations.json
    - |
      critical_count=$(cat violations.json | jq '.summary.critical_count')
      major_count=$(cat violations.json | jq '.summary.major_count')
      echo "Found $critical_count critical, $major_count major violations"
      
      if [ "$critical_count" -gt 0 ]; then
        echo "❌ Critical violations must be fixed"
        exit 1
      fi
  artifacts:
    reports:
      junit: violations.json
    paths:
      - violations.json
    expire_in: 1 week
  only:
    - merge_requests
```

---

## Specialized Workflows

### 13. Security-focused Review

Focus on security-critical patterns.

```bash
# Security-only scan
patingin review --severity critical

# Security scan for entire feature
patingin review --since origin/main --severity critical

# Fix security issues interactively with Claude Code
patingin review --severity critical --fix

# Security report for compliance
patingin review --severity critical --json > security-scan.json
```

### 14. Performance Review

Focus on performance-related anti-patterns.

```bash
# Look for performance issues
patingin rules --search "performance"

# Review performance-critical files
patingin review --language elixir --suggest

# Get performance improvement suggestions
patingin review --suggest | grep -i performance
```

### 15. Language Migration

Manage gradual language adoption or migration.

```bash
# Focus on new TypeScript files
patingin review --language typescript

# Review JavaScript → TypeScript migration
patingin review --since migration-start --language typescript

# Ensure consistent patterns in new language
patingin rules --typescript
```

### 16. Hotfix Workflow

Quick validation for emergency fixes.

```bash
# Create hotfix branch
git checkout -b hotfix/critical-security-fix

# Make minimal changes
vim lib/security.ex

# Quick security check
patingin review --severity critical

# Fix issues interactively with Claude Code
patingin review --fix

# Verify fix
patingin review

# Deploy quickly
git add . && git commit -m "Fix critical security issue"
```

---

## Custom Rule Workflows

### 17. Team Convention Enforcement

Enforce team-specific coding conventions.

```bash
# Define team conventions
patingin rules add --project --elixir "Use team error handling pattern"
patingin rules add --project --elixir "Follow team module organization"

# Validate against team conventions
patingin review --project

# Share conventions in repository
echo "team_rules_version: 1.2" >> .patingin.yml
git add .patingin.yml
git commit -m "Update team coding conventions"
```

### 18. Legacy Code Modernization

Gradually improve legacy codebases.

```bash
# Focus on modified legacy files
patingin review --severity major

# Modernize patterns with Claude Code
patingin review --fix

# Track improvement over time
patingin review --json > modernization-$(date +%Y%m%d).json
```

---

## Troubleshooting Workflows

### 19. Debug Patingin Issues

```bash
# Check environment
patingin setup

# Verbose logging
RUST_LOG=debug patingin review

# Test with minimal scope
patingin review --staged --language elixir

# Verify git integration
git status
git diff --name-only
```

### 20. Performance Optimization

```bash
# Time execution
time patingin review

# Profile large repositories
patingin review --language elixir  # Focus on one language
patingin review --staged           # Smaller scope

# Check project size
patingin setup | grep "Project size"
```

---

## Best Practices

### Frequency
- Run `patingin review` after each meaningful change
- Use `--staged` before every commit
- Use `--since origin/main` before PR submission

### Team Adoption
1. Start with `--severity critical` only
2. Gradually add `--severity major`
3. Introduce team rules incrementally
4. Share configurations via git

### Performance
- Use language filters for large multi-language projects
- Use `--staged` for pre-commit hooks (faster)
- Use `--since` with specific commits for focused reviews

### Integration
- Always use `--json` in CI/CD for structured processing
- Use `--no-color` in non-interactive environments
- Set appropriate exit code handling for critical violations