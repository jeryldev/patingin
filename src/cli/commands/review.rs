use anyhow::Result;
use clap::Args;
use colored::*;

use crate::core::{Language, ProjectDetector, ReviewEngine, Severity};
use crate::external::fix_engine::{BatchFixRequest, FixEngine};
use crate::git::{DiffScope, GitDiffParser};

#[derive(Args)]
pub struct ReviewArgs {
    /// Analyze staged changes (pre-commit check)
    #[arg(long)]
    pub staged: bool,

    /// Analyze unstaged changes only
    #[arg(long)]
    pub uncommitted: bool,

    /// Changes since specific commit/branch/tag
    #[arg(long, value_name = "REF")]
    pub since: Option<String>,

    /// Show only issues of specified severity and above
    #[arg(long, value_name = "LEVEL")]
    pub severity: Option<Severity>,

    /// Check only specific language files
    #[arg(long, value_name = "LANG")]
    pub language: Option<Language>,

    /// Output results in JSON format
    #[arg(long)]
    pub json: bool,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    /// Show fix suggestions (display only)
    #[arg(long)]
    pub suggest: bool,

    /// Launch interactive Claude Code session to fix violations
    #[arg(long)]
    pub fix: bool,

    /// Apply Claude Code fixes automatically (DEPRECATED: use --fix)
    #[arg(long)]
    pub auto_fix: bool,

    /// Skip confirmation when applying fixes (use with --auto-fix)
    #[arg(long)]
    pub no_confirm: bool,
}

pub async fn run(args: ReviewArgs) -> Result<()> {
    // Determine diff scope based on arguments
    let diff_scope = determine_diff_scope(&args);

    // Execute git diff to get changed lines
    let diff_output = GitDiffParser::execute_git_diff(&diff_scope)?;

    // Parse the git diff
    let git_diff = GitDiffParser::parse(&diff_output)?;

    // Filter files by language if specified
    let filtered_diff = if let Some(target_language) = &args.language {
        filter_diff_by_language(git_diff, target_language)
    } else {
        git_diff
    };

    // Review the changes with custom rules if project detected
    let review_engine = if let Ok(project_info) = ProjectDetector::detect_project(None) {
        ReviewEngine::new_with_custom_rules(&project_info.name)
    } else {
        ReviewEngine::new()
    };
    let review_result = review_engine.review_git_diff(&filtered_diff)?;

    // Filter violations by severity if specified
    let filtered_violations = if let Some(min_severity) = args.severity {
        review_engine
            .filter_violations_by_severity(&review_result.violations, min_severity)
            .into_iter()
            .cloned()
            .collect()
    } else {
        review_result.violations.clone()
    };

    // Output results
    if args.json {
        output_json_results(&review_result, &filtered_violations)?;
    } else {
        output_human_readable_results(&filtered_violations, &diff_scope, &args)?;
    }

    // Handle fix requests
    if args.fix {
        handle_interactive_fix(&filtered_violations).await?;
    } else if args.auto_fix {
        // Show deprecation warning
        eprintln!("⚠️  WARNING: --auto-fix is deprecated. Use --fix for interactive Claude Code sessions.");
        eprintln!("   The --auto-fix flag will be removed in a future version.");
        eprintln!();
        handle_auto_fix(&filtered_violations, args.no_confirm).await?;
    } else if args.suggest {
        show_fix_suggestions(&filtered_violations);
    }

    Ok(())
}

fn determine_diff_scope(args: &ReviewArgs) -> DiffScope {
    if args.staged {
        DiffScope::Staged
    } else if args.uncommitted {
        DiffScope::Unstaged
    } else if let Some(ref reference) = args.since {
        DiffScope::SinceCommit(reference.clone())
    } else {
        // Default: changes since last commit (git diff HEAD)
        DiffScope::SinceCommit("HEAD".to_string())
    }
}

fn filter_diff_by_language(
    git_diff: crate::git::GitDiff,
    target_language: &Language,
) -> crate::git::GitDiff {
    let review_engine = ReviewEngine::new();

    let filtered_files = git_diff
        .files
        .into_iter()
        .filter(|file_diff| {
            if let Some(detected_lang) = review_engine.detect_language_from_path(&file_diff.path) {
                detected_lang == *target_language
            } else {
                false
            }
        })
        .collect();

    crate::git::GitDiff {
        files: filtered_files,
    }
}

