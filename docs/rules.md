# Rules Management Guide

Comprehensive guide to managing, customizing, and extending Patingin's anti-pattern rules.

## Overview

Patingin uses a flexible rule system that combines:
- **51 built-in rules** across 7 languages
- **Unlimited custom rules** per project
- **Smart rule detection** based on project languages
- **Centralized configuration** for team consistency

---

## Built-in Rules

Patingin includes carefully curated rules for common anti-patterns.

### Elixir Rules (13 rules)

#### Critical Severity

**`dynamic_atom_creation`** - Dynamic Atom Creation
- **Pattern**: `String.to_atom(.*)`
- **Issue**: Creating atoms from uncontrolled input can exhaust atom table
- **Fix**: Use `String.to_existing_atom/1` or explicit atom mapping
- **Auto-fixable**: ✅ Yes

```elixir
# Bad
atom = String.to_atom(user_input)

# Good  
atom = String.to_existing_atom(user_input)
# Or use explicit mapping
case user_input do
  "admin" -> :admin
  "user" -> :user
  _ -> :unknown
end
```

**`mass_assignment_vulnerability`** - Mass Assignment in Changesets
- **Pattern**: `cast\(.*,\s*:all\)`
- **Issue**: Accepting all parameters can lead to security vulnerabilities
- **Fix**: Explicitly list allowed fields
- **Auto-fixable**: ❌ No (requires domain knowledge)

**`sql_injection_ecto`** - SQL Injection in Ecto
- **Pattern**: `query\(.*#\{.*\}`
- **Issue**: String interpolation in queries enables SQL injection
- **Fix**: Use parameterized queries or `Ecto.Query` macros
- **Auto-fixable**: ❌ No (context-dependent)

#### Major Severity

**`long_parameter_list`** - Long Parameter Lists
- **Pattern**: `def\s+\w+\([^)]*,[^)]*,[^)]*,[^)]*,[^)]*,`
- **Issue**: Functions with many parameters are hard to use and maintain
- **Fix**: Group related parameters into structs or maps
- **Auto-fixable**: ❌ No (requires design decisions)

**`structs_32_plus_fields`** - Large Structs
- **Pattern**: Complex pattern detecting 32+ fields
- **Issue**: Structs with 32+ fields cause VM performance issues
- **Fix**: Split into smaller, focused structs
- **Auto-fixable**: ❌ No (requires architectural changes)

**`namespace_trespassing`** - Namespace Violations
- **Pattern**: `defmodule\s+(?!YourApp\.)\w+`
- **Issue**: Defining modules outside your application namespace
- **Fix**: Use proper namespace prefix
- **Auto-fixable**: ✅ Yes

**`ecto_schemas_in_migrations`** - Schemas in Migrations
- **Pattern**: `YourApp\.[A-Z]\w*` in migration files
- **Issue**: Using application schemas in migrations creates coupling
- **Fix**: Use raw SQL or embedded schemas
- **Auto-fixable**: ❌ No (requires migration strategy)

#### Warning Severity

**`comments_overuse`** - Excessive Comments
- **Pattern**: Lines with >70% comments
- **Issue**: Over-commenting can indicate unclear code
- **Fix**: Improve code clarity, reduce redundant comments
- **Auto-fixable**: ❌ No (subjective)

**`complex_else_in_with`** - Complex with-else
- **Pattern**: `with.*else.*` with complex else blocks
- **Issue**: Complex else handling defeats with's purpose
- **Fix**: Simplify or use case statements
- **Auto-fixable**: ❌ No (requires logic restructuring)

### JavaScript Rules (8 rules)

#### Critical Severity

**`eval_usage`** - Eval Usage
- **Pattern**: `\\beval\\s*\\(`
- **Issue**: `eval()` can execute arbitrary code, major security risk
- **Fix**: Use `JSON.parse()` for data, avoid dynamic code execution
- **Auto-fixable**: ❌ No (security-critical, needs manual review)

