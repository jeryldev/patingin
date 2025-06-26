# Commands Reference

Comprehensive guide to all Patingin commands, options, and usage patterns.

## Overview

Patingin provides three main commands:
- **`patingin review`** - Analyze git changes for anti-patterns
- **`patingin rules`** - Manage and customize rules
- **`patingin setup`** - Environment diagnostics

---

## `patingin review`

Analyze git diff changes for anti-pattern violations with intelligent scope detection.

### Syntax
```bash
patingin review [OPTIONS]
```

### Git Scope Options

#### Default Behavior
```bash
patingin review
# Analyzes: git diff HEAD (changes since last commit)
```

#### Staged Changes
```bash
patingin review --staged
# Analyzes: git diff --cached (staged changes only)
# Use case: Pre-commit validation
```

#### Unstaged Changes
```bash
patingin review --uncommitted  
# Analyzes: git diff (unstaged changes only)
# Use case: Work-in-progress review
```

#### Since Specific Reference
```bash
patingin review --since origin/main
patingin review --since HEAD~3
patingin review --since v1.2.0
patingin review --since feature/parent-branch
# Analyzes: git diff <reference>
# Use case: PR preparation, feature review
```

### Filtering Options

#### Severity Filtering
```bash
patingin review --severity critical    # Only critical violations
patingin review --severity major       # Critical + major violations
patingin review --severity warning     # All violations (default)
```

#### Language Filtering
```bash
patingin review --language elixir      # Only Elixir files
patingin review --language javascript  # Only JavaScript files
patingin review --language python      # Only Python files
```

#### Available Languages
- `elixir` - Elixir source files
- `javascript` - JavaScript source files  
- `typescript` - TypeScript source files
- `python` - Python source files
- `rust` - Rust source files
- `zig` - Zig source files
- `sql` - SQL source files

### Output Options

#### JSON Output
```bash
patingin review --json
# Outputs structured JSON for CI/CD integration
```

#### Disable Colors
```bash
patingin review --no-color
# Plain text output (useful for logs)
```

### Fix Options

#### Show Fix Suggestions
```bash
patingin review --suggest
# Display-only mode: shows what could be fixed
```

#### Apply Automatic Fixes
```bash
patingin review --fix
# Interactive mode: asks for confirmation before each fix
```

#### Batch Apply Fixes
```bash
patingin review --fix --no-confirm
# Non-interactive mode: applies all fixes automatically
```

### Example Combinations

#### Pre-commit Hook
```bash
patingin review --staged --severity critical --no-color
```

#### PR Review
```bash
patingin review --since origin/main --json > violations.json
```

#### Focus on Security
```bash
patingin review --severity critical --fix
```

#### Language-specific Review
```bash
patingin review --language elixir --suggest
```

### Exit Codes

- `0` - Success (no violations or non-critical violations)
- `1` - Critical violations found
- `2` - Command error (invalid arguments, git errors, etc.)

---

## `patingin rules`

Browse, search, and manage anti-pattern rules for your projects.

### Syntax
```bash
patingin rules [OPTIONS] [COMMAND]
```

### Listing Rules

#### List All Applicable Rules
```bash
patingin rules
# Shows all rules applicable to current project
# Automatically detects project languages
```

#### Language-specific Rules
```bash
patingin rules --elixir         # Only Elixir rules
patingin rules --javascript     # Only JavaScript rules
patingin rules --python         # Only Python rules
patingin rules --rust           # Only Rust rules
patingin rules --typescript     # Only TypeScript rules
patingin rules --zig            # Only Zig rules
patingin rules --sql            # Only SQL rules
```

#### Rule Scope Filtering
```bash
patingin rules --global         # Only built-in global rules
patingin rules --project        # Only current project's custom rules
patingin rules --all-projects   # Custom rules from all projects
```

### Searching Rules

#### Search by Keyword
```bash
patingin rules --search "atom"         # Find rules containing "atom"
patingin rules --search "security"     # Find security-related rules
patingin rules --search "performance"  # Find performance rules
```

#### Get Rule Details
```bash
patingin rules --detail dynamic_atom_creation
# Shows complete rule information:
# - Description and rationale
# - Code examples (good vs bad)
# - Fix suggestions
# - Source documentation links
```

### Adding Custom Rules

#### Add Project-specific Rule
```bash
patingin rules add --project --elixir "Use gettext for translations"
patingin rules add --project --javascript "Use team logger instead of console.log"
patingin rules add --project --python "Follow team docstring format"
```

