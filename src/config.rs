use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct HealthConfig {
    #[serde(default)]
    pub required_services: Vec<String>,

    #[serde(default)]
    pub optional_services: Vec<String>,

    #[serde(default = "default_ignore_exact")]
    pub ignore_exact: Vec<String>,

    #[serde(default = "default_ignore_prefixes")]
    pub ignore_prefixes: Vec<String>,
}

fn default_ignore_exact() -> Vec<String> {
    vec!["time-first-boot".into(), "local".into()]
}

fn default_ignore_prefixes() -> Vec<String> {
    vec!["getty.".into(), "agetty.".into()]
}
pub fn from_toml_str(s: &str) -> Result<HealthConfig> {
    toml::from_str::<HealthConfig>(s).context("invalid config TOML")
}

pub fn from_file(path: &Path) -> Result<HealthConfig> {
    let s = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    from_toml_str(&s).with_context(|| format!("failed to parse config file {}", path.display()))
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            required_services: vec![],
            optional_services: vec![],
            ignore_exact: default_ignore_exact(),
            ignore_prefixes: default_ignore_prefixes(),
        }
    }
}