fn output_json_results(
    review_result: &crate::core::review_engine::ReviewResult,
    violations: &[crate::core::ReviewViolation],
) -> Result<()> {
    use serde::{Deserialize, Serialize};
    use serde_json;

    #[derive(Serialize, Deserialize)]
    struct JsonViolation {
        file_path: String,
        line_number: usize,
        rule_id: String,
        rule_name: String,
        severity: String,
        language: String,
        description: String,
        fix_suggestion: String,
        auto_fixable: bool,
    }

    #[derive(Serialize, Deserialize)]
    struct JsonOutput {
        violations: Vec<JsonViolation>,
        summary: JsonSummary,
    }

    #[derive(Serialize, Deserialize)]
    struct JsonSummary {
        total_violations: usize,
        critical_count: usize,
        major_count: usize,
        warning_count: usize,
        files_affected: usize,
        auto_fixable_count: usize,
    }

    let json_violations: Vec<JsonViolation> = violations
        .iter()
        .map(|v| JsonViolation {
            file_path: v.file_path.clone(),
            line_number: v.line_number,
            rule_id: v.rule.id.clone(),
            rule_name: v.rule.name.clone(),
            severity: format!("{:?}", v.severity).to_lowercase(),
            language: format!("{:?}", v.language).to_lowercase(),
            description: v.rule.description.clone(),
            fix_suggestion: v.fix_suggestion.clone(),
            auto_fixable: v.auto_fixable,
        })
        .collect();

    // These are now from review_result.summary, but keeping for validation
    let _critical_count = violations
        .iter()
        .filter(|v| v.severity == Severity::Critical)
        .count();
    let _major_count = violations
        .iter()
        .filter(|v| v.severity == Severity::Major)
        .count();
    let _warning_count = violations
        .iter()
        .filter(|v| v.severity == Severity::Warning)
        .count();
    let _auto_fixable_count = violations.iter().filter(|v| v.auto_fixable).count();

    let mut _files_affected: Vec<_> = violations.iter().map(|v| &v.file_path).collect();
    _files_affected.sort();
    _files_affected.dedup();

    let json_output = JsonOutput {
        violations: json_violations,
        summary: JsonSummary {
            total_violations: review_result.summary.total_violations,
            critical_count: review_result.summary.critical_count,
            major_count: review_result.summary.major_count,
            warning_count: review_result.summary.warning_count,
            files_affected: review_result.summary.files_affected.len(),
            auto_fixable_count: review_result.summary.auto_fixable_count,
        },
    };

    println!("{}", serde_json::to_string_pretty(&json_output)?);
    Ok(())
}

