use anyhow::Result;
use git2::Repository;
use std::path::Path;
use std::process::Command;

pub struct GitIntegration {
    repo: Repository,
}

impl GitIntegration {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::discover(path)?;
        Ok(Self { repo })
    }

    #[allow(dead_code)]
    pub fn get_changed_files(&self) -> Result<Vec<String>> {
        // TODO: Implement getting changed files
        Ok(vec![])
    }

    pub fn get_current_branch(&self) -> Result<String> {
        match self.repo.head() {
            Ok(head) => {
                let branch = head.shorthand().unwrap_or("HEAD");
                Ok(branch.to_string())
            }
            Err(_) => {
                // Handle unborn branch or detached HEAD
                Ok("(no branch)".to_string())
            }
        }
    }
}

// Git diff parsing structures and functionality
#[derive(Debug, Clone, PartialEq)]
pub enum DiffScope {
    /// git diff (unstaged changes)
    Unstaged,
    /// git diff --cached (staged changes)
    Staged,
    /// git diff <commit/branch/tag> (changes since specific reference)
    SinceCommit(String),
}

#[derive(Debug, Clone)]
pub struct ChangedLine {
    pub line_number: usize,
    pub content: String,
    #[allow(dead_code)]
    pub change_type: ChangeType,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Added,
    Removed,
    #[allow(dead_code)]
    Modified,
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: String,
    pub added_lines: Vec<ChangedLine>,
    pub removed_lines: Vec<ChangedLine>,
}

#[derive(Debug, Clone)]
pub struct GitDiff {
    pub files: Vec<FileDiff>,
}

pub struct GitDiffParser;

impl GitDiffParser {
    pub fn parse(diff_output: &str) -> Result<GitDiff> {
        let mut files = Vec::new();
        let mut current_file: Option<FileDiff> = None;
        let mut current_line_number = 0;
        let mut context_lines: Vec<String> = Vec::new();

        for line in diff_output.lines() {
            if line.starts_with("diff --git") {
                // Save previous file if exists
                if let Some(file) = current_file.take() {
                    files.push(file);
                }

                // Extract file path from "diff --git a/path b/path"
                if let Some(path) = Self::extract_file_path(line) {
                    current_file = Some(FileDiff {
                        path,
                        added_lines: Vec::new(),
                        removed_lines: Vec::new(),
                    });
                }
            } else if line.starts_with("@@") {
                // Parse hunk header to get line numbers
                current_line_number = Self::parse_hunk_header(line).unwrap_or(0);
                context_lines.clear();
            } else if line.starts_with('+') && !line.starts_with("+++") {
                // Added line
                if let Some(ref mut file) = current_file {
                    let content = line[1..].to_string(); // Remove '+' prefix
                    let changed_line = ChangedLine {
                        line_number: current_line_number,
                        content,
                        change_type: ChangeType::Added,
                        context_before: context_lines.clone(),
                        context_after: Vec::new(), // Will be filled later if needed
                    };
                    file.added_lines.push(changed_line);
                }
                current_line_number += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                // Removed line
                if let Some(ref mut file) = current_file {
                    let content = line[1..].to_string(); // Remove '-' prefix
                    let changed_line = ChangedLine {
                        line_number: current_line_number,
                        content,
                        change_type: ChangeType::Removed,
                        context_before: context_lines.clone(),
                        context_after: Vec::new(),
                    };
                    file.removed_lines.push(changed_line);
                }
                // Don't increment line number for removed lines
            } else if line.starts_with(' ') {
                // Context line
                context_lines.push(line[1..].to_string());
                // Keep only last 3 context lines
                if context_lines.len() > 3 {
                    context_lines.remove(0);
                }
                current_line_number += 1;
            } else if !line.starts_with("index")
                && !line.starts_with("---")
                && !line.starts_with("+++")
            {
                // Other lines (binary files, etc.)
                continue;
            }
        }

        // Add the last file
        if let Some(file) = current_file {
            files.push(file);
        }

        Ok(GitDiff { files })
    }

    #[allow(dead_code)]
    pub fn build_git_command(scope: &DiffScope) -> String {
        match scope {
            DiffScope::Unstaged => "git diff".to_string(),
            DiffScope::Staged => "git diff --cached".to_string(),
            DiffScope::SinceCommit(reference) => format!("git diff {}", reference),
        }
    }

    pub fn execute_git_diff(scope: &DiffScope) -> Result<String> {
        Self::execute_git_diff_in_dir(scope, None)
    }

    pub fn execute_git_diff_in_dir(
        scope: &DiffScope,
        working_dir: Option<&Path>,
    ) -> Result<String> {
        let command_parts: Vec<&str> = match scope {
            DiffScope::Unstaged => vec!["git", "diff"],
            DiffScope::Staged => vec!["git", "diff", "--cached"],
            DiffScope::SinceCommit(reference) => vec!["git", "diff", reference],
        };

        let mut command = Command::new(command_parts[0]);
        command.args(&command_parts[1..]);

        if let Some(dir) = working_dir {
            command.current_dir(dir);
        }

        let output = command.output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Git diff command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn extract_file_path(diff_line: &str) -> Option<String> {
        // Parse "diff --git a/path b/path" to extract path
        let parts: Vec<&str> = diff_line.split_whitespace().collect();
        if parts.len() >= 4 {
            let a_path = parts[2];
            if a_path.starts_with("a/") {
                return Some(a_path[2..].to_string());
            }
        }
        None
    }

    fn parse_hunk_header(hunk_line: &str) -> Option<usize> {
        // Parse "@@ -15,6 +15,9 @@" to extract starting line number for new version
        if let Some(plus_pos) = hunk_line.find(" +") {
            let after_plus = &hunk_line[plus_pos + 2..];
            if let Some(comma_pos) = after_plus.find(',') {
                let line_num_str = &after_plus[..comma_pos];
                return line_num_str.parse().ok();
            } else if let Some(space_pos) = after_plus.find(' ') {
                let line_num_str = &after_plus[..space_pos];
                return line_num_str.parse().ok();
            }
        }
        None
    }
}

#[cfg(test)]
mod git_diff_tests {
    use super::*;

