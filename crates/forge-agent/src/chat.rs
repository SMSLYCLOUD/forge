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
                content:
                    "Forge AI ready. Type a message or use /explain, /fix, /test, /refactor, /doc"
                        .into(),
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
