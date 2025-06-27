use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub settings: Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub auto_fix: bool,
    pub severity_threshold: String,
    pub focus_languages: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            settings: Settings {
                auto_fix: false,
                severity_threshold: "major".to_string(),
                focus_languages: vec![],
            },
        }
    }
}

impl Config {
    #[allow(dead_code)]
    pub fn load<P: AsRef<Path>>(_path: P) -> Result<Self> {
        // TODO: Implement config loading
        Ok(Self::default())
    }

    #[allow(dead_code)]
    pub fn save<P: AsRef<Path>>(&self, _path: P) -> Result<()> {
        // TODO: Implement config saving
        Ok(())
    }
}
