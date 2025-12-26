use serde::Deserialize;
use anyhow::{Context, Result};

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
