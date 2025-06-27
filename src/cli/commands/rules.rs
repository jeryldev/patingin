use crate::core::registry::PatternRegistry;
use crate::core::{CustomRule, CustomRulesManager, Language, ProjectDetector, Severity};
use anyhow::Result;
use clap::Args;
use std::collections::HashMap;
use std::env;

#[derive(Args)]
pub struct RulesArgs {
    /// Show only Elixir rules
    #[arg(long)]
    pub elixir: bool,

    /// Show only JavaScript rules
    #[arg(long)]
    pub javascript: bool,

    /// Show only TypeScript rules
    #[arg(long)]
    pub typescript: bool,

    /// Show only Python rules
    #[arg(long)]
    pub python: bool,

    /// Show only Rust rules
    #[arg(long)]
    pub rust: bool,

    /// Show only Zig rules
    #[arg(long)]
    pub zig: bool,

    /// Show only SQL rules
    #[arg(long)]
    pub sql: bool,

    /// Show only global built-in rules
    #[arg(long)]
    pub global: bool,

    /// Show only current project's custom rules
    #[arg(long)]
    pub project: bool,

    /// Show custom rules for all projects
    #[arg(long)]
    pub all_projects: bool,

    /// Search rules by keyword
    #[arg(long, value_name = "QUERY")]
    pub search: Option<String>,

    /// Show detailed rule explanation with examples
    #[arg(long, value_name = "RULE_ID")]
    pub detail: Option<String>,

    /// Add rule to project (requires language flag)
    #[arg(long)]
    pub add: bool,

    /// Remove specific project rule
    #[arg(long, value_name = "RULE_ID")]
    pub remove: Option<String>,

    /// Edit existing project rule
    #[arg(long, value_name = "RULE_ID")]
    pub edit: Option<String>,

    /// Rule description when adding
    #[arg(value_name = "DESCRIPTION")]
    pub description: Option<String>,
}

pub async fn run(args: RulesArgs) -> Result<()> {
    use crate::core::registry::PatternRegistry;

    let mut registry = PatternRegistry::new();
    registry.load_built_in_patterns()?;

    // Handle specific rule detail view first
    if let Some(rule_id) = &args.detail {
        return show_rule_detail(&registry, rule_id);
    }

    // Handle rule management operations first (before --project display)
    if args.add {
        return handle_add_rule(&args);
    }

    if let Some(rule_id) = &args.remove {
        return handle_remove_rule(rule_id);
    }

    if let Some(rule_id) = &args.edit {
        return handle_edit_rule(rule_id);
    }

    // Determine which languages to show rules for
    let target_languages = determine_target_languages(&args)?;

    // Load custom rules if --project flag is used for display
    if args.project {
        let project_info = ProjectDetector::detect_project(None)?;
        let project_name = project_info.name.clone();

        // For --project flag, only show custom rules
        let manager = CustomRulesManager::new();
        let custom_patterns = manager.get_project_rules(&project_name)?;

        if custom_patterns.is_empty() {
            println!("📋 No custom rules found for project '{}'", project_name);
            println!("💡 Add custom rules with: patingin rules --add --project --<language> \"rule description\"");
            return Ok(());
        }

        let mut custom_registry = PatternRegistry::new();
        for pattern in custom_patterns {
            custom_registry.add_pattern(pattern);
        }

        // Show only custom rules
        return show_custom_rules(&custom_registry, &project_name, &target_languages);
    }

    // Get rules based on filters
    let all_rules: Vec<_> = if let Some(query) = &args.search {
        registry.search_patterns(query)
    } else {
        // Get rules for target languages
        target_languages
            .iter()
            .flat_map(|lang| registry.get_patterns_for_language(lang))
            .collect()
    };

    // Show organized rule listing
    show_organized_rules(&all_rules, &target_languages, &args)
}

