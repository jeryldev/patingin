use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use regex::Regex;

use crate::core::{AntiPattern, Language, Severity, DetectionMethod};
use crate::core::registry::PatternRegistry;
use crate::git::{GitDiff, ChangedLine};

#[derive(Debug, Clone)]
pub struct ReviewViolation {
    pub rule: AntiPattern,
    pub file_path: String,
    pub line_number: usize,
    pub content: String,
    pub severity: Severity,
    pub language: Language,
    pub fix_suggestion: String,
    pub auto_fixable: bool,
    #[allow(dead_code)] // Used in tests and context display
    pub context_before: Vec<String>,
    #[allow(dead_code)] // Used in tests and context display
    pub context_after: Vec<String>,
    #[allow(dead_code)] // Used in AI integration and tests
    pub confidence: f64,
}

#[derive(Debug)]
pub struct ReviewResult {
    pub violations: Vec<ReviewViolation>,
    #[allow(dead_code)] // Used in tests and JSON output
    pub files_with_violations: HashMap<String, Vec<ReviewViolation>>,
    pub summary: ReviewSummary,
}

#[derive(Debug)]
pub struct ReviewSummary {
    pub total_violations: usize,
    pub critical_count: usize,
    pub major_count: usize,
    pub warning_count: usize,
    pub files_affected: Vec<String>,
    pub auto_fixable_count: usize,
}

pub struct ReviewEngine {
    registry: PatternRegistry,
}

impl ReviewEngine {
    pub fn new() -> Self {
        let mut registry = PatternRegistry::new();
        registry.load_built_in_patterns().expect("Failed to load built-in patterns");
        
        Self { registry }
    }

    pub fn new_with_custom_rules(project_name: &str) -> Self {
        let mut registry = PatternRegistry::new();
        registry.load_built_in_patterns().expect("Failed to load built-in patterns");
        
        // Load custom rules for the project
        if let Err(e) = registry.load_custom_rules(project_name) {
            eprintln!("Warning: Failed to load custom rules for {}: {}", project_name, e);
        }
        
        Self { registry }
    }

    pub fn review_changed_lines(&self, file_path: &str, changed_lines: &[ChangedLine]) -> Result<Vec<ReviewViolation>> {
        let mut violations = Vec::new();
        
        // Get patterns for this specific file (more efficient than language detection)
        let patterns = self.registry.get_patterns_for_file(file_path);
        
        if patterns.is_empty() {
            return Ok(violations); // Skip if no patterns match this file type
        }
        
        // Still detect language for violation metadata
        let language = self.detect_language_from_path(file_path).unwrap_or(Language::JavaScript);
        
        // Check each changed line against patterns
        for changed_line in changed_lines {
            for pattern in &patterns {
                if let Some(violation) = self.check_line_against_pattern(
                    file_path, 
                    changed_line, 
                    pattern, 
                    language.clone()
                )? {
                    violations.push(violation);
                }
            }
        }
        
        Ok(violations)
    }

    pub fn review_git_diff(&self, git_diff: &GitDiff) -> Result<ReviewResult> {
        let mut all_violations = Vec::new();
        let mut files_with_violations = HashMap::new();
        
        for file_diff in &git_diff.files {
            let violations = self.review_changed_lines(&file_diff.path, &file_diff.added_lines)?;
            
            if !violations.is_empty() {
                files_with_violations.insert(file_diff.path.clone(), violations.clone());
                all_violations.extend(violations);
            }
        }
        
        let summary = self.create_review_summary(&all_violations);
        
        Ok(ReviewResult {
            violations: all_violations,
            files_with_violations,
            summary,
        })
    }

