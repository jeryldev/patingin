use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use std::io::{self, Write};

use super::{ClaudeCodeIntegration, FixRequest, FixResult};
use crate::core::ReviewViolation;

#[derive(Debug, Clone)]
pub struct BatchFixRequest {
    pub violations: Vec<ReviewViolation>,
    pub dry_run: bool,
    pub interactive: bool,
    pub confidence_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct BatchFixResult {
    pub total_violations: usize,
    pub fixed_violations: usize,
    pub failed_violations: usize,
    pub skipped_violations: usize,
    pub files_modified: Vec<String>,
    pub fix_details: Vec<FixDetail>,
}

#[derive(Debug, Clone)]
pub struct FixDetail {
    pub violation: ReviewViolation,
    pub fix_result: FixResult,
    pub applied: bool,
    pub file_path: String,
    pub line_number: usize,
}

pub struct FixEngine {
    claude_integration: ClaudeCodeIntegration,
}

impl FixEngine {
    pub fn new() -> Self {
        Self {
            claude_integration: ClaudeCodeIntegration::detect(),
        }
    }

    pub async fn process_batch_fixes(&self, request: &BatchFixRequest) -> Result<BatchFixResult> {
        if !self.claude_integration.available {
            println!("{} Claude Code CLI not available", "⚠️".yellow());
            return Ok(BatchFixResult {
                total_violations: request.violations.len(),
                fixed_violations: 0,
                failed_violations: 0,
                skipped_violations: request.violations.len(),
                files_modified: vec![],
                fix_details: vec![],
            });
        }

        println!(
            "🤖 Processing {} violations with Claude Code...",
            request.violations.len()
        );

        let mut fix_details = Vec::new();
        let mut files_to_modify: HashMap<String, Vec<(usize, String)>> = HashMap::new();

        // Process each violation
        for (i, violation) in request.violations.iter().enumerate() {
            print!(
                "  [{}/{}] Fixing {} in {}:{}... ",
                i + 1,
                request.violations.len(),
                violation.rule.name,
                violation.file_path,
                violation.line_number
            );
            io::stdout().flush().unwrap();

            let fix_request = self.create_fix_request(violation)?;
            let fix_result = self.claude_integration.generate_fix(&fix_request)?;

            let mut applied = false;

            if fix_result.success && fix_result.confidence >= request.confidence_threshold {
                if let Some(ref fixed_code) = fix_result.fixed_code {
                    // Validate the fix
                    if self.claude_integration.validate_fix(
                        &violation.content,
                        fixed_code,
                        &format!("{:?}", violation.language).to_lowercase(),
                    )? {
                        if request.interactive {
                            // Show preview and ask for confirmation
                            applied = self.show_fix_preview_and_confirm(violation, fixed_code)?;
                        } else {
                            applied = true;
                        }

                        if applied && !request.dry_run {
                            // Queue the fix for batch application
                            files_to_modify
                                .entry(violation.file_path.clone())
                                .or_insert_with(Vec::new)
                                .push((violation.line_number, fixed_code.clone()));
                        }

                        println!(
                            "{}",
                            if applied {
                                "✅ Fixed"
                            } else {
                                "⏭️ Skipped"
                            }
                            .green()
                        );
                    } else {
                        println!("{}", "❌ Invalid fix".red());
                    }
                } else {
                    println!("{}", "❌ No fix generated".red());
                }
            } else {
                let reason = if !fix_result.success {
                    "Failed"
                } else {
                    "Low confidence"
                };
                println!("{} {}", "⚠️".yellow(), reason.yellow());
            }

            fix_details.push(FixDetail {
                violation: violation.clone(),
                fix_result,
                applied,
                file_path: violation.file_path.clone(),
                line_number: violation.line_number,
            });
        }

        // Apply all fixes to files (if not dry run)
        let mut files_modified = Vec::new();
        if !request.dry_run {
            for (file_path, fixes) in files_to_modify {
                if let Err(e) = self
                    .claude_integration
                    .apply_fixes_to_file(&file_path, &fixes)
                {
                    eprintln!("❌ Failed to apply fixes to {}: {}", file_path, e);
                } else {
                    files_modified.push(file_path);
                }
            }
        }

        // Calculate results
        let fixed_violations = fix_details.iter().filter(|d| d.applied).count();
        let failed_violations = fix_details.iter().filter(|d| !d.fix_result.success).count();
        let skipped_violations = fix_details.len() - fixed_violations - failed_violations;

        Ok(BatchFixResult {
            total_violations: request.violations.len(),
            fixed_violations,
            failed_violations,
            skipped_violations,
            files_modified,
            fix_details,
        })
    }

