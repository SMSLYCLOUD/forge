use forge_debug::DebugClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Breakpoint {
    pub file: String,
    pub line: usize,
    pub verified: bool,
}

#[derive(Clone, Debug)]
pub struct StackFrame {
    pub id: i64,
    pub name: String,
    pub line: usize,
    pub file: String,
}

pub struct DebugUi {
    pub client: Option<Arc<Mutex<DebugClient>>>,
    pub breakpoints: Vec<Breakpoint>,
    pub stack_frames: Vec<StackFrame>,
    pub variables: HashMap<String, String>,
    pub active_frame_id: Option<i64>,
}

impl DebugUi {
    pub fn new() -> Self {
        Self {
            client: None,
            breakpoints: Vec::new(),
            stack_frames: Vec::new(),
            variables: HashMap::new(),
            active_frame_id: None,
        }
    }

    pub fn toggle_breakpoint(&mut self, file: String, line: usize, rt: &Arc<Runtime>) {
        if let Some(idx) = self
            .breakpoints
            .iter()
            .position(|bp| bp.file == file && bp.line == line)
        {
            self.breakpoints.remove(idx);
        } else {
            self.breakpoints.push(Breakpoint {
                file: file.clone(),
                line,
                verified: false,
            });
        }

        if let Some(client) = &self.client {
            let client = client.clone();
            let bps: Vec<_> = self
                .breakpoints
                .iter()
                .filter(|bp| bp.file == file)
                .map(|bp| serde_json::json!({ "line": bp.line }))
                .collect();
            let file_clone = file.clone();

            rt.spawn(async move {
                let mut c = client.lock().await;
                let args = serde_json::json!({
                    "source": { "path": file_clone },
                    "breakpoints": bps
                });
                let _ = c.send_request("setBreakpoints", Some(args)).await;
            });
        }
    }

    pub async fn start_debug(&mut self, program: &str) {
        // Placeholder
        let mut client = DebugClient::new();
        // Assume "codelldb" or similar adapter
        if let Ok(_) = client.launch("codelldb", &["--port", "0"]) {
            // Initialize DAP session
            self.client = Some(Arc::new(Mutex::new(client)));
        }
    }

    pub async fn step_over(&mut self) {
        if let Some(client) = &self.client {
            let client = client.clone();
            // TODO: spawn or return future? for now just hold lock if async
            // But step_over is async, so we can await lock
            let mut c = client.lock().await;
            let _ = c.send_request("next", None).await;
        }
    }

    // UI rendering logic would go here
    pub fn render_text(&self) -> String {
        let mut text = String::new();

        text.push_str("  RUN AND DEBUG\n\n");

        if self.client.is_none() {
            text.push_str("  No active debug session.\n");
            text.push_str("  Press F5 to start (stub).\n");
            return text;
        }

        // Breakpoints
        text.push_str("  BREAKPOINTS\n");
        if self.breakpoints.is_empty() {
             text.push_str("    No breakpoints.\n");
        } else {
            for bp in &self.breakpoints {
                let status = if bp.verified { "●" } else { "○" };
                let path = std::path::Path::new(&bp.file);
                let file = path.file_name().map(|s| s.to_string_lossy()).unwrap_or_default();
                text.push_str(&format!("    {} {}:{}\n", status, file, bp.line));
            }
        }
        text.push_str("\n");

        // Call Stack
        text.push_str("  CALL STACK\n");
        if self.stack_frames.is_empty() {
            text.push_str("    (Paused or Running)\n");
        } else {
            for frame in &self.stack_frames {
                let arrow = if Some(frame.id) == self.active_frame_id { "→" } else { " " };
                text.push_str(&format!("   {} {}\n", arrow, frame.name));
            }
        }
        text.push_str("\n");

        // Variables
        text.push_str("  VARIABLES\n");
        if self.variables.is_empty() {
            text.push_str("    No variables.\n");
        } else {
            for (name, val) in &self.variables {
                text.push_str(&format!("    {}: {}\n", name, val));
            }
        }

        text
    }
}
