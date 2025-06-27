# Patingin Documentation

Welcome to the comprehensive documentation for Patingin - the all-seeing code guardian for git-aware code review and anti-pattern detection.

## Quick Navigation

### Getting Started

- **[Installation Guide](setup.md)** - Complete setup instructions for all platforms
- **[Main README](../README.md)** - Project overview and quick start

### Core Features

- **[Commands Reference](commands.md)** - Detailed guide to all CLI commands
- **[Workflows Guide](workflows.md)** - Common development workflows and examples
- **[Rules Management](rules.md)** - Managing and customizing anti-pattern rules
- **[AI Integration](ai-integration.md)** - Claude Code setup and interactive fixing

---

## Documentation Overview

### 📚 [Commands Reference](commands.md)

Comprehensive guide to all Patingin commands, options, and usage patterns.

**Covers:**

- `patingin review` - Git-aware code analysis
- `patingin rules` - Rule management and customization
- `patingin setup` - Environment diagnostics
- All command options, flags, and examples
- Exit codes and error handling

### 🔧 [Workflows Guide](workflows.md)

Common development workflows and integration patterns.

**Covers:**

- Daily development workflows
- Feature branch workflows
- Team collaboration patterns
- CI/CD integration examples
- Pre-commit hook setup
- Custom workflow patterns

### 🤖 [AI Integration Guide](ai-integration.md)

Comprehensive guide to AI-powered code fixing with Claude Code.

**Covers:**

- Claude Code CLI setup and authentication
- Interactive vs batch fixing modes
- Confidence scoring and validation
- Fix quality and safety measures
- Integration workflows and best practices
- Troubleshooting AI features

### 📋 [Rules Management Guide](rules.md)

Managing, customizing, and extending anti-pattern rules.

**Covers:**

- 47 built-in rules across 7 languages
- Creating custom project-specific rules
- Team rule management and sharing
- Rule configuration and syntax
- Advanced pattern matching
- Rule lifecycle and best practices

### ⚙️ [Installation and Setup Guide](setup.md)

Complete installation, configuration, and integration guide.

**Covers:**

- Multiple installation methods
- System requirements and dependencies
- IDE/editor integration
- Git integration and hooks
- CI/CD pipeline setup
- Environment configuration
- Troubleshooting common issues

---

## Quick Reference

### Essential Commands

```bash
# Quick code review
patingin review

# Apply interactive fixes
patingin review --fix

# Pre-commit check
patingin review --staged --severity critical

# View available rules
patingin rules

# Environment status
patingin setup
```

### Key Features

- **🎯 Git-aware**: Only analyzes your changes, not entire codebases
- **⚡ Fast**: <100ms startup, O(1) rule lookup, pre-compiled patterns
- **🤖 AI-powered**: Claude Code integration for interactive fixes
- **🔧 Customizable**: 47 built-in rules + unlimited custom rules
- **🌐 Multi-language**: Elixir, JavaScript, TypeScript, Python, Rust, Zig, SQL
- **👥 Team-friendly**: Shared configurations and collaborative workflows

### Supported Languages

| Language   | Rules | Interactive | Examples                            |
| ---------- | ----- | ----------- | ----------------------------------- |
| Elixir     | 13    | ✅          | Dynamic atoms, GenServer patterns   |
| JavaScript | 8     | ✅          | Console logs, eval usage, promises  |
| TypeScript | 3     | ✅          | Type safety, async patterns         |
| Python     | 8     | ✅          | Import patterns, exception handling |
| Rust       | 6     | ✅          | Memory safety, error handling       |
| Zig        | 3     | ✅          | Memory management, safety           |
| SQL        | 7     | ✅          | Injection prevention, optimization  |

---

## Common Use Cases

### For Individual Developers

1. **Daily Code Review**: `patingin review` after each change
2. **Pre-commit Validation**: `patingin review --staged` before commits
3. **Interactive Fixing**: `patingin review --fix` for collaborative improvements
4. **Learning Best Practices**: `patingin rules --detail <rule_id>` for education

### For Teams

1. **Shared Standards**: Team rules in `.patingin.yml`
2. **PR Quality Gates**: CI integration with violation limits
3. **Onboarding**: Consistent code quality education
4. **Legacy Modernization**: Gradual improvement of existing codebases

### For Organizations

1. **Security Compliance**: Critical violation detection and prevention
2. **Performance Monitoring**: Performance anti-pattern detection
3. **Code Quality Metrics**: JSON output for dashboards and reporting
4. **Interactive Remediation**: Large-scale code improvements

---

## Integration Examples

### GitHub Actions

```yaml
- name: Code Quality Check
  run: patingin review --since origin/main --json > violations.json
```

### Pre-commit Hook

```bash
#!/bin/sh
patingin review --staged --severity critical --no-color
```

### VS Code Task

```json
{
  "label": "Patingin Review",
  "command": "patingin",
  "args": ["review", "--fix"]
}
```

---

## Getting Help

### Built-in Help

```bash
patingin --help              # Main help
patingin review --help       # Command-specific help
patingin setup              # Environment diagnostics
```

### Documentation

- **GitHub Repository**: [https://github.com/jeryldev/patingin](https://github.com/jeryldev/patingin)
- **Issue Tracker**: [Report bugs and feature requests](https://github.com/jeryldev/patingin/issues)
- **Discussions**: [Community discussions and Q&A](https://github.com/jeryldev/patingin/discussions)

### Community

- **Contributing**: See [CONTRIBUTING.md](../CONTRIBUTING.md) for development guidelines
- **Code of Conduct**: See [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md)

---

## What's Next?

1. **Start with [Setup Guide](setup.md)** - Get Patingin installed and configured
2. **Try the [Quick Start](../README.md#quick-start)** - Run your first review
3. **Explore [Workflows](workflows.md)** - Find patterns that fit your development style
4. **Set up [AI Integration](ai-integration.md)** - Enable interactive code fixing
5. **Customize [Rules](rules.md)** - Tailor Patingin to your team's standards

**Ready to improve your code quality?** Start with the installation guide and experience focused, intelligent code review.

```bash
patingin review --fix
```

