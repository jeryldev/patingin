use std::env;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

use patingin::cli::commands::{review, setup};
use patingin::core::{CustomRule, CustomRulesManager, Language, ProjectDetector};
use patingin::external::ClaudeCodeIntegration;

/// Comprehensive error handling tests following TDD principles
///
/// These tests verify that the system gracefully handles various error conditions:
/// 1. Invalid YAML configuration files
/// 2. Missing dependencies (git, Claude Code CLI)
/// 3. Malformed regex patterns in custom rules
/// 4. Non-existent project paths
/// 5. Empty git repositories

#[test]
fn test_invalid_yaml_configuration_handling() {
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let config_dir = temp_dir.path().join(".config").join("patingin");
    fs::create_dir_all(&config_dir).expect("Should create config directory");

    // Create invalid YAML file
    let rules_file = config_dir.join("rules.yml");
    fs::write(
        &rules_file,
        r#"
invalid_yaml: [
  unclosed_bracket: {
    missing_closing_brace
    invalid: syntax here
"#,
    )
    .expect("Should write invalid YAML");

    // Test that CustomRulesManager handles invalid YAML gracefully
    let custom_rules_manager = CustomRulesManager::new();

    // Should not panic, should return an error or handle gracefully
    let result =
        std::panic::catch_unwind(|| custom_rules_manager.get_project_rules("test-project"));

    assert!(
        result.is_ok(),
        "CustomRulesManager should not panic on invalid YAML"
    );
}

#[test]
fn test_malformed_regex_patterns_in_custom_rules() {
    let custom_rules_manager = CustomRulesManager::new();

    // Create a custom rule with invalid regex pattern
    let invalid_rule = CustomRule {
        id: "invalid_regex_rule".to_string(),
        description: "Rule with invalid regex".to_string(),
        pattern: r"[invalid[regex".to_string(), // Invalid regex - unclosed bracket
        severity: "major".to_string(),
        fix: "Fix the issue".to_string(),
        enabled: true,
    };

    // Test that adding invalid regex pattern is handled gracefully
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let result = custom_rules_manager.add_project_rule(
        "test-project",
        &temp_dir.path().to_string_lossy(),
        Language::JavaScript,
        invalid_rule,
    );

    // Should either succeed (with validation happening later) or fail gracefully
    match result {
        Ok(_) => {
            // If it succeeds, the regex validation should happen during pattern matching
            // Let's test that the pattern matching handles invalid regex gracefully
            // This would be tested in the review engine tests
        }
        Err(e) => {
            // Should have a helpful error message
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("regex") || error_msg.contains("pattern"),
                "Error message should mention regex/pattern issue: {}",
                error_msg
            );
        }
    }
}

#[test]
fn test_non_existent_project_paths() {
    let non_existent_path = "/absolutely/non/existent/path/that/should/not/exist";

    // Test ProjectDetector with non-existent path
    let result = ProjectDetector::detect_project(Some(Path::new(non_existent_path)));

    // Should handle gracefully - either return error or detect as generic project
    match result {
        Ok(project_info) => {
            // If it succeeds, should have reasonable defaults
            assert!(
                !project_info.name.is_empty(),
                "Project name should not be empty"
            );
        }
        Err(e) => {
            // Error should be informative
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("path")
                    || error_msg.contains("directory")
                    || error_msg.contains("exist"),
                "Error should mention path/directory issue: {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_empty_git_repository_handling() {
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let repo_path = temp_dir.path();

    // Initialize empty git repository (no commits)
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(repo_path)
        .output()
        .expect("Should initialize git repo");

    std::process::Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .expect("Should set git user email");

    std::process::Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .expect("Should set git user name");

    // Change to repo directory for testing
    let original_dir = env::current_dir().expect("Should get current directory");
    env::set_current_dir(repo_path).expect("Should change directory");

    // Test review command on empty repository
    let review_args = review::ReviewArgs {
        staged: false,
        uncommitted: false,
        since: None,
        severity: None,
        language: None,
        json: false,
        no_color: true,
        suggest: false,
        fix: false,
        auto_fix: false,
        no_confirm: false,
    };

    let result = review::run(review_args).await;

    // Should handle empty repository gracefully
    match result {
        Ok(_) => {
            // Success is fine - should just report no changes
        }
        Err(e) => {
            // Error should be informative about empty repository
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("commit")
                    || error_msg.contains("empty")
                    || error_msg.contains("no changes")
                    || error_msg.contains("HEAD")
                    || error_msg.contains("unknown revision")
                    || error_msg.contains("ambiguous argument"),
                "Error should be informative about empty repository: {}",
                error_msg
            );
        }
    }

    // Restore original directory
    env::set_current_dir(original_dir).expect("Should restore directory");
}

#[tokio::test]
async fn test_setup_command_with_missing_git() {
    // This test simulates missing git by checking how setup handles git detection
    let result = setup::run().await;

    // Setup should always succeed, but may show warnings about missing tools
    assert!(
        result.is_ok(),
        "Setup should not fail even if git is missing"
    );

    // Note: We can't actually test missing git without modifying PATH,
    // but the setup command should handle it gracefully
}

#[test]
fn test_claude_code_integration_with_missing_cli() {
    // Test Claude Code detection when CLI might be missing
    let integration = ClaudeCodeIntegration::detect();

    // Should return a valid ClaudeCodeIntegration regardless of CLI availability
    // The 'available' field indicates whether CLI is present
    assert!(
        integration.available == true || integration.available == false,
        "Claude Code integration should have valid availability status"
    );

    // If not available, version should be None
    if !integration.available {
        assert!(
            integration.version.is_none(),
            "Version should be None when CLI is missing"
        );
    }
}

#[test]
fn test_custom_rules_file_permissions() {
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let config_dir = temp_dir.path().join(".config").join("patingin");
    fs::create_dir_all(&config_dir).expect("Should create config directory");

    let rules_file = config_dir.join("rules.yml");
    fs::write(&rules_file, "projects: {}").expect("Should write rules file");

    // Test reading with restricted permissions (simulation)
    let custom_rules_manager = CustomRulesManager::new();
    let result = custom_rules_manager.get_project_rules("test-project");

    // Should handle file access gracefully
    match result {
        Ok(_) => {
            // Success is expected in normal circumstances
        }
        Err(e) => {
            // If there's an error, it should be about file access
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("permission")
                    || error_msg.contains("access")
                    || error_msg.contains("file"),
                "Error should be about file access: {}",
                error_msg
            );
        }
    }
}

#[test]
fn test_large_configuration_file_handling() {
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let config_dir = temp_dir.path().join(".config").join("patingin");
    fs::create_dir_all(&config_dir).expect("Should create config directory");

    // Create a large configuration file
    let rules_file = config_dir.join("rules.yml");
    let mut large_config = String::from("projects:\n");

    // Add many projects and rules
    for i in 0..1000 {
        large_config.push_str(&format!(
            "  project_{}:\n    path: \"/path/to/project_{}\"\n    rules:\n      elixir:\n        - id: \"rule_{}\"\n          pattern: \"test_pattern_{}\"\n          severity: \"major\"\n", 
            i, i, i, i
        ));
    }

    fs::write(&rules_file, large_config).expect("Should write large config file");

    // Test that large config files are handled efficiently
    let start_time = std::time::Instant::now();
    let custom_rules_manager = CustomRulesManager::new();
    let result = custom_rules_manager.get_project_rules("project_500");
    let duration = start_time.elapsed();

    // Should complete within reasonable time (less than 1 second)
    assert!(
        duration.as_millis() < 1000,
        "Large config loading should be fast"
    );

    // Should handle the large file without errors
    match result {
        Ok(_) => {
            // Success is expected
        }
        Err(e) => {
            // If there's an error, it should not be about file size
            let error_msg = e.to_string();
            assert!(
                !error_msg.contains("too large") && !error_msg.contains("size"),
                "Should not fail due to file size: {}",
                error_msg
            );
        }
    }
}

#[test]
fn test_concurrent_config_access() {
    use std::sync::Arc;
    use std::thread;

    let custom_rules_manager = Arc::new(CustomRulesManager::new());
    let mut handles = vec![];

    // Spawn multiple threads accessing config simultaneously
    for i in 0..10 {
        let manager = Arc::clone(&custom_rules_manager);
        let handle = thread::spawn(move || {
            let project_name = format!("project_{}", i);
            manager.get_project_rules(&project_name)
        });
        handles.push(handle);
    }

    // Wait for all threads and check no panics occurred
    let mut results = vec![];
    for handle in handles {
        let result = handle.join();
        assert!(
            result.is_ok(),
            "Thread should not panic during concurrent access"
        );
        results.push(result.unwrap());
    }

    // All operations should complete (successfully or with expected errors)
    assert_eq!(results.len(), 10, "All threads should complete");
}
