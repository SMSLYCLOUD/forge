# JULES IMPLEMENTATION — PART 4 OF 4
# Tasks 13-16: AI Agent, Extension Store, Adaptive UI Modes, Network Layer

> **CRITICAL**: Complete Parts 1-3 first.
> These tasks create NEW crates. Update workspace `Cargo.toml` for each new crate.

---

## TASK 13: Built-in AI Agent

### Step 1: Update workspace `Cargo.toml`

Add to `[workspace]` members:

```toml
members = [
    # ... existing members ...
    "crates/forge-agent",
]
```

Add to `[workspace.dependencies]`:

```toml
# AI Agent
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls", "gzip", "brotli"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
rand = "0.8"
```

### Step 2: Create `crates/forge-agent/Cargo.toml`

```toml
[package]
name = "forge-agent"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
reqwest.workspace = true
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
toml.workspace = true
anyhow.workspace = true
tracing.workspace = true
futures.workspace = true

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
```

### Step 3: Create `crates/forge-agent/src/lib.rs`

```rust
pub mod config;
pub mod provider;
pub mod agent;
pub mod chat;
```

### Step 4: Create `crates/forge-agent/src/config.rs`

```rust
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

fn default_provider() -> String { "ollama".into() }
fn default_openai_model() -> String { "gpt-4o".into() }
fn default_openai_url() -> String { "https://api.openai.com/v1".into() }
fn default_anthropic_model() -> String { "claude-sonnet-4-20250514".into() }
fn default_google_model() -> String { "gemini-2.0-flash".into() }
fn default_ollama_url() -> String { "http://localhost:11434".into() }
fn default_ollama_model() -> String { "codellama:7b".into() }
fn default_openrouter_model() -> String { "anthropic/claude-sonnet-4-20250514".into() }

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
```

Add `dirs-next = "2"` to `forge-agent/Cargo.toml` dependencies.

### Step 5: Create `crates/forge-agent/src/provider.rs`

```rust
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
```

Add `async-trait = "0.1"` and `dirs-next = "2"` to `forge-agent/Cargo.toml`.

### Step 6: Create `crates/forge-agent/src/agent.rs`