**Note:** Language flag is required when adding custom rules.

#### Add Global Rule Sets
```bash
patingin rules add --elixir         # Add all global Elixir rules to project
patingin rules add --javascript     # Add all global JavaScript rules to project
```

### Managing Custom Rules

#### Remove Rule
```bash
patingin rules remove --project rule_id
# Removes specific custom rule from current project
```

#### Edit Rule
```bash
patingin rules edit --project rule_id
# Opens rule configuration for editing
```

### Example Output

```
📋 Rules for Your Project

📁 Project Languages: Elixir (95%), JavaScript (5%)

🔸 Elixir Rules (13 rules)
  🔴 Critical: dynamic_atom_creation, sql_injection_ecto  
  🟡 Major: long_parameter_list, namespace_trespassing
  🔵 Warning: comments_overuse, non_assertive_map_access

⚙️ Project Rules (2 custom rules)
  🟡 Major: team_genserver_convention
  
Total: 15 rules active for this project
```

---

## `patingin setup`

Comprehensive status check of development environment and patingin configuration.

### Syntax
```bash
patingin setup
```

### What It Checks

#### Project Information
- Project name and path detection
- Language analysis and percentages
- Git repository status

#### Tool Integration
- Claude Code CLI availability and version
- Authentication status
- Auto-fix capability

#### Configuration
- Rules configuration files
- Custom rules count
- Project-specific settings

#### Performance Metrics
- Project size estimation
- Expected scan time
- Memory usage estimation

### Example Output

```
🔍 Patingin Environment Status

📁 Project Information:
  Name: my-elixir-app (detected from mix.exs)
  Path: /Users/dev/code/my-elixir-app
  Languages: Elixir (95%), JavaScript (5%)

🌿 Git Status:
  Repository: ✅ Valid git repository
  Current branch: feature/user-auth
  Base branch: main (detected)
  Ahead by: 5 commits, Behind by: 0 commits
  Working directory: 3 files modified, 1 file staged

🤖 Claude Code Integration:
  CLI Available: ✅ v1.2.3
  Authentication: ✅ Authenticated as user@example.com
  Auto-fix capability: ✅ Ready

📋 Rules Configuration:
  Global rules: 13 Elixir, 8 JavaScript  
  Project rules: 2 custom Elixir rules
  Rules file: ~/.config/patingin/rules.yml ✅

⚡ Performance:
  Project size: 145 files, ~12K lines
  Expected scan time: <2 seconds

All systems ready! 🚀
```

---

## Global Options

Options available for all commands:

### Help
```bash
patingin --help              # Show main help
patingin review --help       # Show review command help
patingin rules --help        # Show rules command help
patingin setup --help        # Show setup command help
```

### Version
```bash
patingin --version           # Show version information
```

---

## Environment Variables

### Configuration
```bash
PATINGIN_CONFIG_PATH=/custom/path/config.yml    # Custom config file location
PATINGIN_RULES_PATH=/custom/path/rules.yml      # Custom rules file location
```

### Logging
```bash
RUST_LOG=debug              # Enable debug logging
RUST_LOG=patingin=info      # Patingin-specific logging
```

### Claude Code Integration
```bash
CLAUDE_API_KEY=your_key     # Claude Code authentication
```

---

## Configuration Files

### Rules Configuration
Location: `~/.config/patingin/rules.yml`

```yaml
projects:
  my-elixir-app:
    path: "/Users/dev/code/my-elixir-app"
    git_root: true
    rules:
      elixir:
        - id: "use_gettext"
          description: "Use gettext for translations"
          pattern: "\"[^\"]*\"\\s*(?!\\|>\\s*gettext)"
          severity: "warning"
          fix: "Wrap with gettext()"
```

### Project Configuration  
Location: `.patingin.yml` (in project root)

```yaml
version: 1.0
base_branch: "master"

rules:
  elixir:
    comments_overuse:
      enabled: false
      
ignore_paths:
  - "test/**/*"
  - "deps/**/*"
  - "_build/**/*"
```

---

## Performance Tips

### Faster Scans
- Use `--language` flag to focus on specific languages
- Use `--severity critical` for quick security checks
- Use `--staged` for pre-commit hooks (smaller scope)

### CI/CD Integration
- Use `--json` output for structured processing
- Use `--no-color` for clean log output
- Set appropriate exit code handling

### Large Projects
- Use `--since` with specific commits to limit scope
- Focus on changed files with default `patingin review`
- Use language filtering for multi-language projects