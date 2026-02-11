#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoToLine {
    pub visible: bool,
    pub input: String,
}

impl Default for GoToLine {
    fn default() -> Self {
        Self {
            visible: false,
            input: String::new(),
        }
    }
}

impl GoToLine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.visible = true;
        self.input.clear();
    }

    pub fn cancel(&mut self) {
        self.visible = false;
        self.input.clear();
    }

    pub fn type_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn confirm(&self) -> Option<(usize, Option<usize>)> {
        if self.input.is_empty() {
            return None;
        }

        let parts: Vec<&str> = self.input.split(':').collect();

        let line_str = parts[0].trim();
        if let Ok(line) = line_str.parse::<usize>() {
            let line_idx = if line > 0 { line - 1 } else { 0 };

            if parts.len() > 1 {
                let col_str = parts[1].trim();
                if let Ok(col) = col_str.parse::<usize>() {
                    let col_idx = if col > 0 { col - 1 } else { 0 };
                    return Some((line_idx, Some(col_idx)));
                }
            }

            return Some((line_idx, None));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line() {
        let mut gtl = GoToLine::new();
        gtl.input = "10".to_string();

        let res = gtl.confirm();
        assert_eq!(res, Some((9, None)));
    }

    #[test]
    fn test_parse_line_col() {
        let mut gtl = GoToLine::new();
        gtl.input = "10:5".to_string();

        let res = gtl.confirm();
        assert_eq!(res, Some((9, Some(4))));
    }

    #[test]
    fn test_invalid() {
        let mut gtl = GoToLine::new();
        gtl.input = "abc".to_string();
        assert_eq!(gtl.confirm(), None);
    }
}
