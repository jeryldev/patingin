#[cfg(test)]
mod git_diff_tests {
    use super::*;
    use std::time::Instant;

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
        let added_lines: Vec<_> = file_diff.added_lines.iter()
            .map(|line| &line.content)
            .collect();
        assert!(added_lines.iter().any(|line| line.contains("String.to_existing_atom")));
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
            assert!(!git_cmd.is_empty(), "Git command should not be empty for scope: {:?}", scope);
            
            // Commands should start with "git diff"
            assert!(git_cmd.starts_with("git diff"), "Command should start with 'git diff': {}", git_cmd);
        }
    }

    #[test]
    fn test_changed_line_extraction() {
        let diff_output = r#"diff --git a/test.ex b/test.ex
index 1234567..abcdefg 100644
--- a/test.ex
+++ b/test.ex
@@ -1,5 +1,6 @@
 defmodule Test do
   def bad_function do
+    # This is a new line with an issue
+    String.to_atom("dynamic")
   end
 end"#;

        let parsed = GitDiffParser::parse(diff_output).expect("Should parse diff");
        let file_diff = &parsed.files[0];
        
        assert_eq!(file_diff.added_lines.len(), 2);
        
        // Check line numbers are correct
        let line_numbers: Vec<_> = file_diff.added_lines.iter()
            .map(|line| line.line_number)
            .collect();
        assert!(line_numbers.contains(&3)); // Comment line
        assert!(line_numbers.contains(&4)); // String.to_atom line
        
        // Check content is preserved
        let content_with_issue = file_diff.added_lines.iter()
            .find(|line| line.content.contains("String.to_atom"))
            .expect("Should find line with String.to_atom");
        assert_eq!(content_with_issue.line_number, 4);
    }

    #[test]
    fn test_context_lines_extraction() {
        let diff_output = r#"diff --git a/context_test.ex b/context_test.ex
index 1234567..abcdefg 100644
--- a/context_test.ex
+++ b/context_test.ex
@@ -8,10 +8,11 @@ defmodule ContextTest do
   # Context before
   def some_function do
     # More context
+    problematic_line = String.to_atom(input)
     # Context after
     result
   end
   # More context"#;

        let parsed = GitDiffParser::parse(diff_output).expect("Should parse diff with context");
        let file_diff = &parsed.files[0];
        
        let problematic_line = file_diff.added_lines.iter()
            .find(|line| line.content.contains("String.to_atom"))
            .expect("Should find problematic line");
        
        // Should have context before and after
        assert!(!problematic_line.context_before.is_empty(), "Should have context before");
        assert!(!problematic_line.context_after.is_empty(), "Should have context after");
        
        // Context should contain relevant lines
        assert!(problematic_line.context_before.iter().any(|line| line.contains("some_function")));
        assert!(problematic_line.context_after.iter().any(|line| line.contains("result")));
    }

    #[test]
    fn test_git_diff_performance() {
        // Test with a large diff to ensure performance
        let mut large_diff = String::from("diff --git a/large_file.ex b/large_file.ex\nindex 1234567..abcdefg 100644\n--- a/large_file.ex\n+++ b/large_file.ex\n");
        
        // Generate a diff with 1000 lines
        for i in 1..=1000 {
            large_diff.push_str(&format!("@@ -{},1 +{},1 @@\n", i, i));
            large_diff.push_str(&format!("+  line_{} = String.to_atom(\"test_{}\")\n", i, i));
        }
        
        let start = Instant::now();
        let parsed = GitDiffParser::parse(&large_diff).expect("Should parse large diff");
        let duration = start.elapsed();
        
        assert_eq!(parsed.files.len(), 1);
        assert_eq!(parsed.files[0].added_lines.len(), 1000);
        
        // Should parse quickly even with many lines
        assert!(duration.as_millis() < 100, "Large diff parsing should be < 100ms, took: {:?}", duration);
    }

    #[test]
    fn test_diff_scope_commands() {
        // Test that different scopes generate correct git commands
        
        let unstaged_cmd = GitDiffParser::build_git_command(&DiffScope::Unstaged);
        assert_eq!(unstaged_cmd, "git diff");
        
        let staged_cmd = GitDiffParser::build_git_command(&DiffScope::Staged);
        assert_eq!(staged_cmd, "git diff --cached");
        
        let since_commit_cmd = GitDiffParser::build_git_command(&DiffScope::SinceCommit("HEAD~3".to_string()));
        assert_eq!(since_commit_cmd, "git diff HEAD~3");
        
        let since_branch_cmd = GitDiffParser::build_git_command(&DiffScope::SinceCommit("origin/main".to_string()));
        assert_eq!(since_branch_cmd, "git diff origin/main");
    }

    #[test]
    fn test_empty_diff_handling() {
        let empty_diff = "";
        let parsed = GitDiffParser::parse(empty_diff).expect("Should handle empty diff");
        assert_eq!(parsed.files.len(), 0);
        
        let no_changes_diff = "diff --git a/unchanged.ex b/unchanged.ex\nindex 1234567..1234567 100644\n--- a/unchanged.ex\n+++ b/unchanged.ex";
        let parsed = GitDiffParser::parse(no_changes_diff).expect("Should handle diff with no changes");
        assert_eq!(parsed.files.len(), 1);
        assert_eq!(parsed.files[0].added_lines.len(), 0);
        assert_eq!(parsed.files[0].removed_lines.len(), 0);
    }

    #[test]
    fn test_binary_file_diff_handling() {
        let binary_diff = r#"diff --git a/image.png b/image.png
index 1234567..abcdefg 100644
Binary files a/image.png and b/image.png differ"#;

        let parsed = GitDiffParser::parse(binary_diff).expect("Should handle binary file diff");
        
        // Should recognize binary file but not try to parse lines
        if !parsed.files.is_empty() {
            let file_diff = &parsed.files[0];
            assert_eq!(file_diff.path, "image.png");
            assert_eq!(file_diff.added_lines.len(), 0);
            assert_eq!(file_diff.removed_lines.len(), 0);
        }
    }

    #[test]
    fn test_line_number_accuracy() {
        let diff_output = r#"diff --git a/line_test.ex b/line_test.ex
index 1234567..abcdefg 100644
--- a/line_test.ex
+++ b/line_test.ex
@@ -15,6 +15,9 @@ defmodule LineTest do
   def existing_function do
     existing_line
   end
+
+  def new_function do
+    String.to_atom("test")
+  end
 end"#;

        let parsed = GitDiffParser::parse(diff_output).expect("Should parse diff");
        let file_diff = &parsed.files[0];
        
        // Find the problematic line
        let atom_line = file_diff.added_lines.iter()
            .find(|line| line.content.contains("String.to_atom"))
            .expect("Should find String.to_atom line");
        
        // Line number should be calculated correctly from diff header
        // @@ -15,6 +15,9 means starting at line 15, the new version adds lines
        assert!(atom_line.line_number > 15, "Line number should be greater than 15");
        assert!(atom_line.line_number <= 25, "Line number should be reasonable");
    }
}