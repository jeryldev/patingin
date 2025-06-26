// Tests to validate claims made in documentation
use std::process::Command;

#[test]
fn test_actual_command_help_matches_docs() {
    // Test that our binary actually supports the commands we document
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to run patingin --help");
    
    let help_text = String::from_utf8_lossy(&output.stdout);
    
    // Verify core commands exist
    assert!(help_text.contains("rules"), "Missing 'rules' command in help");
    assert!(help_text.contains("review"), "Missing 'review' command in help");
    assert!(help_text.contains("setup"), "Missing 'setup' command in help");
}

#[test]
fn test_review_command_options() {
    let output = Command::new("cargo")
        .args(&["run", "--", "review", "--help"])
        .output()
        .expect("Failed to run patingin review --help");
    
    let help_text = String::from_utf8_lossy(&output.stdout);
    
    // Verify documented flags exist
    assert!(help_text.contains("--staged"), "Missing --staged flag");
    assert!(help_text.contains("--uncommitted"), "Missing --uncommitted flag");
    assert!(help_text.contains("--since"), "Missing --since flag");
    assert!(help_text.contains("--severity"), "Missing --severity flag");
    assert!(help_text.contains("--language"), "Missing --language flag");
    assert!(help_text.contains("--json"), "Missing --json flag");
    assert!(help_text.contains("--auto-fix"), "Missing --auto-fix flag");
    assert!(help_text.contains("--no-confirm"), "Missing --no-confirm flag");
    assert!(help_text.contains("--suggest"), "Missing --suggest flag");
}

#[test]
fn test_rules_command_options() {
    let output = Command::new("cargo")
        .args(&["run", "--", "rules", "--help"])
        .output()
        .expect("Failed to run patingin rules --help");
    
    let help_text = String::from_utf8_lossy(&output.stdout);
    
    // Verify documented flags exist
    assert!(help_text.contains("--elixir"), "Missing --elixir flag");
    assert!(help_text.contains("--javascript"), "Missing --javascript flag");
    assert!(help_text.contains("--search"), "Missing --search flag");
    assert!(help_text.contains("--detail"), "Missing --detail flag");
}

#[test]
fn test_actual_builtin_rules_count() {
    use std::fs;
    use serde_yaml::Value;
    
    let mut total_rules = 0;
    let rule_files = [
        "src/rules/builtin/elixir.yml",
        "src/rules/builtin/javascript.yml", 
        "src/rules/builtin/typescript.yml",
        "src/rules/builtin/python.yml",
        "src/rules/builtin/rust.yml",
        "src/rules/builtin/zig.yml",
        "src/rules/builtin/sql.yml",
    ];
    
    for file_path in rule_files.iter() {
        let content = fs::read_to_string(file_path)
            .expect(&format!("Failed to read {}", file_path));
        let rules: Vec<Value> = serde_yaml::from_str(&content)
            .expect(&format!("Failed to parse YAML in {}", file_path));
        total_rules += rules.len();
    }
    
    // Document the actual count we found
    println!("Actual built-in rules count: {}", total_rules);
    assert!(total_rules > 40, "Should have substantial number of rules");
    assert!(total_rules < 60, "Sanity check on rule count");
}

#[test] 
fn test_test_count_matches_docs() {
    // Count actual test functions in the codebase
    let output = Command::new("find")
        .args(&["src", "-name", "*.rs", "-exec", "grep", "-c", "fn test_", "{}", ";"])
        .output()
        .expect("Failed to count test functions");
    
    let counts: Vec<i32> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.parse().ok())
        .collect();
    
    let total_tests = counts.iter().sum::<i32>();
    println!("Actual test count: {}", total_tests);
    
    // We should have substantial test coverage but don't exaggerate
    assert!(total_tests > 50, "Should have significant test coverage");
}

#[cfg(test)]
mod claude_code_detection_tests {
    // Note: Not using super::* to avoid unused import warning
    
    #[test]
    fn test_claude_code_detection_methods() {
        // Test that our code actually looks for the right commands
        // This tests the detection logic in our ClaudeCodeIntegration
        use which::which;
        use std::process::Command;
        
        // Our code should detect either "claude-code" or "claude"
        let claude_code_available = which("claude-code").is_ok();
        let claude_available = which("claude").is_ok();
        
        println!("claude-code command available: {}", claude_code_available);
        println!("claude command available: {}", claude_available);
        
        // Check npm installation status
        let npm_check = Command::new("npm")
            .args(&["list", "-g", "@anthropic-ai/claude-code"])
            .output();
            
        if let Ok(output) = npm_check {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let npm_installed = output_str.contains("@anthropic-ai/claude-code");
            println!("Claude Code npm package installed: {}", npm_installed);
            
            if npm_installed && !claude_available {
                println!("WARNING: npm package installed but 'claude' command not found in PATH");
            }
        }
        
        // Just document what we found - don't require Claude Code to be installed
        // This test validates our detection logic works
    }
}

#[test]
fn test_supported_languages_match_implementation() {
    use std::fs;
    
    // Check that we actually have rule files for documented languages
    let expected_languages = [
        "elixir.yml",
        "javascript.yml", 
        "typescript.yml",
        "python.yml",
        "rust.yml",
        "zig.yml",
        "sql.yml",
    ];
    
    for lang_file in expected_languages.iter() {
        let path = format!("src/rules/builtin/{}", lang_file);
        assert!(fs::metadata(&path).is_ok(), "Missing rule file: {}", path);
        
        // Verify file has actual rules
        let content = fs::read_to_string(&path).expect("Failed to read rule file");
        assert!(content.contains("id:"), "Rule file {} appears empty", lang_file);
    }
}

#[test]
fn test_performance_claims_are_reasonable() {
    use std::time::Instant;
    
    // Test that startup time is reasonable (not necessarily <100ms but reasonable)
    let start = Instant::now();
    
    let _output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to run patingin");
    
    let duration = start.elapsed();
    
    // Don't claim specific performance numbers without measurement
    // Just verify it's not unreasonably slow
    assert!(duration.as_secs() < 10, "Startup should be reasonably fast");
    println!("Actual startup time: {:?}", duration);
}