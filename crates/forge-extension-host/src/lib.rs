use deno_core::{op2, JsRuntime, RuntimeOptions, Extension, Op};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

pub mod protocol;
use protocol::HostMessage;

// Embed the API script
const VSCODE_API: &str = include_str!("api.js");

#[derive(Clone)]
struct HostState {
    outbox: Arc<Mutex<VecDeque<HostMessage>>>,
}

#[op2(fast)]
fn op_show_info(#[state] state: &HostState, #[string] msg: String) {
    state.outbox.lock().unwrap().push_back(HostMessage::ShowInfo { message: msg });
}

pub struct ExtensionHost {
    runtime: JsRuntime,
    outbox: Arc<Mutex<VecDeque<HostMessage>>>,
}

impl ExtensionHost {
    pub fn new() -> Self {
        let outbox = Arc::new(Mutex::new(VecDeque::new()));
        let state = HostState { outbox: outbox.clone() };

        let ext = Extension {
            name: "forge_ext",
            ops: std::borrow::Cow::Borrowed(&[op_show_info::DECL]),
            op_state_fn: Some(Box::new(move |s: &mut deno_core::OpState| {
                s.put(state.clone());
            })),
            ..Default::default()
        };

        let mut runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![ext],
            ..Default::default()
        });

        // Initialize API
        runtime.execute_script("vscode_api.js", VSCODE_API.to_string()).expect("Failed to init VS Code API");
        Self { runtime, outbox }
    }

    pub fn poll_messages(&self) -> Vec<HostMessage> {
        let mut outbox = self.outbox.lock().unwrap();
        outbox.drain(..).collect()
    }

    pub async fn run_script(&mut self, code: &str) -> Result<()> {
        self.runtime.execute_script("<anon>", code.to_string())?;
        self.runtime.run_event_loop(deno_core::PollEventLoopOptions::default()).await?;
        Ok(())
    }
}

impl Default for ExtensionHost {
    fn default() -> Self {
        Self::new()
    }
}
