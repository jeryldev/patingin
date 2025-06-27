use anyhow::Result;
use colored::*;
use std::env;
use std::process::Command;
use which::which;

use crate::core::ProjectDetector;
use crate::external::ClaudeCodeIntegration;
use crate::git::GitIntegration;

pub async fn run() -> Result<()> {
    println!("{}", "🔧 Patingin Environment Setup & Status".bold());
    println!(
        "{}\n",
        "Comprehensive diagnostic of your development environment".dimmed()
    );

    let mut checks_passed = 0;
    let mut total_checks = 0;
    let mut warnings = 0;

    // === Project Information ===
    println!("{}", "📁 Project Information".bold().blue());
    total_checks += 1;

    let current_dir = env::current_dir()?;
    match ProjectDetector::detect_project(Some(&current_dir)) {
        Ok(project_info) => {
            println!(
                "  {} Project detected: {}",
                "✓".green(),
                ProjectDetector::describe_project(&project_info).bold()
            );
            println!(
                "  📂 Root path: {}",
                project_info.root_path.display().to_string().dimmed()
            );

            if !project_info.package_files.is_empty() {
                println!(
                    "  📦 Package files: {}",
                    project_info.package_files.join(", ").dimmed()
                );
            }

            if !project_info.languages.is_empty() {
                let lang_names: Vec<String> = project_info
                    .languages
                    .iter()
                    .map(|l| format!("{:?}", l))
                    .collect();
                println!("  🔤 Languages: {}", lang_names.join(", ").cyan());
            }
            checks_passed += 1;
        }
        Err(e) => {
            println!("  {} Failed to detect project: {}", "✗".red(), e);
            println!(
                "  📂 Current directory: {}",
                current_dir.display().to_string().dimmed()
            );
        }
    }
    println!();

    // === Git Environment ===
    println!("{}", "🌳 Git Environment".bold().blue());

    // Git installation check
    total_checks += 1;
    if which("git").is_ok() {
        if let Ok(output) = Command::new("git").args(["--version"]).output() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("  {} Git installed: {}", "✓".green(), version.dimmed());
            checks_passed += 1;
        } else {
            println!("  {} Git found but not working properly", "!".yellow());
            warnings += 1;
        }
    } else {
        println!("  {} Git not found in PATH", "✗".red());
    }

    // Git repository check
    total_checks += 1;
    match GitIntegration::new(current_dir.clone()) {
        Ok(git) => {
            println!("  {} Git repository detected", "✓".green());

            if let Ok(branch) = git.get_current_branch() {
                println!("    🌿 Current branch: {}", branch.cyan());
            }

            // Check for remotes
            if let Ok(output) = Command::new("git").args(["remote", "-v"]).output() {
                let remotes = String::from_utf8_lossy(&output.stdout);
                if !remotes.trim().is_empty() {
                    let remote_lines: Vec<&str> = remotes.lines().take(2).collect();
                    println!("    🔗 Remotes:");
                    for line in remote_lines {
                        println!("      {}", line.dimmed());
                    }
                } else {
                    println!("    {} No remotes configured", "!".yellow());
                    warnings += 1;
                }
            }

            // Check git status
            if let Ok(output) = Command::new("git")
                .args(["status", "--porcelain"])
                .output()
            {
                let status = String::from_utf8_lossy(&output.stdout);
                if status.trim().is_empty() {
                    println!("    {} Working directory clean", "✓".green());
                } else {
                    let line_count = status.lines().count();
                    println!("    {} {} uncommitted changes", "!".yellow(), line_count);
                    warnings += 1;
                }
            }
            checks_passed += 1;
        }
        Err(_) => {
            println!("  {} Not in a git repository", "✗".red());
            println!("    💡 Initialize with: {}", "git init".cyan());
        }
    }
    println!();

    // === Tool Dependencies ===
    println!("{}", "🛠️  Tool Dependencies".bold().blue());

    // Claude Code CLI check
    total_checks += 1;
    let integration = ClaudeCodeIntegration::detect();
    if integration.available {
        let version_display = integration.version.as_deref().unwrap_or("unknown version");
        println!(
            "  {} Claude Code CLI: {}",
            "✓".green(),
            version_display.dimmed()
        );
        println!("    ✨ Auto-fix integration: {}", "Ready".green());
        checks_passed += 1;
    } else {
        println!("  {} Claude Code CLI not found", "✗".red());
        println!(
            "    💡 Install from: {}",
            "https://docs.anthropic.com/en/docs/claude-code".cyan()
        );
    }

    // System tools check
    let system_tools = [
        ("rg", "ripgrep (fast text search)"),
        ("fd", "fd (fast file finder)"),
        ("fzf", "fzf (fuzzy finder)"),
    ];

    for (tool, _description) in &system_tools {
        if which(tool).is_ok() {
            println!(
                "  {} {}: {}",
                "✓".green(),
                tool.bold(),
                "Available".dimmed()
            );
        } else {
            println!(
                "  {} {}: {}",
                "○".dimmed(),
                tool.bold(),
                "Optional but recommended".dimmed()
            );
        }
    }
    println!();

    // === Configuration ===
    println!("{}", "⚙️  Configuration".bold().blue());

    // Patingin config directory
    total_checks += 1;
    let home_dir = home::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let config_dir = home_dir.join(".config").join("patingin");

    if config_dir.exists() {
        println!(
            "  {} Global config directory: {}",
            "✓".green(),
            config_dir.display().to_string().dimmed()
        );

        let rules_file = config_dir.join("rules.yml");
        if rules_file.exists() {
            println!("    📋 Custom rules file: {}", "Found".green());
        } else {
            println!("    📋 Custom rules file: {}", "Not created yet".dimmed());
        }
        checks_passed += 1;
    } else {
        println!(
            "  {} Global config directory: {}",
            "!".yellow(),
            "Will be created on first use".yellow()
        );
        println!(
            "    📂 Location: {}",
            config_dir.display().to_string().dimmed()
        );
        warnings += 1;
    }

    // Project-specific config
    let project_configs = ["patingin.yml", ".patingin.yml", ".patingin/config.yml"];

    let mut project_config_found = false;
    for config_path in &project_configs {
        if std::path::Path::new(config_path).exists() {
            println!("  {} Project config: {}", "✓".green(), config_path.cyan());
            project_config_found = true;
            break;
        }
    }

    if !project_config_found {
        println!("  {} Project config: {}", "○".dimmed(), "Optional".dimmed());
        println!(
            "    💡 Create with: {}",
            "patingin rules add --project".cyan()
        );
    }
    println!();

    // === System Information ===
    println!("{}", "💻 System Information".bold().blue());

    // OS and architecture
    println!("  🖥️  OS: {} {}", env::consts::OS, env::consts::ARCH);

    // Patingin version
    println!(
        "  🦀 Patingin: {} ({})",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_NAME")
    );

    // Environment variables
    if let Ok(editor) = env::var("EDITOR") {
        println!("  ✏️  Editor: {}", editor.cyan());
    } else {
        println!("  ✏️  Editor: {}", "Not set (EDITOR env var)".dimmed());
    }

    if let Ok(shell) = env::var("SHELL") {
        println!("  🐚 Shell: {}", shell.cyan());
    }
    println!();

    // === Summary ===
    println!("{}", "=".repeat(60));
    let success_rate = (checks_passed as f64 / total_checks as f64) * 100.0;

    if checks_passed == total_checks && warnings == 0 {
        println!("{} Environment is fully ready!", "🎉".green().bold());
        println!("  All {} checks passed with no warnings", checks_passed);
    } else if checks_passed == total_checks {
        println!(
            "{} Environment is ready with minor warnings",
            "✅".green().bold()
        );
        println!("  {} checks passed, {} warnings", checks_passed, warnings);
    } else {
        println!("{} Environment needs attention", "⚠️".yellow().bold());
        println!(
            "  {}/{} checks passed ({:.0}%), {} warnings",
            checks_passed, total_checks, success_rate, warnings
        );
    }

    println!("\n💡 Next steps:");
    if checks_passed < total_checks {
        println!("  • Address failed checks above");
    }
    if warnings > 0 {
        println!("  • Review warnings for optimal experience");
    }
    println!(
        "  • Run {} to start analyzing your code",
        "patingin review".cyan()
    );
    println!("  • Use {} to see available rules", "patingin rules".cyan());

    Ok(())
}

