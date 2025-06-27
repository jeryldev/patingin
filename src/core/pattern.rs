use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Elixir,
    JavaScript,
    TypeScript,
    Python,
    Rust,
    Zig,
    Sql,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Elixir => write!(f, "elixir"),
            Language::JavaScript => write!(f, "javascript"),
            Language::TypeScript => write!(f, "typescript"),
            Language::Python => write!(f, "python"),
            Language::Rust => write!(f, "rust"),
            Language::Zig => write!(f, "zig"),
            Language::Sql => write!(f, "sql"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Major,
    Warning,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "critical"),
            Severity::Major => write!(f, "major"),
            Severity::Warning => write!(f, "warning"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DetectionMethod {
    Regex { pattern: String },
    Ast { pattern: String },
    LineCount { threshold: usize, pattern: String },
    Ratio { threshold: f64, pattern: String },
    Custom { pattern: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    pub bad: String,
    pub good: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPattern {
    pub id: String,
    pub name: String,
    pub language: Language,
    pub severity: Severity,
    pub description: String,
    pub detection_method: DetectionMethod,
    pub fix_suggestion: String,
    pub source_url: Option<String>,
    pub claude_code_fixable: bool,
    pub examples: Vec<CodeExample>,
    pub tags: Vec<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl AntiPattern {
    pub fn matches_file_extension(&self, extension: &str) -> bool {
        match self.language {
            Language::Elixir => matches!(extension, "ex" | "exs"),
            Language::JavaScript => matches!(extension, "js" | "jsx" | "mjs"),
            Language::TypeScript => matches!(extension, "ts" | "tsx"),
            Language::Python => matches!(extension, "py"),
            Language::Rust => matches!(extension, "rs"),
            Language::Zig => matches!(extension, "zig"),
            Language::Sql => matches!(extension, "sql"),
        }
    }
}
