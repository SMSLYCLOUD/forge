use lsp_types::{Diagnostic, DiagnosticSeverity, PublishDiagnosticsParams};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct DiagnosticHandler {
    pub diagnostics: HashMap<PathBuf, Vec<Diagnostic>>,
}

impl DiagnosticHandler {
    pub fn new() -> Self {
        Self {
            diagnostics: HashMap::new(),
        }
    }

    pub fn handle_publish_diagnostics(&mut self, params: PublishDiagnosticsParams) {
        if let Ok(url) = url::Url::parse(params.uri.as_str()) {
            if let Ok(path) = url.to_file_path() {
                self.diagnostics.insert(path, params.diagnostics);
            }
        }
    }

    pub fn get_diagnostics(&self, path: &PathBuf) -> Option<&Vec<Diagnostic>> {
        self.diagnostics.get(path)
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics.values().flatten().filter(|d| d.severity == Some(DiagnosticSeverity::ERROR)).count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics.values().flatten().filter(|d| d.severity == Some(DiagnosticSeverity::WARNING)).count()
    }
}
