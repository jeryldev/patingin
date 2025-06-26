# Installation and Setup Guide

Complete guide to installing, configuring, and integrating Patingin into your development workflow.

## System Requirements

### Minimum Requirements
- **Operating System**: Linux, macOS, or Windows
- **Rust**: 1.70+ (for building from source)
- **Git**: 2.20+ (for git integration)
- **Memory**: 512MB RAM
- **Storage**: 50MB disk space

### Recommended
- **Rust**: Latest stable version
- **Git**: Latest version
- **Claude Code CLI**: For AI-powered fixes
- **Terminal**: Modern terminal with color support

---

## Installation Methods

### Option 1: Install from Source (Recommended)

```bash
# 1. Clone the repository
git clone https://github.com/your-org/patingin.git
cd patingin

# 2. Build and install
cargo install --path .

# 3. Verify installation
patingin --version
```

### Option 2: Install from Crates.io

```bash
# Install directly from crates.io (when published)
cargo install patingin

# Verify installation
patingin --version
```

### Option 3: Download Pre-built Binaries

```bash
# Download from GitHub releases (when available)
wget https://github.com/your-org/patingin/releases/latest/download/patingin-linux-x64.tar.gz
tar -xzf patingin-linux-x64.tar.gz
sudo mv patingin /usr/local/bin/

# Verify installation
patingin --version
```

### Option 4: Install via Package Managers

#### Homebrew (macOS)
```bash
# Add tap (when available)
brew tap your-org/patingin
brew install patingin
```

#### APT (Ubuntu/Debian)
```bash
# Add repository (when available)
echo "deb [trusted=yes] https://repo.patingin.dev/apt stable main" | sudo tee /etc/apt/sources.list.d/patingin.list
sudo apt update
sudo apt install patingin
```

#### Chocolatey (Windows)
```powershell
# Install via Chocolatey (when available)
choco install patingin
```

---

## Initial Setup

### 1. Verify Installation

```bash
# Check version and basic functionality
patingin --version

# Run environment check
patingin setup
```

### 2. First Project Setup

```bash
# Navigate to your project
cd /path/to/your/project

# Check project detection
patingin setup

# Run first review
patingin review
```

### 3. Configuration Directory

Patingin uses the standard config directory:

```bash
# Linux/macOS
~/.config/patingin/

# Windows
%APPDATA%\patingin\
```

Create the directory if it doesn't exist:

```bash
# Linux/macOS
mkdir -p ~/.config/patingin

# Windows
mkdir %APPDATA%\patingin
```

---

## Claude Code Integration Setup

### 1. Install Claude Code CLI

```bash
# Install Node.js 18+ first (if not already installed)
# https://nodejs.org/

# Install Claude Code CLI via npm
npm install -g @anthropic-ai/claude-code

# Verify installation
claude --version
```

### 2. Authentication

```bash
# Login interactively (follow prompts)
claude auth login

# Verify authentication 
claude auth status
```

### 3. Test Integration

```bash
# Verify Patingin can detect Claude Code
patingin setup

# Should show:
# 🤖 Claude Code Integration:
#   CLI Available: ✅ v1.2.3
#   Authentication: ✅ Authenticated
#   Auto-fix capability: ✅ Ready

# Test auto-fix functionality
patingin review --suggest
```

---

## Project Configuration

### 1. Basic Project Configuration

Create `.patingin.yml` in your project root:

```yaml
version: 1.0
project_name: "my-awesome-project"
base_branch: "main"

# Language settings
languages:
  elixir:
    severity_threshold: "major"
  javascript:
    severity_threshold: "warning"

# Ignore patterns
ignore_paths:
  - "test/**/*"
  - "deps/**/*"
  - "_build/**/*"
  - "node_modules/**/*"
  - "dist/**/*"

# AI integration
ai_integration:
  enabled: true
  confidence_threshold: 0.8
```

### 2. Team Configuration

For team projects, commit `.patingin.yml` to git:

```bash
# Create team configuration
vim .patingin.yml

# Commit to repository
git add .patingin.yml
git commit -m "Add Patingin team configuration"
git push

# Team members automatically inherit settings
git pull
patingin review  # Uses team settings
```

### 3. Custom Rules Setup

Set up project-specific rules:

```bash
# Add custom rules
patingin rules add --project --elixir "Use gettext for translations"
patingin rules add --project --javascript "Use team logger"

# View project rules
patingin rules --project

# Rules are stored in ~/.config/patingin/rules.yml
```

---

## IDE/Editor Integration

### Visual Studio Code

Create `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Patingin Review",
      "type": "shell",
      "command": "patingin",
      "args": ["review"],
      "group": "build",
      "presentation": {
        "reveal": "always",
        "panel": "new"
      },
      "problemMatcher": {
        "pattern": {
          "regexp": "^(.*):(\\d+)\\s+(\\w+)\\s+(.*)$",
          "file": 1,
          "line": 2,
          "severity": 3,
          "message": 4
        }
      }
    },
    {
      "label": "Patingin Auto-fix",
      "type": "shell",
      "command": "patingin",
      "args": ["review", "--fix"],
      "group": "build"
    }
  ]
}
```

