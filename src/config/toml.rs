use crate::algorithm::Algorithm;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct TomlConfig {
    pub listen: Option<String>,
    pub backends: Option<Vec<String>>,
    pub algorithm: Option<Algorithm>,
    pub health_interval: Option<u64>,
    pub health_timeout: Option<u64>,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
}

impl TomlConfig {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}