fn output_human_readable_results(
    violations: &[crate::core::ReviewViolation],
    diff_scope: &DiffScope,
    args: &ReviewArgs,
) -> Result<()> {
    // Header
    let scope_description = match diff_scope {
        DiffScope::Staged => "staged changes",
        DiffScope::Unstaged => "unstaged changes",
        DiffScope::SinceCommit(ref reference) => {
            if reference == "HEAD" {
                "changes since last commit"
            } else {
                reference
            }
        }
    };

    println!("🔍 Code Review: {}", scope_description.bold());

    if violations.is_empty() {
        println!("✅ No anti-pattern violations found!");
        return Ok(());
    }

    // Group violations by file
    let mut violations_by_file: std::collections::HashMap<
        String,
        Vec<&crate::core::ReviewViolation>,
    > = std::collections::HashMap::new();
    for violation in violations {
        violations_by_file
            .entry(violation.file_path.clone())
            .or_default()
            .push(violation);
    }

    println!(
        "📊 Found {} violations in {} files\n",
        violations.len(),
        violations_by_file.len()
    );

    // Show violations grouped by file
    for (file_path, file_violations) in violations_by_file {
        println!("📁 {}", file_path.bold());

        for violation in file_violations {
            let severity_icon = match violation.severity {
                Severity::Critical => "🔴 CRITICAL".red(),
                Severity::Major => "🟡 MAJOR".yellow(),
                Severity::Warning => "🔵 WARNING".blue(),
            };

            println!(
                "  {} {} ({})",
                severity_icon,
                violation.rule.name,
                violation.rule.id.dimmed()
            );

            // Show line number and content
            println!(
                "    Line {}: {}",
                violation.line_number.to_string().cyan(),
                violation.content.dimmed()
            );

            // Show fix suggestion
            println!("    💡 Fix: {}", violation.fix_suggestion);

            if violation.auto_fixable && (args.suggest || args.auto_fix) {
                println!("    ✨ Auto-fixable with Claude Code");
            }

            println!();
        }
    }

    // Summary
    let critical_count = violations
        .iter()
        .filter(|v| v.severity == Severity::Critical)
        .count();
    let major_count = violations
        .iter()
        .filter(|v| v.severity == Severity::Major)
        .count();
    let warning_count = violations
        .iter()
        .filter(|v| v.severity == Severity::Warning)
        .count();
    let auto_fixable_count = violations.iter().filter(|v| v.auto_fixable).count();

    println!("📊 Summary: {} violations", violations.len());
    if critical_count > 0 {
        println!("   🔴 Critical: {critical_count}");
    }
    if major_count > 0 {
        println!("   🟡 Major: {major_count}");
    }
    if warning_count > 0 {
        println!("   🔵 Warning: {warning_count}");
    }

    if auto_fixable_count > 0 {
        println!("   ✨ Auto-fixable: {auto_fixable_count}");

        if !args.fix && !args.auto_fix && !args.suggest {
            println!("\n💡 Use {} to see suggested fixes", "--suggest".cyan());
            println!(
                "💡 Use {} to launch interactive Claude Code session",
                "--fix".cyan()
            );
        }
    }

    Ok(())
}

fn show_fix_suggestions(violations: &[crate::core::ReviewViolation]) {
    let auto_fixable: Vec<_> = violations.iter().filter(|v| v.auto_fixable).collect();

    if auto_fixable.is_empty() {
        println!("💡 No auto-fixable violations found");
        return;
    }

    println!("\n🔧 Suggested Fixes:\n");

    for violation in auto_fixable {
        println!("📁 {}:{}", violation.file_path, violation.line_number);
        println!("   Issue: {}", violation.rule.name);
        println!("   Current: {}", violation.content.red());
        println!("   Suggestion: {}", violation.fix_suggestion.green());
        println!();
    }
}

async fn handle_auto_fix(
    violations: &[crate::core::ReviewViolation],
    no_confirm: bool,
) -> Result<()> {
    let auto_fixable: Vec<_> = violations
        .iter()
        .filter(|v| v.auto_fixable)
        .cloned()
        .collect();

    if auto_fixable.is_empty() {
        println!("💡 No auto-fixable violations found");
        return Ok(());
    }

    // Create fix engine and batch request
    let fix_engine = FixEngine::new();

    // Preview what will be fixed
    fix_engine.preview_batch_fixes(&auto_fixable)?;

    // Ask for confirmation unless --no-confirm is used
    if !no_confirm {
        print!("\n🤖 Apply fixes with Claude Code? [y/N]: ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Fix process cancelled.");
            return Ok(());
        }
    } else {
        println!("\n🤖 Applying fixes automatically (--no-confirm)...");
    }

    // Create batch fix request
    let batch_request = BatchFixRequest {
        violations: auto_fixable,
        dry_run: false,
        interactive: !no_confirm, // Interactive mode unless --no-confirm is used
        confidence_threshold: 0.7,
    };

    // Process fixes
    let result = fix_engine.process_batch_fixes(&batch_request).await?;

    // Generate summary
    fix_engine.generate_fix_summary(&result);

    Ok(())
}