**`unhandled_promise`** - Unhandled Promise Rejection
- **Pattern**: `\.then\\s*\\([^)]*\\)\\s*[^.c]`
- **Issue**: Promises without catch handlers can cause silent failures
- **Fix**: Add `.catch()` handler
- **Auto-fixable**: ✅ Yes

#### Major Severity

**`console_log_production`** - Console Statements
- **Pattern**: `console\\.(log|warn|error|info|debug)\\s*\\(`
- **Issue**: Console statements should not be in production code
- **Fix**: Remove or replace with proper logging framework
- **Auto-fixable**: ✅ Yes

**`var_declaration`** - Var Declaration
- **Pattern**: `\\bvar\\s+\\w+`
- **Issue**: `var` has function scope and hoisting issues
- **Fix**: Use `const` or `let` with block scope
- **Auto-fixable**: ✅ Yes

**`double_equals`** - Loose Equality
- **Pattern**: `[^=!]==[^=]|[^=!]!=[^=]`
- **Issue**: `==` and `!=` perform type coercion
- **Fix**: Use `===` and `!==` for strict equality
- **Auto-fixable**: ✅ Yes

### Python Rules (8 rules)

**`wildcard_imports`** - Wildcard Imports
- **Pattern**: `from\s+\w+\s+import\s+\*`
- **Issue**: Wildcard imports pollute namespace and hide dependencies
- **Fix**: Import specific names or use qualified imports
- **Auto-fixable**: ❌ No (requires knowing what's used)

**`bare_except`** - Bare Except Clauses
- **Pattern**: `except:\s*$`
- **Issue**: Catching all exceptions can hide bugs
- **Fix**: Catch specific exception types
- **Auto-fixable**: ❌ No (requires exception analysis)

### Additional Languages

- **TypeScript** (3 rules) - Type safety, async patterns
- **Rust** (6 rules) - Memory safety, error handling  
- **Zig** (3 rules) - Memory management, safety
- **SQL** (7 rules) - Injection prevention, optimization

---

## Custom Rules

### Rule Configuration File

Custom rules are stored in `~/.config/patingin/rules.yml`:

```yaml
projects:
  my-elixir-app:
    path: "/Users/dev/code/my-elixir-app"
    git_root: true
    rules:
      elixir:
        - id: "use_gettext"
          name: "Use Gettext for Translations"
          description: "Hard-coded strings should use gettext for i18n"
          severity: "warning"
          pattern: '"[^"]*"\\s*(?!\\|>\\s*gettext)'
          fix_suggestion: "Wrap string with gettext()"
          claude_code_fixable: true
          enabled: true
          
        - id: "team_genserver_pattern"
          name: "Team GenServer Convention"
          description: "Use async calls in GenServer for better performance"
          severity: "major"
          pattern: "GenServer\\.call.*:sync"
          fix_suggestion: "Replace with GenServer.cast for async operation"
          claude_code_fixable: false
          enabled: true
      
      javascript:
        - id: "team_logging"
          name: "Team Logging Framework"
          description: "Use team logger instead of console.log"
          severity: "major"
          pattern: "console\\.log\\("
          fix_suggestion: "Replace with logger.debug() or logger.info()"
          claude_code_fixable: true
          enabled: true
```

### Adding Custom Rules

#### Via Command Line

```bash
# Add Elixir rule
patingin rules add --project --elixir "Use gettext for translations"

# Add JavaScript rule
patingin rules add --project --javascript "Use team logger instead of console.log"

# Add Python rule
patingin rules add --project --python "Follow team docstring format"
```

#### Via Configuration File

Create or edit `~/.config/patingin/rules.yml`:

```bash
# Create config directory
mkdir -p ~/.config/patingin

# Edit rules file
vim ~/.config/patingin/rules.yml
```

### Rule Properties

**Required Fields:**
- `id` - Unique identifier for the rule
- `name` - Human-readable rule name
- `description` - Explanation of the anti-pattern
- `severity` - `critical`, `major`, or `warning`
- `pattern` - Regular expression to match violations
- `fix_suggestion` - How to fix the violation

**Optional Fields:**
- `claude_code_fixable` - Whether AI can auto-fix (default: `false`)
- `enabled` - Whether rule is active (default: `true`)
- `source_url` - Documentation link
- `examples` - Code examples
- `tags` - Categorization tags

### Rule Examples

#### Security Rule
```yaml
- id: "hardcoded_secrets"
  name: "Hardcoded Secrets"
  description: "Secrets should not be hardcoded in source"
  severity: "critical"
  pattern: '(password|secret|key)\\s*=\\s*"[^"]+"'
  fix_suggestion: "Move to environment variables or config files"
  claude_code_fixable: false
  tags: ["security", "secrets"]
```

#### Performance Rule
```yaml
- id: "inefficient_enum"
  name: "Inefficient Enum Usage"
  description: "Use Stream for large collections"
  severity: "warning"
  pattern: "Enum\\.(map|filter|reduce).*Enum\\.(map|filter|reduce)"
  fix_suggestion: "Use Stream for chained operations on large collections"
  claude_code_fixable: true
  tags: ["performance", "elixir"]
```

#### Team Convention Rule
```yaml
- id: "module_doc_required"
  name: "Module Documentation Required"
  description: "All public modules must have @moduledoc"
  severity: "major"
  pattern: "defmodule\\s+[A-Z]\\w*(?!.*@moduledoc)"
  fix_suggestion: "Add @moduledoc to describe module purpose"
  claude_code_fixable: false
  tags: ["documentation", "team"]
```

---

## Project-specific Configuration

### `.patingin.yml`

Place in your project root for team-shared configuration:

```yaml
version: 1.0
project_name: "my-elixir-app"
base_branch: "main"

# Rule overrides
rules:
  elixir:
    # Disable specific rule
    comments_overuse:
      enabled: false
    
    # Adjust severity
    long_parameter_list:
      severity: "warning"  # Downgrade from major
    
    # Custom team rule
    team_genserver_pattern:
      name: "Team GenServer Convention"
      severity: "major"
      description: "Use async calls in GenServer"
      pattern: "GenServer\\.call.*:sync"
      fix_suggestion: "Replace with GenServer.cast"

# Ignore patterns
ignore_paths:
  - "test/**/*"
  - "deps/**/*" 
  - "_build/**/*"
  - "assets/vendor/**/*"

# Language-specific settings
languages:
  elixir:
    severity_threshold: "major"
  javascript:
    severity_threshold: "warning"

# AI integration settings
ai_integration:
  enabled: true
  confidence_threshold: 0.8
```

### Team Workflow

```bash
# 1. Create team configuration
vim .patingin.yml

# 2. Commit to repository
git add .patingin.yml
git commit -m "Add team code quality rules"

# 3. Team members benefit automatically
git pull
patingin rules  # Shows team rules
```

---

## Rule Management Commands

### Listing Rules

```bash
# All applicable rules
patingin rules

# Language-specific
patingin rules --elixir
patingin rules --javascript

# By scope
patingin rules --global         # Built-in rules only
patingin rules --project        # Project custom rules only
patingin rules --all-projects   # All custom rules
```

### Searching Rules

```bash
# Search by keyword
patingin rules --search "atom"      # Find atom-related rules
patingin rules --search "security"  # Find security rules
patingin rules --search "performance" # Find performance rules

# Get detailed information
patingin rules --detail dynamic_atom_creation
```

### Managing Custom Rules

```bash
# Add new rule
patingin rules add --project --elixir "Description here"

# Remove rule
patingin rules remove --project rule_id

# Edit rule (opens in editor)
patingin rules edit --project rule_id

# Disable rule temporarily
patingin rules disable --project rule_id

# Re-enable rule
patingin rules enable --project rule_id
```

---

## Advanced Rule Patterns

### Complex Regex Patterns

#### Multi-line Patterns
```yaml
pattern: "def\\s+\\w+\\([^)]*\\)\\s*do[\\s\\S]*?end"
# Matches entire function definitions
```

#### Negative Lookahead
```yaml
pattern: "String\\.to_atom\\((?!:existing)"
# Matches String.to_atom but not String.to_atom(:existing)
```

#### Context-aware Patterns
```yaml
pattern: "(?<=defmodule\\s)[A-Z]\\w*(?=\\s+do)"
# Matches module names with proper context
```

### Language-specific Considerations

#### Elixir Patterns
- Use `\\b` for word boundaries
- Account for pipe operators `|>`
- Consider macro usage patterns

#### JavaScript Patterns  
- Handle both function and arrow syntax
- Consider async/await patterns
- Account for destructuring

#### Python Patterns
- Handle indentation sensitivity
- Consider both function and method definitions
- Account for decorators

### Testing Custom Rules

```bash
# Test rule on specific files
patingin review --language elixir lib/specific_file.ex

# Test with debug output
RUST_LOG=debug patingin review --language elixir

# Validate pattern syntax
echo "test code" | grep -P "your_pattern_here"
```

---

## Rule Categories

### Security Rules
- SQL injection prevention
- XSS vulnerability detection
- Hardcoded secrets
- Input validation

### Performance Rules
- Inefficient algorithms
- Memory leaks
- Unnecessary allocations
- Database N+1 queries

### Maintainability Rules
- Code complexity
- Documentation requirements
- Naming conventions
- Architecture violations

### Team Convention Rules
- Coding standards
- Library usage patterns
- Error handling conventions
- Testing requirements

---

## Best Practices

### Rule Creation
1. **Start Simple** - Begin with basic patterns, refine over time
2. **Test Thoroughly** - Validate patterns on real codebases
3. **Document Well** - Include clear descriptions and examples
4. **Consider Context** - Some violations may be intentional

### Team Adoption
1. **Gradual Introduction** - Add rules incrementally
2. **Team Consensus** - Discuss rules with team before adding
3. **Regular Review** - Periodically review and update rules
4. **Exception Handling** - Provide ways to disable rules when needed

### Pattern Quality
- **Precise** - Avoid false positives
- **Comprehensive** - Cover edge cases
- **Performant** - Efficient regex patterns
- **Maintainable** - Clear and documented

### Rule Lifecycle

```bash
# 1. Create and test rule
patingin rules add --project --elixir "New pattern"

# 2. Validate on codebase
patingin review --language elixir

# 3. Refine pattern if needed
patingin rules edit --project new_pattern_id

# 4. Share with team
git add .patingin.yml
git commit -m "Add new team rule"

# 5. Monitor and adjust
patingin review --json | jq '.violations[] | select(.rule.id == "new_pattern_id")'
```

---

## Troubleshooting

### Common Issues

#### Rule Not Triggering
```bash
# Check rule is enabled
patingin rules --project | grep rule_id

# Test pattern manually
echo "test code" | grep -P "pattern"

# Enable debug logging
RUST_LOG=debug patingin review
```

#### Too Many False Positives
```bash
# Refine pattern
patingin rules edit --project rule_id

# Add negative lookahead
pattern: "bad_pattern(?!good_context)"

# Temporarily disable
patingin rules disable --project rule_id
```

#### Performance Issues
```bash
# Check pattern complexity
# Avoid excessive backtracking
# Use atomic groups where possible
# Test on large files
```

### Debug Commands

```bash
# Validate configuration
patingin setup

# Test specific rule
RUST_LOG=patingin::rules=debug patingin review

# Check pattern compilation
RUST_LOG=patingin::pattern=debug patingin review
```

This comprehensive rule system makes Patingin adaptable to any team's coding standards while providing robust built-in protection against common anti-patterns.