```rust
use crate::config::AgentConfig;
use crate::provider::{self, ChatMessage, LlmProvider, LlmResponse};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Agent request from UI to background thread
#[derive(Debug, Clone)]
pub enum AgentRequest {
    Chat {
        message: String,
        context: EditorContext,
    },
    SlashCommand {
        command: String,
        context: EditorContext,
    },
    InlineCompletion {
        context: EditorContext,
    },
    Cancel,
}

/// Editor context sent with each request
#[derive(Debug, Clone, Default)]
pub struct EditorContext {
    pub file_path: Option<String>,
    pub file_content: Option<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub selection: Option<String>,
    pub language: String,
    pub confidence_score: Option<f32>,
}

/// Agent response from background thread to UI
#[derive(Debug, Clone)]
pub enum AgentResponse {
    /// Partial streaming response
    StreamChunk(String),
    /// Complete response
    Complete(LlmResponse),
    /// Inline completion suggestion
    InlineSuggestion(String),
    /// Error occurred
    Error(String),
    /// Agent status changed
    StatusChange(AgentStatus),
}

/// Agent status for status bar
#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Ready,
    Thinking,
    Streaming,
    Error(String),
    Offline,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ready => write!(f, "Ready"),
            Self::Thinking => write!(f, "Thinking..."),
            Self::Streaming => write!(f, "Streaming..."),
            Self::Error(e) => write!(f, "Error: {}", e),
            Self::Offline => write!(f, "Offline"),
        }
    }
}

/// The AI Agent that runs on a background tokio runtime
pub struct Agent {
    /// Channel to send requests to the agent
    pub request_tx: mpsc::UnboundedSender<AgentRequest>,
    /// Channel to receive responses from the agent
    pub response_rx: mpsc::UnboundedReceiver<AgentResponse>,
}

impl Agent {
    /// Create and start the agent on a background thread
    pub fn start() -> Self {
        let (req_tx, mut req_rx) = mpsc::unbounded_channel::<AgentRequest>();
        let (resp_tx, resp_rx) = mpsc::unbounded_channel::<AgentResponse>();

        // Spawn background tokio runtime
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime for AI agent");

            rt.block_on(async move {
                let config = AgentConfig::load();
                let provider = match provider::create_provider(&config) {
                    Ok(p) => p,
                    Err(e) => {
                        let _ = resp_tx.send(AgentResponse::Error(format!("Provider init failed: {}", e)));
                        let _ = resp_tx.send(AgentResponse::StatusChange(AgentStatus::Error(e.to_string())));
                        // Keep thread alive for future requests
                        loop {
                            match req_rx.recv().await {
                                Some(_) => {
                                    let _ = resp_tx.send(AgentResponse::Error(
                                        "No AI provider configured. Edit ~/.forge/agent.toml".into()
                                    ));
                                }
                                None => return,
                            }
                        }
                    }
                };

                let mut history: Vec<ChatMessage> = vec![ChatMessage {
                    role: "system".into(),
                    content: SYSTEM_PROMPT.into(),
                }];

                loop {
                    match req_rx.recv().await {
                        Some(AgentRequest::Chat { message, context }) => {
                            let _ = resp_tx.send(AgentResponse::StatusChange(AgentStatus::Thinking));

                            // Build context-aware user message
                            let user_msg = build_context_message(&message, &context);
                            history.push(ChatMessage {
                                role: "user".into(),
                                content: user_msg,
                            });

                            match provider.chat(&history).await {
                                Ok(response) => {
                                    history.push(ChatMessage {
                                        role: "assistant".into(),
                                        content: response.content.clone(),
                                    });
                                    let _ = resp_tx.send(AgentResponse::Complete(response));
                                    let _ = resp_tx.send(AgentResponse::StatusChange(AgentStatus::Ready));
                                }
                                Err(e) => {
                                    // Remove the failed user message from history
                                    history.pop();
                                    let _ = resp_tx.send(AgentResponse::Error(e.to_string()));
                                    let _ = resp_tx.send(AgentResponse::StatusChange(AgentStatus::Error(e.to_string())));
                                }
                            }
                        }
                        Some(AgentRequest::SlashCommand { command, context }) => {
                            let _ = resp_tx.send(AgentResponse::StatusChange(AgentStatus::Thinking));

                            let prompt = match command.as_str() {
                                "/explain" => format!("Explain this code concisely:\n```{}\n{}\n```",
                                    context.language,
                                    context.selection.as_deref().unwrap_or("(no selection)")),
                                "/fix" => format!("Fix any bugs in this code. Return only the corrected code:\n```{}\n{}\n```",
                                    context.language,
                                    context.selection.as_deref().unwrap_or("(no selection)")),
                                "/test" => format!("Generate unit tests for this code:\n```{}\n{}\n```",
                                    context.language,
                                    context.selection.as_deref().unwrap_or("(no selection)")),
                                "/refactor" => format!("Refactor this code for clarity and performance:\n```{}\n{}\n```",
                                    context.language,
                                    context.selection.as_deref().unwrap_or("(no selection)")),
                                "/doc" => format!("Generate documentation comments for this code:\n```{}\n{}\n```",
                                    context.language,
                                    context.selection.as_deref().unwrap_or("(no selection)")),
                                _ => format!("Unknown command: {}", command),
                            };

                            let messages = vec![
                                ChatMessage { role: "system".into(), content: SYSTEM_PROMPT.into() },
                                ChatMessage { role: "user".into(), content: prompt },
                            ];

                            match provider.chat(&messages).await {
                                Ok(response) => {
                                    let _ = resp_tx.send(AgentResponse::Complete(response));
                                    let _ = resp_tx.send(AgentResponse::StatusChange(AgentStatus::Ready));
                                }
                                Err(e) => {
                                    let _ = resp_tx.send(AgentResponse::Error(e.to_string()));
                                    let _ = resp_tx.send(AgentResponse::StatusChange(AgentStatus::Ready));
                                }
                            }
                        }
                        Some(AgentRequest::InlineCompletion { context }) => {
                            // Lightweight completion request
                            let prompt = format!(
                                "Complete the following code. Return ONLY the completion, no explanation:\n```{}\n{}\n```\nCursor is at line {}, col {}.",
                                context.language,
                                context.file_content.as_deref().unwrap_or(""),
                                context.cursor_line + 1,
                                context.cursor_col + 1,
                            );

                            let messages = vec![
                                ChatMessage { role: "system".into(), content: "You are a code completion engine. Return ONLY the code that should be inserted. No explanations.".into() },
                                ChatMessage { role: "user".into(), content: prompt },
                            ];

                            match provider.chat(&messages).await {
                                Ok(response) => {
                                    let _ = resp_tx.send(AgentResponse::InlineSuggestion(response.content));
                                }
                                Err(_) => {
                                    // Silently fail for inline completions
                                }
                            }
                        }
                        Some(AgentRequest::Cancel) => {
                            let _ = resp_tx.send(AgentResponse::StatusChange(AgentStatus::Ready));
                        }
                        None => return, // Channel closed
                    }
                }
            });
        });

        Self {
            request_tx: req_tx,
            response_rx: resp_rx,
        }
    }

    /// Send a chat message (non-blocking)
    pub fn send_chat(&self, message: String, context: EditorContext) {
        let _ = self.request_tx.send(AgentRequest::Chat { message, context });
    }

    /// Send a slash command (non-blocking)
    pub fn send_slash_command(&self, command: String, context: EditorContext) {
        let _ = self.request_tx.send(AgentRequest::SlashCommand { command, context });
    }

    /// Request inline completion (non-blocking)
    pub fn request_completion(&self, context: EditorContext) {
        let _ = self.request_tx.send(AgentRequest::InlineCompletion { context });
    }

    /// Poll for responses (call every frame, non-blocking)
    pub fn poll_response(&mut self) -> Option<AgentResponse> {
        self.response_rx.try_recv().ok()
    }
}

fn build_context_message(message: &str, context: &EditorContext) -> String {
    let mut parts = Vec::new();

    if let Some(path) = &context.file_path {
        parts.push(format!("File: {}", path));
    }
    parts.push(format!("Language: {}", context.language));
    parts.push(format!("Cursor: line {}, col {}", context.cursor_line + 1, context.cursor_col + 1));

    if let Some(score) = context.confidence_score {
        parts.push(format!("Confidence Score: {:.1}%", score));
    }

    if let Some(sel) = &context.selection {
        parts.push(format!("Selected code:\n```\n{}\n```", sel));
    }

    parts.push(format!("\nUser: {}", message));
    parts.join("\n")
}

const SYSTEM_PROMPT: &str = r#"You are Forge AI, a coding assistant built into the Forge IDE.

Rules:
1. Be concise. Developers hate verbosity.
2. When showing code, use the correct language in fenced code blocks.
3. When fixing bugs, show only the changed lines with context.
4. When explaining, use bullet points.
5. If asked to generate code, return ONLY the code unless explanation was requested.
6. You have access to the user's current file, cursor position, and selection.
7. Respect the user's coding style visible in the file.
8. For Rust code: prefer safe code, avoid unwrap(), use proper error handling.
"#;
```