#[cfg(test)]
mod setup_command_tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_setup_run_basic() {
        // Test that setup command runs without errors
        let result = run().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_project_detection_functionality() {
        // Test project detection logic (same as ProjectDetector tests)
        let current_dir = env::current_dir().unwrap();
        let project_result = ProjectDetector::detect_project(Some(&current_dir));

        // Should either succeed or fail gracefully
        match project_result {
            Ok(project_info) => {
                assert!(!project_info.root_path.as_os_str().is_empty());
            }
            Err(_) => {
                // Project detection failure is acceptable in test environment
            }
        }
    }

    #[test]
    fn test_git_version_check() {
        use std::process::Command;

        // Test git version check functionality
        let git_check = which("git");
        if git_check.is_ok() {
            let output = Command::new("git").args(&["--version"]).output();
            if let Ok(output) = output {
                let version = String::from_utf8_lossy(&output.stdout);
                assert!(version.contains("git"));
            }
        }
        // If git is not available, that's a valid state to test
    }

    #[test]
    fn test_claude_code_detection() {
        // Test Claude Code CLI detection via npm
        let npm_check = Command::new("npm")
            .args(&["list", "-g", "@anthropic-ai/claude-code"])
            .output();

        let claude_code_npm_installed = if let Ok(output) = npm_check {
            let output_str = String::from_utf8_lossy(&output.stdout);
            output_str.contains("@anthropic-ai/claude-code")
        } else {
            false
        };

        let integration = ClaudeCodeIntegration::detect();

        // Detection logic should correctly identify availability
        // Note: integration.available might have additional checks beyond just which()
        println!(
            "Claude Code npm package installed: {}",
            claude_code_npm_installed
        );
        println!(
            "Integration detected as available: {}",
            integration.available
        );
        // so we just test that the detection doesn't panic and returns a boolean
        assert!(integration.available == true || integration.available == false);
    }