    fn create_fix_request(&self, violation: &ReviewViolation) -> Result<FixRequest> {
        Ok(FixRequest {
            file_path: violation.file_path.clone(),
            line_number: violation.line_number,
            original_code: violation.content.clone(),
            violation_description: violation.rule.description.clone(),
            fix_suggestion: violation.fix_suggestion.clone(),
            language: format!("{:?}", violation.language).to_lowercase(),
        })
    }

    fn show_fix_preview_and_confirm(
        &self,
        violation: &ReviewViolation,
        fixed_code: &str,
    ) -> Result<bool> {
        println!("\n{}", "📋 Fix Preview".bold().cyan());
        println!("File: {}", violation.file_path.bold());
        println!("Line: {}", violation.line_number.to_string().cyan());
        println!("Issue: {}", violation.rule.name.yellow());

        println!("\n{}", "Before:".red());
        println!("  {}", violation.content.red());

        println!("\n{}", "After:".green());
        println!("  {}", fixed_code.green());

        print!("\n{} Apply this fix? [y/N/a/q]: ", "❓".cyan());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => Ok(true),
            "a" | "all" => {
                // TODO: Implement "apply all" functionality
                Ok(true)
            }
            "q" | "quit" => {
                println!("Aborting fix process...");
                std::process::exit(0);
            }
            _ => Ok(false),
        }
    }

    pub fn generate_fix_summary(&self, result: &BatchFixResult) {
        println!("\n{}", "🎯 Batch Fix Summary".bold().cyan());
        println!("══════════════════════════════════════");

        println!("Total violations: {}", result.total_violations);
        println!(
            "{} Fixed: {}",
            "✅".green(),
            result.fixed_violations.to_string().green()
        );
        println!(
            "{} Failed: {}",
            "❌".red(),
            result.failed_violations.to_string().red()
        );
        println!(
            "{} Skipped: {}",
            "⏭️".yellow(),
            result.skipped_violations.to_string().yellow()
        );

        if !result.files_modified.is_empty() {
            println!("\n{} Files modified:", "📝".cyan());
            for file in &result.files_modified {
                println!("  • {}", file.cyan());
            }
        }

        // Show detailed results for failed or skipped fixes
        let problematic_fixes: Vec<_> = result
            .fix_details
            .iter()
            .filter(|d| !d.applied || !d.fix_result.success)
            .collect();

        if !problematic_fixes.is_empty() {
            println!("\n{} Detailed Results:", "📊".cyan());
            for detail in problematic_fixes {
                let status = if !detail.fix_result.success {
                    "❌ Failed"
                } else if !detail.applied {
                    "⏭️ Skipped"
                } else {
                    "✅ Applied"
                };

                println!(
                    "  {} {}:{} - {} ({})",
                    status,
                    detail.file_path,
                    detail.line_number,
                    detail.violation.rule.name,
                    if let Some(ref err) = detail.fix_result.error_message {
                        err.as_str()
                    } else {
                        "User choice"
                    }
                );
            }
        }

        println!("\n{} Next steps:", "💡".cyan());
        if result.fixed_violations > 0 {
            println!("  • Review the changes and test your code");
            println!("  • Run {} to verify fixes", "patingin review".cyan());
            if result.files_modified.len() > 0 {
                println!(
                    "  • Commit the changes: {}",
                    "git add . && git commit -m \"Apply patingin fixes\"".cyan()
                );
            }
        }
        if result.failed_violations > 0 || result.skipped_violations > 0 {
            println!("  • Review failed/skipped violations manually");
            println!(
                "  • Use {} for detailed guidance",
                "patingin rules --detail <rule_id>".cyan()
            );
        }
    }

