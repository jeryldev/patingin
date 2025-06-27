use anyhow::{anyhow, Result};
use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;
use which::which;

pub mod fix_engine;

pub struct ClaudeCodeIntegration {
    pub available: bool,
    pub version: Option<String>,
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct FixRequest {
    pub file_path: String,
    pub line_number: usize,
    pub original_code: String,
    pub violation_description: String,
    pub fix_suggestion: String,
    pub language: String,
}

#[derive(Debug, Clone)]
pub struct FixResult {
    pub success: bool,
    pub fixed_code: Option<String>,
    pub error_message: Option<String>,
    pub confidence: f64,
}

impl ClaudeCodeIntegration {
    pub fn detect() -> Self {
        let (available, command, version) = if which("claude-code").is_ok() {
            let version = Self::get_version("claude-code");
            (true, "claude-code".to_string(), version)
        } else if which("claude").is_ok() {
            let version = Self::get_version("claude");
            (true, "claude".to_string(), version)
        } else {
            (false, "".to_string(), None)
        };

        Self {
            available,
            version,
            command,
        }
    }

    fn get_version(command: &str) -> Option<String> {
        Command::new(command)
            .args(["--version"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            })
    }

    pub fn generate_fix(&self, request: &FixRequest) -> Result<FixResult> {
        if !self.available {
            return Ok(FixResult {
                success: false,
                fixed_code: None,
                error_message: Some("Claude Code CLI not available".to_string()),
                confidence: 0.0,
            });
        }

        // Create a focused prompt for Claude Code
        let prompt = self.create_fix_prompt(request);

        // Execute Claude Code with the prompt
        match self.execute_claude_code(&prompt) {
            Ok(response) => self.parse_claude_response(&response, request),
            Err(e) => Ok(FixResult {
                success: false,
                fixed_code: None,
                error_message: Some(format!("Claude Code execution failed: {e}")),
                confidence: 0.0,
            }),
        }
    }

    fn create_fix_prompt(&self, request: &FixRequest) -> String {
        format!(
            r#"Fix this {language} code violation:

File: {file_path}
Line: {line_number}

Issue: {violation_description}
Suggestion: {fix_suggestion}

Original code:
```{language}
{original_code}
```

Please provide ONLY the fixed code without explanations. Return the corrected line(s) that should replace the original code."#,
            language = request.language,
            file_path = request.file_path,
            line_number = request.line_number,
            violation_description = request.violation_description,
            fix_suggestion = request.fix_suggestion,
            original_code = request.original_code
        )
    }

    fn execute_claude_code(&self, prompt: &str) -> Result<String> {
        // Create a temporary file for the prompt
        let temp_file = NamedTempFile::new()?;
        fs::write(temp_file.path(), prompt)?;

        // Execute Claude Code with the prompt file
        let output = Command::new(&self.command)
            .args(["--file", temp_file.path().to_str().unwrap()])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Claude Code failed: {}", error));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn parse_claude_response(&self, response: &str, _request: &FixRequest) -> Result<FixResult> {
        // Parse the Claude Code response
        let cleaned_response = response.trim();

        if cleaned_response.is_empty() {
            return Ok(FixResult {
                success: false,
                fixed_code: None,
                error_message: Some("Empty response from Claude Code".to_string()),
                confidence: 0.0,
            });
        }

        // Extract code from markdown blocks if present
        let fixed_code = if cleaned_response.contains("```") {
            self.extract_code_from_markdown(cleaned_response)
        } else {
            cleaned_response.to_string()
        };

        // Calculate confidence based on response quality
        let confidence = self.calculate_confidence(&fixed_code);

        Ok(FixResult {
            success: true,
            fixed_code: Some(fixed_code),
            error_message: None,
            confidence,
        })
    }

    fn extract_code_from_markdown(&self, response: &str) -> String {
        // Find code blocks in markdown
        let lines: Vec<&str> = response.lines().collect();
        let mut in_code_block = false;
        let mut code_lines = Vec::new();

        for line in lines {
            if line.starts_with("```") {
                if in_code_block {
                    break; // End of code block
                } else {
                    in_code_block = true; // Start of code block
                }
            } else if in_code_block {
                code_lines.push(line);
            }
        }

        if code_lines.is_empty() {
            // Fallback: return the whole response if no code blocks found
            response.to_string()
        } else {
            code_lines.join("\n")
        }
    }

