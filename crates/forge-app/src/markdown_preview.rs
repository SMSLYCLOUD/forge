#[derive(Debug, Clone, PartialEq)]
pub enum MarkdownNode {
    Heading { level: u8, text: String },
    Paragraph(String),
    CodeBlock { language: String, code: String },
    ListItem(String),
    Bold(String),
    Italic(String),
    Link { text: String, url: String },
    HorizontalRule,
}

pub fn parse_markdown(text: &str) -> Vec<MarkdownNode> {
    let mut nodes = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            let language = trimmed.trim_start_matches("```").trim().to_string();
            let mut code = String::new();
            i += 1;
            while i < lines.len() {
                let code_line = lines[i];
                if code_line.trim().starts_with("```") {
                    break;
                }
                code.push_str(code_line);
                code.push('\n');
                i += 1;
            }
            nodes.push(MarkdownNode::CodeBlock { language, code });
        } else if trimmed.starts_with('#') {
            let mut level = 0;
            for c in trimmed.chars() {
                if c == '#' {
                    level += 1;
                } else {
                    break;
                }
            }
            if level > 0 && level <= 6 {
                let text = trimmed[level..].trim().to_string();
                nodes.push(MarkdownNode::Heading {
                    level: level as u8,
                    text,
                });
            } else {
                nodes.push(MarkdownNode::Paragraph(trimmed.to_string()));
            }
        } else if trimmed.starts_with("- ") {
            nodes.push(MarkdownNode::ListItem(trimmed[2..].to_string()));
        } else if trimmed == "---" {
            nodes.push(MarkdownNode::HorizontalRule);
        } else if !trimmed.is_empty() {
            // Very basic parsing for bold, italic, link
            // Note: This is a simplistic parser as requested, not a full MD parser
            if trimmed.len() >= 4 && trimmed.starts_with("**") && trimmed.ends_with("**") {
                nodes.push(MarkdownNode::Bold(
                    trimmed[2..trimmed.len() - 2].to_string(),
                ));
            } else if trimmed.len() >= 2 && trimmed.starts_with('*') && trimmed.ends_with('*') {
                nodes.push(MarkdownNode::Italic(
                    trimmed[1..trimmed.len() - 1].to_string(),
                ));
            } else if trimmed.starts_with('[') && trimmed.contains("](") && trimmed.ends_with(')') {
                // simple link parser
                if let Some(close_bracket) = trimmed.find(']') {
                    if let Some(open_paren) = trimmed.find('(') {
                        if close_bracket < open_paren {
                            let text = trimmed[1..close_bracket].to_string();
                            let url = trimmed[open_paren + 1..trimmed.len() - 1].to_string();
                            nodes.push(MarkdownNode::Link { text, url });
                            i += 1;
                            continue;
                        }
                    }
                }
                nodes.push(MarkdownNode::Paragraph(trimmed.to_string()));
            } else {
                nodes.push(MarkdownNode::Paragraph(trimmed.to_string()));
            }
        }
        i += 1;
    }

    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headings_parsed() {
        let md = "# Heading 1\n## Heading 2";
        let nodes = parse_markdown(md);
        assert_eq!(nodes.len(), 2);
        match &nodes[0] {
            MarkdownNode::Heading { level, text } => {
                assert_eq!(*level, 1);
                assert_eq!(text, "Heading 1");
            }
            _ => panic!("Expected Heading"),
        }
        match &nodes[1] {
            MarkdownNode::Heading { level, text } => {
                assert_eq!(*level, 2);
                assert_eq!(text, "Heading 2");
            }
            _ => panic!("Expected Heading"),
        }
    }

    #[test]
    fn test_code_blocks_parsed() {
        let md = "```rust\nfn main() {}\n```";
        let nodes = parse_markdown(md);
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            MarkdownNode::CodeBlock { language, code } => {
                assert_eq!(language, "rust");
                assert_eq!(code, "fn main() {}\n");
            }
            _ => panic!("Expected CodeBlock"),
        }
    }

    #[test]
    fn test_bold_italic_parsed() {
        let md = "**bold**\n*italic*";
        let nodes = parse_markdown(md);
        assert_eq!(nodes.len(), 2);
        match &nodes[0] {
            MarkdownNode::Bold(text) => assert_eq!(text, "bold"),
            _ => panic!("Expected Bold"),
        }
        match &nodes[1] {
            MarkdownNode::Italic(text) => assert_eq!(text, "italic"),
            _ => panic!("Expected Italic"),
        }
    }

    #[test]
    fn test_list_items() {
        let md = "- item 1\n- item 2";
        let nodes = parse_markdown(md);
        assert_eq!(nodes.len(), 2);
        match &nodes[0] {
            MarkdownNode::ListItem(text) => assert_eq!(text, "item 1"),
            _ => panic!("Expected ListItem"),
        }
    }
}
