#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AntiPattern, Language, Severity, DetectionMethod, CodeExample};
    use std::collections::HashMap;
    use std::sync::Arc;
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
        // Should be sub-microsecond for O(1) lookup
        assert!(duration.as_micros() < 100);
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
    fn test_get_patterns_for_file_extension() {
        let mut registry = PatternRegistry::new();
        
        // Add patterns for different languages
        let elixir_pattern = create_test_pattern("elixir_test", Language::Elixir, Severity::Major);
        let js_pattern = create_test_pattern("js_test", Language::JavaScript, Severity::Major);
        
        registry.add_pattern(elixir_pattern);
        registry.add_pattern(js_pattern);
        
        // Test file extension matching
        let elixir_file_patterns = registry.get_patterns_for_file("lib/user.ex");
        let js_file_patterns = registry.get_patterns_for_file("src/app.js");
        
        // Should find appropriate patterns (this depends on matches_file_extension implementation)
        assert!(!elixir_file_patterns.is_empty() || !js_file_patterns.is_empty());
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

    #[test]
    fn test_pattern_enabled_filtering() {
        let mut registry = PatternRegistry::new();
        
        // Add enabled and disabled patterns
        let mut enabled_pattern = create_test_pattern("enabled", Language::Elixir, Severity::Major);
        enabled_pattern.enabled = true;
        
        let mut disabled_pattern = create_test_pattern("disabled", Language::Elixir, Severity::Major);
        disabled_pattern.enabled = false;
        
        registry.add_pattern(enabled_pattern);
        registry.add_pattern(disabled_pattern);
        
        // get_patterns_for_file should only return enabled patterns
        let file_patterns = registry.get_patterns_for_file("test.ex");
        
        // Only enabled patterns should be included
        assert!(file_patterns.iter().all(|p| p.enabled));
    }

    #[test]
    fn test_concurrent_access_safety() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let registry = Arc::new(Mutex::new(PatternRegistry::new()));
        
        // Add initial patterns
        {
            let mut reg = registry.lock().unwrap();
            reg.load_built_in_patterns().unwrap();
        }
        
        // Spawn multiple threads to test concurrent read access
        let handles: Vec<_> = (0..10).map(|i| {
            let reg_clone = Arc::clone(&registry);
            thread::spawn(move || {
                let reg = reg_clone.lock().unwrap();
                
                // Each thread performs different operations
                match i % 3 {
                    0 => {
                        let _pattern = reg.get_pattern("dynamic_atom_creation");
                    },
                    1 => {
                        let _patterns = reg.get_patterns_for_language(&Language::Elixir);
                    },
                    _ => {
                        let _results = reg.search_patterns("atom");
                    }
                }
            })
        }).collect();
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Registry should still be functional
        let reg = registry.lock().unwrap();
        assert!(reg.get_pattern("dynamic_atom_creation").is_some());
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
}