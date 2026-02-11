use anyhow::Result;

#[derive(Debug, Clone)]
struct EmmetNode {
    tag: String,
    id: Option<String>,
    classes: Vec<String>,
    children: Vec<EmmetNode>,
    siblings: Vec<EmmetNode>,
    count: usize,
    self_closing: bool,
    text: Option<String>,
}

impl EmmetNode {
    fn new(tag: &str) -> Self {
        let self_closing = matches!(tag, "img" | "br" | "hr" | "input" | "meta" | "link");
        Self {
            tag: tag.to_string(),
            id: None,
            classes: Vec::new(),
            children: Vec::new(),
            siblings: Vec::new(),
            count: 1,
            self_closing,
            text: None,
        }
    }

    fn render(&self) -> String {
        let mut output = String::new();
        for _ in 0..self.count {
            output.push('<');
            output.push_str(&self.tag);

            if let Some(id) = &self.id {
                output.push_str(&format!(" id=\"{}\"", id));
            }

            if !self.classes.is_empty() {
                output.push_str(&format!(" class=\"{}\"", self.classes.join(" ")));
            }

            if self.self_closing && self.children.is_empty() && self.text.is_none() {
                output.push_str(" />");
            } else {
                output.push('>');
                if let Some(text) = &self.text {
                    output.push_str(text);
                }
                for child in &self.children {
                    output.push_str(&child.render());
                }
                output.push_str(&format!("</{}>", self.tag));
            }
        }

        for sibling in &self.siblings {
            output.push_str(&sibling.render());
        }

        output
    }
}

pub fn expand_abbreviation(abbr: &str) -> Result<String> {
    if abbr.is_empty() {
        return Ok(String::new());
    }

    let (node, remainder) = parse_node(abbr)?;
    let mut root = node;
    let mut current_remainder = remainder;

    while let Some(rem) = current_remainder {
        if rem.starts_with('+') {
            let (sibling, next_rem) = parse_node(&rem[1..])?;
            root.siblings.push(sibling);
            current_remainder = next_rem;
        } else {
            break;
        }
    }

    Ok(root.render())
}

fn parse_node(input: &str) -> Result<(EmmetNode, Option<String>)> {
    let mut chars = input.chars().peekable();
    let mut tag_part = String::new();

    while let Some(&c) = chars.peek() {
        if c == '>' || c == '+' || c == '*' || c == '{' {
            break;
        }
        tag_part.push(c);
        chars.next();
    }

    if tag_part.is_empty() {
        // If the abbreviation starts with `.` or `#`, we treat it as implicit div.
        // But the loop above would consume them into tag_part.
        // If tag_part is empty, it means we hit a special char immediately?
        // e.g. `>...` or `+...`
        anyhow::bail!("Invalid abbreviation: empty tag");
    }

    let (tag, id, classes) = parse_tag_part(&tag_part);
    let mut node = EmmetNode::new(&tag);
    node.id = id;
    node.classes = classes;

    // Check for Text `{...}`
    if let Some(&c) = chars.peek() {
        if c == '{' {
            chars.next(); // consume {
            let mut text = String::new();
            while let Some(&tc) = chars.peek() {
                if tc == '}' {
                    chars.next();
                    break;
                }
                text.push(tc);
                chars.next();
            }
            node.text = Some(text);
        }
    }

    // Check for Multiplication `*`
    if let Some(&c) = chars.peek() {
        if c == '*' {
            chars.next(); // consume *
            let mut count_str = String::new();
            while let Some(&dc) = chars.peek() {
                if dc.is_ascii_digit() {
                    count_str.push(dc);
                    chars.next();
                } else {
                    break;
                }
            }
            if let Ok(c) = count_str.parse::<usize>() {
                node.count = c;
            }
        }
    }

    // Check for Children `>`
    if let Some(&c) = chars.peek() {
        if c == '>' {
            chars.next(); // consume >
            let rest: String = chars.collect();

            // Recursive call for children
            let (child, remainder) = parse_node(&rest)?;
            let mut current_child = child;
            let mut child_rem = remainder;

            // Handle siblings of the child (nested structure)
            // e.g. div>p+span
            // p is child. span is sibling of p (so also child of div).
            // parse_node returns p. remainder is "+span".

            // We need to attach these siblings to `current_child.siblings`?
            // Yes, because `render` iterates siblings.
            // If `div` has child `p`. And `p` has sibling `span`.
            // `div.render()` calls `p.render()`.
            // `p.render()` calls `span.render()`.
            // Result: `<div><p></p><span></span></div>`. Correct.

            while let Some(rem) = child_rem {
                if rem.starts_with('+') {
                    let (sibling, next_rem) = parse_node(&rem[1..])?;
                    current_child.siblings.push(sibling);

                    // NOTE: We keep appending to the FIRST child's siblings list.
                    // A -> B -> C
                    // This assumes parse_node returns single node.
                    // But if we have `div>p+span+a`.
                    // `parse_node("p+span+a")` -> returns `p`, remainder `+span+a`.
                    // Loop 1: `parse_node("span+a")` -> returns `span`, remainder `+a`.
                    // `p.siblings.push(span)`.
                    // Loop 2: `parse_node("a")` -> returns `a`, remainder None.
                    // `p.siblings.push(a)`.
                    // `p.siblings` = `[span, a]`.
                    // `p.render()` -> renders `p`, then `span`, then `a`.
                    // Correct.

                    child_rem = next_rem;
                } else {
                    child_rem = Some(rem);
                    break;
                }
            }

            node.children.push(current_child);
            return Ok((node, child_rem));
        }
    }

    let rest: String = chars.collect();
    let remainder = if rest.is_empty() { None } else { Some(rest) };

    Ok((node, remainder))
}