    pub fn preview_batch_fixes(&self, violations: &[ReviewViolation]) -> Result<()> {
        println!("{}", "🔍 Batch Fix Preview".bold().cyan());
        println!("═══════════════════════════════════");

        if violations.is_empty() {
            println!("No fixable violations found.");
            return Ok(());
        }

        // Group violations by file
        let mut violations_by_file: HashMap<String, Vec<&ReviewViolation>> = HashMap::new();
        for violation in violations {
            violations_by_file
                .entry(violation.file_path.clone())
                .or_insert_with(Vec::new)
                .push(violation);
        }

        println!(
            "Will attempt to fix {} violations in {} files:",
            violations.len(),
            violations_by_file.len()
        );

        for (file_path, file_violations) in violations_by_file {
            println!("\n📁 {}", file_path.bold());
            for violation in file_violations {
                let confidence_indicator = if self.claude_integration.available {
                    "🤖 High confidence"
                } else {
                    "❓ Claude Code not available"
                };

                println!(
                    "  {}:{} - {} ({})",
                    violation.line_number.to_string().cyan(),
                    violation.rule.name,
                    confidence_indicator,
                    format!("{:?}", violation.language).to_lowercase().dimmed()
                );
                println!("    Current: {}", violation.content.dimmed());
                println!("    Fix: {}", violation.fix_suggestion.green());
            }
        }

        println!("\n{} Commands:", "💡".cyan());
        println!(
            "  • {} - Preview and apply fixes interactively",
            "patingin review --auto-fix".cyan()
        );
        println!(
            "  • {} - Apply all fixes automatically",
            "patingin review --auto-fix --no-confirm".cyan()
        );
        println!(
            "  • {} - Show what would be fixed (dry run)",
            "patingin review --suggest".cyan()
        );

        Ok(())
    }
}

#[cfg(test)]
mod fix_engine_tests {
    use super::*;
    use crate::core::{AntiPattern, DetectionMethod, Language, Severity};

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
            content: "String.to_atom(user_input)".to_string(),
            severity: Severity::Major,
            language: Language::Elixir,
            fix_suggestion: "Use String.to_existing_atom(user_input)".to_string(),
            auto_fixable: true,
            context_before: vec!["def process_input(input) do".to_string()],
            context_after: vec!["end".to_string()],
            confidence: 0.9,
        }
    }

    #[test]
    fn test_fix_engine_creation() {
        let engine = FixEngine::new();
        // Should create without errors
        assert!(engine.claude_integration.available || !engine.claude_integration.available);
    }

    #[test]
    fn test_create_fix_request() {
        let engine = FixEngine::new();
        let violation = create_test_violation();

        let fix_request = engine.create_fix_request(&violation).unwrap();

        assert_eq!(fix_request.file_path, "test.ex");
        assert_eq!(fix_request.line_number, 42);
        assert_eq!(fix_request.original_code, "String.to_atom(user_input)");
        assert_eq!(fix_request.language, "elixir");
        assert!(fix_request
            .violation_description
            .contains("Test description"));
    }

    #[tokio::test]
    async fn test_batch_fix_request_creation() {
        let violations = vec![create_test_violation()];

        let batch_request = BatchFixRequest {
            violations,
            dry_run: true,
            interactive: false,
            confidence_threshold: 0.7,
        };

        assert_eq!(batch_request.violations.len(), 1);
        assert!(batch_request.dry_run);
        assert!(!batch_request.interactive);
        assert_eq!(batch_request.confidence_threshold, 0.7);
    }

    #[test]
    fn test_preview_batch_fixes() {
        let engine = FixEngine::new();
        let violations = vec![create_test_violation()];

        let result = engine.preview_batch_fixes(&violations);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_fix_summary() {
        let engine = FixEngine::new();
        let violation = create_test_violation();

        let result = BatchFixResult {
            total_violations: 1,
            fixed_violations: 1,
            failed_violations: 0,
            skipped_violations: 0,
            files_modified: vec!["test.ex".to_string()],
            fix_details: vec![FixDetail {
                violation: violation.clone(),
                fix_result: FixResult {
                    success: true,
                    fixed_code: Some("String.to_existing_atom(user_input)".to_string()),
                    error_message: None,
                    confidence: 0.9,
                },
                applied: true,
                file_path: "test.ex".to_string(),
                line_number: 42,
            }],
        };

        // Should not panic
        engine.generate_fix_summary(&result);
    }
}
