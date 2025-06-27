use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

use crate::schema::SchemaOutputTier;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_tier: SchemaOutputTier,
    pub pretty_output: bool,
    pub validate_schema: bool,
    pub output_directory: Option<PathBuf>,
    pub file_extensions: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_tier: SchemaOutputTier::Standard,
            pretty_output: false,
            validate_schema: false,
            output_directory: None,
            file_extensions: vec!["json".to_string()],
        }
    }
}

impl Config {
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let config: Config = if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
            toml::from_str(&content)
                .map_err(|e| AppError::SchemaGeneration(format!("Invalid TOML config: {}", e)))?
        } else {
            serde_json::from_str(&content)
                .map_err(|e| AppError::SchemaGeneration(format!("Invalid JSON config: {}", e)))?
        };

        Ok(config)
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
            toml::to_string_pretty(self)
                .map_err(|e| AppError::SchemaGeneration(format!("Failed to serialize config to TOML: {}", e)))?
        } else {
            serde_json::to_string_pretty(self)?
        };

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        Ok(())
    }

    pub fn merge_with_args(&mut self, tier: Option<SchemaOutputTier>, pretty: bool, validate: bool) {
        if let Some(t) = tier {
            self.default_tier = t;
        }
        if pretty {
            self.pretty_output = true;
        }
        if validate {
            self.validate_schema = true;
        }
    }
}