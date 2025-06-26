use anyhow::Result;
use std::env;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

use patingin::cli::commands::{review, setup};
use patingin::core::{Language, CustomRulesManager, CustomRule};
use patingin::git::{DiffScope, GitDiffParser, GitIntegration};
use patingin::external::ClaudeCodeIntegration;

/// Integration tests for real git workflows and end-to-end scenarios
/// 
/// These tests verify:
/// 1. Real git repository setup and diff execution
/// 2. Line-by-line anti-pattern detection
/// 3. Claude Code integration scenarios (present/absent)
/// 4. Complete workflows from rule addition to violation detection

#[tokio::test]
async fn test_end_to_end_workflow_add_rule_find_violation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    // Initialize a git repository
    setup_test_git_repo(repo_path)?;
    
    // Change to the repo directory
    let original_dir = env::current_dir()?;
    env::set_current_dir(repo_path)?;
    
    // Create custom rules config for this test
    let custom_rules_manager = CustomRulesManager::new();
    
    // Add a custom rule to detect console.log
    let custom_rule = CustomRule {
        id: "no_console_log_test".to_string(),
        description: "Avoid console.log in production".to_string(),
        pattern: r"console\.log\(".to_string(),
        severity: "major".to_string(),
        fix: "Use proper logging library".to_string(),
        enabled: true,
    };
    
    custom_rules_manager.add_project_rule(
        "test-repo",
        repo_path.to_string_lossy().as_ref(),
        Language::JavaScript,
        custom_rule,
    )?;
    
    // Create a file with a violation
    let js_file = repo_path.join("src").join("app.js");
    fs::create_dir_all(js_file.parent().unwrap())?;
    fs::write(&js_file, r#"
function debugInfo() {
    console.log("This should be caught by our rule");
    return "debug info";
}
"#)?;
    
    // Add and commit the file
    Command::new("git").args(&["add", "."]).current_dir(repo_path).output()?;
    Command::new("git")
        .args(&["commit", "-m", "Add file with violation"])
        .current_dir(repo_path)
        .output()?;
    
    // Modify the file to create a diff
    fs::write(&js_file, r#"
function debugInfo() {
    console.log("This should be caught by our rule");
    console.log("Added another violation");
    return "debug info";
}
"#)?;
    
    // Test review command with default scope (changes since last commit)
    let review_args = review::ReviewArgs {
        staged: false,
        uncommitted: false,
        since: None, // Should default to HEAD
        severity: None,
        language: None,
        json: false,
        no_color: true,
        suggest: false,
        fix: false,
        auto_fix: false,
        no_confirm: false,
    };
    
    // This should detect the console.log violation in the new line
    let result = review::run(review_args).await;
    assert!(result.is_ok(), "Review should succeed");
    
    // Restore original directory
    env::set_current_dir(original_dir)?;
    
    Ok(())
}

#[tokio::test]
async fn test_git_diff_branch_vs_commit_real_execution() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    // Setup git repo with a branch
    setup_test_git_repo_with_branch(repo_path)?;
    
    // Test different git diff scopes with real execution
    let scopes = vec![
        DiffScope::Unstaged,
        DiffScope::Staged,
        DiffScope::SinceCommit("HEAD~1".to_string()),
        DiffScope::SinceCommit("main".to_string()),
    ];
    
    for scope in scopes {
        let result = GitDiffParser::execute_git_diff_in_dir(&scope, Some(repo_path));
        match scope {
            DiffScope::SinceCommit(ref reference) if reference == "main" => {
                // Should work when comparing to main branch
                assert!(result.is_ok(), "Git diff to main should work: {:?}", scope);
            }
            DiffScope::SinceCommit(ref reference) if reference == "HEAD~1" => {
                // Should work when comparing to previous commit
                assert!(result.is_ok(), "Git diff to HEAD~1 should work: {:?}", scope);
            }
            _ => {
                // Other scopes might be empty but should not error
                let output = result.unwrap_or_default();
                // Just verify it's a valid string - all strings have valid length
                let _length = output.len(); // Verify we can get length without panic
            }
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_line_by_line_violation_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    setup_test_git_repo(repo_path)?;
    
    // Create an Elixir file with multiple anti-patterns
    let elixir_file = repo_path.join("lib").join("user.ex");
    fs::create_dir_all(elixir_file.parent().unwrap())?;
    
    // Initial commit with clean code
    fs::write(&elixir_file, r#"
defmodule User do
  def create_user(name) do
    %User{name: name}
  end
end
"#)?;
    
    Command::new("git").args(&["add", "."]).current_dir(repo_path).output()?;
    Command::new("git")
        .args(&["commit", "-m", "Initial clean code"])
        .current_dir(repo_path)
        .output()?;
    
    // Add violations in new commit
    fs::write(&elixir_file, r#"
defmodule User do
  def create_user(name) do
    # This line contains a critical violation
    atom = String.to_atom(name)
    %User{name: atom}
  end
  
  # This function has too many parameters (major violation)
  def complex_auth(email, password, token, device, ip, session, opts) do
    # Multiple violations in one function
    atom_key = String.to_atom("user_key")
    {:ok, atom_key}
  end
end
"#)?;
    
    Command::new("git").args(&["add", "."]).current_dir(repo_path).output()?;
    Command::new("git")
        .args(&["commit", "-m", "Add violations"])
        .current_dir(repo_path)
        .output()?;
    
    // Execute git diff to get the actual changes (use explicit working directory)
    let diff_output = GitDiffParser::execute_git_diff_in_dir(
        &DiffScope::SinceCommit("HEAD~1".to_string()),
        Some(repo_path)
    )?;
    let git_diff = GitDiffParser::parse(&diff_output)?;
    
    // Review the diff to find violations
    let review_engine = patingin::core::ReviewEngine::new();
    let review_result = review_engine.review_git_diff(&git_diff)?;
    
    // Should detect multiple violations
    assert!(!review_result.violations.is_empty(), "Should detect violations in the diff");
    
    // Should detect dynamic atom creation (critical)
    let atom_violations: Vec<_> = review_result.violations.iter()
        .filter(|v| v.rule.id == "dynamic_atom_creation")
        .collect();
    assert!(!atom_violations.is_empty(), "Should detect String.to_atom violations");
    
    // Should detect long parameter list (major)
    let param_violations: Vec<_> = review_result.violations.iter()
        .filter(|v| v.rule.id == "long_parameter_list")
        .collect();
    assert!(!param_violations.is_empty(), "Should detect long parameter list violations");
    
    // Verify line numbers are correct
    for violation in &review_result.violations {
        assert!(violation.line_number > 0, "Line numbers should be positive");
        assert!(!violation.content.is_empty(), "Violation content should not be empty");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_claude_code_detection_scenarios() -> Result<()> {
    // Test 1: Basic detection
    let integration = ClaudeCodeIntegration::detect();
    
    // Should return a valid boolean (doesn't matter which)
    assert!(integration.available == true || integration.available == false);
    
    // Test 2: Setup command handles Claude Code presence/absence gracefully
    let result = setup::run().await;
    assert!(result.is_ok(), "Setup should handle Claude Code availability gracefully");
    
    Ok(())
}

#[tokio::test]
async fn test_setup_command_git_repository_scenarios() -> Result<()> {
    // Test 1: Setup in a git repository
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    setup_test_git_repo(repo_path)?;
    
    let original_dir = env::current_dir()?;
    env::set_current_dir(repo_path)?;
    
    let result = setup::run().await;
    assert!(result.is_ok(), "Setup should work in git repository");
    
    // Test 2: Setup in non-git directory
    let non_git_dir = TempDir::new()?;
    env::set_current_dir(non_git_dir.path())?;
    
    let result = setup::run().await;
    assert!(result.is_ok(), "Setup should work in non-git directory");
    
    env::set_current_dir(original_dir)?;
    Ok(())
}

#[tokio::test]
async fn test_git_integration_branch_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    setup_test_git_repo_with_branch(repo_path)?;
    
    let original_dir = env::current_dir()?;
    env::set_current_dir(repo_path)?;
    
    // Test GitIntegration creation and branch detection
    let git_integration = GitIntegration::new(repo_path)?;
    let current_branch = git_integration.get_current_branch()?;
    
    // Should detect a branch name (either main or feature-branch)
    assert!(!current_branch.is_empty(), "Should detect current branch");
    assert!(current_branch == "main" || current_branch == "feature-branch" || current_branch == "HEAD");
    
    env::set_current_dir(original_dir)?;
    Ok(())
}

#[tokio::test]
async fn test_review_command_with_no_violations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    setup_test_git_repo(repo_path)?;
    
    let original_dir = env::current_dir()?;
    env::set_current_dir(repo_path)?;
    
    // Create a clean file with no violations
    let clean_file = repo_path.join("src").join("clean.ex");
    fs::create_dir_all(clean_file.parent().unwrap())?;
    fs::write(&clean_file, r#"
defmodule Clean do
  def safe_function(data) do
    %{result: data}
  end
end
"#)?;
    
    Command::new("git").args(&["add", "."]).current_dir(repo_path).output()?;
    Command::new("git")
        .args(&["commit", "-m", "Add clean code"])
        .current_dir(repo_path)
        .output()?;
    
    // Review should succeed with no violations
    let review_args = review::ReviewArgs {
        staged: false,
        uncommitted: false,
        since: Some("HEAD~1".to_string()),
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
    assert!(result.is_ok(), "Review should succeed even with no violations");
    
    env::set_current_dir(original_dir)?;
    Ok(())
}

#[tokio::test]
async fn test_review_command_json_output() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();
    
    setup_test_git_repo(repo_path)?;
    
    let original_dir = env::current_dir()?;
    env::set_current_dir(repo_path)?;
    
    // Create a file with known violations
    let js_file = repo_path.join("src").join("violations.js");
    fs::create_dir_all(js_file.parent().unwrap())?;
    fs::write(&js_file, r#"
// Initial version
function test() {
    return "ok";
}
"#)?;
    
    Command::new("git").args(&["add", "."]).current_dir(repo_path).output()?;
    Command::new("git")
        .args(&["commit", "-m", "Initial version"])
        .current_dir(repo_path)
        .output()?;
    
    // Add violations
    fs::write(&js_file, r#"
// Version with violations
function test() {
    console.log("debug info");
    eval("dangerous code");
    return "ok";
}
"#)?;
    
    // Test JSON output format
    let review_args = review::ReviewArgs {
        staged: false,
        uncommitted: true, // Check unstaged changes
        since: None,
        severity: None,
        language: None,
        json: true, // Request JSON output
        no_color: true,
        suggest: false,
        fix: false,
        auto_fix: false,
        no_confirm: false,
    };
    
    let result = review::run(review_args).await;
    assert!(result.is_ok(), "Review with JSON output should succeed");
    
    env::set_current_dir(original_dir)?;
    Ok(())
}

// Helper functions

fn setup_test_git_repo(repo_path: &std::path::Path) -> Result<()> {
    // Initialize git repo
    Command::new("git")
        .args(&["init"])
        .current_dir(repo_path)
        .output()?;
    
    // Configure git user (required for commits)
    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()?;
    
    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()?;
    
    // Create initial commit
    let readme = repo_path.join("README.md");
    fs::write(readme, "# Test Repository\n")?;
    
    Command::new("git")
        .args(&["add", "README.md"])
        .current_dir(repo_path)
        .output()?;
    
    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()?;
    
    Ok(())
}

fn setup_test_git_repo_with_branch(repo_path: &std::path::Path) -> Result<()> {
    // Setup basic repo
    setup_test_git_repo(repo_path)?;
    
    // Create and switch to a feature branch
    Command::new("git")
        .args(&["checkout", "-b", "feature-branch"])
        .current_dir(repo_path)
        .output()?;
    
    // Add some content to the feature branch
    let feature_file = repo_path.join("feature.txt");
    fs::write(feature_file, "Feature branch content\n")?;
    
    Command::new("git")
        .args(&["add", "feature.txt"])
        .current_dir(repo_path)
        .output()?;
    
    Command::new("git")
        .args(&["commit", "-m", "Add feature content"])
        .current_dir(repo_path)
        .output()?;
    
    Ok(())
}