Add keyboard shortcuts in `.vscode/keybindings.json`:

```json
[
  {
    "key": "ctrl+shift+p",
    "command": "workbench.action.tasks.runTask",
    "args": "Patingin Review"
  },
  {
    "key": "ctrl+shift+f",
    "command": "workbench.action.tasks.runTask",
    "args": "Patingin Auto-fix"
  }
]
```

### Vim/Neovim

Add to your `.vimrc` or `init.vim`:

```vim
" Patingin integration
nnoremap <leader>pr :!patingin review<CR>
nnoremap <leader>pf :!patingin review --fix<CR>
nnoremap <leader>ps :!patingin review --suggest<CR>

" Quick fix for current file
nnoremap <leader>pq :!patingin review --uncommitted<CR>
```

### Emacs

Add to your `.emacs` or `init.el`:

```elisp
;; Patingin integration
(defun patingin-review ()
  "Run patingin review on current project"
  (interactive)
  (compile "patingin review"))

(defun patingin-auto-fix ()
  "Run patingin auto-fix on current project"
  (interactive)
  (compile "patingin review --fix"))

(global-set-key (kbd "C-c p r") 'patingin-review)
(global-set-key (kbd "C-c p f") 'patingin-auto-fix)
```

---

## Git Integration

### Pre-commit Hooks

#### Option 1: Direct Git Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/sh
echo "🔍 Running Patingin pre-commit check..."

# Check staged changes for critical violations
if ! patingin review --staged --severity critical --no-color; then
    echo ""
    echo "❌ Critical violations found in staged changes."
    echo "💡 Fix them with: patingin review --staged --fix"
    echo "💡 Or skip with: git commit --no-verify"
    exit 1
fi

echo "✅ Pre-commit check passed!"
exit 0
```

Make it executable:

```bash
chmod +x .git/hooks/pre-commit
```

#### Option 2: pre-commit Framework

Install `pre-commit` and create `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: patingin
        name: Patingin Code Review
        entry: patingin review --staged --severity critical --no-color
        language: system
        pass_filenames: false
        always_run: true
```

Install the hook:

```bash
pip install pre-commit
pre-commit install
```

### Commit Message Templates

Create `.gitmessage` template:

```
# <type>: <description>

# More detailed explanatory text, if necessary

# Patingin check:
# ✅ Ran `patingin review --staged`
# ✅ No critical violations found
# ✅ Applied auto-fixes where appropriate

# Types: feat, fix, docs, style, refactor, test, chore
```

Configure git to use it:

```bash
git config commit.template .gitmessage
```

---

## CI/CD Integration

### GitHub Actions

Create `.github/workflows/patingin.yml`:

```yaml
name: Code Quality with Patingin

on:
  pull_request:
    branches: [ main, develop ]
  push:
    branches: [ main ]

jobs:
  patingin:
    runs-on: ubuntu-latest
    
    steps:
    - name: Checkout code
      uses: actions/checkout@v3
      with:
        fetch-depth: 0  # Need full history for --since
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Cache cargo dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Install Patingin
      run: cargo install patingin
    
    - name: Setup Claude Code (optional)
      if: ${{ secrets.CLAUDE_API_KEY }}
      env:
        CLAUDE_API_KEY: ${{ secrets.CLAUDE_API_KEY }}
      run: |
        npm install -g @anthropic-ai/claude-code
        # Note: Authentication setup depends on your deployment requirements
    
    - name: Run Patingin Review
      run: |
        if [ "${{ github.event_name }}" = "pull_request" ]; then
          # For PRs: check changes since base branch
          patingin review --since origin/${{ github.base_ref }} --json > patingin-results.json
        else
          # For pushes: check last commit
          patingin review --json > patingin-results.json
        fi
    
    - name: Check Results
      run: |
        critical_count=$(jq '.summary.critical_count' patingin-results.json)
        major_count=$(jq '.summary.major_count' patingin-results.json)
        
        echo "Found $critical_count critical and $major_count major violations"
        
        if [ "$critical_count" -gt 0 ]; then
          echo "❌ Critical violations found:"
          jq -r '.violations[] | select(.severity == "critical") | "\(.file_path):\(.line_number): \(.rule.name)"' patingin-results.json
          exit 1
        fi
        
        if [ "$major_count" -gt 5 ]; then
          echo "⚠️ Too many major violations ($major_count > 5)"
          exit 1
        fi
    
    - name: Upload Results
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: patingin-results
        path: patingin-results.json
    
    - name: Comment PR (optional)
      if: github.event_name == 'pull_request' && always()
      uses: actions/github-script@v6
      with:
        script: |
          const fs = require('fs');
          const results = JSON.parse(fs.readFileSync('patingin-results.json'));
          
          const { critical_count, major_count, warning_count } = results.summary;
          
          const comment = `## 🔍 Patingin Code Review
          
          **Summary:** ${critical_count} critical, ${major_count} major, ${warning_count} warning violations
          
          ${critical_count > 0 ? '❌ Critical issues must be fixed before merge' : '✅ No critical issues found'}
          
          <details><summary>View Details</summary>
          
          \`\`\`json
          ${JSON.stringify(results, null, 2)}
          \`\`\`
          
          </details>`;
          
          github.rest.issues.createComment({
            issue_number: context.issue.number,
            owner: context.repo.owner,
            repo: context.repo.repo,
            body: comment
          });