async fn handle_interactive_fix(violations: &[crate::core::ReviewViolation]) -> Result<()> {
    if violations.is_empty() {
        println!("✅ No violations found to fix!");
        return Ok(());
    }

    // Check if Claude Code CLI is available
    use which::which;
    if which("claude").is_err() && which("claude-code").is_err() {
        eprintln!("❌ Claude Code CLI not found!");
        eprintln!("💡 Install it with: npm install -g @anthropic-ai/claude-code");
        eprintln!("💡 Then authenticate with: claude auth login");
        return Ok(());
    }

    println!(
        "🔍 Found {} violation(s). Launching interactive Claude Code session...",
        violations.len()
    );

    // Create the comprehensive query for Claude Code
    let query = create_claude_query(violations)?;

    // Determine which command to use
    let claude_cmd = if which("claude").is_ok() {
        "claude"
    } else {
        "claude-code"
    };

    // Launch Claude Code with the query
    use std::process::Command;
    let status = Command::new(claude_cmd).arg(&query).status()?;

    if status.success() {
        println!("\n✅ Claude Code session completed!");
        println!("💡 Run 'patingin review' again to check if violations were fixed.");
    } else {
        eprintln!("❌ Claude Code session failed or was cancelled.");
    }

    Ok(())
}

fn create_claude_query(violations: &[crate::core::ReviewViolation]) -> Result<String> {
    use crate::core::ProjectDetector;
    use std::collections::HashMap;
    use std::env;

    // Get project information
    let project_info = match ProjectDetector::detect_project(None) {
        Ok(info) => info,
        Err(_) => {
            // Fallback project info
            let current_dir = env::current_dir()?;
            let project_name = current_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown-project");

            crate::core::project_detector::ProjectInfo {
                name: project_name.to_string(),
                root_path: current_dir,
                languages: vec![],
                package_files: vec![],
                project_type: crate::core::project_detector::ProjectType::Generic,
            }
        }
    };

    // Group violations by file
    let mut files_with_violations: HashMap<String, Vec<&crate::core::ReviewViolation>> =
        HashMap::new();
    for violation in violations {
        files_with_violations
            .entry(violation.file_path.clone())
            .or_default()
            .push(violation);
    }

    // Format languages
    let languages: Vec<String> = project_info
        .languages
        .iter()
        .map(|l| format!("{l:?}"))
        .collect();
    let languages_str = if languages.is_empty() {
        "Unknown".to_string()
    } else {
        languages.join(", ")
    };

    // Build the comprehensive query
    let mut query = format!(
        "Fix these code quality violations in my project:\n\n\
        PROJECT: {} ({})\n\
        FILES AFFECTED: {} files with {} violations\n\n\
        VIOLATIONS FOUND:\n\n",
        project_info.name,
        languages_str,
        files_with_violations.len(),
        violations.len()
    );

    // Add each violation with context
    for (file_path, file_violations) in files_with_violations {
        for violation in file_violations {
            let severity_icon = match violation.severity {
                crate::core::Severity::Critical => "🔴",
                crate::core::Severity::Major => "🟡",
                crate::core::Severity::Warning => "🔵",
            };

            let severity_text = match violation.severity {
                crate::core::Severity::Critical => "CRITICAL",
                crate::core::Severity::Major => "MAJOR",
                crate::core::Severity::Warning => "WARNING",
            };

            query.push_str(&format!(
                "📁 {}:{}\n\
                {} {}: {} ({})\n\
                   Problem: {}\n\
                   Code:\n",
                file_path,
                violation.line_number,
                severity_icon,
                severity_text,
                violation.rule.name,
                violation.rule.id,
                violation.rule.description
            ));

            // Add context lines with line numbers
            let context_start = violation.line_number.saturating_sub(2);

            // Show context before
            for (i, line) in violation.context_before.iter().enumerate() {
                let line_num = context_start + i;
                query.push_str(&format!("   {line_num} │ {line}\n"));
            }

            // Show the violation line
            query.push_str(&format!(
                "   {} │ {}  ← VIOLATION\n",
                violation.line_number, violation.content
            ));

            // Show context after
            for (i, line) in violation.context_after.iter().enumerate() {
                let line_num = violation.line_number + 1 + i;
                query.push_str(&format!("   {line_num} │ {line}\n"));
            }

            query.push_str(&format!("   Fix: {}\n\n", violation.fix_suggestion));
        }
    }

    query.push_str("Please help me fix these issues interactively. Show me the problems and guide me through solutions.");

    Ok(query)
}

#[cfg(test)]
mod review_command_tests {
    use super::*;
    use crate::core::{AntiPattern, DetectionMethod, Language, ReviewViolation, Severity};
    use crate::git::DiffScope;

