use forge_debug::DebugClient;
use std::collections::HashMap;

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
    pub client: Option<DebugClient>,
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

    pub fn toggle_breakpoint(&mut self, file: String, line: usize) {
        if let Some(idx) = self.breakpoints.iter().position(|bp| bp.file == file && bp.line == line) {
            self.breakpoints.remove(idx);
        } else {
            self.breakpoints.push(Breakpoint {
                file,
                line,
                verified: false,
            });
        }
        // TODO: Send setBreakpoints request to DAP
    }

    pub async fn start_debug(&mut self, program: &str) {
        // Placeholder
        let mut client = DebugClient::new();
        // Assume "codelldb" or similar adapter
        if let Ok(_) = client.launch("codelldb", &["--port", "0"]) {
             // Initialize DAP session
             self.client = Some(client);
        }
    }

    pub async fn step_over(&mut self) {
        if let Some(client) = &mut self.client {
            // client.send_request("next", ...).await;
        }
    }

    // UI rendering logic would go here
}
