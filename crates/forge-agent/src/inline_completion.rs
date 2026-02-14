#[derive(Debug, Clone, PartialEq)]
pub struct InlineCompletion {
    pub text: String,
    pub line: usize,
    pub col: usize,
    pub ghost: bool,
}

pub struct InlineCompletionProvider {
    pub visible: bool,
    pub completion: Option<InlineCompletion>,
    pub debounce_ms: u64,
}

impl Default for InlineCompletionProvider {
    fn default() -> Self {
        Self {
            visible: false,
            completion: None,
            debounce_ms: 300,
        }
    }
}

impl InlineCompletionProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn suggest(&mut self, context: &str, line: usize, col: usize) -> Option<InlineCompletion> {
        // Extract surrounding context (10 lines before, 5 after)
        let lines: Vec<&str> = context.lines().collect();
        let start_line = line.saturating_sub(10);
        let end_line = std::cmp::min(line + 5, lines.len());

        let _context_snippet = if start_line < end_line {
            lines[start_line..end_line].join("\n")
        } else {
            String::new()
        };

        // TODO: Call AI service (e.g. Gemini/Ollama) here with the prompt
        // For now, return a stub suggestion based on context
        let suggestion = if !context.is_empty() {
            Some(InlineCompletion {
                text: "println!(\"Hello World\");".to_string(),
                line,
                col,
                ghost: true,
            })
        } else {
            None
        };

        self.completion = suggestion.clone();
        self.visible = self.completion.is_some();
        suggestion
    }

    pub fn accept(&mut self) -> Option<String> {
        if let Some(completion) = &self.completion {
            let text = completion.text.clone();
            self.completion = None;
            self.visible = false;
            Some(text)
        } else {
            None
        }
    }

    pub fn dismiss(&mut self) {
        self.completion = None;
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_creates_completion() {
        let mut provider = InlineCompletionProvider::new();
        let context = "fn main() {\n    \n}";
        let completion = provider.suggest(context, 1, 4);

        assert!(completion.is_some());
        let completion = completion.unwrap();
        assert_eq!(completion.line, 1);
        assert_eq!(completion.col, 4);
        assert!(completion.ghost);
        assert!(provider.is_visible());
    }

    #[test]
    fn test_accept_returns_text_and_clears() {
        let mut provider = InlineCompletionProvider::new();
        let context = "fn main() {\n    \n}";
        provider.suggest(context, 1, 4);

        assert!(provider.is_visible());

        let accepted = provider.accept();
        assert!(accepted.is_some());
        assert_eq!(accepted.unwrap(), "println!(\"Hello World\");");

        assert!(!provider.is_visible());
        assert!(provider.completion.is_none());
    }

    #[test]
    fn test_dismiss_clears_completion() {
        let mut provider = InlineCompletionProvider::new();
        let context = "fn main() {\n    \n}";
        provider.suggest(context, 1, 4);

        assert!(provider.is_visible());

        provider.dismiss();

        assert!(!provider.is_visible());
        assert!(provider.completion.is_none());
    }
}