fn determine_target_languages(args: &RulesArgs) -> Result<Vec<Language>> {
    let mut languages = Vec::new();

    // Check individual language flags
    if args.elixir {
        languages.push(Language::Elixir);
    }
    if args.javascript {
        languages.push(Language::JavaScript);
    }
    if args.typescript {
        languages.push(Language::TypeScript);
    }
    if args.python {
        languages.push(Language::Python);
    }
    if args.rust {
        languages.push(Language::Rust);
    }
    if args.zig {
        languages.push(Language::Zig);
    }
    if args.sql {
        languages.push(Language::Sql);
    }

    // If specific languages requested, return them
    if !languages.is_empty() {
        return Ok(languages);
    }

    // If showing global, project, or all-projects, show all languages
    if args.global || args.project || args.all_projects || args.search.is_some() {
        return Ok(vec![
            Language::Elixir,
            Language::JavaScript,
            Language::TypeScript,
            Language::Python,
            Language::Rust,
            Language::Zig,
            Language::Sql,
        ]);
    }

    // Default: detect project languages using ProjectDetector
    let current_dir = env::current_dir()?;
    match ProjectDetector::detect_project(Some(&current_dir)) {
        Ok(project_info) => {
            if project_info.languages.is_empty() {
                // No languages detected, show all
                Ok(vec![
                    Language::Elixir,
                    Language::JavaScript,
                    Language::TypeScript,
                    Language::Python,
                    Language::Rust,
                    Language::Zig,
                    Language::Sql,
                ])
            } else {
                Ok(project_info.languages)
            }
        }
        Err(_) => {
            // Fallback to all languages if detection fails
            Ok(vec![
                Language::Elixir,
                Language::JavaScript,
                Language::TypeScript,
                Language::Python,
                Language::Rust,
                Language::Zig,
                Language::Sql,
            ])
        }
    }
}

fn show_rule_detail(
    registry: &crate::core::registry::PatternRegistry,
    rule_id: &str,
) -> Result<()> {
    use colored::*;

    if let Some(rule) = registry.get_pattern(rule_id) {
        println!("Rule: {}", rule.name.bold());
        println!("ID: {}", rule.id);
        println!("Language: {}", rule.language);
        println!(
            "Severity: {}",
            match rule.severity {
                crate::core::Severity::Critical => "CRITICAL".red(),
                crate::core::Severity::Major => "MAJOR".yellow(),
                crate::core::Severity::Warning => "WARNING".blue(),
            }
        );
        println!("Description: {}", rule.description);
        println!("Fix: {}", rule.fix_suggestion);
        if let Some(url) = &rule.source_url {
            println!("Source: {}", url);
        }
        println!(
            "Claude Code Fixable: {}",
            if rule.claude_code_fixable {
                "Yes".green()
            } else {
                "No".red()
            }
        );

        if !rule.examples.is_empty() {
            println!("\nExamples:");
            for example in &rule.examples {
                println!("  Bad:  {}", example.bad.red());
                println!("  Good: {}", example.good.green());
                println!("  Why:  {}", example.explanation);
            }
        }
    } else {
        println!("Rule '{}' not found", rule_id);
    }
    Ok(())
}

