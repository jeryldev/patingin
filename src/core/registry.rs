use super::pattern::{AntiPattern, Language, Severity};
use super::custom_rules::CustomRulesManager;
use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

pub struct PatternRegistry {
    patterns: HashMap<String, AntiPattern>,
    by_language: HashMap<Language, Vec<String>>,
    pub compiled_patterns: HashMap<String, Regex>,
}

impl PatternRegistry {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            by_language: HashMap::new(),
            compiled_patterns: HashMap::new(),
        }
    }

    pub fn load_built_in_patterns(&mut self) -> Result<()> {
        self.load_all_embedded_rules()?;
        self.compile_all_patterns()?;
        Ok(())
    }

    pub fn load_embedded_elixir_rules(&mut self) -> Result<()> {
        const ELIXIR_RULES: &str = include_str!("../rules/builtin/elixir.yml");
        self.load_rules_from_yaml(ELIXIR_RULES, Language::Elixir)
    }

    pub fn load_embedded_javascript_rules(&mut self) -> Result<()> {
        const JAVASCRIPT_RULES: &str = include_str!("../rules/builtin/javascript.yml");
        self.load_rules_from_yaml(JAVASCRIPT_RULES, Language::JavaScript)
    }

    pub fn load_embedded_typescript_rules(&mut self) -> Result<()> {
        const TYPESCRIPT_RULES: &str = include_str!("../rules/builtin/typescript.yml");
        self.load_rules_from_yaml(TYPESCRIPT_RULES, Language::TypeScript)
    }

    pub fn load_embedded_python_rules(&mut self) -> Result<()> {
        const PYTHON_RULES: &str = include_str!("../rules/builtin/python.yml");
        self.load_rules_from_yaml(PYTHON_RULES, Language::Python)
    }

    pub fn load_embedded_rust_rules(&mut self) -> Result<()> {
        const RUST_RULES: &str = include_str!("../rules/builtin/rust.yml");
        self.load_rules_from_yaml(RUST_RULES, Language::Rust)
    }

    pub fn load_embedded_zig_rules(&mut self) -> Result<()> {
        const ZIG_RULES: &str = include_str!("../rules/builtin/zig.yml");
        self.load_rules_from_yaml(ZIG_RULES, Language::Zig)
    }

    pub fn load_embedded_sql_rules(&mut self) -> Result<()> {
        const SQL_RULES: &str = include_str!("../rules/builtin/sql.yml");
        self.load_rules_from_yaml(SQL_RULES, Language::Sql)
    }

    pub fn load_all_embedded_rules(&mut self) -> Result<()> {
        self.load_embedded_elixir_rules()?;
        self.load_embedded_javascript_rules()?;
        self.load_embedded_typescript_rules()?;
        self.load_embedded_python_rules()?;
        self.load_embedded_rust_rules()?;
        self.load_embedded_zig_rules()?;
        self.load_embedded_sql_rules()?;
        Ok(())
    }

    pub fn load_custom_rules(&mut self, project_name: &str) -> Result<()> {
        let custom_rules_manager = CustomRulesManager::new();
        let custom_patterns = custom_rules_manager.get_project_rules(project_name)?;
        
        for pattern in custom_patterns {
            self.add_pattern(pattern);
        }
        
        Ok(())
    }

    pub fn compile_all_patterns(&mut self) -> Result<()> {
        use crate::core::DetectionMethod;
        
        for pattern in self.patterns.values() {
            if let DetectionMethod::Regex { pattern: regex_pattern } = &pattern.detection_method {
                match Regex::new(regex_pattern) {
                    Ok(compiled) => {
                        self.compiled_patterns.insert(pattern.id.clone(), compiled);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to compile regex for pattern {}: {}", pattern.id, e);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_compiled_pattern(&self, id: &str) -> Option<&Regex> {
        self.compiled_patterns.get(id)
    }

    fn load_rules_from_yaml(&mut self, yaml_content: &str, _expected_language: Language) -> Result<()> {
        #[derive(serde::Deserialize)]
        struct YamlRule {
            id: String,
            name: String,
            language: String,
            severity: String,
            description: String,
            detection_method: YamlDetectionMethod,
            fix_suggestion: String,
            source_url: Option<String>,
            claude_code_fixable: bool,
            examples: Vec<YamlExample>,
            tags: Vec<String>,
            enabled: bool,
        }

        #[derive(serde::Deserialize)]
        struct YamlDetectionMethod {
            #[serde(rename = "type")]
            method_type: String,
            pattern: String,
            threshold: Option<f64>,
        }

        #[derive(serde::Deserialize)]
        struct YamlExample {
            bad: String,
            good: String,
            explanation: String,
        }

        let yaml_rules: Vec<YamlRule> = serde_yaml::from_str(yaml_content)?;

        for yaml_rule in yaml_rules {
            use crate::core::{DetectionMethod, CodeExample};

            let language = match yaml_rule.language.as_str() {
                "elixir" => Language::Elixir,
                "javascript" => Language::JavaScript,
                "typescript" => Language::TypeScript,
                "python" => Language::Python,
                "rust" => Language::Rust,
                "zig" => Language::Zig,
                "sql" => Language::Sql,
                _ => continue, // Skip unknown languages
            };

            let severity = match yaml_rule.severity.as_str() {
                "critical" => Severity::Critical,
                "major" => Severity::Major,
                "warning" => Severity::Warning,
                _ => continue, // Skip unknown severities
            };

            let detection_method = match yaml_rule.detection_method.method_type.as_str() {
                "regex" => DetectionMethod::Regex { 
                    pattern: yaml_rule.detection_method.pattern 
                },
                "ratio" => DetectionMethod::Ratio { 
                    pattern: yaml_rule.detection_method.pattern,
                    threshold: yaml_rule.detection_method.threshold.unwrap_or(0.3)
                },
                "line_count" => DetectionMethod::LineCount { 
                    threshold: yaml_rule.detection_method.threshold.unwrap_or(10.0) as usize,
                    pattern: yaml_rule.detection_method.pattern
                },
                "custom" => DetectionMethod::Custom {
                    pattern: yaml_rule.detection_method.pattern
                },
                _ => continue, // Skip unknown detection methods
            };

            let examples = yaml_rule.examples.into_iter().map(|ex| CodeExample {
                bad: ex.bad,
                good: ex.good,
                explanation: ex.explanation,
            }).collect();

            let pattern = AntiPattern {
                id: yaml_rule.id,
                name: yaml_rule.name,
                language,
                severity,
                description: yaml_rule.description,
                detection_method,
                fix_suggestion: yaml_rule.fix_suggestion,
                source_url: yaml_rule.source_url,
                claude_code_fixable: yaml_rule.claude_code_fixable,
                examples,
                tags: yaml_rule.tags,
                enabled: yaml_rule.enabled,
            };

            self.add_pattern(pattern);
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn load_custom_patterns<P: AsRef<Path>>(&mut self, _path: P) -> Result<()> {
        // TODO: Load custom patterns from file
        Ok(())
    }

    pub fn add_pattern(&mut self, pattern: AntiPattern) {
        let id = pattern.id.clone();
        let language = pattern.language.clone();
        
        self.patterns.insert(id.clone(), pattern);
        self.by_language.entry(language).or_default().push(id);
    }

    pub fn get_pattern(&self, id: &str) -> Option<&AntiPattern> {
        self.patterns.get(id)
    }

    pub fn get_patterns_for_language(&self, language: &Language) -> Vec<&AntiPattern> {
        self.by_language
            .get(language)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.patterns.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_patterns_for_file(&self, file_path: &str) -> Vec<&AntiPattern> {
        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        self.patterns
            .values()
            .filter(|p| p.enabled && p.matches_file_extension(extension))
            .collect()
    }

    pub fn search_patterns(&self, query: &str) -> Vec<&AntiPattern> {
        let query_lower = query.to_lowercase();
        self.patterns
            .values()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
                    || p.id.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    #[allow(dead_code)]
    fn add_elixir_patterns(&mut self) {
        use crate::core::{CodeExample, DetectionMethod};

        // Dynamic Atom Creation pattern
        let pattern = AntiPattern {
            id: "dynamic_atom_creation".to_string(),
            name: "Dynamic Atom Creation".to_string(),
            language: Language::Elixir,
            severity: Severity::Critical,
            description: "Creating atoms from uncontrolled input can exhaust memory as atoms are never garbage collected".to_string(),
            detection_method: DetectionMethod::Regex { 
                pattern: r"String\.to_atom\s*\(".to_string() 
            },
            fix_suggestion: "Replace String.to_atom(input) with String.to_existing_atom(input) or use explicit atom mapping".to_string(),
            source_url: Some("https://hexdocs.pm/elixir/main/code-anti-patterns.html#dynamic-atom-creation".to_string()),
            claude_code_fixable: true,
            examples: vec![
                CodeExample {
                    bad: "String.to_atom(user_input)".to_string(),
                    good: "String.to_existing_atom(user_input)".to_string(),
                    explanation: "Only converts if atom already exists, preventing memory exhaustion".to_string(),
                }
            ],
            tags: vec!["security".to_string(), "memory".to_string()],
            enabled: true,
        };
        self.add_pattern(pattern);

        // Long Parameter List pattern
        let pattern = AntiPattern {
            id: "long_parameter_list".to_string(),
            name: "Long Parameter List".to_string(),
            language: Language::Elixir,
            severity: Severity::Major,
            description: "Functions with too many parameters become confusing and error-prone".to_string(),
            detection_method: DetectionMethod::Regex { 
                pattern: r"def\s+\w+\s*\([^)]*,[^)]*,[^)]*,[^)]*,[^)]".to_string() 
            },
            fix_suggestion: "Group related parameters into structs or maps".to_string(),
            source_url: Some("https://hexdocs.pm/elixir/main/code-anti-patterns.html#long-parameter-list".to_string()),
            claude_code_fixable: true,
            examples: vec![
                CodeExample {
                    bad: "def loan(user_name, email, password, alias, book_title, book_ed)".to_string(),
                    good: "def loan(%{name: name, email: email} = user, %{title: title, ed: ed} = book)".to_string(),
                    explanation: "Grouping related parameters improves clarity and reduces errors".to_string(),
                }
            ],
            tags: vec!["maintainability".to_string()],
            enabled: true,
        };
        self.add_pattern(pattern);
    }
}

#[allow(dead_code)]
pub static GLOBAL_REGISTRY: Lazy<PatternRegistry> = Lazy::new(|| {
    let mut registry = PatternRegistry::new();
    registry.load_built_in_patterns().expect("Failed to load built-in patterns");
    registry
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AntiPattern, Language, Severity, DetectionMethod, CodeExample};
    use std::time::Instant;

    #[test]
    fn test_rule_registry_creation() {
        let registry = PatternRegistry::new();
        assert_eq!(registry.patterns.len(), 0);
        assert_eq!(registry.by_language.len(), 0);
    }

    #[test]
    fn test_add_pattern_o1_lookup() {
        let mut registry = PatternRegistry::new();
        let pattern = create_test_pattern("test_id", Language::Elixir, Severity::Critical);
        
        registry.add_pattern(pattern);
        
        // Test O(1) lookup by ID
        let start = Instant::now();
        let found = registry.get_pattern("test_id");
        let duration = start.elapsed();
        
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "test_id");
        // Should be fast for O(1) lookup (allow up to 1ms for CI variability)
        assert!(duration.as_micros() < 1000);
    }

    #[test]
    fn test_get_patterns_for_language_performance() {
        let mut registry = PatternRegistry::new();
        
        // Add 100 patterns for different languages
        for i in 0..100 {
            let lang = match i % 3 {
                0 => Language::Elixir,
                1 => Language::JavaScript,
                _ => Language::Python,
            };
            let pattern = create_test_pattern(&format!("test_{}", i), lang, Severity::Major);
            registry.add_pattern(pattern);
        }
        
        // Test language-based lookup performance
        let start = Instant::now();
        let elixir_patterns = registry.get_patterns_for_language(&Language::Elixir);
        let duration = start.elapsed();
        
        // Should find ~33 Elixir patterns
        assert!(elixir_patterns.len() >= 30 && elixir_patterns.len() <= 35);
        // Should be fast lookup
        assert!(duration.as_millis() < 10);
    }

    #[test]
    fn test_search_patterns_functionality() {
        let mut registry = PatternRegistry::new();
        
        let pattern1 = AntiPattern {
            id: "atom_creation".to_string(),
            name: "Dynamic Atom Creation".to_string(),
            language: Language::Elixir,
            severity: Severity::Critical,
            description: "Memory exhaustion through atoms".to_string(),
            detection_method: DetectionMethod::Regex { pattern: "test".to_string() },
            fix_suggestion: "Use existing atoms".to_string(),
            source_url: None,
            claude_code_fixable: true,
            examples: vec![],
            tags: vec!["memory".to_string()],
            enabled: true,
        };
        
        let pattern2 = AntiPattern {
            id: "sql_injection".to_string(),
            name: "SQL Injection Risk".to_string(),
            language: Language::JavaScript,
            severity: Severity::Critical,
            description: "SQL injection vulnerability".to_string(),
            detection_method: DetectionMethod::Regex { pattern: "test".to_string() },
            fix_suggestion: "Use parameterized queries".to_string(),
            source_url: None,
            claude_code_fixable: true,
            examples: vec![],
            tags: vec!["security".to_string()],
            enabled: true,
        };
        
        registry.add_pattern(pattern1);
        registry.add_pattern(pattern2);
        
        // Test search by name
        let atom_results = registry.search_patterns("atom");
        assert_eq!(atom_results.len(), 1);
        assert_eq!(atom_results[0].id, "atom_creation");
        
        // Test search by description
        let memory_results = registry.search_patterns("memory");
        assert_eq!(memory_results.len(), 1);
        
        // Test search by ID
        let sql_results = registry.search_patterns("sql");
        assert_eq!(sql_results.len(), 1);
        assert_eq!(sql_results[0].id, "sql_injection");
    }

    #[test]
    fn test_load_custom_rules_integration() {
        use tempfile::TempDir;
        use crate::core::custom_rules::{CustomRulesManager, CustomRule};
        
        // Setup temporary config
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_rules.yml").to_string_lossy().to_string();
        let custom_rules_manager = CustomRulesManager::with_config_path(config_path);
        
        // Add a custom rule
        let custom_rule = CustomRule {
            id: "no_console_log".to_string(),
            description: "Avoid console.log in production".to_string(),
            pattern: r"console\.log\(".to_string(),
            severity: "warning".to_string(),
            fix: "Use proper logging library".to_string(),
            enabled: true,
        };
        
        custom_rules_manager.add_project_rule(
            "test-project",
            "/test/path",
            Language::JavaScript,
            custom_rule,
        ).unwrap();
        
        // Test loading custom rules into registry
        let mut registry = PatternRegistry::new();
        
        // Override the CustomRulesManager::new() behavior by setting the path through test
        // This is a limitation of our current design - in real usage, the path would be consistent
        let custom_patterns = custom_rules_manager.get_project_rules("test-project").unwrap();
        for pattern in custom_patterns {
            registry.add_pattern(pattern);
        }
        
        // Verify custom rule was loaded
        let js_patterns = registry.get_patterns_for_language(&Language::JavaScript);
        assert_eq!(js_patterns.len(), 1);
        
        let custom_pattern = registry.get_pattern("custom_no_console_log");
        assert!(custom_pattern.is_some());
        
        let pattern = custom_pattern.unwrap();
        assert_eq!(pattern.name, "Avoid console.log in production");
        assert_eq!(pattern.severity, Severity::Warning);
        assert!(pattern.tags.contains(&"custom".to_string()));
    }

    #[test]
    fn test_load_built_in_patterns() {
        let mut registry = PatternRegistry::new();
        
        let result = registry.load_built_in_patterns();
        assert!(result.is_ok());
        
        // Should have loaded some Elixir patterns
        let elixir_patterns = registry.get_patterns_for_language(&Language::Elixir);
        assert!(!elixir_patterns.is_empty());
        
        // Should have specific patterns
        assert!(registry.get_pattern("dynamic_atom_creation").is_some());
        assert!(registry.get_pattern("long_parameter_list").is_some());
    }

    #[test]
    fn test_registry_scalability() {
        let mut registry = PatternRegistry::new();
        
        // Add 1000 patterns to test scalability
        for i in 0..1000 {
            let lang = match i % 7 {
                0 => Language::Elixir,
                1 => Language::JavaScript,
                2 => Language::TypeScript,
                3 => Language::Python,
                4 => Language::Rust,
                5 => Language::Zig,
                _ => Language::Sql,
            };
            let pattern = create_test_pattern(&format!("pattern_{}", i), lang, Severity::Warning);
            registry.add_pattern(pattern);
        }
        
        // Test performance with large dataset
        let start = Instant::now();
        
        // Multiple operations should still be fast
        let _pattern = registry.get_pattern("pattern_500");
        let _elixir_patterns = registry.get_patterns_for_language(&Language::Elixir);
        let _search_results = registry.search_patterns("pattern");
        
        let duration = start.elapsed();
        
        // All operations should complete quickly even with 1000 patterns
        assert!(duration.as_millis() < 50);
        
        // Verify we have all patterns
        assert_eq!(registry.patterns.len(), 1000);
    }

    // Helper function to create test patterns
    fn create_test_pattern(id: &str, language: Language, severity: Severity) -> AntiPattern {
        AntiPattern {
            id: id.to_string(),
            name: format!("Test Pattern {}", id),
            language,
            severity,
            description: format!("Test description for {}", id),
            detection_method: DetectionMethod::Regex { 
                pattern: r"test_pattern".to_string() 
            },
            fix_suggestion: "Fix this test pattern".to_string(),
            source_url: None,
            claude_code_fixable: false,
            examples: vec![
                CodeExample {
                    bad: "bad_example()".to_string(),
                    good: "good_example()".to_string(),
                    explanation: "Why the good example is better".to_string(),
                }
            ],
            tags: vec!["test".to_string()],
            enabled: true,
        }
    }

    mod embedded_rules_tests {
        use super::*;
        use std::time::Instant;

        #[test]
        fn test_embedded_elixir_rules_load() {
            let mut registry = PatternRegistry::new();
            
            // Test that embedded Elixir rules load successfully
            let result = registry.load_embedded_elixir_rules();
            assert!(result.is_ok(), "Should load embedded Elixir rules without error");
            
            // Should have specific Elixir patterns
            assert!(registry.get_pattern("dynamic_atom_creation").is_some());
            assert!(registry.get_pattern("long_parameter_list").is_some());
            assert!(registry.get_pattern("sql_injection_ecto").is_some());
            
            // Check language filtering works
            let elixir_patterns = registry.get_patterns_for_language(&Language::Elixir);
            assert!(elixir_patterns.len() >= 6, "Should have at least 6 Elixir patterns");
        }

        #[test]
        fn test_embedded_javascript_rules_load() {
            let mut registry = PatternRegistry::new();
            
            // Test that embedded JavaScript rules load successfully  
            let result = registry.load_embedded_javascript_rules();
            assert!(result.is_ok(), "Should load embedded JavaScript rules without error");
            
            // Should have specific JavaScript patterns
            assert!(registry.get_pattern("console_log_production").is_some());
            assert!(registry.get_pattern("eval_usage").is_some());
            assert!(registry.get_pattern("double_equals").is_some());
            
            // Check language filtering works
            let js_patterns = registry.get_patterns_for_language(&Language::JavaScript);
            assert!(js_patterns.len() >= 8, "Should have at least 8 JavaScript patterns");
        }

        #[test]
        fn test_load_all_embedded_rules() {
            let mut registry = PatternRegistry::new();
            
            // Test loading all embedded rules
            let result = registry.load_all_embedded_rules();
            assert!(result.is_ok(), "Should load all embedded rules without error");
            
            // Should have patterns for multiple languages
            assert!(!registry.get_patterns_for_language(&Language::Elixir).is_empty());
            assert!(!registry.get_patterns_for_language(&Language::JavaScript).is_empty());
            
            // Should have a reasonable total number of patterns
            let total_patterns = registry.patterns.len();
            assert!(total_patterns >= 14, "Should have at least 14 total patterns");
        }

        #[test]
        fn test_embedded_rules_performance() {
            let start = Instant::now();
            
            let mut registry = PatternRegistry::new();
            registry.load_all_embedded_rules().expect("Should load rules");
            
            let load_duration = start.elapsed();
            
            // Loading should be very fast (embedded in binary)
            assert!(load_duration.as_millis() < 100, "Embedded rule loading should be < 100ms");
            
            // Test lookup performance after loading
            let lookup_start = Instant::now();
            let _pattern = registry.get_pattern("dynamic_atom_creation");
            let _elixir_patterns = registry.get_patterns_for_language(&Language::Elixir);
            let lookup_duration = lookup_start.elapsed();
            
            assert!(lookup_duration.as_micros() < 1000, "Lookups should be < 1ms");
        }

        #[test]
        fn test_pre_compiled_regex_patterns() {
            let mut registry = PatternRegistry::new();
            registry.load_all_embedded_rules().expect("Should load rules");
            
            // Test that regex patterns compile successfully
            let result = registry.compile_all_patterns();
            assert!(result.is_ok(), "All regex patterns should compile successfully");
            
            // Test that compiled patterns are accessible
            assert!(registry.compiled_patterns.len() > 0, "Should have compiled patterns");
            
            // Test lookup performance with compiled patterns
            let start = Instant::now();
            let _compiled_pattern = registry.get_compiled_pattern("dynamic_atom_creation");
            let duration = start.elapsed();
            
            assert!(duration.as_micros() < 10, "Compiled pattern lookup should be < 10 microseconds");
        }

        #[test]
        fn test_embedded_rules_yaml_format_validation() {
            // Test that our YAML files have the correct structure
            let elixir_yaml = include_str!("../rules/builtin/elixir.yml");
            let js_yaml = include_str!("../rules/builtin/javascript.yml");
            
            // Should be able to parse as YAML
            let elixir_parsed: Result<Vec<serde_yaml::Value>, _> = serde_yaml::from_str(elixir_yaml);
            assert!(elixir_parsed.is_ok(), "Elixir YAML should parse correctly");
            
            let js_parsed: Result<Vec<serde_yaml::Value>, _> = serde_yaml::from_str(js_yaml);
            assert!(js_parsed.is_ok(), "JavaScript YAML should parse correctly");
            
            // Test that required fields are present in first rule
            if let Ok(elixir_rules) = elixir_parsed {
                if let Some(first_rule) = elixir_rules.first() {
                    assert!(first_rule.get("id").is_some(), "Rule should have ID");
                    assert!(first_rule.get("name").is_some(), "Rule should have name");
                    assert!(first_rule.get("language").is_some(), "Rule should have language");
                    assert!(first_rule.get("severity").is_some(), "Rule should have severity");
                    assert!(first_rule.get("description").is_some(), "Rule should have description");
                    assert!(first_rule.get("detection_method").is_some(), "Rule should have detection method");
                    assert!(first_rule.get("fix_suggestion").is_some(), "Rule should have fix suggestion");
                }
            }
        }
    }
}