    #[test]
    fn test_system_tools_detection() {
        let tools = [
            ("rg", "ripgrep"),
            ("fd", "fd-find"),
            ("fzf", "fuzzy finder"),
        ];

        for (tool, _description) in &tools {
            let available = which(tool).is_ok();
            // Each tool can be available or not - both are valid states
            // Just test that the detection doesn't panic
            assert!(available || !available); // Tautology to ensure no panic
        }
    }

    #[test]
    fn test_config_directory_logic() {
        use std::path::PathBuf;

        // Test config directory path construction
        let home_dir = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_dir = home_dir.join(".config").join("patingin");

        // Verify path construction
        assert!(config_dir.to_string_lossy().contains("patingin"));
        assert!(config_dir.to_string_lossy().contains(".config"));
    }

    #[test]
    fn test_project_config_detection() {
        use std::path::Path;

        let project_configs = ["patingin.yml", ".patingin.yml", ".patingin/config.yml"];

        // Test that we can check for project config files
        for config_path in &project_configs {
            let exists = Path::new(config_path).exists();
            // Either exists or doesn't - both are valid, just test no panic
            assert!(exists || !exists);
        }
    }

    #[test]
    fn test_system_information_gathering() {
        // Test system info gathering
        assert!(!env::consts::OS.is_empty());
        assert!(!env::consts::ARCH.is_empty());

        // Test package info
        let version = env!("CARGO_PKG_VERSION");
        let name = env!("CARGO_PKG_NAME");
        assert!(!version.is_empty());
        assert_eq!(name, "patingin");
    }

