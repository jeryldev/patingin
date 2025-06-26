#[cfg(test)]
mod review_engine_tests {
    use super::*;
    use crate::git::{DiffScope, GitDiffParser, ChangedLine, ChangeType};
    use crate::core::{Language, Severity};
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
        
        // Create many changed lines to test performance
        let mut changed_lines = Vec::new();
        for i in 1..=1000 {
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
        assert_eq!(violations.len(), 1000);
        
        // Should be fast even with many lines
        assert!(duration.as_millis() < 500, "Review should be fast, took: {:?}", duration);
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
    fn test_violation_severity_filtering() {
        let engine = ReviewEngine::new();
        
        let changed_lines = vec![
            ChangedLine {
                line_number: 10,
                content: "String.to_atom(input)".to_string(), // Critical
                change_type: ChangeType::Added,
                context_before: vec![],
                context_after: vec![],
            },
            ChangedLine {
                line_number: 20,
                content: "def long_func(a, b, c, d, e, f) do".to_string(), // Major
                change_type: ChangeType::Added,
                context_before: vec![],
                context_after: vec![],
            },
            ChangedLine {
                line_number: 30,
                content: "# Lots of comments everywhere".to_string(), // Warning
                change_type: ChangeType::Added,
                context_before: vec![],
                context_after: vec![],
            },
        ];
        
        let all_violations = engine.review_changed_lines("test.ex", &changed_lines)
            .expect("Should review all lines");
        
        let critical_violations = engine.filter_violations_by_severity(&all_violations, Severity::Critical);
        let major_violations = engine.filter_violations_by_severity(&all_violations, Severity::Major);
        
        // Critical filter should only include critical violations
        assert!(critical_violations.iter().all(|v| v.severity == Severity::Critical));
        
        // Major filter should include critical and major
        assert!(major_violations.iter().all(|v| v.severity >= Severity::Major));
        assert!(critical_violations.len() <= major_violations.len());
    }

    #[test]
    fn test_review_result_summary() {
        let engine = ReviewEngine::new();
        
        let changed_lines = vec![
            ChangedLine {
                line_number: 10,
                content: "String.to_atom(input)".to_string(), // Critical
                change_type: ChangeType::Added,
                context_before: vec![],
                context_after: vec![],
            },
            ChangedLine {
                line_number: 20,
                content: "def long_func(a, b, c, d, e) do".to_string(), // Major
                change_type: ChangeType::Added,
                context_before: vec![],
                context_after: vec![],
            },
        ];
        
        let violations = engine.review_changed_lines("test.ex", &changed_lines)
            .expect("Should review lines");
        
        let summary = engine.create_review_summary(&violations);
        
        assert_eq!(summary.total_violations, violations.len());
        assert!(summary.critical_count > 0);
        assert!(summary.major_count >= 0);
        assert!(summary.warning_count >= 0);
        assert_eq!(summary.total_violations, summary.critical_count + summary.major_count + summary.warning_count);
        assert!(!summary.files_affected.is_empty());
    }

    #[test]
    fn test_context_aware_detection() {
        let engine = ReviewEngine::new();
        
        // Test that context helps with more accurate detection
        let line_with_context = ChangedLine {
            line_number: 15,
            content: "atom = String.to_atom(input)".to_string(),
            change_type: ChangeType::Added,
            context_before: vec![
                "def handle_user_input(input) do".to_string(),
                "  # Processing user data".to_string(),
            ],
            context_after: vec![
                "  process_atom(atom)".to_string(),
                "end".to_string(),
            ],
        };
        
        let violations = engine.review_changed_lines("lib/user.ex", &vec![line_with_context])
            .expect("Should review with context");
        
        assert!(!violations.is_empty(), "Should detect violation");
        
        let violation = &violations[0];
        assert!(!violation.context_before.is_empty(), "Should preserve context before");
        assert!(!violation.context_after.is_empty(), "Should preserve context after");
    }

    #[test]
    fn test_auto_fixable_detection() {
        let engine = ReviewEngine::new();
        
        let changed_lines = vec![
            ChangedLine {
                line_number: 10,
                content: "String.to_atom(user_input)".to_string(),
                change_type: ChangeType::Added,
                context_before: vec![],
                context_after: vec![],
            },
        ];
        
        let violations = engine.review_changed_lines("lib/user.ex", &changed_lines)
            .expect("Should review lines");
        
        let auto_fixable_violations: Vec<_> = violations.iter()
            .filter(|v| v.auto_fixable)
            .collect();
        
        assert!(!auto_fixable_violations.is_empty(), "Should have auto-fixable violations");
        
        // Check that auto-fixable violations have Claude Code integration
        for violation in auto_fixable_violations {
            assert!(!violation.fix_suggestion.is_empty(), "Auto-fixable violation should have fix suggestion");
            assert!(violation.auto_fixable, "Should be marked as auto-fixable");
        }
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
            assert_eq!(detected_lang, Some(expected_lang), "Should detect {} for {}", expected_lang, file_path);
        }
        
        // Test unknown extension
        let unknown_lang = engine.detect_language_from_path("README.md");
        assert_eq!(unknown_lang, None, "Should return None for unknown extensions");
    }
}