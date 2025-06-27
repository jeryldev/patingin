use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::Language;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub root_path: PathBuf,
    pub languages: Vec<Language>,
    pub project_type: ProjectType,
    pub package_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    Git,
    Elixir,
    JavaScript,
    TypeScript,
    Python,
    Rust,
    Zig,
    Generic,
}

pub struct ProjectDetector;

impl ProjectDetector {
    /// Detect project information using the hierarchy: git root → package files → current directory
    pub fn detect_project(starting_path: Option<&Path>) -> Result<ProjectInfo> {
        let current_dir = starting_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        // Step 1: Try to find git root
        if let Some(git_root) = Self::find_git_root(&current_dir)? {
            let project_info = Self::analyze_project(&git_root)?;
            return Ok(project_info);
        }

        // Step 2: Try to find package files by walking up the directory tree
        if let Some(package_root) = Self::find_package_root(&current_dir)? {
            let project_info = Self::analyze_project(&package_root)?;
            return Ok(project_info);
        }

        // Step 3: Fallback to current directory analysis
        Self::analyze_project(&current_dir)
    }

    /// Find the git repository root by walking up the directory tree
    fn find_git_root(start_path: &Path) -> Result<Option<PathBuf>> {
        let mut current = start_path.to_path_buf();

        loop {
            let git_dir = current.join(".git");
            if git_dir.exists() {
                return Ok(Some(current));
            }

            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => break,
            }
        }

