#[cfg(test)]
mod embedded_rules_tests {
    use super::*;
    use crate::core::{Language, Severity};
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
    fn test_embedded_rule_validation() {
        let mut registry = PatternRegistry::new();
        registry.load_all_embedded_rules().expect("Should load rules");
        
        // Test that all loaded patterns have required fields
        for pattern in registry.patterns.values() {
            assert!(!pattern.id.is_empty(), "Pattern ID should not be empty");
            assert!(!pattern.name.is_empty(), "Pattern name should not be empty");
            assert!(!pattern.description.is_empty(), "Pattern description should not be empty");
            assert!(!pattern.fix_suggestion.is_empty(), "Pattern fix suggestion should not be empty");
            
            // Test that severity is valid
            match pattern.severity {
                Severity::Critical | Severity::Major | Severity::Warning => {},
                // Any other value should fail
            }
            
            // Test that detection method is configured
            match &pattern.detection_method {
                crate::core::DetectionMethod::Regex { pattern: regex_pattern } => {
                    assert!(!regex_pattern.is_empty(), "Regex pattern should not be empty");
                }
                _ => {} // Other detection methods are valid too
            }
        }
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
    fn test_rule_registry_lazy_loading() {
        // Test that global registry loads successfully
        let start = Instant::now();
        
        // Access global registry (should trigger lazy loading)
        let elixir_patterns = GLOBAL_REGISTRY.get_patterns_for_language(&Language::Elixir);
        
        let duration = start.elapsed();
        
        assert!(!elixir_patterns.is_empty(), "Global registry should have Elixir patterns");
        assert!(duration.as_millis() < 200, "Lazy loading should be fast");
        
        // Second access should be even faster (already loaded)
        let start2 = Instant::now();
        let _js_patterns = GLOBAL_REGISTRY.get_patterns_for_language(&Language::JavaScript);
        let duration2 = start2.elapsed();
        
        assert!(duration2.as_micros() < 100, "Second access should be very fast");
    }

    #[test]
    fn test_rule_search_across_embedded_rules() {
        let mut registry = PatternRegistry::new();
        registry.load_all_embedded_rules().expect("Should load rules");
        
        // Test search functionality across all embedded rules
        let security_patterns = registry.search_patterns("security");
        assert!(!security_patterns.is_empty(), "Should find security-related patterns");
        
        let memory_patterns = registry.search_patterns("memory");
        assert!(!memory_patterns.is_empty(), "Should find memory-related patterns");
        
        let sql_patterns = registry.search_patterns("sql");
        assert!(!sql_patterns.is_empty(), "Should find SQL-related patterns");
        
        // Test search performance
        let start = Instant::now();
        let _results = registry.search_patterns("injection");
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 10, "Search should be fast");
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

    #[test]
    fn test_rule_registry_thread_safety() {
        use std::sync::Arc;
        use std::thread;
        
        // Test that global registry can be accessed from multiple threads
        let handles: Vec<_> = (0..5).map(|i| {
            thread::spawn(move || {
                let patterns = GLOBAL_REGISTRY.get_patterns_for_language(&Language::Elixir);
                assert!(!patterns.is_empty(), "Thread {} should get Elixir patterns", i);
                
                let search_results = GLOBAL_REGISTRY.search_patterns("atom");
                assert!(!search_results.is_empty(), "Thread {} should get search results", i);
            })
        }).collect();
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread should complete successfully");
        }
    }
}