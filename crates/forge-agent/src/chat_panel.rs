use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    pub timestamp: SystemTime,
}

pub struct ChatPanel {
    pub visible: bool,
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub model: String,
}

impl Default for ChatPanel {
    fn default() -> Self {
        Self {
            visible: false,
            messages: Vec::new(),
            input: String::new(),
            model: "gpt-4".to_string(),
        }
    }
}

impl ChatPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.visible = true;
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn type_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn backspace(&mut self) {
        self.input.pop();
    }

    pub fn send(&mut self) -> Option<String> {
        if self.input.trim().is_empty() {
            return None;
        }

        let content = self.input.clone();
        self.messages.push(ChatMessage {
            role: ChatRole::User,
            content: content.clone(),
            timestamp: SystemTime::now(),
        });
        self.input.clear();
        Some(content)
    }

    pub fn add_response(&mut self, text: String) {
        self.messages.push(ChatMessage {
            role: ChatRole::Assistant,
            content: text,
            timestamp: SystemTime::now(),
        });
    }

    pub fn clear_history(&mut self) {
        self.messages.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_returns_message_and_clears_input() {
        let mut panel = ChatPanel::new();
        panel.input = "Hello AI".to_string();

        let msg = panel.send();
        assert!(msg.is_some());
        assert_eq!(msg.unwrap(), "Hello AI");
        assert!(panel.input.is_empty());
        assert_eq!(panel.messages.len(), 1);
        assert_eq!(panel.messages[0].role, ChatRole::User);
        assert_eq!(panel.messages[0].content, "Hello AI");
    }

    #[test]
    fn test_messages_accumulate() {
        let mut panel = ChatPanel::new();
        panel.input = "Q1".to_string();
        panel.send();
        panel.add_response("A1".to_string());

        assert_eq!(panel.messages.len(), 2);
        assert_eq!(panel.messages[0].content, "Q1");
        assert_eq!(panel.messages[1].content, "A1");
        assert_eq!(panel.messages[1].role, ChatRole::Assistant);
    }

    #[test]
    fn test_clear_works() {
        let mut panel = ChatPanel::new();
        panel.input = "Hi".to_string();
        panel.send();
        assert!(!panel.messages.is_empty());

        panel.clear_history();
        assert!(panel.messages.is_empty());
    }

    #[test]
    fn test_input_manipulation() {
        let mut panel = ChatPanel::new();
        panel.type_char('a');
        panel.type_char('b');
        assert_eq!(panel.input, "ab");
        panel.backspace();
        assert_eq!(panel.input, "a");
    }

    #[test]
    fn test_visibility_toggle() {
        let mut panel = ChatPanel::new();
        assert!(!panel.visible);
        panel.open();
        assert!(panel.visible);
        panel.close();
        assert!(!panel.visible);
        panel.toggle();
        assert!(panel.visible);
    }
}