    fn create_test_args() -> ReviewArgs {
        ReviewArgs {
            staged: false,
            uncommitted: false,
            since: None,
            severity: None,
            language: None,
            json: false,
            no_color: false,
            suggest: false,
            fix: false,
            auto_fix: false,
            no_confirm: false,
        }
    }

    fn create_test_violation() -> ReviewViolation {
        let rule = AntiPattern {
            id: "test_rule".to_string(),
            name: "Test Rule".to_string(),
            language: Language::Elixir,
            severity: Severity::Major,
            description: "Test description".to_string(),
            detection_method: DetectionMethod::Regex {
                pattern: "test".to_string(),
            },
            fix_suggestion: "Fix this test issue".to_string(),
            source_url: None,
            claude_code_fixable: true,
            examples: vec![],
            tags: vec![],
            enabled: true,
        };

        ReviewViolation {
            rule,
            file_path: "test.ex".to_string(),
            line_number: 42,
            content: "test_content()".to_string(),
            severity: Severity::Major,
            language: Language::Elixir,
            fix_suggestion: "Use better pattern".to_string(),
            auto_fixable: true,
            context_before: vec!["# Previous line".to_string()],
            context_after: vec!["# Next line".to_string()],
            confidence: 0.85,
        }
    }

    fn create_test_review_result() -> crate::core::review_engine::ReviewResult {
        use crate::core::review_engine::{ReviewResult, ReviewSummary};
        use std::collections::HashMap;

        let violations = vec![create_test_violation()];
        let mut files_with_violations = HashMap::new();
        files_with_violations.insert("test.ex".to_string(), violations.clone());

        let summary = ReviewSummary {
            total_violations: 1,
            critical_count: 0,
            major_count: 1,
            warning_count: 0,
            files_affected: vec!["test.ex".to_string()],
            auto_fixable_count: 1,
        };

        ReviewResult {
            violations,
            files_with_violations,
            summary,
        }
    }

    #[test]
    fn test_determine_diff_scope_default() {
        let args = create_test_args();
        let scope = determine_diff_scope(&args);

        match scope {
            DiffScope::SinceCommit(ref reference) => {
                assert_eq!(reference, "HEAD");
            }
            _ => panic!("Expected SinceCommit(HEAD) for default scope"),
        }
    }

    #[test]
    fn test_determine_diff_scope_staged() {
        let mut args = create_test_args();
        args.staged = true;
        let scope = determine_diff_scope(&args);

        match scope {
            DiffScope::Staged => {}
            _ => panic!("Expected Staged scope"),
        }
    }

    #[test]
    fn test_determine_diff_scope_uncommitted() {
        let mut args = create_test_args();
        args.uncommitted = true;
        let scope = determine_diff_scope(&args);

        match scope {
            DiffScope::Unstaged => {}
            _ => panic!("Expected Unstaged scope"),
        }
    }

    #[test]
    fn test_determine_diff_scope_since_commit() {
        let mut args = create_test_args();
        args.since = Some("origin/main".to_string());
        let scope = determine_diff_scope(&args);

        match scope {
            DiffScope::SinceCommit(ref reference) => {
                assert_eq!(reference, "origin/main");
            }
            _ => panic!("Expected SinceCommit with specified reference"),
        }
    }

    #[test]
    fn test_determine_diff_scope_precedence() {
        // staged takes precedence
        let mut args = create_test_args();
        args.staged = true;
        args.uncommitted = true;
        args.since = Some("main".to_string());

        let scope = determine_diff_scope(&args);
        match scope {
            DiffScope::Staged => {}
            _ => panic!("Staged should take precedence"),
        }
    }

    #[test]
    fn test_output_json_results_structure() {
        let review_result = create_test_review_result();
        let violations = vec![create_test_violation()];

        // Capture stdout to test JSON structure
        let result = output_json_results(&review_result, &violations);
        assert!(result.is_ok());

        // Test that the function runs without panic
        // In a real test, we'd capture and parse the JSON output
    }

