use glyphon::{Attrs, Color, Family, Style, Weight};

pub fn parse_markdown<'a>(text: &str, base_attrs: Attrs<'a>) -> Vec<(String, Attrs<'a>)> {
    let mut spans = Vec::new();
    let mut current_text = String::new();
    let mut chars = text.chars().peekable();

    // Simple state machine for bold/italic/code
    // Limitations: No nesting support in this basic version, focused on chat bubbles.

    while let Some(c) = chars.next() {
        if c == '*' {
            if chars.peek() == Some(&'*') {
                // Bold start/end
                chars.next(); // consume second *
                if !current_text.is_empty() {
                    spans.push((current_text.clone(), base_attrs));
                    current_text.clear();
                }

                // Read until next **
                let mut bold_text = String::new();
                while let Some(bc) = chars.next() {
                    if bc == '*' && chars.peek() == Some(&'*') {
                        chars.next(); // consume
                        break;
                    }
                    bold_text.push(bc);
                }
                spans.push((bold_text, base_attrs.weight(Weight::BOLD)));
            } else {
                // Italic start/end
                if !current_text.is_empty() {
                    spans.push((current_text.clone(), base_attrs));
                    current_text.clear();
                }

                let mut italic_text = String::new();
                while let Some(ic) = chars.next() {
                    if ic == '*' { break; }
                    italic_text.push(ic);
                }
                spans.push((italic_text, base_attrs.style(Style::Italic)));
            }
        } else if c == '`' {
            // Code block or inline code
            // Simple check: if next two are ``, it's a block
            let is_block = if chars.peek() == Some(&'`') {
                chars.next();
                if chars.peek() == Some(&'`') {
                    chars.next();
                    true
                } else {
                    false // just double backtick? treat as text for now or empty inline
                }
            } else {
                false
            };

            if !current_text.is_empty() {
                spans.push((current_text.clone(), base_attrs));
                current_text.clear();
            }

            let mut code_text = String::new();
            if is_block {
                // Read until ```
                while let Some(cc) = chars.next() {
                    if cc == '`' {
                        if chars.peek() == Some(&'`') {
                            chars.next();
                            if chars.peek() == Some(&'`') {
                                chars.next();
                                break;
                            } else {
                                code_text.push('`');
                                code_text.push('`');
                            }
                        } else {
                            code_text.push(cc);
                        }
                    } else {
                        code_text.push(cc);
                    }
                }
            } else {
                // Inline `
                while let Some(cc) = chars.next() {
                    if cc == '`' { break; }
                    code_text.push(cc);
                }
            }

            // Render code with monospace font and slight color change
            let code_attrs = base_attrs
                .family(Family::Monospace)
                .color(Color::rgb(206, 145, 120)); // VS Code orange-ish string color
            spans.push((code_text, code_attrs));

        } else {
            current_text.push(c);
        }
    }

    if !current_text.is_empty() {
        spans.push((current_text, base_attrs));
    }

    spans
}
