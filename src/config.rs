use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::{fs, net::IpAddr, path::Path};

#[derive(Debug, Clone, Deserialize)]
pub struct BridgeConfig {
    pub upstream: UpstreamConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub history: HistoryConfig,
    #[serde(default)]
    pub reasoning: ReasoningConfig,
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

/// Optional per-model reasoning capability overrides.
///
/// When present, the translator uses these values instead of inferring
/// reasoning parameters from the model name. Useful for models served via
/// LiteLLM under custom aliases.
///
/// Example YAML:
/// ```yaml
/// reasoning:
///   supports_thinking: true
///   supports_effort: false
///   thinking_param: thinking
///   effort_param: ""
///   effort_value_mode: ""
///   output_format: ""
/// ```
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ReasoningConfig {
    pub supports_thinking: Option<bool>,
    pub supports_effort: Option<bool>,
    pub thinking_param: Option<String>,
    pub effort_param: Option<String>,
    pub effort_value_mode: Option<String>,
    pub output_format: Option<String>,
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
        let config: Self = serde_yaml::from_str(&text)
            .with_context(|| format!("failed to parse config {}", path.display()))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.upstream.base_url.trim().is_empty() {
            bail!("upstream.base_url must not be empty");
        }
        if self.upstream.chat_endpoint.trim().is_empty() {
            bail!("upstream.chat_endpoint must not be empty");
        }
        if !self.upstream.chat_endpoint.starts_with('/') {
            bail!(
                "upstream.chat_endpoint must start with '/': got '{}'",
                self.upstream.chat_endpoint
            );
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(yaml: &str) -> Result<BridgeConfig> {
        let config: BridgeConfig = serde_yaml::from_str(yaml).with_context(|| "parse error")?;
        config.validate()?;
        Ok(config)
    }

    #[test]
    fn rejects_empty_base_url() {
        let err = parse("upstream:\n  base_url: \"\"").unwrap_err();
        assert!(err.to_string().contains("base_url"), "{err}");
    }

    #[test]
    fn rejects_empty_chat_endpoint() {
        let err = parse("upstream:\n  base_url: \"http://localhost:4000\"\n  chat_endpoint: \"\"")
            .unwrap_err();
        assert!(err.to_string().contains("chat_endpoint"), "{err}");
    }

    #[test]
    fn rejects_chat_endpoint_without_leading_slash() {
        let err = parse(
            "upstream:\n  base_url: \"http://localhost:4000\"\n  chat_endpoint: \"v1/chat/completions\"",
        )
        .unwrap_err();
        assert!(err.to_string().contains("chat_endpoint"), "{err}");
    }

    #[test]
    fn accepts_valid_minimal_config() {
        let cfg = parse("upstream:\n  base_url: \"http://localhost:4000\"").unwrap();
        assert_eq!(cfg.upstream.base_url, "http://localhost:4000");
        assert_eq!(cfg.upstream.chat_endpoint, "/v1/chat/completions");
        assert_eq!(cfg.history.max_cached_responses, 512);
    }

    #[test]
    fn parses_reasoning_config() {
        let cfg = parse(
            "upstream:\n  base_url: \"http://localhost:4000\"\nreasoning:\n  supports_thinking: true\n  thinking_param: thinking",
        )
        .unwrap();
        assert_eq!(cfg.reasoning.supports_thinking, Some(true));
        assert_eq!(cfg.reasoning.thinking_param.as_deref(), Some("thinking"));
    }

    #[test]
    fn history_max_cached_responses_is_configurable() {
        let cfg = parse(
            "upstream:\n  base_url: \"http://localhost:4000\"\nhistory:\n  max_cached_responses: 64",
        )
        .unwrap();
        assert_eq!(cfg.history.max_cached_responses, 64);
    }
}
