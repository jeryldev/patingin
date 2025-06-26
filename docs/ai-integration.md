# AI Integration Guide

Comprehensive guide to Patingin's AI-powered code fixing with Claude Code integration.

## Overview

Patingin integrates with **Claude Code CLI** to provide intelligent, context-aware code fixes for detected anti-patterns. This enables automated code improvement while maintaining safety and control.

### Key Features
- **Intelligent Fix Generation** - Context-aware code improvements
- **Batch Processing** - Fix multiple violations efficiently
- **Interactive Confirmation** - Preview and approve fixes
- **Confidence Scoring** - Quality assessment of generated fixes
- **Multi-language Support** - Works across all supported languages

---

## Setup and Installation

### 1. Install Claude Code CLI

First, install Node.js 18+ and then the Claude Code CLI:

```bash
# Install Node.js 18+ (if not already installed)
# https://nodejs.org/

# Install Claude Code CLI via npm
npm install -g @anthropic-ai/claude-code

# Verify installation (note: command is 'claude', not 'claude-code')
claude --version
```

### 2. Authentication

Set up Claude Code authentication:

```bash
# Login to Claude Code (follow interactive prompts)
claude auth login

# Authentication options:
# - Anthropic Console (default)
# - Claude App (Pro/Max plan required)
# - Enterprise platforms (Bedrock/Vertex AI)
```

### 3. Verify Integration

Check that Patingin can detect Claude Code:

```bash
patingin setup
# Should show something like:
# ✓ Claude Code CLI: 1.0.35 (Claude Code)
#   ✨ Auto-fix integration: Ready
```

---

## Basic Usage

### 1. Preview Mode

See what fixes would be applied without making changes:

```bash
# Show fix suggestions
patingin review --suggest

# Example output:
# 🔧 Suggested Fixes:
# 
# 📁 lib/user.ex:42
#    Issue: Dynamic Atom Creation
#    Current: String.to_atom(user_input)
#    Suggestion: String.to_existing_atom(user_input)
```

### 2. Interactive Mode

Apply fixes with confirmation for each change:

```bash
# Launch interactive Claude Code session
patingin review --fix

# Example interaction:
# 📍 Fix Preview
# File: lib/user.ex
# Line: 42
# Issue: Dynamic Atom Creation
# 
# Before:
#   String.to_atom(user_input)
# 
# After:
#   String.to_existing_atom(user_input)
# 
# ❓ Apply this fix? [y/N/a/q]: y
```

### 3. Batch Mode

Apply all fixes automatically without prompts:

```bash
# This workflow is deprecated - use interactive --fix instead
# patingin review --auto-fix --no-confirm

# Example output:
# 🤖 Processing 5 violations with Claude Code...
#   [1/5] Fixing Dynamic Atom Creation in lib/user.ex:42... ✅ Fixed
#   [2/5] Fixing Console.log in Production in app.js:15... ✅ Fixed
#   [3/5] Fixing Long Parameter List in lib/auth.ex:28... ⚠️ Low confidence
#   [4/5] Fixing Eval Usage in script.js:10... ❌ Invalid fix
#   [5/5] Fixing Var Declaration in old.js:5... ✅ Fixed
```

---

## Advanced Features

### Confidence Scoring

Patingin evaluates the quality of AI-generated fixes:

```bash
# Interactive mode with Claude Code (recommended)
patingin review --fix

# Confidence factors:
# - Code structure quality (0.1 boost)
# - Syntax correctness (0.1 boost)
# - Explanation detection (-0.3 penalty)
# - Base confidence: 0.7
```

**Confidence Levels:**
- **0.9-1.0**: Very high confidence, safe to auto-apply
- **0.7-0.9**: Good confidence, recommend review
- **0.5-0.7**: Medium confidence, manual review needed
- **0.0-0.5**: Low confidence, likely needs manual fix

### Batch Processing Configuration

Customize batch fix behavior:

```bash
# Use interactive Claude Code session for best results
patingin review --fix
```

### Language-specific Fixing

Focus AI fixes on specific languages:

