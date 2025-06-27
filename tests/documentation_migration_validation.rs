use std::fs;
use std::path::Path;

/// Tests to ensure documentation migration from --auto-fix to --fix is complete
///
/// Following TDD principles, these tests validate that:
/// 1. No documentation files contain --auto-fix references
/// 2. All documentation properly uses --fix for interactive mode
/// 3. Deprecation notices are in place where needed

#[test]
fn test_readme_uses_fix_not_auto_fix() {
    let readme_content = fs::read_to_string("README.md").expect("README.md should exist");

    let auto_fix_count = readme_content.matches("--auto-fix").count();
    let fix_count = readme_content.matches("--fix").count();

    assert_eq!(
        auto_fix_count, 0,
        "README.md should not contain --auto-fix references"
    );
    assert!(
        fix_count > 0,
        "README.md should contain --fix references for interactive mode"
    );
}

#[test]
fn test_ai_integration_docs_use_fix_not_auto_fix() {
    let docs_content =
        fs::read_to_string("docs/ai-integration.md").expect("docs/ai-integration.md should exist");

    let auto_fix_count = docs_content.matches("--auto-fix").count();
    let fix_count = docs_content.matches("--fix").count();

    assert_eq!(
        auto_fix_count, 0,
        "ai-integration.md should not contain --auto-fix references"
    );
    assert!(
        fix_count > 0,
        "ai-integration.md should contain --fix references for interactive mode"
    );
}

#[test]
fn test_workflows_docs_use_fix_not_auto_fix() {
    let docs_content =
        fs::read_to_string("docs/workflows.md").expect("docs/workflows.md should exist");

    let auto_fix_count = docs_content.matches("--auto-fix").count();
    let fix_count = docs_content.matches("--fix").count();

    assert_eq!(
        auto_fix_count, 0,
        "workflows.md should not contain --auto-fix references"
    );
    assert!(
        fix_count > 0,
        "workflows.md should contain --fix references for interactive mode"
    );
}

#[test]
fn test_commands_docs_use_fix_not_auto_fix() {
    let docs_content =
        fs::read_to_string("docs/commands.md").expect("docs/commands.md should exist");

    let auto_fix_count = docs_content.matches("--auto-fix").count();
    let fix_count = docs_content.matches("--fix").count();

    assert_eq!(
        auto_fix_count, 0,
        "commands.md should not contain --auto-fix references"
    );
    assert!(
        fix_count > 0,
        "commands.md should contain --fix references for interactive mode"
    );
}

#[test]
fn test_setup_docs_use_fix_not_auto_fix() {
    let docs_content = fs::read_to_string("docs/setup.md").expect("docs/setup.md should exist");

    let auto_fix_count = docs_content.matches("--auto-fix").count();
    let fix_count = docs_content.matches("--fix").count();

    assert_eq!(
        auto_fix_count, 0,
        "setup.md should not contain --auto-fix references"
    );
    assert!(
        fix_count > 0,
        "setup.md should contain --fix references for interactive mode"
    );
}

#[test]
fn test_all_markdown_files_migration_complete() {
    let markdown_files = [
        "README.md",
        "docs/README.md",
        "docs/ai-integration.md",
        "docs/workflows.md",
        "docs/commands.md",
        "docs/setup.md",
        "docs/rules.md",
    ];

    for file_path in &markdown_files {
        if Path::new(file_path).exists() {
            let content = fs::read_to_string(file_path)
                .unwrap_or_else(|_| panic!("Should be able to read {}", file_path));

            let auto_fix_count = content.matches("--auto-fix").count();
            assert_eq!(
                auto_fix_count, 0,
                "{} should not contain --auto-fix references after migration",
                file_path
            );
        }
    }
}

#[test]
fn test_interactive_fix_examples_are_present() {
    let readme_content = fs::read_to_string("README.md").expect("README.md should exist");

    // Should contain examples of the interactive fix workflow
    assert!(
        readme_content.contains("patingin review --fix"),
        "README should contain interactive fix examples"
    );

    // Should mention Claude Code integration
    assert!(
        readme_content.contains("Claude Code") || readme_content.contains("claude"),
        "README should mention Claude Code integration"
    );
}