### Step 7: Create `crates/forge-agent/src/chat.rs`

```rust
/// Chat history for display in AI panel
#[derive(Debug, Clone)]
pub struct ChatHistory {
    pub messages: Vec<ChatDisplayMessage>,
}

#[derive(Debug, Clone)]
pub struct ChatDisplayMessage {
    pub role: ChatRole,
    pub content: String,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatRole {
    User,
    Assistant,
    System,
    Error,
}

impl ChatHistory {
    pub fn new() -> Self {
        Self {
            messages: vec![ChatDisplayMessage {
                role: ChatRole::System,
                content: "Forge AI ready. Type a message or use /explain, /fix, /test, /refactor, /doc".into(),
                timestamp: std::time::SystemTime::now(),
            }],
        }
    }

    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(ChatDisplayMessage {
            role: ChatRole::User,
            content,
            timestamp: std::time::SystemTime::now(),
        });
    }

    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push(ChatDisplayMessage {
            role: ChatRole::Assistant,
            content,
            timestamp: std::time::SystemTime::now(),
        });
    }

    pub fn add_error_message(&mut self, content: String) {
        self.messages.push(ChatDisplayMessage {
            role: ChatRole::Error,
            content,
            timestamp: std::time::SystemTime::now(),
        });
    }
}

impl Default for ChatHistory {
    fn default() -> Self {
        Self::new()
    }
}
```