fn handle_add_rule(args: &RulesArgs) -> Result<()> {
    if !args.project {
        println!("❌ Error: --project flag is required when adding rules");
        println!("💡 Example: patingin rules add --project --elixir \"avoid IO.puts in production code\"");
        return Ok(());
    }

    // Determine language
    let language = get_language_from_args(args)?;

    // Get project information
    let project_info = ProjectDetector::detect_project(None)?;
    let project_name = project_info.name.clone();
    let project_path = project_info.root_path.to_string_lossy().to_string();

    // Get rule description from args
    let description = match &args.description {
        Some(desc) => desc.clone(),
        None => {
            println!("❌ Error: Rule description is required");
            println!("💡 Example: patingin rules add --project --elixir \"avoid IO.puts in production code\"");
            return Ok(());
        }
    };

    // Create interactive prompt for additional rule details
    println!("📋 Adding custom rule to project: {}", project_name);
    println!("🏷️  Language: {:?}", language);
    println!("📝 Description: {}", description);
    println!();

    // For now, create a simple regex pattern based on description
    // In the future, this could be an interactive prompt
    let rule_id = description
        .to_lowercase()
        .replace(" ", "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>();

    let pattern = description
        .split_whitespace()
        .last()
        .unwrap_or("TODO")
        .to_string();

    let custom_rule = CustomRule {
        id: rule_id.clone(),
        description: description.clone(),
        pattern,
        severity: "warning".to_string(), // Default to warning
        fix: "Review and fix according to team guidelines".to_string(),
        enabled: true,
    };

    // Add rule using CustomRulesManager
    let manager = CustomRulesManager::new();
    manager.add_project_rule(&project_name, &project_path, language, custom_rule)?;

    println!("✅ Successfully added custom rule: {}", rule_id);
    println!("📁 Saved to: ~/.config/patingin/rules.yml");
    println!("💡 You can edit the pattern and settings in the config file");

    Ok(())
}

fn get_language_from_args(args: &RulesArgs) -> Result<Language> {
    match (
        args.elixir,
        args.javascript,
        args.typescript,
        args.python,
        args.rust,
        args.zig,
        args.sql,
    ) {
        (true, false, false, false, false, false, false) => Ok(Language::Elixir),
        (false, true, false, false, false, false, false) => Ok(Language::JavaScript),
        (false, false, true, false, false, false, false) => Ok(Language::TypeScript),
        (false, false, false, true, false, false, false) => Ok(Language::Python),
        (false, false, false, false, true, false, false) => Ok(Language::Rust),
        (false, false, false, false, false, true, false) => Ok(Language::Zig),
        (false, false, false, false, false, false, true) => Ok(Language::Sql),
        _ => {
            anyhow::bail!("Please specify exactly one language flag (--elixir, --javascript, --typescript, --python, --rust, --zig, --sql)");
        }
    }
}

fn handle_remove_rule(rule_id: &str) -> Result<()> {
    // Get project information
    let project_info = ProjectDetector::detect_project(None)?;
    let project_name = project_info.name.clone();

    // Remove rule using CustomRulesManager
    let manager = CustomRulesManager::new();
    let removed = manager.remove_project_rule(&project_name, rule_id)?;

    if removed {
        println!("✅ Successfully removed custom rule: {}", rule_id);
        println!("📁 Updated: ~/.config/patingin/rules.yml");
    } else {
        println!(
            "❌ Rule '{}' not found in project '{}'",
            rule_id, project_name
        );
        println!("💡 Use 'patingin rules --project' to see available custom rules");
    }

    Ok(())
}

fn handle_edit_rule(rule_id: &str) -> Result<()> {
    println!("Edit rule '{}' functionality not yet implemented", rule_id);
    // TODO: Implement rule editing in ~/.config/patingin/rules.yml
    Ok(())
}

fn show_custom_rules(
    registry: &PatternRegistry,
    project_name: &str,
    target_languages: &[Language],
) -> Result<()> {
    println!("📋 Custom Rules for Project: {}", project_name);
    println!();

    let mut total_rules = 0;

    for language in target_languages {
        let patterns = registry.get_patterns_for_language(language);
        if patterns.is_empty() {
            continue;
        }

        let (critical_count, major_count, warning_count) = count_patterns_by_severity(&patterns);
        total_rules += patterns.len();

        let (emoji, name) = get_language_display_info(language);
        println!("{} {} ({} rules)", emoji, name, patterns.len());
        if critical_count > 0 {
            println!("  🔴 Critical: {}", critical_count);
        }
        if major_count > 0 {
            println!("  🟡 Major: {}", major_count);
        }
        if warning_count > 0 {
            println!("  🔵 Warning: {}", warning_count);
        }

        // Show all rules
        for pattern in patterns.iter() {
            let severity_icon = match pattern.severity {
                Severity::Critical => "🔴",
                Severity::Major => "🟡",
                Severity::Warning => "🔵",
            };
            let rule_name = pattern.name.clone();
            let rule_id = pattern.id.strip_prefix("custom_").unwrap_or(&pattern.id);
            println!("    {} {} ({})", severity_icon, rule_name, rule_id);
        }

        // Show all rules - no truncation
        println!();
    }

    println!("Total: {} custom rules", total_rules);
    println!();
    println!("💡 Use --detail <rule_id> to see detailed info about a specific rule");
    println!("💡 Use 'remove <rule_id>' to remove a custom rule");
    println!("💡 Edit ~/.config/patingin/rules.yml to modify rule patterns and settings");

    Ok(())
}

fn show_organized_rules(
    rules: &[&crate::core::AntiPattern],
    target_languages: &[Language],
    args: &RulesArgs,
) -> Result<()> {
    use colored::*;

    let current_dir = env::current_dir()?;
    let project_info = ProjectDetector::detect_project(Some(&current_dir)).ok();

    // Show project context if we detected project info and not showing specific flags
    if !args.global
        && !args.project
        && !args.all_projects
        && args.search.is_none()
        && !args.elixir
        && !args.javascript
        && !args.typescript
        && !args.python
        && !args.rust
        && !args.zig
        && !args.sql
    {
        if let Some(ref info) = project_info {
            println!("📋 Rules for Your Project\n");

            // Show project information
            println!(
                "📁 Project: {}",
                ProjectDetector::describe_project(info).bold()
            );
            println!("📂 Path: {}", info.root_path.display().to_string().dimmed());

            if !info.package_files.is_empty() {
                println!(
                    "📦 Package files: {}",
                    info.package_files.join(", ").dimmed()
                );
            }

            println!();
        } else {
            println!("📋 Available Anti-pattern Rules\n");
        }
    } else {
        println!("📋 Available Anti-pattern Rules\n");
    }

    // Group rules by language
    let mut rules_by_language: HashMap<Language, Vec<&crate::core::AntiPattern>> = HashMap::new();
    for rule in rules {
        rules_by_language
            .entry(rule.language.clone())
            .or_default()
            .push(rule);
    }

    // Show rules grouped by language
    for language in target_languages {
        if let Some(lang_rules) = rules_by_language.get(language) {
            if lang_rules.is_empty() {
                continue;
            }

            let (emoji, name) = get_language_display_info(language);
            let critical_count = lang_rules
                .iter()
                .filter(|p| p.severity == Severity::Critical)
                .count();
            let major_count = lang_rules
                .iter()
                .filter(|p| p.severity == Severity::Major)
                .count();
            let warning_count = lang_rules
                .iter()
                .filter(|p| p.severity == Severity::Warning)
                .count();

            println!("{} {} ({} rules)", emoji, name.bold(), lang_rules.len());

            if critical_count > 0 {
                println!("  {} Critical: {}", "🔴".red(), critical_count);
            }
            if major_count > 0 {
                println!("  {} Major: {}", "🟡".yellow(), major_count);
            }
            if warning_count > 0 {
                println!("  {} Warning: {}", "🔵".blue(), warning_count);
            }

            // Show all rules for this language
            for rule in lang_rules.iter() {
                let severity_str = match rule.severity {
                    Severity::Critical => "CRITICAL".red(),
                    Severity::Major => "MAJOR".yellow(),
                    Severity::Warning => "WARNING".blue(),
                };

                println!("    {} {} ({})", severity_str, rule.name, rule.id.dimmed());
            }

            // Show all rules - no truncation
            println!();
        }
    }

    let total_rules = rules.len();
    let total_languages = rules_by_language.len();

    println!(
        "Total: {} rules across {} languages",
        total_rules, total_languages
    );

    if !args.global && !args.project && project_info.is_some() {
        println!(
            "\n💡 Use {} to see rules for all languages",
            "--global".cyan()
        );
        println!(
            "💡 Use {} to see project-specific custom rules",
            "--project".cyan()
        );
    }

    println!(
        "💡 Use {} to see detailed info about a specific rule",
        "patingin rules --detail <rule_id>".cyan()
    );

    Ok(())
}

fn count_patterns_by_severity(patterns: &[&crate::core::AntiPattern]) -> (usize, usize, usize) {
    let critical_count = patterns
        .iter()
        .filter(|p| p.severity == Severity::Critical)
        .count();
    let major_count = patterns
        .iter()
        .filter(|p| p.severity == Severity::Major)
        .count();
    let warning_count = patterns
        .iter()
        .filter(|p| p.severity == Severity::Warning)
        .count();
    (critical_count, major_count, warning_count)
}

fn get_language_display_info(language: &Language) -> (&'static str, &'static str) {
    match language {
        Language::Elixir => ("⚗️", "Elixir"),
        Language::JavaScript => ("📜", "JavaScript"),
        Language::TypeScript => ("🔷", "TypeScript"),
        Language::Python => ("🐍", "Python"),
        Language::Rust => ("🦀", "Rust"),
        Language::Zig => ("⚡", "Zig"),
        Language::Sql => ("🗃️", "SQL"),
    }
}

#[cfg(test)]
mod rules_command_tests {
    use super::*;

    fn create_test_args() -> RulesArgs {
        RulesArgs {
            elixir: false,
            javascript: false,
            typescript: false,
            python: false,
            rust: false,
            zig: false,
            sql: false,
            global: false,
            project: false,
            all_projects: false,
            search: None,
            detail: None,
            add: false,
            remove: None,
            edit: None,
            description: None,
        }
    }

    #[tokio::test]
    async fn test_get_language_from_args_single_language() {
        let mut args = create_test_args();
        args.elixir = true;

        let result = get_language_from_args(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Language::Elixir);
    }

    #[tokio::test]
    async fn test_get_language_from_args_multiple_languages() {
        let mut args = create_test_args();
        args.elixir = true;
        args.javascript = true;

        let result = get_language_from_args(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("exactly one language"));
    }

    #[tokio::test]
    async fn test_get_language_from_args_no_language() {
        let args = create_test_args();

        let result = get_language_from_args(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("exactly one language"));
    }

    #[tokio::test]
    async fn test_all_language_flags() {
        for (flag_name, expected_language) in [
            ("elixir", Language::Elixir),
            ("javascript", Language::JavaScript),
            ("typescript", Language::TypeScript),
            ("python", Language::Python),
            ("rust", Language::Rust),
            ("zig", Language::Zig),
            ("sql", Language::Sql),
        ] {
            let mut args = create_test_args();
            match flag_name {
                "elixir" => args.elixir = true,
                "javascript" => args.javascript = true,
                "typescript" => args.typescript = true,
                "python" => args.python = true,
                "rust" => args.rust = true,
                "zig" => args.zig = true,
                "sql" => args.sql = true,
                _ => unreachable!(),
            }

            let result = get_language_from_args(&args);
            assert!(result.is_ok(), "Failed for language: {}", flag_name);
            assert_eq!(result.unwrap(), expected_language);
        }
    }

    #[tokio::test]
    async fn test_count_patterns_by_severity() {
        use crate::core::{AntiPattern, DetectionMethod, Severity};

        let patterns = vec![
            AntiPattern {
                id: "critical1".to_string(),
                name: "Critical Pattern".to_string(),
                language: Language::Elixir,
                severity: Severity::Critical,
                description: "Critical".to_string(),
                detection_method: DetectionMethod::Regex {
                    pattern: "test".to_string(),
                },
                fix_suggestion: "Fix".to_string(),
                source_url: None,
                claude_code_fixable: false,
                examples: vec![],
                tags: vec![],
                enabled: true,
            },
            AntiPattern {
                id: "major1".to_string(),
                name: "Major Pattern".to_string(),
                language: Language::Elixir,
                severity: Severity::Major,
                description: "Major".to_string(),
                detection_method: DetectionMethod::Regex {
                    pattern: "test".to_string(),
                },
                fix_suggestion: "Fix".to_string(),
                source_url: None,
                claude_code_fixable: false,
                examples: vec![],
                tags: vec![],
                enabled: true,
            },
            AntiPattern {
                id: "warning1".to_string(),
                name: "Warning Pattern".to_string(),
                language: Language::Elixir,
                severity: Severity::Warning,
                description: "Warning".to_string(),
                detection_method: DetectionMethod::Regex {
                    pattern: "test".to_string(),
                },
                fix_suggestion: "Fix".to_string(),
                source_url: None,
                claude_code_fixable: false,
                examples: vec![],
                tags: vec![],
                enabled: true,
            },
        ];

        let pattern_refs: Vec<&AntiPattern> = patterns.iter().collect();
        let (critical, major, warning) = count_patterns_by_severity(&pattern_refs);

        assert_eq!(critical, 1);
        assert_eq!(major, 1);
        assert_eq!(warning, 1);
    }

    #[tokio::test]
    async fn test_get_language_display_info() {
        let test_cases = [
            (Language::Elixir, ("⚗️", "Elixir")),
            (Language::JavaScript, ("📜", "JavaScript")),
            (Language::TypeScript, ("🔷", "TypeScript")),
            (Language::Python, ("🐍", "Python")),
            (Language::Rust, ("🦀", "Rust")),
            (Language::Zig, ("⚡", "Zig")),
            (Language::Sql, ("🗃️", "SQL")),
        ];

        for (language, expected) in test_cases {
            let result = get_language_display_info(&language);
            assert_eq!(result, expected);
        }
    }

    #[tokio::test]
    async fn test_rules_basic_functionality() {
        // Test that the basic rules command runs without errors
        let args = create_test_args();
        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rules_with_elixir_filter() {
        let mut args = create_test_args();
        args.elixir = true;

        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rules_add_without_project_flag() {
        let mut args = create_test_args();
        args.add = true;
        args.elixir = true;
        args.description = Some("test rule".to_string());

        let result = run(args).await;
        assert!(result.is_ok()); // Should succeed but show error message
    }

    #[tokio::test]
    async fn test_rules_search() {
        let mut args = create_test_args();
        args.search = Some("atom".to_string());

        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rules_project_empty() {
        let mut args = create_test_args();
        args.project = true;

        let result = run(args).await;
        assert!(result.is_ok()); // Should show "no custom rules" message
    }
}
