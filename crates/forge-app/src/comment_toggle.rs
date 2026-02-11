#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    JavaScript,
    Python,
    Html,
    Css,
    Go,
    Other,
}

pub struct CommentToggler;

impl CommentToggler {
    fn get_comment_syntax(lang: Language) -> (&'static str, Option<&'static str>) {
        match lang {
            Language::Rust | Language::JavaScript | Language::Go | Language::Other => ("//", None),
            Language::Python => ("#", None),
            Language::Html => ("<!--", Some("-->")),
            Language::Css => ("/*", Some("*/")),
        }
    }

    pub fn toggle_line(line: &str, lang: Language) -> String {
        let (start_token, end_token) = Self::get_comment_syntax(lang);
        let trimmed = line.trim_start();

        // Check if line is commented
        if trimmed.starts_with(start_token) {
            // Uncomment
            // Remove start token
            // If end token exists, remove it from end

            // We need to preserve indentation?
            // "   // code" -> "   code"
            // "   //code" -> "   code"

            // Find where comment starts in original line
            if let Some(idx) = line.find(start_token) {
                let prefix = &line[..idx];
                let content_with_end = &line[idx + start_token.len()..];

                // Remove optional space after start token
                let content = content_with_end
                    .strip_prefix(' ')
                    .unwrap_or(content_with_end);

                // Remove end token if present
                let final_content = if let Some(end) = end_token {
                    if content.trim_end().ends_with(end) {
                        // Remove end token
                        let end_idx = content.rfind(end).unwrap_or(content.len());
                        let mut inner = &content[..end_idx];
                        if inner.ends_with(' ') {
                            inner = &inner[..inner.len() - 1];
                        }
                        inner
                    } else {
                        content
                    }
                } else {
                    content
                };

                return format!("{}{}", prefix, final_content);
            }
        }

        // Comment
        // "   code" -> "   // code"
        // Preserve indentation
        let indent_len = line.len() - trimmed.len();
        let indent = &line[..indent_len];
        let content = trimmed;

        if let Some(end) = end_token {
            format!("{}{}{}{}{}{}", indent, start_token, " ", content, " ", end)
        } else {
            format!("{}{}{}{}", indent, start_token, " ", content)
        }
    }

    pub fn toggle_block(lines: &[&str], lang: Language) -> Vec<String> {
        let (start_token, end_token) = Self::get_comment_syntax(lang);

        let all_commented = lines
            .iter()
            .all(|line| line.trim().is_empty() || line.trim_start().starts_with(start_token));

        lines
            .iter()
            .map(|line| {
                if all_commented {
                    if line.trim().is_empty() {
                        line.to_string()
                    } else {
                        Self::toggle_line(line, lang)
                    }
                } else {
                    let is_commented = line.trim_start().starts_with(start_token);
                    if is_commented {
                        // Re-implement comment logic to double comment
                        let indent_len = line.len() - line.trim_start().len();
                        let indent = &line[..indent_len];
                        let content = line.trim_start();

                        if let Some(e) = end_token {
                            format!("{}{}{}{}{}{}", indent, start_token, " ", content, " ", e)
                        } else {
                            format!("{}{}{}{}", indent, start_token, " ", content)
                        }
                    } else {
                        Self::toggle_line(line, lang)
                    }
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_line_rust() {
        let line = "    let x = 1;";
        let commented = CommentToggler::toggle_line(line, Language::Rust);
        assert_eq!(commented, "    // let x = 1;");

        let uncommented = CommentToggler::toggle_line(&commented, Language::Rust);
        assert_eq!(uncommented, "    let x = 1;");
    }

    #[test]
    fn test_toggle_line_html() {
        let line = "<div>";
        let commented = CommentToggler::toggle_line(line, Language::Html);
        assert!(commented.contains("<!-- <div> -->")); // exact spacing might vary

        let uncommented = CommentToggler::toggle_line(&commented, Language::Html);
        assert_eq!(uncommented, "<div>");
    }

    #[test]
    fn test_toggle_block() {
        let lines = vec!["a", "b"];
        // Comment all
        let commented = CommentToggler::toggle_block(&lines, Language::Rust);
        assert_eq!(commented[0], "// a");
        assert_eq!(commented[1], "// b");

        // Uncomment all
        let uncommented = CommentToggler::toggle_block(
            &commented.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            Language::Rust,
        );
        assert_eq!(uncommented[0], "a");
        assert_eq!(uncommented[1], "b");
    }
}