### Run `cargo check --package forge-agent` — fix ALL errors.

---

## TASK 14: Extension Store (Stubbed)

Extensions require wasmtime which is a heavy dependency. Create the data structures and UI hooks now; WASM runtime can be added later.

### Create `crates/forge-app/src/extensions.rs`

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Extension manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub icon: Option<String>,
}

/// Extension state
#[derive(Debug, Clone, PartialEq)]
pub enum ExtensionState {
    Active,
    Disabled,
    Error(String),
}

/// Loaded extension
#[derive(Debug, Clone)]
pub struct Extension {
    pub manifest: ExtensionManifest,
    pub state: ExtensionState,
    pub install_path: PathBuf,
}

/// Extension registry
pub struct ExtensionRegistry {
    pub installed: Vec<Extension>,
    pub store_path: PathBuf,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        let store_path = dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".forge")
            .join("extensions");

        // Create directory if it doesn't exist
        let _ = std::fs::create_dir_all(&store_path);

        Self {
            installed: Vec::new(),
            store_path,
        }
    }

    /// Load installed extensions from disk
    pub fn load_installed(&mut self) {
        self.installed.clear();

        if let Ok(entries) = std::fs::read_dir(&self.store_path) {
            for entry in entries.flatten() {
                let manifest_path = entry.path().join("forge-ext.toml");
                if manifest_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = toml::from_str::<ExtensionManifest>(&content) {
                            self.installed.push(Extension {
                                manifest,
                                state: ExtensionState::Active,
                                install_path: entry.path(),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Get installed extension count
    pub fn count(&self) -> usize {
        self.installed.len()
    }

    /// List of built-in available extensions (hardcoded for now)
    pub fn available() -> Vec<ExtensionManifest> {
        vec![
            ExtensionManifest {
                id: "forge-ext.word-count".into(),
                name: "Word Count".into(),
                version: "1.0.0".into(),
                author: "Forge".into(),
                description: "Displays word count in status bar".into(),
                icon: None,
            },
            ExtensionManifest {
                id: "forge-ext.bracket-pair".into(),
                name: "Bracket Pair Colorizer".into(),
                version: "1.0.0".into(),
                author: "Forge".into(),
                description: "Colors matching brackets".into(),
                icon: None,
            },
        ]
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

Add `dirs-next = "2"` to `forge-app/Cargo.toml`.

### Update module declarations

```rust
mod extensions;
```

### Run `cargo check --package forge-app` — fix ALL errors.

---

## TASK 15: Adaptive UI Modes

### Create file: `crates/forge-app/src/modes.rs`

```rust
use crate::ui::LayoutConstants;
use serde::{Deserialize, Serialize};

/// UI modes that rearrange the layout for max effectiveness
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiMode {
    Standard,
    Focus,
    Performance,
    Debug,
    Zen,
    Review,
}

/// Layout configuration determined by the mode
#[derive(Clone, Debug)]
pub struct ModeLayoutConfig {
    pub activity_bar: bool,
    pub tab_bar: bool,
    pub breadcrumbs: bool,
    pub gutter: bool,
    pub status_bar: bool,
    pub ai_panel_allowed: bool,
    pub sidebar_allowed: bool,
    pub center_editor: bool,
    pub max_editor_width: Option<f32>,
    pub cursor_blink: bool,
    pub animations: bool,
    pub show_frame_time: bool,
}

impl UiMode {
    pub fn all() -> &'static [UiMode] {
        &[
            UiMode::Standard,
            UiMode::Focus,
            UiMode::Performance,
            UiMode::Debug,
            UiMode::Zen,
            UiMode::Review,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Standard => "🖥️ Standard",
            Self::Focus => "🎯 Focus",
            Self::Performance => "⚡ Perf",
            Self::Debug => "🐛 Debug",
            Self::Zen => "🧘 Zen",
            Self::Review => "📝 Review",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            Self::Standard => "Ctrl+Shift+1",
            Self::Focus => "Ctrl+Shift+F",
            Self::Performance => "Ctrl+Shift+H",
            Self::Debug => "F5",
            Self::Zen => "Ctrl+K Z",
            Self::Review => "Ctrl+Shift+R",
        }
    }

    pub fn layout_config(&self) -> ModeLayoutConfig {
        match self {
            Self::Standard => ModeLayoutConfig {
                activity_bar: true,
                tab_bar: true,
                breadcrumbs: true,
                gutter: true,
                status_bar: true,
                ai_panel_allowed: true,
                sidebar_allowed: true,
                center_editor: false,
                max_editor_width: None,
                cursor_blink: true,
                animations: true,
                show_frame_time: true,
            },
            Self::Focus => ModeLayoutConfig {
                activity_bar: false,
                tab_bar: true,
                breadcrumbs: false,
                gutter: true,
                status_bar: true,
                ai_panel_allowed: false,
                sidebar_allowed: false,
                center_editor: true,
                max_editor_width: Some(800.0),
                cursor_blink: true,
                animations: true,
                show_frame_time: false,
            },
            Self::Performance => ModeLayoutConfig {
                activity_bar: false,
                tab_bar: false,
                breadcrumbs: false,
                gutter: true,
                status_bar: true,
                ai_panel_allowed: false,
                sidebar_allowed: false,
                center_editor: false,
                max_editor_width: None,
                cursor_blink: false,
                animations: false,
                show_frame_time: true,
            },
            Self::Debug => ModeLayoutConfig {
                activity_bar: true,
                tab_bar: true,
                breadcrumbs: false,
                gutter: true,
                status_bar: true,
                ai_panel_allowed: false,
                sidebar_allowed: true,
                center_editor: false,
                max_editor_width: None,
                cursor_blink: true,
                animations: true,
                show_frame_time: true,
            },
            Self::Zen => ModeLayoutConfig {
                activity_bar: false,
                tab_bar: false,
                breadcrumbs: false,
                gutter: false,
                status_bar: false,
                ai_panel_allowed: false,
                sidebar_allowed: false,
                center_editor: true,
                max_editor_width: Some(700.0),
                cursor_blink: true,
                animations: false,
                show_frame_time: false,
            },
            Self::Review => ModeLayoutConfig {
                activity_bar: true,
                tab_bar: true,
                breadcrumbs: true,
                gutter: true,
                status_bar: true,
                ai_panel_allowed: true,
                sidebar_allowed: false,
                center_editor: false,
                max_editor_width: None,
                cursor_blink: true,
                animations: true,
                show_frame_time: false,
            },
        }
    }

    /// Cycle to next mode
    pub fn next(&self) -> Self {
        match self {
            Self::Standard => Self::Focus,
            Self::Focus => Self::Performance,
            Self::Performance => Self::Debug,
            Self::Debug => Self::Zen,
            Self::Zen => Self::Review,
            Self::Review => Self::Standard,
        }
    }

    /// Previous mode
    pub fn prev(&self) -> Self {
        match self {
            Self::Standard => Self::Review,
            Self::Focus => Self::Standard,
            Self::Performance => Self::Focus,
            Self::Debug => Self::Performance,
            Self::Zen => Self::Debug,
            Self::Review => Self::Zen,
        }
    }
}

impl Default for UiMode {
    fn default() -> Self {
        Self::Standard
    }
}
```

### Integration in application.rs:

Add `current_mode: UiMode` to `ForgeApplication` struct. In the render loop, apply the mode's layout config:

```rust
// In render(), before computing layout:
let mode_config = self.current_mode.layout_config();
self.cursor_renderer.set_blink_enabled(mode_config.cursor_blink);
self.status_bar_state.mode_indicator = self.current_mode.label().to_string();

// Compute layout respecting mode
// If mode hides activity bar, set its width to 0, etc.
```

### Update module declarations

```rust
mod modes;
```

### Run `cargo check --package forge-app` — fix ALL errors.

---

## TASK 16: Next-Gen Network Layer

### Step 1: Update workspace Cargo.toml

Add to members:

```toml
"crates/forge-net",
```

Add to `[workspace.dependencies]`:

```toml
rand = "0.8"
```

### Step 2: Create `crates/forge-net/Cargo.toml`

```toml
[package]
name = "forge-net"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
reqwest.workspace = true
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true
rand.workspace = true
```

### Step 3: Create `crates/forge-net/src/lib.rs`

```rust
pub mod client;
pub mod retry;
pub mod health;

pub use client::{ForgeNet, ConnectionState, NetConfig};
pub use retry::RetryPolicy;
```

### Step 4: Create `crates/forge-net/src/retry.rs`

```rust
use std::time::Duration;
use rand::Rng;

/// Retry policy with exponential backoff and decorrelated jitter
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub max_attempts: u32,
}

impl RetryPolicy {
    pub fn default_policy() -> Self {
        Self {
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            max_attempts: 5,
        }
    }

    /// Decorrelated jitter: min(cap, random(base, prev_delay * 3))
    pub fn next_delay(&self, _attempt: u32, prev_delay: Duration) -> Duration {
        let max_ms = self.max_delay.as_millis() as u64;
        let base_ms = self.base_delay.as_millis() as u64;
        let prev_ms = prev_delay.as_millis() as u64;
        let upper = prev_ms.saturating_mul(3).min(max_ms);
        if upper <= base_ms {
            return self.base_delay;
        }
        let delay_ms = rand::thread_rng().gen_range(base_ms..=upper);
        Duration::from_millis(delay_ms)
    }

    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::default_policy()
    }
}
```

### Step 5: Create `crates/forge-net/src/health.rs`

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use crate::ConnectionState;

/// Background health monitor
pub struct HealthMonitor {
    state: Arc<RwLock<ConnectionState>>,
    check_interval: Duration,
}

impl HealthMonitor {
    pub fn new(state: Arc<RwLock<ConnectionState>>, check_interval: Duration) -> Self {
        Self { state, check_interval }
    }

    /// Run health check loop (call from tokio spawn)
    pub async fn run(&self) {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        loop {
            tokio::time::sleep(self.check_interval).await;

            let start = Instant::now();
            let result = client
                .head("https://httpbin.org/status/200")
                .send()
                .await;

            let new_state = match result {
                Ok(resp) if resp.status().is_success() => {
                    let latency = start.elapsed().as_millis() as u32;
                    ConnectionState::Online { latency_ms: latency }
                }
                Ok(_) => ConnectionState::Degraded { error_rate: 0.5 },
                Err(_) => ConnectionState::Offline {
                    since: Instant::now(),
                    queued: 0,
                },
            };

            if let Ok(mut state) = self.state.try_write() {
                *state = new_state;
            }
        }
    }
}
```

### Step 6: Create `crates/forge-net/src/client.rs`

```rust
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use crate::retry::RetryPolicy;
use crate::health::HealthMonitor;
use anyhow::Result;

/// Connection state for status bar display
#[derive(Debug, Clone)]
pub enum ConnectionState {
    Online { latency_ms: u32 },
    Degraded { error_rate: f32 },
    Offline { since: Instant, queued: usize },
    Reconnecting { attempt: u32 },
}

impl ConnectionState {
    pub fn display(&self) -> String {
        match self {
            Self::Online { latency_ms } => format!("🌐 Online {}ms", latency_ms),
            Self::Degraded { error_rate } => format!("⚠️ Degraded {:.0}%", error_rate * 100.0),
            Self::Offline { queued, .. } => format!("🔴 Offline ({} queued)", queued),
            Self::Reconnecting { attempt } => format!("🔄 Retry #{}", attempt),
        }
    }
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Online { latency_ms: 0 }
    }
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetConfig {
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub max_retries: u32,
    pub health_check_interval: Duration,
}

impl Default for NetConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            max_retries: 5,
            health_check_interval: Duration::from_secs(5),
        }
    }
}

/// The network client with auto-retry and health monitoring
pub struct ForgeNet {
    client: reqwest::Client,
    retry_policy: RetryPolicy,
    pub state: Arc<RwLock<ConnectionState>>,
}

impl ForgeNet {
    pub fn new(config: NetConfig) -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(20)
            .pool_idle_timeout(Duration::from_secs(90))
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .gzip(true)
            .brotli(true)
            .tcp_nodelay(true)
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        let state = Arc::new(RwLock::new(ConnectionState::default()));

        // Start health monitor
        let health_state = state.clone();
        let health_interval = config.health_check_interval;
        tokio::spawn(async move {
            let monitor = HealthMonitor::new(health_state, health_interval);
            monitor.run().await;
        });

        Self {
            client,
            retry_policy: RetryPolicy::default(),
            state,
        }
    }

    /// Send a request with automatic retry
    pub async fn request(&self, request: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let mut attempt = 0u32;
        let mut prev_delay = self.retry_policy.base_delay;

        loop {
            // We need to clone the request for each attempt
            // Unfortunately reqwest::RequestBuilder doesn't implement Clone
            // So the caller should pass a closure or use this simpler approach
            match request.try_clone() {
                Some(req) => {
                    match req.send().await {
                        Ok(resp) if resp.status().is_success() => {
                            return Ok(resp);
                        }
                        Ok(resp) if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS => {
                            // Rate limited
                            let retry_after = resp.headers()
                                .get("retry-after")
                                .and_then(|v| v.to_str().ok())
                                .and_then(|v| v.parse::<u64>().ok())
                                .unwrap_or(1);
                            tokio::time::sleep(Duration::from_secs(retry_after)).await;
                            continue;
                        }
                        Ok(resp) => {
                            // Non-retryable error status
                            return Ok(resp);
                        }
                        Err(e) if self.retry_policy.should_retry(attempt) => {
                            attempt += 1;
                            let delay = self.retry_policy.next_delay(attempt, prev_delay);
                            prev_delay = delay;
                            tracing::warn!("Request failed (attempt {}): {}, retrying in {:?}", attempt, e, delay);
                            tokio::time::sleep(delay).await;
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                None => {
                    anyhow::bail!("Request cannot be retried (body was already consumed)");
                }
            }
        }
    }

    /// Get current connection state (non-blocking, for UI)
    pub fn current_state(&self) -> ConnectionState {
        self.state.try_read()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Get the underlying reqwest client for direct use
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}
```

### Run `cargo check --package forge-net` — fix ALL errors.

---

## FINAL STEP: Update workspace Cargo.toml

Make sure ALL new crates are in the workspace members list:

```toml
[workspace]
members = [
    "crates/forge-core",
    "crates/forge-renderer",
    "crates/forge-window",
    "crates/forge-app",
    "crates/forge-config",
    "crates/forge-theme",
    "crates/forge-input",
    "crates/forge-confidence",
    "crates/forge-propagation",
    "crates/forge-semantic",
    "crates/forge-bayesnet",
    "crates/forge-ml",
    "crates/forge-anticipation",
    "crates/forge-immune",
    "crates/forge-developer",
    "crates/forge-surfaces",
    "crates/forge-feedback",
    "crates/forge-agent",
    "crates/forge-net",
]
```

Add ALL new workspace dependencies:

```toml
[workspace.dependencies]
# ... existing deps ...

# AI Agent
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls", "gzip", "brotli"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
async-trait = "0.1"
dirs-next = "2"
rand = "0.8"

# Rectangle Renderer
bytemuck = { version = "1", features = ["derive"] }
```

### Run `cargo check --workspace` — fix ALL errors.
### Run `cargo test --workspace` — fix ALL failures.
### Run `cargo build --release --package forge-app` — must succeed.