```

### GitLab CI

Create `.gitlab-ci.yml`:

```yaml
stages:
  - quality

variables:
  CARGO_HOME: "$CI_PROJECT_DIR/.cargo"
  CARGO_TARGET_DIR: "$CI_PROJECT_DIR/target"

cache:
  paths:
    - .cargo/
    - target/

patingin-review:
  stage: quality
  image: rust:latest
  
  before_script:
    - apt-get update && apt-get install -y git jq
    - cargo install patingin
  
  script:
    - git fetch origin $CI_DEFAULT_BRANCH
    - patingin review --since origin/$CI_DEFAULT_BRANCH --json > patingin-results.json
    - |
      critical_count=$(jq '.summary.critical_count' patingin-results.json)
      major_count=$(jq '.summary.major_count' patingin-results.json)
      
      echo "Found $critical_count critical, $major_count major violations"
      
      if [ "$critical_count" -gt 0 ]; then
        echo "Critical violations found:"
        jq -r '.violations[] | select(.severity == "critical")' patingin-results.json
        exit 1
      fi
  
  artifacts:
    reports:
      junit: patingin-results.json
    paths:
      - patingin-results.json
    expire_in: 1 week
  
  only:
    - merge_requests
```

---

## Environment Configuration

### Environment Variables

```bash
# Patingin configuration
export PATINGIN_CONFIG_PATH="/custom/path/config.yml"
export PATINGIN_RULES_PATH="/custom/path/rules.yml"
export PATINGIN_LOG_LEVEL="info"

# Claude Code integration
export CLAUDE_API_KEY="your_api_key"
export CLAUDE_CODE_PATH="/custom/path/claude-code"

# Performance tuning
export PATINGIN_MAX_FILE_SIZE="1048576"  # 1MB
export PATINGIN_PARALLEL_JOBS="4"

# Logging
export RUST_LOG="patingin=info"
export RUST_BACKTRACE="1"
```

### Shell Integration

Add to your shell profile (`.bashrc`, `.zshrc`, etc.):

```bash
# Patingin aliases
alias pr='patingin review'
alias prf='patingin review --fix'
alias prs='patingin review --suggest'
alias prst='patingin review --staged'

# Quick project check
function pcheck() {
    echo "🔍 Quick Patingin check..."
    patingin review --severity critical
}

# Auto-fix and commit
function pautofix() {
    echo "🤖 Auto-fixing issues..."
    patingin review --fix --no-confirm
    if [ $? -eq 0 ]; then
        echo "✅ Auto-fixes applied successfully"
        git add -u
        echo "Files staged for commit"
    fi
}
```

---

## Troubleshooting

### Common Installation Issues

#### Rust Compilation Errors
```bash
# Update Rust toolchain
rustup update stable

# Clear cargo cache
cargo clean
rm -rf ~/.cargo/registry/cache

# Retry installation
cargo install --path . --force
```

#### Permission Issues
```bash
# Linux/macOS: Install to user directory
cargo install --path . --root ~/.local
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc

# Or use sudo for system-wide install
sudo cargo install --path . --root /usr/local
```

#### Git Integration Issues
```bash
# Check git version
git --version  # Should be 2.20+

# Verify git repository
git status

# Check git configuration
git config --list
```

### Performance Issues

#### Large Repositories
```bash
# Use language filtering
patingin review --language elixir

# Use smaller scope
patingin review --staged
patingin review --since HEAD~1

# Check project size
patingin setup | grep "Project size"
```

#### Memory Usage
```bash
# Monitor memory usage
/usr/bin/time -v patingin review

# Reduce parallel processing
export PATINGIN_PARALLEL_JOBS=1

# Skip large files
export PATINGIN_MAX_FILE_SIZE=524288  # 512KB
```

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug patingin review

# Specific component logging
RUST_LOG=patingin::git=debug patingin review
RUST_LOG=patingin::rules=debug patingin review

# Trace all activity
RUST_LOG=trace patingin review 2> debug.log
```

### Getting Help

```bash
# Built-in help
patingin --help
patingin review --help
patingin rules --help

# Environment check
patingin setup

# Version information
patingin --version
```

---

## Uninstallation

### Remove Patingin

```bash
# If installed via cargo
cargo uninstall patingin

# If installed manually
rm -f /usr/local/bin/patingin
# or
rm -f ~/.local/bin/patingin
```

### Clean Configuration

```bash
# Remove configuration (optional)
rm -rf ~/.config/patingin

# Remove project configurations
find . -name ".patingin.yml" -delete
```

### Remove Git Hooks

```bash
# Remove pre-commit hook
rm -f .git/hooks/pre-commit

# Remove pre-commit framework integration
pre-commit uninstall
```

This comprehensive setup guide ensures Patingin integrates smoothly into any development environment and workflow.