    pub fn filter_violations_by_severity<'a>(&self, violations: &'a [ReviewViolation], min_severity: Severity) -> Vec<&'a ReviewViolation> {
        violations.iter()
            .filter(|v| v.severity >= min_severity)
            .collect()
    }

    pub fn create_review_summary(&self, violations: &[ReviewViolation]) -> ReviewSummary {
        let total_violations = violations.len();
        let critical_count = violations.iter().filter(|v| v.severity == Severity::Critical).count();
        let major_count = violations.iter().filter(|v| v.severity == Severity::Major).count();
        let warning_count = violations.iter().filter(|v| v.severity == Severity::Warning).count();
        let auto_fixable_count = violations.iter().filter(|v| v.auto_fixable).count();
        
        let mut files_affected: Vec<String> = violations.iter()
            .map(|v| v.file_path.clone())
            .collect();
        files_affected.sort();
        files_affected.dedup();
        
        ReviewSummary {
            total_violations,
            critical_count,
            major_count,
            warning_count,
            files_affected,
            auto_fixable_count,
        }
    }

    pub fn detect_language_from_path(&self, file_path: &str) -> Option<Language> {
        let path = Path::new(file_path);
        let extension = path.extension()?.to_str()?;
        
        match extension.to_lowercase().as_str() {
            "ex" | "exs" => Some(Language::Elixir),
            "js" | "jsx" | "mjs" | "cjs" => Some(Language::JavaScript),
            "ts" | "tsx" => Some(Language::TypeScript),
            "py" | "pyw" | "pyi" => Some(Language::Python),
            "rs" => Some(Language::Rust),
            "zig" => Some(Language::Zig),
            "sql" | "psql" | "mysql" => Some(Language::Sql),
            _ => None,
        }
    }

    fn check_line_against_pattern(
        &self,
        file_path: &str,
        changed_line: &ChangedLine,
        pattern: &AntiPattern,
        language: Language,
    ) -> Result<Option<ReviewViolation>> {
        // Skip disabled patterns
        if !pattern.enabled {
            return Ok(None);
        }
        
        let matched = match &pattern.detection_method {
            DetectionMethod::Regex { pattern: regex_pattern } => {
                // Use pre-compiled regex if available
                if let Some(compiled_regex) = self.registry.get_compiled_pattern(&pattern.id) {
                    compiled_regex.is_match(&changed_line.content)
                } else {
                    // Fallback to creating regex on the fly
                    match Regex::new(regex_pattern) {
                        Ok(regex) => regex.is_match(&changed_line.content),
                        Err(_) => false, // Skip patterns with invalid regex
                    }
                }
            },
            DetectionMethod::Ratio { pattern: regex_pattern, threshold } => {
                // For ratio-based detection, check if pattern appears frequently enough
                match Regex::new(regex_pattern) {
                    Ok(regex) => {
                        let matches = regex.find_iter(&changed_line.content).count();
                        let total_chars = changed_line.content.len();
                        if total_chars > 0 {
                            let ratio = matches as f64 / total_chars as f64;
                            ratio >= *threshold
                        } else {
                            false
                        }
                    },
                    Err(_) => false,
                }
            },
            DetectionMethod::LineCount { threshold: _, pattern: _ } => {
                // Line count detection would need more context (entire function/file)
                // For now, skip this detection method for single lines
                false
            },
            _ => false, // Other detection methods not implemented yet
        };
        
        if matched {
            let violation = ReviewViolation {
                rule: pattern.clone(),
                file_path: file_path.to_string(),
                line_number: changed_line.line_number,
                content: changed_line.content.clone(),
                severity: pattern.severity,
                language,
                fix_suggestion: pattern.fix_suggestion.clone(),
                auto_fixable: pattern.claude_code_fixable,
                context_before: changed_line.context_before.clone(),
                context_after: changed_line.context_after.clone(),
                confidence: 0.85, // Default confidence score
            };
            
            Ok(Some(violation))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod review_engine_tests {
    use super::*;
    use crate::git::{ChangeType, GitDiffParser};
    use std::time::Instant;

    #[test]
    fn test_review_changed_lines_basic() {
        let engine = ReviewEngine::new();
        
        // Create test changed lines with anti-patterns
        let changed_lines = vec![
            ChangedLine {
                line_number: 42,
                content: "atom = String.to_atom(user_input)".to_string(),
                change_type: ChangeType::Added,
                context_before: vec!["def create_user(input) do".to_string()],
                context_after: vec!["  %User{name: atom}".to_string()],
            },
            ChangedLine {
                line_number: 45,
                content: "# This is just a comment".to_string(),
                change_type: ChangeType::Added,
                context_before: vec![],
                context_after: vec![],
            },
        ];
        
        let violations = engine.review_changed_lines("lib/user.ex", &changed_lines)
            .expect("Should review changed lines");
        
        // Should detect the dynamic atom creation anti-pattern
        assert!(violations.len() > 0, "Should detect violations");
        
        let atom_violation = violations.iter()
            .find(|v| v.rule.id == "dynamic_atom_creation")
            .expect("Should find dynamic atom creation violation");
        
        assert_eq!(atom_violation.line_number, 42);
        assert_eq!(atom_violation.severity, Severity::Critical);
        assert!(atom_violation.fix_suggestion.contains("String.to_existing_atom"));
    }

    #[test]
    fn test_review_engine_performance() {
        let engine = ReviewEngine::new();
        
        // Create moderate number of changed lines to test performance (CI/CD friendly)
        let mut changed_lines = Vec::new();
        for i in 1..=200 {
            changed_lines.push(ChangedLine {
                line_number: i,
                content: format!("atom_{} = String.to_atom(\"test_{}\")", i, i),
                change_type: ChangeType::Added,
                context_before: vec![],
                context_after: vec![],
            });
        }
        
        let start = Instant::now();
        let violations = engine.review_changed_lines("test.ex", &changed_lines)
            .expect("Should handle large review");
        let duration = start.elapsed();
        
        // Should detect violations in all lines
        assert_eq!(violations.len(), 200);
        
        // Should be fast even with many lines (CI/CD timeout adjusted)
        assert!(duration.as_millis() < 2000, "Review should be fast, took: {:?}", duration);
    }

    #[test]
    fn test_review_multiple_languages() {
        let engine = ReviewEngine::new();
        
        let elixir_lines = vec![
            ChangedLine {
                line_number: 10,
                content: "String.to_atom(dynamic_input)".to_string(),
                change_type: ChangeType::Added,
                context_before: vec![],
                context_after: vec![],
            },
        ];
        
        let javascript_lines = vec![
            ChangedLine {
                line_number: 20,
                content: "console.log('debug info')".to_string(),
                change_type: ChangeType::Added,
                context_before: vec![],
                context_after: vec![],
            },
        ];
        
        let elixir_violations = engine.review_changed_lines("lib/user.ex", &elixir_lines)
            .expect("Should review Elixir");
        let js_violations = engine.review_changed_lines("src/app.js", &javascript_lines)
            .expect("Should review JavaScript");
        
        // Should detect language-specific anti-patterns
        assert!(!elixir_violations.is_empty(), "Should detect Elixir violations");
        assert!(!js_violations.is_empty(), "Should detect JavaScript violations");
        
        // Violations should be for correct languages
        assert!(elixir_violations.iter().all(|v| v.language == Language::Elixir));
        assert!(js_violations.iter().all(|v| v.language == Language::JavaScript));
    }

    #[test]
    fn test_review_diff_integration() {
        let diff_output = r#"diff --git a/lib/user.ex b/lib/user.ex
index 1234567..abcdefg 100644
--- a/lib/user.ex
+++ b/lib/user.ex
@@ -10,7 +10,8 @@ defmodule User do
   def create_user(name) do
     # Old implementation
-    atom = String.to_atom(name)
+    # New implementation still has issue
+    atom = String.to_atom(dynamic_name)
     %User{name: atom}
   end
 end"#;

        let git_diff = GitDiffParser::parse(diff_output).expect("Should parse diff");
        let engine = ReviewEngine::new();
        
        let review_result = engine.review_git_diff(&git_diff).expect("Should review diff");
        
        assert!(!review_result.violations.is_empty(), "Should find violations in diff");
        
        // Should have file-level grouping
        assert!(review_result.files_with_violations.contains_key("lib/user.ex"));
        
        let user_file_violations = &review_result.files_with_violations["lib/user.ex"];
        assert!(!user_file_violations.is_empty(), "Should have violations in user.ex");
        
        // Should detect the atom creation issue
        assert!(user_file_violations.iter().any(|v| v.rule.id == "dynamic_atom_creation"));
    }

    #[test]
    fn test_file_extension_language_detection() {
        let engine = ReviewEngine::new();
        
        let test_files = vec![
            ("lib/user.ex", Language::Elixir),
            ("src/app.js", Language::JavaScript),
            ("src/component.tsx", Language::TypeScript),
            ("scripts/deploy.py", Language::Python),
            ("src/main.rs", Language::Rust),
            ("lib/math.zig", Language::Zig),
            ("migrations/001_users.sql", Language::Sql),
        ];
        
        for (file_path, expected_lang) in test_files {
            let detected_lang = engine.detect_language_from_path(file_path);
            assert_eq!(detected_lang, Some(expected_lang.clone()), "Should detect {} for {}", expected_lang, file_path);
        }
        
        // Test unknown extension
        let unknown_lang = engine.detect_language_from_path("README.md");
        assert_eq!(unknown_lang, None, "Should return None for unknown extensions");
    }

    #[test]
    fn test_create_review_summary() {
        let engine = ReviewEngine::new();
        
        // Create mock violations with different severities
        let violations = vec![
            ReviewViolation {
                rule: AntiPattern {
                    id: "test1".to_string(),
                    name: "Test Critical".to_string(),
                    language: Language::Elixir,
                    severity: Severity::Critical,
                    description: "Test".to_string(),
                    detection_method: DetectionMethod::Regex { pattern: "test".to_string() },
                    fix_suggestion: "Fix".to_string(),
                    source_url: None,
                    claude_code_fixable: true,
                    examples: vec![],
                    tags: vec![],
                    enabled: true,
                },
                file_path: "test.ex".to_string(),
                line_number: 1,
                content: "test".to_string(),
                severity: Severity::Critical,
                language: Language::Elixir,
                fix_suggestion: "Fix".to_string(),
                auto_fixable: true,
                context_before: vec![],
                context_after: vec![],
                confidence: 0.9,
            },
        ];
        
        let summary = engine.create_review_summary(&violations);
        
        assert_eq!(summary.total_violations, 1);
        assert_eq!(summary.critical_count, 1);
        assert_eq!(summary.auto_fixable_count, 1);
        assert_eq!(summary.files_affected, vec!["test.ex"]);
    }
}