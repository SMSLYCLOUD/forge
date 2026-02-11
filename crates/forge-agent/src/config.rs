use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// AI Agent configuration, loaded from ~/.forge/agent.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub openai: OpenAiConfig,
    #[serde(default)]
    pub anthropic: AnthropicConfig,
    #[serde(default)]
    pub google: GoogleConfig,
    #[serde(default)]
    pub ollama: OllamaConfig,
    #[serde(default)]
    pub openrouter: OpenRouterConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: Option<String>,
    #[serde(default = "default_openai_model")]
    pub model: String,
    #[serde(default = "default_openai_url")]
    pub base_url: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub api_key: Option<String>,
    #[serde(default = "default_anthropic_model")]
    pub model: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoogleConfig {
    pub api_key: Option<String>,
    #[serde(default = "default_google_model")]
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    #[serde(default = "default_ollama_url")]
    pub base_url: String,
    #[serde(default = "default_ollama_model")]
    pub model: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    pub api_key: Option<String>,
    #[serde(default = "default_openrouter_model")]
    pub model: String,
}

fn default_provider() -> String {
    "ollama".into()
}
fn default_openai_model() -> String {
    "gpt-4o".into()
}
fn default_openai_url() -> String {
    "https://api.openai.com/v1".into()
}
fn default_anthropic_model() -> String {
    "claude-sonnet-4-20250514".into()
}
fn default_google_model() -> String {
    "gemini-2.0-flash".into()
}
fn default_ollama_url() -> String {
    "http://localhost:11434".into()
}
fn default_ollama_model() -> String {
    "codellama:7b".into()
}
fn default_openrouter_model() -> String {
    "anthropic/claude-sonnet-4-20250514".into()
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: default_ollama_url(),
            model: default_ollama_model(),
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            openai: OpenAiConfig::default(),
            anthropic: AnthropicConfig::default(),
            google: GoogleConfig::default(),
            ollama: OllamaConfig::default(),
            openrouter: OpenRouterConfig::default(),
        }
    }
}

impl AgentConfig {
    /// Load config from ~/.forge/agent.toml
    pub fn load() -> Self {
        let config_path = Self::config_path();
        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => tracing::warn!("Failed to parse agent.toml: {}", e),
                },
                Err(e) => tracing::warn!("Failed to read agent.toml: {}", e),
            }
        }
        Self::default()
    }

    /// Save config to ~/.forge/agent.toml
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".forge")
            .join("agent.toml")
    }
}
