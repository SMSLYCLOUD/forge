#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub message: String,
    pub severity: Severity,
}

pub struct ProblemsPanel {
    pub diagnostics: Vec<Diagnostic>,
}

impl Default for ProblemsPanel {
    fn default() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }
}

impl ProblemsPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, d: Diagnostic) {
        self.diagnostics.push(d);
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    pub fn filter(&self, severity: Severity) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == severity)
            .collect()
    }

    pub fn count_by_severity(&self) -> (usize, usize, usize, usize) {
        let mut errors = 0;
        let mut warnings = 0;
        let mut infos = 0;
        let mut hints = 0;

        for d in &self.diagnostics {
            match d.severity {
                Severity::Error => errors += 1,
                Severity::Warning => warnings += 1,
                Severity::Info => infos += 1,
                Severity::Hint => hints += 1,
            }
        }

        (errors, warnings, infos, hints)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_count() {
        let mut panel = ProblemsPanel::new();
        panel.add(Diagnostic {
            file: "main.rs".to_string(),
            line: 1,
            col: 1,
            message: "Error".to_string(),
            severity: Severity::Error,
        });
        panel.add(Diagnostic {
            file: "lib.rs".to_string(),
            line: 2,
            col: 5,
            message: "Warning".to_string(),
            severity: Severity::Warning,
        });

        let (e, w, i, h) = panel.count_by_severity();
        assert_eq!(e, 1);
        assert_eq!(w, 1);
        assert_eq!(i, 0);
        assert_eq!(h, 0);
    }

    #[test]
    fn test_filter() {
        let mut panel = ProblemsPanel::new();
        panel.add(Diagnostic {
            file: "main.rs".to_string(),
            line: 1,
            col: 1,
            message: "Error".to_string(),
            severity: Severity::Error,
        });

        let errors = panel.filter(Severity::Error);
        assert_eq!(errors.len(), 1);

        let warnings = panel.filter(Severity::Warning);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut panel = ProblemsPanel::new();
        panel.add(Diagnostic {
            file: "main.rs".to_string(),
            line: 1,
            col: 1,
            message: "Error".to_string(),
            severity: Severity::Error,
        });

        panel.clear();
        assert!(panel.diagnostics.is_empty());
    }
}