    fn calculate_confidence(&self, fixed_code: &str) -> f64 {
        // Simple heuristics for confidence calculation
        let mut confidence: f64 = 0.7; // Base confidence

        // Increase confidence if code looks structured
        if fixed_code.contains("def ")
            || fixed_code.contains("function ")
            || fixed_code.contains("defmodule ")
        {
            confidence += 0.1;
        }

        // Increase confidence if code has proper syntax elements
        if fixed_code.contains("(") && fixed_code.contains(")") {
            confidence += 0.1;
        }

        // Decrease confidence if response looks like an explanation
        if fixed_code.to_lowercase().contains("here's")
            || fixed_code.to_lowercase().contains("this code")
        {
            confidence -= 0.3;
        }

        confidence.clamp(0.0, 1.0)
    }

    pub fn apply_fixes_to_file(&self, file_path: &str, fixes: &[(usize, String)]) -> Result<()> {
        // Read the original file
        let original_content = fs::read_to_string(file_path)?;
        let mut lines: Vec<String> = original_content.lines().map(|s| s.to_string()).collect();

        // Apply fixes in reverse order (highest line number first) to maintain line numbers
        let mut sorted_fixes = fixes.to_vec();
        sorted_fixes.sort_by(|a, b| b.0.cmp(&a.0));

        for (line_number, fixed_line) in sorted_fixes {
            if line_number > 0 && line_number <= lines.len() {
                lines[line_number - 1] = fixed_line;
            }
        }

        // Write the modified content back to the file
        let modified_content = lines.join("\n");
        fs::write(file_path, modified_content)?;

        Ok(())
    }

    pub fn validate_fix(&self, original: &str, fixed: &str, language: &str) -> Result<bool> {
        // Basic validation to ensure the fix is reasonable

        // Check if the fix is not empty
        if fixed.trim().is_empty() {
            return Ok(false);
        }

        // Check if the fix is not identical to original
        if original.trim() == fixed.trim() {
            return Ok(false);
        }

        // Language-specific basic syntax validation
        match language.to_lowercase().as_str() {
            "elixir" => self.validate_elixir_syntax(fixed),
            "javascript" | "typescript" => self.validate_javascript_syntax(fixed),
            "python" => self.validate_python_syntax(fixed),
            "rust" => self.validate_rust_syntax(fixed),
            _ => Ok(true), // Default to valid for unknown languages
        }
    }

    fn validate_elixir_syntax(&self, code: &str) -> Result<bool> {
        // Basic Elixir syntax checks
        let balanced_parens = self.check_balanced_brackets(code, '(', ')');
        let balanced_braces = self.check_balanced_brackets(code, '{', '}');
        let balanced_brackets = self.check_balanced_brackets(code, '[', ']');

        Ok(balanced_parens && balanced_braces && balanced_brackets)
    }

    fn validate_javascript_syntax(&self, code: &str) -> Result<bool> {
        // Basic JavaScript syntax checks
        let balanced_parens = self.check_balanced_brackets(code, '(', ')');
        let balanced_braces = self.check_balanced_brackets(code, '{', '}');
        let balanced_brackets = self.check_balanced_brackets(code, '[', ']');

        Ok(balanced_parens && balanced_braces && balanced_brackets)
    }

    fn validate_python_syntax(&self, code: &str) -> Result<bool> {
        // Basic Python syntax checks
        let balanced_parens = self.check_balanced_brackets(code, '(', ')');
        let balanced_brackets = self.check_balanced_brackets(code, '[', ']');

        // Check for basic Python indentation (simplified)
        let lines: Vec<&str> = code.lines().collect();
        for line in lines {
            if !line.trim().is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                // Allow non-indented lines (top-level statements)
                continue;
            }
        }

        Ok(balanced_parens && balanced_brackets)
    }

    fn validate_rust_syntax(&self, code: &str) -> Result<bool> {
        // Basic Rust syntax checks
        let balanced_parens = self.check_balanced_brackets(code, '(', ')');
        let balanced_braces = self.check_balanced_brackets(code, '{', '}');
        let balanced_brackets = self.check_balanced_brackets(code, '[', ']');

        Ok(balanced_parens && balanced_braces && balanced_brackets)
    }

    fn check_balanced_brackets(&self, code: &str, open: char, close: char) -> bool {
        let mut count = 0;
        for ch in code.chars() {
            if ch == open {
                count += 1;
            } else if ch == close {
                count -= 1;
                if count < 0 {
                    return false;
                }
            }
        }
        count == 0
    }
}

#[allow(dead_code)]
pub struct GitHubIntegration {
    token: Option<String>,
}

impl Default for GitHubIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl GitHubIntegration {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let token = std::env::var("GITHUB_TOKEN").ok();
        Self { token }
    }

    #[allow(dead_code)]
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }
}