```bash
# Fix only Elixir violations
patingin review --language elixir --fix

# Fix only JavaScript violations
patingin review --language javascript --fix

# Fix only security-critical issues
patingin review --severity critical --fix
```

---

## Fix Quality and Validation

### Syntax Validation

Patingin validates fixes before applying them:

**Elixir Validation:**
- Balanced parentheses, brackets, braces
- Basic syntax structure

**JavaScript/TypeScript Validation:**
- Balanced parentheses, brackets, braces
- Basic syntax structure

**Python Validation:**
- Balanced parentheses, brackets
- Basic indentation checks

**Rust Validation:**
- Balanced parentheses, brackets, braces
- Basic syntax structure

### Fix Examples

#### Elixir Fixes

```elixir
# Dynamic Atom Creation
# Before:
String.to_atom(user_input)

# After:
String.to_existing_atom(user_input)

# Long Parameter List
# Before:
def authenticate(email, password, token, device, ip, session, opts) do

# After:
def authenticate(%AuthRequest{} = request) do
```

#### JavaScript Fixes

```javascript
// Console.log in Production
// Before:
console.log("Debug info:", data);

// After:
logger.debug("Debug info:", data);

// Var Declaration
// Before:
var userName = "John";

// After:
const userName = "John";

// Unhandled Promise
// Before:
fetch('/api/data').then(response => process(response))

// After:
fetch('/api/data')
  .then(response => process(response))
  .catch(error => handleError(error))
```

#### Python Fixes

```python
# Import Organization
# Before:
from module import *

# After:
from module import specific_function, another_function

# Exception Handling
# Before:
try:
    risky_operation()
except:
    pass

# After:
try:
    risky_operation()
except SpecificException as e:
    logger.error(f"Operation failed: {e}")
    raise
```

---

## Integration Workflows

### Development Workflow

```bash
# 1. Regular development
vim lib/user.ex

# 2. Quick fix check
patingin review --suggest

# 3. Apply fixes interactively
patingin review --fix

# 4. Review all remaining issues
patingin review --fix

# 5. Final verification
patingin review
```

### CI/CD Integration

```yaml
# .github/workflows/auto-fix.yml
name: Auto-fix Code Issues

on:
  pull_request:
    branches: [ main ]

jobs:
  auto-fix:
    runs-on: ubuntu-latest
    if: contains(github.event.pull_request.labels.*.name, 'auto-fix')
    
    steps:
    - uses: actions/checkout@v3
      with:
        token: ${{ secrets.AUTO_FIX_TOKEN }}
        fetch-depth: 0
    
    - name: Setup Claude Code
      run: |
        npm install -g @anthropic-ai/claude-code
        # Note: Authentication setup varies by deployment
    
    - name: Install Patingin
      run: cargo install patingin
    
    - name: Apply Auto-fixes (Deprecated)
      run: |
        # Note: --auto-fix is deprecated, consider using --fix in interactive mode
        # patingin review --since origin/main --auto-fix --no-confirm --confidence 0.8
    
    - name: Commit fixes
      run: |
        git config user.name "Patingin Bot"
        git config user.email "bot@patingin.dev"
        git add -A
        if ! git diff --cached --quiet; then
          git commit -m "🤖 Auto-fix code quality issues
          
          Applied by Patingin with Claude Code
          
          Co-authored-by: ${{ github.actor }} <${{ github.actor }}@users.noreply.github.com>"
          git push
        fi
```

### Team Adoption Strategy

```bash
# Phase 1: Manual preview only
patingin review --suggest

# Phase 2: Interactive Claude Code sessions
patingin review --fix

# Phase 3: Regular interactive fixing
patingin review --fix

# Phase 4: Streamlined interactive workflow
patingin review --fix
```

---

## Troubleshooting

### Common Issues

#### Claude Code Not Found
```bash
# Check installation via npm
npm list -g @anthropic-ai/claude-code

# Check if command is available
which claude

# Verify PATH includes npm global bin
echo $PATH

# Reinstall if needed
npm install -g @anthropic-ai/claude-code
```

#### Authentication Issues
```bash
# Check authentication
claude-code auth status

# Re-authenticate
claude-code auth login

# Check API key
echo $CLAUDE_API_KEY
```

