use crate::{algorithm::Algorithm, backend::Backend};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TomlConfig {
    listen: String,
    backends: Vec<Backend>,
    algorithm: Algorithm,
    health_interval: u64,
    health_timeout: u64,
}

impl TomlConfig {
    pub fn new(path: &str) -> Self {
        let content = std::fs::read_to_string(path).unwrap();
        toml::from_str(&content).unwrap()
    }
}
