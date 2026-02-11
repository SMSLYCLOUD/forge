#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSegment {
    pub label: String,
    pub tooltip: String,
}

#[allow(clippy::too_many_arguments)]
pub fn build_segments(
    branch: &str,
    errors: usize,
    warnings: usize,
    line: usize,
    col: usize,
    encoding: &str,
    line_ending: &str,
    language: &str,
) -> Vec<StatusSegment> {
    let mut segments = Vec::new();

    if !branch.is_empty() {
        segments.push(StatusSegment {
            label: format!("Branch: {}", branch),
            tooltip: "Git Branch".to_string(),
        });
    }

    if errors > 0 || warnings > 0 {
        segments.push(StatusSegment {
            label: format!("{}E {}W", errors, warnings),
            tooltip: format!("{} Errors, {} Warnings", errors, warnings),
        });
    }

    segments.push(StatusSegment {
        label: format!("Ln {}, Col {}", line + 1, col + 1),
        tooltip: "Cursor Position".to_string(),
    });

    segments.push(StatusSegment {
        label: encoding.to_string(),
        tooltip: "File Encoding".to_string(),
    });

    segments.push(StatusSegment {
        label: line_ending.to_string(),
        tooltip: "Line Ending".to_string(),
    });

    segments.push(StatusSegment {
        label: language.to_string(),
        tooltip: "Language Mode".to_string(),
    });

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_segments() {
        let segments = build_segments("main", 2, 1, 10, 5, "UTF-8", "LF", "Rust");

        assert_eq!(segments.len(), 6);
        assert_eq!(segments[0].label, "Branch: main");
        assert_eq!(segments[1].label, "2E 1W");
        assert_eq!(segments[2].label, "Ln 11, Col 6");
        assert_eq!(segments[3].label, "UTF-8");
        assert_eq!(segments[4].label, "LF");
        assert_eq!(segments[5].label, "Rust");
    }

    #[test]
    fn test_no_branch_no_errors() {
        let segments = build_segments("", 0, 0, 0, 0, "UTF-8", "LF", "Rust");
        // Branch skipped. Errors skipped.
        // 1. Cursor
        // 2. Encoding
        // 3. Line Ending
        // 4. Language
        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0].label, "Ln 1, Col 1");
    }
}
