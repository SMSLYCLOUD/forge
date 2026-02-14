pub mod client;

pub use client::DebugClient;

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub value: String,
    pub type_name: String,
    pub children: Vec<Variable>,
    pub expanded: bool,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
    pub variables: Vec<Variable>,
    pub expanded: bool,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub id: usize,
    pub name: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct DebugSession {
    pub active: bool,
    pub scopes: Vec<Scope>,
    pub stack_frames: Vec<StackFrame>,
    pub current_frame_idx: Option<usize>,
}

impl DebugSession {
    pub fn new() -> Self {
        Self {
            active: false,
            scopes: Vec::new(),
            stack_frames: Vec::new(),
            current_frame_idx: None,
        }
    }

    pub fn mock() -> Self {
        let vars = vec![
            Variable {
                name: "counter".into(),
                value: "42".into(),
                type_name: "i32".into(),
                children: vec![],
                expanded: false,
            },
            Variable {
                name: "config".into(),
                value: "Struct".into(),
                type_name: "Config".into(),
                children: vec![
                    Variable { name: "debug".into(), value: "true".into(), type_name: "bool".into(), children: vec![], expanded: false },
                    Variable { name: "port".into(), value: "8080".into(), type_name: "u16".into(), children: vec![], expanded: false },
                ],
                expanded: true,
            },
        ];

        Self {
            active: true,
            scopes: vec![Scope { name: "Local".into(), variables: vars, expanded: true }],
            stack_frames: vec![
                StackFrame { id: 1, name: "main".into(), file: "main.rs".into(), line: 20 },
                StackFrame { id: 2, name: "init".into(), file: "lib.rs".into(), line: 45 },
            ],
            current_frame_idx: Some(0),
        }
    }
}

impl Default for DebugSession {
    fn default() -> Self {
        Self::new()
    }
}