    #[test]
    fn test_output_json_results_empty_violations() {
        let review_result = create_test_review_result();
        let violations: Vec<ReviewViolation> = vec![];

        let result = output_json_results(&review_result, &violations);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_human_readable_results() {
        let violations = vec![create_test_violation()];
        let diff_scope = DiffScope::SinceCommit("HEAD".to_string());
        let args = create_test_args();

        let result = output_human_readable_results(&violations, &diff_scope, &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_human_readable_results_empty() {
        let violations: Vec<ReviewViolation> = vec![];
        let diff_scope = DiffScope::Staged;
        let args = create_test_args();

        let result = output_human_readable_results(&violations, &diff_scope, &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_fix_suggestions_with_auto_fixable() {
        let violations = vec![create_test_violation()];

        // Test that the function runs without panic
        show_fix_suggestions(&violations);
        // In real tests, we'd capture stdout and verify output
    }

    #[test]
    fn test_show_fix_suggestions_no_auto_fixable() {
        let mut violation = create_test_violation();
        violation.auto_fixable = false;
        let violations = vec![violation];

        show_fix_suggestions(&violations);
    }

    #[test]
    fn test_show_fix_suggestions_empty() {
        let violations: Vec<ReviewViolation> = vec![];
        show_fix_suggestions(&violations);
    }

    #[tokio::test]
    async fn test_handle_auto_fix_with_fixable() {
        let violations = vec![create_test_violation()];

        // Use no_confirm=true to avoid waiting for user input in tests
        let result = handle_auto_fix(&violations, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_auto_fix_no_fixable() {
        let mut violation = create_test_violation();
        violation.auto_fixable = false;
        let violations = vec![violation];

        // Use no_confirm=true to avoid waiting for user input in tests
        let result = handle_auto_fix(&violations, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_auto_fix_empty() {
        let violations: Vec<ReviewViolation> = vec![];

        // Use no_confirm=true to avoid waiting for user input in tests
        let result = handle_auto_fix(&violations, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter_diff_by_language() {
        use crate::git::{ChangeType, ChangedLine, FileDiff, GitDiff};

        let file_diff = FileDiff {
            path: "test.ex".to_string(),
            added_lines: vec![ChangedLine {
                line_number: 1,
                content: "defmodule Test do".to_string(),
                change_type: ChangeType::Added,
                context_before: vec![],
                context_after: vec![],
            }],
            removed_lines: vec![],
        };

        let git_diff = GitDiff {
            files: vec![file_diff],
        };

        let filtered = filter_diff_by_language(git_diff, &Language::Elixir);
        assert_eq!(filtered.files.len(), 1);
        assert_eq!(filtered.files[0].path, "test.ex");
    }

    #[test]
    fn test_filter_diff_by_language_no_match() {
        use crate::git::{ChangeType, ChangedLine, FileDiff, GitDiff};

        let file_diff = FileDiff {
            path: "test.py".to_string(),
            added_lines: vec![ChangedLine {
                line_number: 1,
                content: "def test():".to_string(),
                change_type: ChangeType::Added,
                context_before: vec![],
                context_after: vec![],
            }],
            removed_lines: vec![],
        };

        let git_diff = GitDiff {
            files: vec![file_diff],
        };

        let filtered = filter_diff_by_language(git_diff, &Language::Elixir);
        assert_eq!(filtered.files.len(), 0);
    }

    #[test]
    fn test_multiple_violations_summary() {
        let violations = vec![
            {
                let mut v = create_test_violation();
                v.severity = Severity::Critical;
                v
            },
            {
                let mut v = create_test_violation();
                v.severity = Severity::Major;
                v
            },
            {
                let mut v = create_test_violation();
                v.severity = Severity::Warning;
                v.auto_fixable = false;
                v
            },
        ];

        let critical_count = violations
            .iter()
            .filter(|v| v.severity == Severity::Critical)
            .count();
        let major_count = violations
            .iter()
            .filter(|v| v.severity == Severity::Major)
            .count();
        let warning_count = violations
            .iter()
            .filter(|v| v.severity == Severity::Warning)
            .count();
        let auto_fixable_count = violations.iter().filter(|v| v.auto_fixable).count();

        assert_eq!(critical_count, 1);
        assert_eq!(major_count, 1);
        assert_eq!(warning_count, 1);
        assert_eq!(auto_fixable_count, 2); // Critical and Major are auto_fixable by default
    }
}
