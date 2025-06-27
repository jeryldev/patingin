# Patingin - All-Seeing Code Guardian

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tests](https://img.shields.io/badge/tests-130%20tests-brightgreen.svg)](#testing)

> **Patingin** (Tagalog: "can I look?") - A lightning-fast, git-aware code review assistant for anti-pattern detection and interactive fixes.

## 🎯 What is Patingin?

Patingin is a **git-aware code review tool** that analyzes only your changes (not entire codebases) to catch anti-patterns and suggest fixes. It integrates with **Claude Code** for intelligent interactive fixes, making code reviews faster and more consistent.

### Core Philosophy

- **Analyze only what changed** - Focus on git diff, not entire projects
- **Show exactly where problems are** - Line-by-line violations with context
- **Fix what can be fixed** - AI-powered interactive corrections
- **Works with any workflow** - No assumptions about branching strategy

## 🚀 Quick Start

### Installation

```bash
# Install from source (requires Rust)
git clone https://github.com/jeryldev/patingin.git
cd patingin
cargo install --path .

# Verify installation
patingin --version
```

### First Review

```bash
# Check your current changes
patingin review

# Get fix suggestions
patingin review --suggest

# Launch interactive Claude Code session (requires Claude Code)
patingin review --fix
```

## 🔍 Core Commands

### `patingin review` - Analyze Changes

Detect anti-patterns in your git changes with intelligent scope detection.

```bash
# Analyze changes since last commit (default)
patingin review

# Pre-commit check
patingin review --staged

# Check specific scope
patingin review --since origin/main
patingin review --uncommitted

# Filter and format
patingin review --severity critical --json
patingin review --language elixir
```

### `patingin rules` - Manage Rules

Browse and customize anti-pattern rules for your projects.

```bash
# List all applicable rules
patingin rules

# Language-specific rules
patingin rules --elixir
patingin rules --javascript

# Search and details
patingin rules --search "atom"
patingin rules --detail dynamic_atom_creation

# Add custom rules
patingin rules add --project --elixir "use gettext for translations"
```

### `patingin setup` - Environment Status

Comprehensive status check of your development environment.

```bash
patingin setup
```

## 🤖 AI Integration

Patingin integrates with **Claude Code** for intelligent interactive fixes:

```bash
# Preview fixes
patingin review --suggest

# Launch interactive Claude Code session
patingin review --fix
```

**Requirements:** Install [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code/setup) via `npm install -g @anthropic-ai/claude-code` for AI features.

## 📋 Supported Languages

- **Elixir** (13 rules) - Dynamic atoms, GenServer patterns, Ecto security
- **JavaScript** (8 rules) - Console logs, eval usage, promise handling
- **TypeScript** (4 rules) - Type safety, async patterns
- **Python** (9 rules) - Import patterns, exception handling
- **Rust** (6 rules) - Memory safety, error handling
- **Zig** (4 rules) - Memory management, safety patterns
- **SQL** (7 rules) - Injection prevention, query optimization

**Total: 51 built-in rules + unlimited custom rules**

## 🔧 Example Workflows

### Daily Development

```bash
# 1. Make changes
vim lib/user.ex

# 2. Check as you work
patingin review

# 3. Fix issues interactively
patingin review --fix

# 4. Commit clean code
git add . && git commit -m "Add user auth"
```

### Pre-commit Hook

```bash
#!/bin/sh
patingin review --staged --severity critical
if [ $? -ne 0 ]; then
  echo "❌ Critical violations found. Fix before committing."
  exit 1
fi
```

### PR Preparation

```bash
# Review all changes for PR
patingin review --since origin/main

# Get JSON for CI integration
patingin review --json > violations.json
```

## 📊 Sample Output

```
🔍 Code Review: changes since last commit
📊 Found 2 violations in 2 files

📁 lib/user.ex
  🔴 CRITICAL Dynamic Atom Creation (dynamic_atom_creation)
    Line 42: String.to_atom(user_input)
    💡 Fix: Use String.to_existing_atom() or explicit mapping
    ✨ Interactively fixable with Claude Code

📁 assets/js/app.js
  🟡 MAJOR Console.log in Production (console_log_production)
    Line 15: console.log("Debug info:", data)
    💡 Fix: Remove console statements or use proper logging
    ✨ Interactively fixable with Claude Code

📊 Summary: 2 violations (1 critical, 1 major)
💡 Use --fix to launch interactive Claude Code session
```

## 🏗️ Architecture

### High-Performance Design

- **Fast startup** - Embedded rules, minimal I/O
- **Efficient rule lookup** - HashMap-based registry
- **Pre-compiled regex** - Reduced compilation overhead
- **Smart caching** - Language detection and project info

### Git Integration

- **Default scope**: Changes since last commit (`git diff HEAD`)
- **Flexible overrides**: `--staged`, `--uncommitted`, `--since <ref>`
- **Line-level analysis**: Only check changed/added lines
- **Branch agnostic**: Works with any workflow

### Storage Strategy

- **Built-in rules**: Embedded in binary (51 rules)
- **Custom rules**: `~/.config/patingin/rules.yml`
- **Smart project detection**: Git root → package files → directory

## 📚 Documentation

- **[Commands Reference](docs/commands.md)** - Detailed command options and examples
- **[Workflows Guide](docs/workflows.md)** - Common development workflows
- **[AI Integration](docs/ai-integration.md)** - Claude Code setup and usage
- **[Rules Management](docs/rules.md)** - Custom rules and configuration
- **[Installation Guide](docs/setup.md)** - Detailed setup instructions

## 🧪 Testing

Patingin has comprehensive test coverage:

```bash
# Run all tests
cargo test

# Test specific modules
cargo test core::
cargo test cli::
cargo test integration

# Performance benchmarks
cargo test --release performance
```

**Test Coverage**: 130+ tests

- Unit tests for all core modules
- CLI command integration tests
- End-to-end workflow tests
- AI integration validation

## 🤝 Contributing

This project is currently in early development and not yet ready for external contributions. Feel free to open issues for suggestions and feedback.

## 🔒 Security

Patingin helps **detect** security anti-patterns but:

- Never modifies files without explicit user confirmation
- Only analyzes code locally (no data sent to external services)
- Claude Code integration is opt-in and requires separate installation

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- Built with ❤️ in Rust for performance and reliability
- Inspired by the need for **focused**, **fast** code review tools
- Integrates with [Claude Code](https://claude.ai/code) for AI-powered fixes
- Name "Patingin" reflects our philosophy: "May I look at your code?"

---

**Ready to improve your code quality?** Install Patingin today and experience focused, intelligent code review.

```bash
patingin review --fix
```

