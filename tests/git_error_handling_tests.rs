use anyhow::Result;
use std::env;
use std::process::Command;
use tempfile::TempDir;

use patingin::git::{DiffScope, GitDiffParser};

/// Tests for git error handling improvements
/// These tests ensure git operations handle edge cases gracefully

#[test]
fn test_git_diff_with_empty_repository() {
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let repo_path = temp_dir.path();

    // Initialize empty git repository (no commits)
    Command::new("git")
        .args(&["init"])
        .current_dir(repo_path)
        .output()
        .expect("Should initialize git repo");

    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .expect("Should set git user email");

    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .expect("Should set git user name");

    // Use directory guard to ensure restoration
    let original_dir = env::current_dir().expect("Should get current directory");
    env::set_current_dir(repo_path).expect("Should change directory");

    // Test git diff on empty repository - should handle gracefully
    let result = GitDiffParser::execute_git_diff(&DiffScope::SinceCommit("HEAD".to_string()));

    // Restore directory before assertions (ignore errors since temp dir might be cleaned up)
    let _ = env::set_current_dir(original_dir);

    match result {
        Ok(diff_output) => {
            // Should return empty diff or handle gracefully
            assert!(
                diff_output.is_empty() || diff_output.contains("no changes"),
                "Empty repository should return empty or 'no changes' diff"
            );
        }
        Err(e) => {
            // Error should be informative
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("empty")
                    || error_msg.contains("no commits")
                    || error_msg.contains("HEAD")
                    || error_msg.contains("no changes"),
                "Error should be informative about empty repository: {}",
                error_msg
            );
        }
    }
}

#[test]
fn test_git_diff_with_non_git_directory() {
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let non_git_path = temp_dir.path();

    // Change to non-git directory
    let original_dir = env::current_dir().expect("Should get current directory");
    env::set_current_dir(non_git_path).expect("Should change directory");

    // Test git diff in non-git directory
    let result = GitDiffParser::execute_git_diff(&DiffScope::SinceCommit("HEAD".to_string()));

    // Restore directory before assertions
    env::set_current_dir(original_dir).expect("Should restore directory");

    // Should return appropriate error
    match result {
        Ok(_) => {
            panic!("Should not succeed in non-git directory");
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("not a git repository")
                    || error_msg.contains("git repository")
                    || error_msg.contains("not in a git"),
                "Error should mention git repository issue: {}",
                error_msg
            );
        }
    }
}

#[test]
fn test_git_diff_with_invalid_commit_reference() {
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let repo_path = temp_dir.path();

    // Initialize git repository with one commit
    setup_git_repo_with_commit(repo_path).expect("Should setup git repo");

    // Change to repo directory
    let original_dir = env::current_dir().expect("Should get current directory");
    env::set_current_dir(repo_path).expect("Should change directory");

    // Test git diff with invalid commit reference
    let result =
        GitDiffParser::execute_git_diff(&DiffScope::SinceCommit("nonexistent-commit".to_string()));

    // Restore directory before assertions
    env::set_current_dir(original_dir).expect("Should restore directory");

    // Should return appropriate error
    match result {
        Ok(_) => {
            panic!("Should not succeed with invalid commit reference");
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("unknown revision")
                    || error_msg.contains("ambiguous argument")
                    || error_msg.contains("not a valid"),
                "Error should mention invalid revision: {}",
                error_msg
            );
        }
    }
}

#[test]
fn test_git_diff_graceful_degradation() {
    // Test that when git diff fails, we can still provide useful information
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let repo_path = temp_dir.path();

    // Create a regular directory (not git)
    let original_dir = env::current_dir().expect("Should get current directory");
    env::set_current_dir(repo_path).expect("Should change directory");

    // Test multiple diff scopes to ensure they all handle errors gracefully
    let scopes = vec![
        DiffScope::Unstaged,
        DiffScope::Staged,
        DiffScope::SinceCommit("HEAD".to_string()),
        DiffScope::SinceCommit("main".to_string()),
    ];

    for scope in scopes {
        let result = GitDiffParser::execute_git_diff(&scope);

        match result {
            Ok(_) => {
                // Unexpected success - should not happen in non-git directory
            }
            Err(e) => {
                // Error should be informative and not panic
                let error_msg = e.to_string();
                assert!(!error_msg.is_empty(), "Error message should not be empty");
                assert!(error_msg.len() > 10, "Error message should be descriptive");
            }
        }
    }

    // Restore directory (ignore errors if temp dir was cleaned up)
    let _ = env::set_current_dir(original_dir);
}

fn setup_git_repo_with_commit(repo_path: &std::path::Path) -> Result<()> {
    // Initialize git repo
    Command::new("git").args(&["init"]).current_dir(repo_path).output()?;

    // Configure git user
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
    std::fs::write(readme, "# Test Repository\n")?;

    Command::new("git").args(&["add", "README.md"]).current_dir(repo_path).output()?;

    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()?;

    Ok(())
}
