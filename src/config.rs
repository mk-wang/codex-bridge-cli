use anyhow::{Context, Result};
use serde::Deserialize;
use std::{fs, net::IpAddr, path::Path};

#[derive(Debug, Clone, Deserialize)]
pub struct BridgeConfig {
    pub upstream: UpstreamConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub history: HistoryConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpstreamConfig {
    pub base_url: String,
    #[serde(default = "default_chat_endpoint")]
    pub chat_endpoint: String,
    pub api_key_env: Option<String>,
    #[serde(default = "default_timeout_secs")]
    pub timeout: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: IpAddr,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryConfig {
    #[serde(default = "default_max_cached_responses")]
    pub max_cached_responses: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_cached_responses: default_max_cached_responses(),
        }
    }
}

impl BridgeConfig {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        serde_yaml::from_str(&text)
            .with_context(|| format!("failed to parse config {}", path.display()))
    }
}

fn default_host() -> IpAddr {
    "127.0.0.1".parse().expect("valid default host")
}

fn default_port() -> u16 {
    4010
}

fn default_chat_endpoint() -> String {
    "/v1/chat/completions".to_string()
}

fn default_timeout_secs() -> u64 {
    300
}

fn default_max_cached_responses() -> usize {
    512
}
