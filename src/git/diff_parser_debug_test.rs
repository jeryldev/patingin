#[cfg(test)]
mod diff_parser_debug_tests {
    use crate::git::{GitDiffParser, DiffScope};
    use crate::core::ReviewEngine;

    #[test]
    fn test_debug_integration_test_diff() {
        let diff_output = r#"diff --git a/user.ex b/user.ex
index ac51456..33dce6e 100644
--- a/user.ex
+++ b/user.ex
@@ -1,5 +1,14 @@
 defmodule User do
   def create_user(name) do
-    %User{name: name}
+    # This line contains a critical violation
+    atom = String.to_atom(name)
+    %User{name: atom}
+  end
+  
+  # This function has too many parameters (major violation)
+  def complex_auth(email, password, token, device, ip, session, opts) do
+    # Multiple violations in one function
+    atom_key = String.to_atom("user_key")
+    {:ok, atom_key}
   end
 end
\ No newline at end of file"#;

        // Parse the diff
        let git_diff = GitDiffParser::parse(diff_output).expect("Should parse diff");
        
        println!("Parsed {} files", git_diff.files.len());
        
        for file in &git_diff.files {
            println!("\nFile: {}", file.path);
            println!("Added lines: {}", file.added_lines.len());
            for line in &file.added_lines {
                println!("  Line {}: '{}'", line.line_number, line.content);
            }
        }
        
        // Now test the review engine
        println!("\n--- Testing Review Engine ---");
        let review_engine = ReviewEngine::new();
        
        let review_result = review_engine.review_git_diff(&git_diff)
            .expect("Should review diff");
            
        println!("Found {} violations", review_result.violations.len());
        for violation in &review_result.violations {
            println!("  - {} at line {}: {}", 
                violation.rule.id, 
                violation.line_number, 
                violation.content);
        }
        
        // Assertions from the original test
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
    }
}