#### Low-quality Fixes
```bash
# Use interactive Claude Code mode
patingin review --fix

# Use interactive mode for review
patingin review --fix

# Check specific language issues
patingin review --language elixir --fix
```

### Debug Mode

```bash
# Use interactive Claude Code mode with debug info
RUST_LOG=debug patingin review --fix

# Check Claude Code execution
RUST_LOG=patingin::external=debug patingin review --fix

# Interactive mode is now the default
patingin review --fix
```

### Performance Optimization

```bash
# Process fewer violations at once
patingin review --severity critical --fix

# Focus on specific files
patingin review --staged --fix

# Use language filtering
patingin review --language elixir --fix
```

---

## Safety and Best Practices

### Safety Measures

1. **Always Review Fixes**: Start with `--suggest` mode
2. **Use Version Control**: Commit before running auto-fix
3. **Test After Fixes**: Run tests after applying fixes
4. **Gradual Adoption**: Start with high-confidence fixes only

### Best Practices

```bash
# 1. Backup before batch fixes
git add . && git commit -m "Before auto-fix"

# 2. Use interactive mode
patingin review --fix

# 3. Review changes
git diff

# 4. Test thoroughly
mix test  # or your test command

# 5. Commit clean fixes
git add .
git commit -m "🤖 Apply Patingin fixes with Claude Code"
```

### When NOT to Use Auto-fix

- **Complex refactoring** - Manual review needed
- **Business logic changes** - Requires domain knowledge
- **Performance-critical code** - Needs benchmarking
- **Public APIs** - Breaking changes require coordination
- **Legacy systems** - May have hidden dependencies

---

## Configuration

### Environment Variables

```bash
# Claude Code configuration
export CLAUDE_API_KEY="your_key"
export CLAUDE_CODE_PATH="/custom/path/claude-code"

# Patingin AI configuration
export PATINGIN_CONFIDENCE_THRESHOLD="0.8"
export PATINGIN_AUTO_FIX_TIMEOUT="30"  # seconds
export PATINGIN_MAX_FIXES_PER_RUN="50"
```

### Project Configuration

```yaml
# .patingin.yml
version: 1.0

ai_integration:
  enabled: true
  confidence_threshold: 0.8
  max_fixes_per_run: 20
  
  # Language-specific settings
  languages:
    elixir:
      confidence_threshold: 0.9  # More conservative for Elixir
    javascript:
      confidence_threshold: 0.7  # More aggressive for JS
      
  # Rule-specific settings
  rules:
    dynamic_atom_creation:
      auto_fix: true
      confidence_threshold: 0.95
    console_log_production:
      auto_fix: true
      confidence_threshold: 0.8
```

---

## API and Extensions

### Custom Fix Providers

Patingin's AI integration is extensible for custom fix providers:

```rust
// Custom fix provider interface
pub trait FixProvider {
    fn generate_fix(&self, request: &FixRequest) -> Result<FixResult>;
    fn validate_fix(&self, original: &str, fixed: &str) -> Result<bool>;
}
```

### Integration with Other Tools

```bash
# Combine with formatters
patingin review --fix
mix format  # Format after fixing

# Combine with linters
patingin review --fix
mix credo --strict  # Additional linting

# Combine with tests
patingin review --fix && mix test
```

---

## Metrics and Monitoring

### Fix Success Tracking

```bash
# Generate fix reports
patingin review --auto-fix --json > fix-report.json

# Analyze fix success rates
cat fix-report.json | jq '.fix_details[] | .applied'

# Track confidence scores
cat fix-report.json | jq '.fix_details[] | .fix_result.confidence'
```

### Team Metrics

```bash
# Weekly fix summary
patingin review --since "1 week ago" --auto-fix --json | \
  jq '{fixed: .fixed_violations, total: .total_violations}'

# Most common fixes
patingin review --auto-fix --json | \
  jq -r '.fix_details[] | .violation.rule.name' | \
  sort | uniq -c | sort -nr
```

This comprehensive AI integration makes Patingin a powerful tool for maintaining code quality while reducing manual effort through intelligent automation.