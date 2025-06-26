use patingin::git::{GitDiffParser, DiffScope};
use patingin::core::ReviewEngine;

fn main() {
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
    match GitDiffParser::parse(diff_output) {
        Ok(git_diff) => {
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
            
            match review_engine.review_git_diff(&git_diff) {
                Ok(review_result) => {
                    println!("Found {} violations", review_result.violations.len());
                    for violation in &review_result.violations {
                        println!("  - {} at line {}: {}", 
                            violation.rule.id, 
                            violation.line_number, 
                            violation.content);
                    }
                }
                Err(e) => {
                    println!("Error reviewing diff: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Error parsing diff: {}", e);
        }
    }
}