    #[test]
    fn test_parse_git_diff_basic() {
        let diff_output = r#"diff --git a/lib/user.ex b/lib/user.ex
index 1234567..abcdefg 100644
--- a/lib/user.ex
+++ b/lib/user.ex
@@ -10,7 +10,8 @@ defmodule User do
   def create_user(name) do
     # Old implementation
-    atom = String.to_atom(name)
+    # New implementation with fix
+    atom = String.to_existing_atom(name)
     %User{name: atom}
   end
 end"#;

        let parsed = GitDiffParser::parse(diff_output).expect("Should parse diff");

        assert_eq!(parsed.files.len(), 1);

        let file_diff = &parsed.files[0];
        assert_eq!(file_diff.path, "lib/user.ex");
        assert!(file_diff.added_lines.len() > 0);
        assert!(file_diff.removed_lines.len() > 0);

        // Should capture the added line with the fix
        let added_lines: Vec<_> = file_diff
            .added_lines
            .iter()
            .map(|line| &line.content)
            .collect();
        assert!(added_lines
            .iter()
            .any(|line| line.contains("String.to_existing_atom")));
    }

    #[test]
    fn test_parse_multiple_files_diff() {
        let diff_output = r#"diff --git a/lib/user.ex b/lib/user.ex
index 1234567..abcdefg 100644
--- a/lib/user.ex
+++ b/lib/user.ex
@@ -10,7 +10,7 @@ defmodule User do
   def create_user(name) do
-    atom = String.to_atom(name)
+    atom = String.to_existing_atom(name)
   end

diff --git a/lib/auth.ex b/lib/auth.ex
index 9876543..fedcba9 100644
--- a/lib/auth.ex
+++ b/lib/auth.ex
@@ -5,7 +5,7 @@ defmodule Auth do
   def authenticate(email, password, token, device, ip, session, opts) do
-    # Long parameter list
+    def authenticate(%{email: email, password: password} = user_data, %{token: token, device: device} = auth_data, opts) do
   end
 end"#;

        let parsed = GitDiffParser::parse(diff_output).expect("Should parse multi-file diff");

        assert_eq!(parsed.files.len(), 2);
        assert!(parsed.files.iter().any(|f| f.path == "lib/user.ex"));
        assert!(parsed.files.iter().any(|f| f.path == "lib/auth.ex"));
    }

    #[test]
    fn test_git_diff_scope_detection() {
        // Test different diff scopes
        let scopes = vec![
            DiffScope::Unstaged,
            DiffScope::Staged,
            DiffScope::SinceCommit("HEAD~1".to_string()),
            DiffScope::SinceCommit("origin/main".to_string()),
        ];

        for scope in scopes {
            let git_cmd = GitDiffParser::build_git_command(&scope);
            assert!(
                !git_cmd.is_empty(),
                "Git command should not be empty for scope: {:?}",
                scope
            );

            // Commands should start with "git diff"
            assert!(
                git_cmd.starts_with("git diff"),
                "Command should start with 'git diff': {}",
                git_cmd
            );
        }
    }

    #[test]
    fn test_diff_scope_commands() {
        // Test that different scopes generate correct git commands

        let unstaged_cmd = GitDiffParser::build_git_command(&DiffScope::Unstaged);
        assert_eq!(unstaged_cmd, "git diff");

        let staged_cmd = GitDiffParser::build_git_command(&DiffScope::Staged);
        assert_eq!(staged_cmd, "git diff --cached");

        let since_commit_cmd =
            GitDiffParser::build_git_command(&DiffScope::SinceCommit("HEAD~3".to_string()));
        assert_eq!(since_commit_cmd, "git diff HEAD~3");

        let since_branch_cmd =
            GitDiffParser::build_git_command(&DiffScope::SinceCommit("origin/main".to_string()));
        assert_eq!(since_branch_cmd, "git diff origin/main");
    }

    #[test]
    fn test_empty_diff_handling() {
        let empty_diff = "";
        let parsed = GitDiffParser::parse(empty_diff).expect("Should handle empty diff");
        assert_eq!(parsed.files.len(), 0);

        let no_changes_diff = "diff --git a/unchanged.ex b/unchanged.ex\nindex 1234567..1234567 100644\n--- a/unchanged.ex\n+++ b/unchanged.ex";
        let parsed =
            GitDiffParser::parse(no_changes_diff).expect("Should handle diff with no changes");
        assert_eq!(parsed.files.len(), 1);
        assert_eq!(parsed.files[0].added_lines.len(), 0);
        assert_eq!(parsed.files[0].removed_lines.len(), 0);
    }
}