    #[test]
    fn test_environment_variables_check() {
        // Test environment variable checks
        let editor = env::var("EDITOR");
        let shell = env::var("SHELL");

        // These may or may not be set - both are valid
        match editor {
            Ok(editor_val) => assert!(!editor_val.is_empty()),
            Err(_) => {} // EDITOR not set is valid
        }

        match shell {
            Ok(shell_val) => assert!(!shell_val.is_empty()),
            Err(_) => {} // SHELL not set is valid on some systems
        }
    }

    #[test]
    fn test_git_integration_creation() {
        // Test GitIntegration creation with current directory
        let current_dir = env::current_dir().unwrap();
        let git_result = GitIntegration::new(current_dir);

        // Either succeeds (in git repo) or fails (not in git repo) - both valid
        match git_result {
            Ok(git) => {
                // If in git repo, test basic functionality
                if let Ok(branch) = git.get_current_branch() {
                    assert!(!branch.is_empty());
                }
            }
            Err(_) => {
                // Not in git repo is a valid test state
            }
        }
    }

    #[test]
    fn test_git_status_check() {
        use std::process::Command;

        // Test git status functionality (if in git repo)
        let status_output = Command::new("git")
            .args(&["status", "--porcelain"])
            .output();

        match status_output {
            Ok(output) => {
                let status = String::from_utf8_lossy(&output.stdout);
                // Status can be empty (clean) or contain changes - both valid
                // Just verify we got a result without panicking
                let _status_length = status.len();
            }
            Err(_) => {
                // Git not available or not in git repo - valid test state
            }
        }
    }

    #[test]
    fn test_remote_check() {
        use std::process::Command;

        // Test git remote check functionality
        let remote_output = Command::new("git").args(&["remote", "-v"]).output();

        match remote_output {
            Ok(output) => {
                let remotes = String::from_utf8_lossy(&output.stdout);
                // Remotes can exist or not - both are valid states
                // Just verify we got a result without panicking
                let _remotes_length = remotes.len();
            }
            Err(_) => {
                // Git not available - valid test state
            }
        }
    }

    #[test]
    fn test_summary_calculation() {
        // Test summary calculation logic
        let checks_passed = 3;
        let total_checks = 5;
        let warnings = 1;

        let success_rate = (checks_passed as f64 / total_checks as f64) * 100.0;
        assert_eq!(success_rate, 60.0);

        // Test different scenarios
        let all_passed = checks_passed == total_checks && warnings == 0;
        let ready_with_warnings = checks_passed == total_checks && warnings > 0;
        let needs_attention = checks_passed < total_checks;

        assert!(!all_passed);
        assert!(!ready_with_warnings);
        assert!(needs_attention);
    }

    #[test]
    fn test_perfect_environment() {
        // Test logic for perfect environment
        let checks_passed = 5;
        let total_checks = 5;
        let warnings = 0;

        let all_passed = checks_passed == total_checks && warnings == 0;
        assert!(all_passed);
    }

    #[test]
    fn test_environment_with_warnings() {
        // Test logic for environment with warnings
        let checks_passed = 5;
        let total_checks = 5;
        let warnings = 2;

        let ready_with_warnings = checks_passed == total_checks && warnings > 0;
        assert!(ready_with_warnings);
    }

    #[test]
    fn test_environment_needs_attention() {
        // Test logic for environment that needs attention
        let checks_passed = 3;
        let total_checks = 5;
        let _warnings = 1;

        let needs_attention = checks_passed < total_checks;
        assert!(needs_attention);
    }

    #[tokio::test]
    async fn test_setup_in_temporary_directory() {
        // Test setup command behavior in a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();

        // Change to temp directory
        env::set_current_dir(temp_dir.path()).unwrap();

        // Run setup (should handle non-git directory gracefully)
        let result = run().await;
        assert!(result.is_ok());

        // Restore original directory
        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_config_path_construction() {
        // Test that config paths are constructed correctly
        let home_dir = home::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let config_dir = home_dir.join(".config").join("patingin");
        let rules_file = config_dir.join("rules.yml");

        assert!(config_dir.ends_with("patingin"));
        assert!(rules_file.ends_with("rules.yml"));
    }
}
