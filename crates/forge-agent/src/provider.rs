use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::config::AgentConfig;

/// A chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,    // "system", "user", "assistant"
    pub content: String,
}

/// Response from the LLM
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub finish_reason: String,
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Clone)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Trait for LLM providers
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse>;
    fn name(&self) -> &str;
    fn model(&self) -> &str;
}

/// OpenAI-compatible provider (works for OpenAI, OpenRouter, and any compatible API)
pub struct OpenAiProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
    name: String,
}

impl OpenAiProvider {
    pub fn new(base_url: &str, api_key: &str, model: &str, name: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            name: name.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": 0.3,
            "max_tokens": 4096,
        });

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error {}: {}", status, body);
        }

        let json: serde_json::Value = response.json().await?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let finish_reason = json["choices"][0]["finish_reason"]
            .as_str()
            .unwrap_or("stop")
            .to_string();

        Ok(LlmResponse {
            content,
            model: self.model.clone(),
            finish_reason,
            usage: None,
        })
    }

    fn name(&self) -> &str { &self.name }
    fn model(&self) -> &str { &self.model }
}

/// Ollama provider (local LLM)
pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for OllamaProvider {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "stream": false,
        });

        let response = self.client
            .post(format!("{}/api/chat", self.base_url))
            .header("Content-Type", "application/json")
            .json(&body)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama error {}: {}", status, body);
        }

        let json: serde_json::Value = response.json().await?;
        let content = json["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(LlmResponse {
            content,
            model: self.model.clone(),
            finish_reason: "stop".to_string(),
            usage: None,
        })
    }

    fn name(&self) -> &str { "ollama" }
    fn model(&self) -> &str { &self.model }
}

/// Anthropic provider
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<LlmResponse> {
        // Separate system message from conversation
        let system_msg = messages.iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let chat_messages: Vec<serde_json::Value> = messages.iter()
            .filter(|m| m.role != "system")
            .map(|m| serde_json::json!({
                "role": m.role,
                "content": m.content,
            }))
            .collect();

        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 4096,
            "system": system_msg,
            "messages": chat_messages,
        });

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic error {}: {}", status, body);
        }

        let json: serde_json::Value = response.json().await?;
        let content = json["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(LlmResponse {
            content,
            model: self.model.clone(),
            finish_reason: "stop".to_string(),
            usage: None,
        })
    }

    fn name(&self) -> &str { "anthropic" }
    fn model(&self) -> &str { &self.model }
}

/// Create the appropriate provider from config
pub fn create_provider(config: &AgentConfig) -> Result<Box<dyn LlmProvider>> {
    match config.provider.as_str() {
        "openai" => {
            let api_key = config.openai.api_key.as_deref()
                .ok_or_else(|| anyhow::anyhow!("OpenAI API key not set in ~/.forge/agent.toml"))?;
            Ok(Box::new(OpenAiProvider::new(
                &config.openai.base_url,
                api_key,
                &config.openai.model,
                "openai",
            )))
        }
        "anthropic" => {
            let api_key = config.anthropic.api_key.as_deref()
                .ok_or_else(|| anyhow::anyhow!("Anthropic API key not set"))?;
            Ok(Box::new(AnthropicProvider::new(api_key, &config.anthropic.model)))
        }
        "ollama" => {
            Ok(Box::new(OllamaProvider::new(
                &config.ollama.base_url,
                &config.ollama.model,
            )))
        }
        "openrouter" => {
            let api_key = config.openrouter.api_key.as_deref()
                .ok_or_else(|| anyhow::anyhow!("OpenRouter API key not set"))?;
            Ok(Box::new(OpenAiProvider::new(
                "https://openrouter.ai/api/v1",
                api_key,
                &config.openrouter.model,
                "openrouter",
            )))
        }
        other => anyhow::bail!("Unknown provider: {}", other),
    }
}
