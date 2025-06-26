use super::pattern::{AntiPattern, Language, Severity, DetectionMethod};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomRulesConfig {
    pub projects: HashMap<String, ProjectRules>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectRules {
    pub path: String,
    pub git_root: bool,
    pub rules: HashMap<String, Vec<CustomRule>>, // language -> rules
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomRule {
    pub id: String,
    pub description: String,
    pub pattern: String,
    pub severity: String,
    pub fix: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

pub struct CustomRulesManager {
    config_path: String,
}

impl CustomRulesManager {
    pub fn new() -> Self {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_path = format!("{}/.config/patingin/rules.yml", home_dir);
        Self { config_path }
    }

    #[allow(dead_code)] // Used in registry.rs and tests
    pub fn with_config_path(config_path: String) -> Self {
        Self { config_path }
    }

    pub fn load_config(&self) -> Result<CustomRulesConfig> {
        if !Path::new(&self.config_path).exists() {
            return Ok(CustomRulesConfig {
                projects: HashMap::new(),
            });
        }

        let content = fs::read_to_string(&self.config_path)?;
        let config: CustomRulesConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_config(&self, config: &CustomRulesConfig) -> Result<()> {
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(&self.config_path).parent() {
            fs::create_dir_all(parent)?;
        }

        let yaml_content = serde_yaml::to_string(config)?;
        fs::write(&self.config_path, yaml_content)?;
        Ok(())
    }

    pub fn add_project_rule(
        &self,
        project_name: &str,
        project_path: &str,
        language: Language,
        rule: CustomRule,
    ) -> Result<()> {
        let mut config = self.load_config()?;
        
        let project_rules = config.projects.entry(project_name.to_string()).or_insert(ProjectRules {
            path: project_path.to_string(),
            git_root: true,
            rules: HashMap::new(),
        });

        let language_key = language.to_string().to_lowercase();
        let rules_for_language = project_rules.rules.entry(language_key).or_insert(Vec::new());
        rules_for_language.push(rule);

        self.save_config(&config)?;
        Ok(())
    }

    pub fn get_project_rules(&self, project_name: &str) -> Result<Vec<AntiPattern>> {
        let config = self.load_config()?;
        let mut patterns = Vec::new();

        if let Some(project_rules) = config.projects.get(project_name) {
            for (language_str, custom_rules) in &project_rules.rules {
                let language = match language_str.as_str() {
                    "elixir" => Language::Elixir,
                    "javascript" => Language::JavaScript,
                    "typescript" => Language::TypeScript,
                    "python" => Language::Python,
                    "rust" => Language::Rust,
                    "zig" => Language::Zig,
                    "sql" => Language::Sql,
                    _ => continue,
                };

                for custom_rule in custom_rules {
                    if custom_rule.enabled {
                        let severity = match custom_rule.severity.as_str() {
                            "critical" => Severity::Critical,
                            "major" => Severity::Major,
                            "warning" => Severity::Warning,
                            _ => Severity::Warning,
                        };

                        let pattern = AntiPattern {
                            id: format!("custom_{}", custom_rule.id),
                            name: custom_rule.description.clone(),
                            language: language.clone(),
                            severity,
                            description: custom_rule.description.clone(),
                            detection_method: DetectionMethod::Regex {
                                pattern: custom_rule.pattern.clone(),
                            },
                            fix_suggestion: custom_rule.fix.clone(),
                            source_url: Some("Custom project rule".to_string()),
                            claude_code_fixable: false,
                            examples: vec![],
                            tags: vec!["custom".to_string()],
                            enabled: true,
                        };
                        patterns.push(pattern);
                    }
                }
            }
        }

        Ok(patterns)
    }

    pub fn remove_project_rule(&self, project_name: &str, rule_id: &str) -> Result<bool> {
        let mut config = self.load_config()?;
        let mut found = false;

        if let Some(project_rules) = config.projects.get_mut(project_name) {
            for rules_for_language in project_rules.rules.values_mut() {
                rules_for_language.retain(|rule| {
                    if rule.id == rule_id {
                        found = true;
                        false
                    } else {
                        true
                    }
                });
            }
        }

        if found {
            self.save_config(&config)?;
        }

        Ok(found)
    }
}

#[cfg(test)]
mod custom_rules_tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_config() -> (TempDir, CustomRulesManager) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_rules.yml").to_string_lossy().to_string();
        let manager = CustomRulesManager::with_config_path(config_path);
        (temp_dir, manager)
    }

    #[test]
    fn test_empty_config_loading() {
        let (_temp_dir, manager) = setup_test_config();
        let config = manager.load_config().unwrap();
        assert_eq!(config.projects.len(), 0);
    }

    #[test]
    fn test_add_project_rule() {
        let (_temp_dir, manager) = setup_test_config();
        
        let custom_rule = CustomRule {
            id: "no_console_log".to_string(),
            description: "Avoid console.log in production".to_string(),
            pattern: r"console\.log\(".to_string(),
            severity: "warning".to_string(),
            fix: "Use proper logging library".to_string(),
            enabled: true,
        };

        manager.add_project_rule(
            "my-app",
            "/home/user/my-app",
            Language::JavaScript,
            custom_rule,
        ).unwrap();

        let config = manager.load_config().unwrap();
        assert_eq!(config.projects.len(), 1);
        assert!(config.projects.contains_key("my-app"));
        
        let project = &config.projects["my-app"];
        assert_eq!(project.path, "/home/user/my-app");
        assert!(project.git_root);
        assert_eq!(project.rules["javascript"].len(), 1);
        assert_eq!(project.rules["javascript"][0].id, "no_console_log");
    }

    #[test]
    fn test_get_project_rules() {
        let (_temp_dir, manager) = setup_test_config();
        
        // Add multiple rules for different languages
        let js_rule = CustomRule {
            id: "no_console_log".to_string(),
            description: "Avoid console.log in production".to_string(),
            pattern: r"console\.log\(".to_string(),
            severity: "warning".to_string(),
            fix: "Use proper logging library".to_string(),
            enabled: true,
        };

        let elixir_rule = CustomRule {
            id: "team_genserver".to_string(),
            description: "Use team GenServer pattern".to_string(),
            pattern: r"GenServer\.call.*:sync".to_string(),
            severity: "major".to_string(),
            fix: "Use async GenServer.cast".to_string(),
            enabled: true,
        };

        manager.add_project_rule("my-app", "/home/user/my-app", Language::JavaScript, js_rule).unwrap();
        manager.add_project_rule("my-app", "/home/user/my-app", Language::Elixir, elixir_rule).unwrap();

        let patterns = manager.get_project_rules("my-app").unwrap();
        assert_eq!(patterns.len(), 2);

        // Check JavaScript rule
        let js_pattern = patterns.iter().find(|p| p.language == Language::JavaScript).unwrap();
        assert_eq!(js_pattern.id, "custom_no_console_log");
        assert_eq!(js_pattern.severity, Severity::Warning);
        assert!(js_pattern.tags.contains(&"custom".to_string()));

        // Check Elixir rule
        let elixir_pattern = patterns.iter().find(|p| p.language == Language::Elixir).unwrap();
        assert_eq!(elixir_pattern.id, "custom_team_genserver");
        assert_eq!(elixir_pattern.severity, Severity::Major);
    }

    #[test]
    fn test_remove_project_rule() {
        let (_temp_dir, manager) = setup_test_config();
        
        let custom_rule = CustomRule {
            id: "test_rule".to_string(),
            description: "Test rule".to_string(),
            pattern: "test".to_string(),
            severity: "warning".to_string(),
            fix: "Fix test".to_string(),
            enabled: true,
        };

        manager.add_project_rule("my-app", "/path", Language::JavaScript, custom_rule).unwrap();
        
        // Verify rule exists
        let patterns = manager.get_project_rules("my-app").unwrap();
        assert_eq!(patterns.len(), 1);

        // Remove rule
        let removed = manager.remove_project_rule("my-app", "test_rule").unwrap();
        assert!(removed);

        // Verify rule is gone
        let patterns = manager.get_project_rules("my-app").unwrap();
        assert_eq!(patterns.len(), 0);

        // Try to remove non-existent rule
        let removed = manager.remove_project_rule("my-app", "non_existent").unwrap();
        assert!(!removed);
    }

    #[test]
    fn test_disabled_rules_not_loaded() {
        let (_temp_dir, manager) = setup_test_config();
        
        let disabled_rule = CustomRule {
            id: "disabled_rule".to_string(),
            description: "This rule is disabled".to_string(),
            pattern: "disabled".to_string(),
            severity: "warning".to_string(),
            fix: "Should not appear".to_string(),
            enabled: false,
        };

        manager.add_project_rule("my-app", "/path", Language::JavaScript, disabled_rule).unwrap();
        
        let patterns = manager.get_project_rules("my-app").unwrap();
        assert_eq!(patterns.len(), 0); // Disabled rule should not be loaded
    }

    #[test]
    fn test_config_persistence() {
        let (_temp_dir, manager) = setup_test_config();
        
        let custom_rule = CustomRule {
            id: "persistent_rule".to_string(),
            description: "This rule should persist".to_string(),
            pattern: "persist".to_string(),
            severity: "major".to_string(),
            fix: "Should be saved".to_string(),
            enabled: true,
        };

        manager.add_project_rule("test-project", "/test/path", Language::Python, custom_rule).unwrap();

        // Create new manager instance with same config path
        let manager2 = CustomRulesManager::with_config_path(manager.config_path.clone());
        let patterns = manager2.get_project_rules("test-project").unwrap();
        
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].id, "custom_persistent_rule");
        assert_eq!(patterns[0].language, Language::Python);
        assert_eq!(patterns[0].severity, Severity::Major);
    }
}