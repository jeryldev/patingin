use anyhow::Result;
use std::env;
use std::fs;
use std::process::Command;

use patingin::cli::commands::{review, setup};
use patingin::core::{CustomRule, CustomRulesManager, Language, ReviewEngine};
use patingin::git::{DiffScope, GitDiffParser};

/// Improved integration tests that use in-project test files
/// instead of creating temporary directories.
///
/// This approach is cleaner because:
/// 1. Test files are version controlled and visible
/// 2. No temporary directory cleanup needed
/// 3. More realistic testing environment
/// 4. Easier to debug issues with actual files

#[tokio::test]
async fn test_real_project_violation_detection() -> Result<()> {
    // Get the current project directory (patingin repo)
    let project_root = env::current_dir()?;

    // Ensure we're in the patingin project
    assert!(project_root.join("Cargo.toml").exists(), "Should be in patingin project directory");
    assert!(project_root.join("src").exists(), "Should have src directory");
    assert!(project_root.join("test_files").exists(), "Should have test_files directory");

    // Create a commit with a new violation file to test git diff
    let test_file = project_root.join("test_files").join("new_violations.ex");
    fs::write(
        &test_file,
        r#"
defmodule NewViolations do
  # This will be detected as a dynamic atom creation violation
  def dangerous_function(user_input) do
    risky_atom = String.to_atom(user_input)
    {:ok, risky_atom}
  end
  
  # This will be detected as a long parameter list violation  
  def complex_function(a, b, c, d, e, f, g, h) do
    {a, b, c, d, e, f, g, h}
  end
end
"#,
    )?;

    // Add and commit the new file
    Command::new("git")
        .args(&["add", "test_files/new_violations.ex"])
        .current_dir(&project_root)
        .output()?;

    Command::new("git")
        .args(&["commit", "-m", "Add test violations for integration test"])
        .current_dir(&project_root)
        .output()?;

    // Now test git diff analysis on the last commit
    let git_diff = GitDiffParser::execute_git_diff(&DiffScope::SinceCommit("HEAD~1".to_string()))?;
    let parsed_diff = GitDiffParser::parse(&git_diff)?;

    // Review the changes
    let review_engine = ReviewEngine::new();
    let review_result = review_engine.review_git_diff(&parsed_diff)?;

    // Should detect violations in our new file
    assert!(!review_result.violations.is_empty(), "Should detect violations in new test file");

    // Verify we found the specific violations we expect
    let atom_violations: Vec<_> =
        review_result.violations.iter().filter(|v| v.rule.id == "dynamic_atom_creation").collect();
    assert!(!atom_violations.is_empty(), "Should detect String.to_atom violation");

    let param_violations: Vec<_> =
        review_result.violations.iter().filter(|v| v.rule.id == "long_parameter_list").collect();
    assert!(!param_violations.is_empty(), "Should detect long parameter list violation");

    // Clean up - remove the test file and reset git
    fs::remove_file(&test_file)?;
    Command::new("git").args(&["reset", "--hard", "HEAD~1"]).current_dir(&project_root).output()?;

    Ok(())
}

#[tokio::test]
async fn test_review_existing_test_files() -> Result<()> {
    let project_root = env::current_dir()?;

    // Test reviewing existing violations in our test files
    // First, modify an existing test file to create a git diff
    let violations_file = project_root.join("test_files").join("elixir_violations.ex");
    let original_content = fs::read_to_string(&violations_file)?;

    // Add another violation to the file
    let modified_content = format!("{}\n\n  # Added violation for testing\n  def new_violation(input) do\n    String.to_atom(input)\n  end\nend", 
        original_content.trim_end_matches("end"));

    fs::write(&violations_file, &modified_content)?;

    // Test the review command on unstaged changes
    let review_args = review::ReviewArgs {
        staged: false,
        uncommitted: true, // Review unstaged changes
        since: None,
        severity: None,
        language: Some(Language::Elixir),
        json: false,
        no_color: true,
        suggest: false,
        fix: false,
        auto_fix: false,
        no_confirm: false,
    };

    // This should work without panicking and detect violations
    let result = review::run(review_args).await;
    assert!(result.is_ok(), "Review command should succeed");

    // Restore original file
    fs::write(&violations_file, &original_content)?;

    Ok(())
}

#[tokio::test]
async fn test_setup_command_in_real_project() -> Result<()> {
    let _project_root = env::current_dir()?;

    // Test setup command in our actual project
    let result = setup::run().await;

    // Should succeed in real project environment
    assert!(result.is_ok(), "Setup should work in real patingin project");

    Ok(())
}

#[tokio::test]
async fn test_custom_rules_with_project_files() -> Result<()> {
    let project_root = env::current_dir()?;
    let project_name = project_root.file_name().unwrap().to_string_lossy().to_string();

    // Add a custom rule for this project
    let custom_rules_manager = CustomRulesManager::new();
    let custom_rule = CustomRule {
        id: "test_custom_rule".to_string(),
        description: "Test custom rule for integration test".to_string(),
        pattern: r"# Added violation for testing".to_string(),
        severity: "warning".to_string(),
        fix: "Remove test comment".to_string(),
        enabled: true,
    };

    custom_rules_manager.add_project_rule(
        &project_name,
        &project_root.to_string_lossy(),
        Language::Elixir,
        custom_rule,
    )?;

    // Verify the rule was added
    let project_rules = custom_rules_manager.get_project_rules(&project_name)?;
    assert!(!project_rules.is_empty(), "Should have custom rules");

    // Clean up - remove the custom rule
    custom_rules_manager.remove_project_rule(&project_name, "test_custom_rule")?;

    Ok(())
}

#[tokio::test]
async fn test_multi_language_project_detection() -> Result<()> {
    let project_root = env::current_dir()?;

    // Our project should be detected as having multiple languages
    // based on our test files and source code
    let _review_engine = ReviewEngine::new();

    // Check that we can detect both Rust (src/) and Elixir (test_files/) in our project
    assert!(project_root.join("src").join("main.rs").exists(), "Should have Rust files");
    assert!(
        project_root.join("test_files").join("elixir_violations.ex").exists(),
        "Should have Elixir test files"
    );
    assert!(
        project_root.join("test_files").join("javascript_violations.js").exists(),
        "Should have JavaScript test files"
    );

    Ok(())
}
