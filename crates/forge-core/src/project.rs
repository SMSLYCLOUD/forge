use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Project {
    /// Root directory of the project
    pub root: PathBuf,
    /// Known files in the project
    pub files: Vec<PathBuf>,
}

impl Project {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            files: Vec::new(),
        }
    }

    /// Scan the project directory for files
    pub fn scan(&mut self) {
        self.files.clear();
        // Basic recursive scan
        self.visit_dir(&self.root.clone());
    }

    fn visit_dir(&mut self, dir: &Path) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if !self.is_ignored(&path) {
                        self.visit_dir(&path);
                    }
                } else {
                    self.files.push(path);
                }
            }
        }
    }

    fn is_ignored(&self, path: &Path) -> bool {
        // Basic ignore logic
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            return name_str.starts_with('.') || name_str == "target" || name_str == "node_modules";
        }
        false
    }
}
