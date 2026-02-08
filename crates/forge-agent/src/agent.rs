use crate::config::AgentConfig;
use crate::provider::{self, ChatMessage, LlmResponse};
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