        Ok(None)
    }

    /// Find project root by looking for package files (mix.exs, package.json, etc.)
    fn find_package_root(start_path: &Path) -> Result<Option<PathBuf>> {
        let package_files = vec![
            "mix.exs",          // Elixir
            "package.json",     // JavaScript/TypeScript
            "pyproject.toml",   // Python
            "requirements.txt", // Python
            "Cargo.toml",       // Rust
            "build.zig",        // Zig
        ];

        let mut current = start_path.to_path_buf();

        loop {
            for package_file in &package_files {
                let package_path = current.join(package_file);
                if package_path.exists() {
                    return Ok(Some(current));
                }
            }

            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => break,
            }
        }

        Ok(None)
    }

    /// Analyze a directory to determine project information
    fn analyze_project(project_root: &Path) -> Result<ProjectInfo> {
        let project_name = Self::determine_project_name(project_root)?;
        let (languages, project_type, package_files) =
            Self::detect_languages_and_type(project_root)?;

        Ok(ProjectInfo {
            name: project_name,
            root_path: project_root.to_path_buf(),
            languages,
            project_type,
            package_files,
        })
    }

    /// Determine project name from directory name or package files
    fn determine_project_name(project_root: &Path) -> Result<String> {
        // Try to get name from package files first
        if let Ok(name) = Self::get_name_from_package_json(project_root) {
            return Ok(name);
        }

        if let Ok(name) = Self::get_name_from_mix_exs(project_root) {
            return Ok(name);
        }

        if let Ok(name) = Self::get_name_from_cargo_toml(project_root) {
            return Ok(name);
        }

        // Fallback to directory name
        Ok(project_root.file_name().and_then(|name| name.to_str()).unwrap_or("unknown").to_string())
    }

    /// Get project name from package.json
    fn get_name_from_package_json(project_root: &Path) -> Result<String> {
        let package_json_path = project_root.join("package.json");
        if !package_json_path.exists() {
            return Err(anyhow::anyhow!("package.json not found"));
        }

        let content =
            fs::read_to_string(&package_json_path).context("Failed to read package.json")?;

        let package_data: serde_json::Value =
            serde_json::from_str(&content).context("Failed to parse package.json")?;

        package_data["name"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No name field in package.json"))
    }

    /// Get project name from mix.exs
    fn get_name_from_mix_exs(project_root: &Path) -> Result<String> {
        let mix_exs_path = project_root.join("mix.exs");
        if !mix_exs_path.exists() {
            return Err(anyhow::anyhow!("mix.exs not found"));
        }

        let content = fs::read_to_string(&mix_exs_path).context("Failed to read mix.exs")?;

        // Simple regex-based extraction (could be improved with proper Elixir parsing)
        if let Some(caps) = regex::Regex::new(r#"app:\s*:(\w+)"#).unwrap().captures(&content) {
            Ok(caps[1].to_string())
        } else {
            Err(anyhow::anyhow!("Could not extract app name from mix.exs"))
        }
    }

    /// Get project name from Cargo.toml
    fn get_name_from_cargo_toml(project_root: &Path) -> Result<String> {
        let cargo_toml_path = project_root.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Err(anyhow::anyhow!("Cargo.toml not found"));
        }

        let content = fs::read_to_string(&cargo_toml_path).context("Failed to read Cargo.toml")?;

        let cargo_data: toml::Value = content.parse().context("Failed to parse Cargo.toml")?;

        cargo_data["package"]["name"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No package.name in Cargo.toml"))
    }

    /// Detect languages and project type from package files and directory structure
    fn detect_languages_and_type(
        project_root: &Path,
    ) -> Result<(Vec<Language>, ProjectType, Vec<String>)> {
        let mut languages = Vec::new();
        let mut package_files = Vec::new();
        let mut project_type = ProjectType::Generic;

        // Check for specific package files
        let package_checks = vec![
            ("mix.exs", Language::Elixir, ProjectType::Elixir),
            ("package.json", Language::JavaScript, ProjectType::JavaScript),
            ("tsconfig.json", Language::TypeScript, ProjectType::TypeScript),
            ("pyproject.toml", Language::Python, ProjectType::Python),
            ("requirements.txt", Language::Python, ProjectType::Python),
            ("Cargo.toml", Language::Rust, ProjectType::Rust),
            ("build.zig", Language::Zig, ProjectType::Zig),
        ];

        for (file_name, language, proj_type) in package_checks {
            let file_path = project_root.join(file_name);
            if file_path.exists() {
                if !languages.contains(&language) {
                    languages.push(language);
                }
                package_files.push(file_name.to_string());

                // Set project type to the first detected type
                if matches!(project_type, ProjectType::Generic) {
                    project_type = proj_type;
                }
            }
        }

        // Check if it's a git repository
        if project_root.join(".git").exists() && matches!(project_type, ProjectType::Generic) {
            project_type = ProjectType::Git;
        }

        // If no specific languages detected, scan for file extensions
        if languages.is_empty() {
            languages = Self::detect_languages_from_files(project_root)?;
        }

        Ok((languages, project_type, package_files))
    }

    /// Detect languages by scanning file extensions in the project
    fn detect_languages_from_files(project_root: &Path) -> Result<Vec<Language>> {
        let mut languages = Vec::new();

        let extension_map = vec![
            (vec!["ex", "exs"], Language::Elixir),
            (vec!["js", "jsx", "mjs", "cjs"], Language::JavaScript),
            (vec!["ts", "tsx"], Language::TypeScript),
            (vec!["py", "pyw", "pyi"], Language::Python),
            (vec!["rs"], Language::Rust),
            (vec!["zig"], Language::Zig),
            (vec!["sql", "psql", "mysql"], Language::Sql),
        ];

        // Walk through directory and collect extensions
        if let Ok(entries) = fs::read_dir(project_root) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(extension) = entry.path().extension() {
                            if let Some(ext_str) = extension.to_str() {
                                for (extensions, language) in &extension_map {
                                    if extensions.contains(&ext_str.to_lowercase().as_str())
                                        && !languages.contains(language)
                                    {
                                        languages.push(language.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(languages)
    }

    /// Get a human-readable description of the project
    pub fn describe_project(project_info: &ProjectInfo) -> String {
        let lang_list = if project_info.languages.is_empty() {
            "unknown".to_string()
        } else {
            project_info
                .languages
                .iter()
                .map(|l| format!("{l:?}").to_lowercase())
                .collect::<Vec<_>>()
                .join(", ")
        };

        format!(
            "{} ({:?} project with {})",
            project_info.name, project_info.project_type, lang_list
        )
    }

    /// Check if the project uses a specific language
    #[allow(dead_code)] // Used in tests and will be used for language filtering
    pub fn project_uses_language(project_info: &ProjectInfo, language: &Language) -> bool {
        project_info.languages.contains(language)
    }
}

#[cfg(test)]
mod project_detector_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_elixir_project() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let project_root = temp_dir.path();

        // Create mix.exs file
        fs::write(
            project_root.join("mix.exs"),
            r#"defmodule MyApp.MixProject do
                 use Mix.Project
                 def project do
                   [
                     app: :my_app,
                     version: "0.1.0"
                   ]
                 end
               end"#,
        )
        .expect("Should write mix.exs");

        let project_info =
            ProjectDetector::analyze_project(project_root).expect("Should detect Elixir project");

        assert_eq!(project_info.name, "my_app");
        assert!(project_info.languages.contains(&Language::Elixir));
        assert!(matches!(project_info.project_type, ProjectType::Elixir));
        assert!(project_info.package_files.contains(&"mix.exs".to_string()));
    }

    #[test]
    fn test_detect_javascript_project() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let project_root = temp_dir.path();

        // Create package.json file
        fs::write(
            project_root.join("package.json"),
            r#"{
                 "name": "my-js-project",
                 "version": "1.0.0",
                 "dependencies": {}
               }"#,
        )
        .expect("Should write package.json");

        let project_info = ProjectDetector::analyze_project(project_root)
            .expect("Should detect JavaScript project");

        assert_eq!(project_info.name, "my-js-project");
        assert!(project_info.languages.contains(&Language::JavaScript));
        assert!(matches!(project_info.project_type, ProjectType::JavaScript));
        assert!(project_info.package_files.contains(&"package.json".to_string()));
    }

    #[test]
    fn test_detect_rust_project() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let project_root = temp_dir.path();

        // Create Cargo.toml file
        fs::write(
            project_root.join("Cargo.toml"),
            r#"[package]
               name = "my-rust-project"
               version = "0.1.0"
               edition = "2021""#,
        )
        .expect("Should write Cargo.toml");

        let project_info =
            ProjectDetector::analyze_project(project_root).expect("Should detect Rust project");

        assert_eq!(project_info.name, "my-rust-project");
        assert!(project_info.languages.contains(&Language::Rust));
        assert!(matches!(project_info.project_type, ProjectType::Rust));
        assert!(project_info.package_files.contains(&"Cargo.toml".to_string()));
    }

    #[test]
    fn test_git_root_detection() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let project_root = temp_dir.path();

        // Create .git directory
        fs::create_dir(project_root.join(".git")).expect("Should create .git dir");

        // Create nested directory
        let nested_dir = project_root.join("src").join("lib");
        fs::create_dir_all(&nested_dir).expect("Should create nested dirs");

        let git_root = ProjectDetector::find_git_root(&nested_dir)
            .expect("Should find git root")
            .expect("Should return Some git root");

        assert_eq!(git_root, project_root);
    }

    #[test]
    fn test_package_root_detection() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let project_root = temp_dir.path();

        // Create mix.exs at root
        fs::write(project_root.join("mix.exs"), "").expect("Should write mix.exs");

        // Create nested directory
        let nested_dir = project_root.join("lib").join("my_app");
        fs::create_dir_all(&nested_dir).expect("Should create nested dirs");

        let package_root = ProjectDetector::find_package_root(&nested_dir)
            .expect("Should find package root")
            .expect("Should return Some package root");

        assert_eq!(package_root, project_root);
    }

    #[test]
    fn test_multi_language_project() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let project_root = temp_dir.path();

        // Create both package.json and mix.exs
        fs::write(project_root.join("package.json"), r#"{"name": "multi-lang"}"#)
            .expect("Should write package.json");
        fs::write(project_root.join("mix.exs"), "").expect("Should write mix.exs");

        let project_info = ProjectDetector::analyze_project(project_root)
            .expect("Should detect multi-language project");

        assert_eq!(project_info.name, "multi-lang");
        assert!(project_info.languages.contains(&Language::JavaScript));
        assert!(project_info.languages.contains(&Language::Elixir));
        assert!(project_info.package_files.len() >= 2);
    }

    #[test]
    fn test_fallback_to_directory_name() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let project_root = temp_dir.path();

        // Create a directory with no package files
        let custom_dir = project_root.join("my-custom-project");
        fs::create_dir(&custom_dir).expect("Should create custom dir");

        let project_info =
            ProjectDetector::analyze_project(&custom_dir).expect("Should analyze generic project");

        assert_eq!(project_info.name, "my-custom-project");
        assert!(matches!(project_info.project_type, ProjectType::Generic));
    }

    #[test]
    fn test_project_uses_language() {
        let project_info = ProjectInfo {
            name: "test".to_string(),
            root_path: PathBuf::from("/test"),
            languages: vec![Language::Elixir, Language::JavaScript],
            project_type: ProjectType::Elixir,
            package_files: vec!["mix.exs".to_string()],
        };

        assert!(ProjectDetector::project_uses_language(&project_info, &Language::Elixir));
        assert!(ProjectDetector::project_uses_language(&project_info, &Language::JavaScript));
        assert!(!ProjectDetector::project_uses_language(&project_info, &Language::Python));
    }

    #[test]
    fn test_describe_project() {
        let project_info = ProjectInfo {
            name: "my-app".to_string(),
            root_path: PathBuf::from("/path/to/my-app"),
            languages: vec![Language::Elixir],
            project_type: ProjectType::Elixir,
            package_files: vec!["mix.exs".to_string()],
        };

        let description = ProjectDetector::describe_project(&project_info);
        assert!(description.contains("my-app"));
        assert!(description.contains("Elixir"));
        assert!(description.contains("elixir"));
    }

    #[test]
    fn test_detect_current_rust_project() {
        // Test with current project directory (should detect this as a Rust project if Cargo.toml exists)
        let current_dir = std::env::current_dir().expect("Should get current dir");

        // Check if we're in a directory with Cargo.toml
        let cargo_toml = current_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(project_info) = ProjectDetector::detect_project(Some(&current_dir)) {
                // Don't assert specific name since tests might run in temp directories
                assert!(!project_info.name.is_empty(), "Project should have a name");
                assert!(project_info.languages.contains(&Language::Rust));
                assert!(matches!(project_info.project_type, ProjectType::Rust));
                assert!(project_info.package_files.contains(&"Cargo.toml".to_string()));

                println!("Detected project: {} in {}", project_info.name, current_dir.display());
                println!("Description: {}", ProjectDetector::describe_project(&project_info));
            } else {
                panic!("Should detect Rust project when Cargo.toml exists");
            }
        } else {
            // Skip test if not in a Rust project directory (e.g., in test temp dir)
            println!("Skipping test - no Cargo.toml found in {}", current_dir.display());
        }
    }
}