fn parse_tag_part(input: &str) -> (String, Option<String>, Vec<String>) {
    let mut tag = String::new();
    let mut id = None;
    let mut classes = Vec::new();

    let mut current_token = String::new();
    let mut mode = 't'; // t=tag, c=class, i=id

    for c in input.chars() {
        if c == '.' || c == '#' {
            if !current_token.is_empty() {
                match mode {
                    't' => tag = current_token.clone(),
                    'c' => classes.push(current_token.clone()),
                    'i' => id = Some(current_token.clone()),
                    _ => {}
                }
                current_token.clear();
            }
            mode = if c == '.' { 'c' } else { 'i' };
        } else {
            current_token.push(c);
        }
    }

    if !current_token.is_empty() {
        match mode {
            't' => tag = current_token,
            'c' => classes.push(current_token),
            'i' => id = Some(current_token),
            _ => {}
        }
    }

    if tag.is_empty() {
        tag = "div".to_string();
    }

    (tag, id, classes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tag() {
        assert_eq!(expand_abbreviation("div").unwrap(), "<div></div>");
        assert_eq!(expand_abbreviation("p").unwrap(), "<p></p>");
    }

    #[test]
    fn test_class_id() {
        assert_eq!(
            expand_abbreviation("div.foo").unwrap(),
            "<div class=\"foo\"></div>"
        );
        assert_eq!(
            expand_abbreviation("div#bar").unwrap(),
            "<div id=\"bar\"></div>"
        );
        assert_eq!(
            expand_abbreviation("div.foo#bar").unwrap(),
            "<div id=\"bar\" class=\"foo\"></div>"
        );
        // Implicit div
        assert_eq!(
            expand_abbreviation(".foo").unwrap(),
            "<div class=\"foo\"></div>"
        );
        assert_eq!(
            expand_abbreviation("#bar").unwrap(),
            "<div id=\"bar\"></div>"
        );
    }

    #[test]
    fn test_child() {
        assert_eq!(
            expand_abbreviation("div>span").unwrap(),
            "<div><span></span></div>"
        );
        assert_eq!(expand_abbreviation("ul>li").unwrap(), "<ul><li></li></ul>");
    }

    #[test]
    fn test_sibling() {
        assert_eq!(expand_abbreviation("div+p").unwrap(), "<div></div><p></p>");
    }

    #[test]
    fn test_multiply() {
        assert_eq!(
            expand_abbreviation("ul>li*3").unwrap(),
            "<ul><li></li><li></li><li></li></ul>"
        );
    }

    #[test]
    fn test_self_closing() {
        assert_eq!(expand_abbreviation("img").unwrap(), "<img />");
    }

    #[test]
    fn test_complex() {
        // div>p+span
        assert_eq!(
            expand_abbreviation("div>p+span").unwrap(),
            "<div><p></p><span></span></div>"
        );
    }

    #[test]
    fn test_text() {
        assert_eq!(expand_abbreviation("p{hello}").unwrap(), "<p>hello</p>");
    }
}
