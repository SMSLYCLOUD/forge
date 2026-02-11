use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Task {
    pub label: String,
    pub command: String,
    pub group: Option<String>,
    pub problem_matcher: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaskConfig {
    pub tasks: Vec<Task>,
}

pub struct TaskRunner {
    pub tasks: Vec<Task>,
}

impl TaskRunner {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn load_tasks(&mut self, config: TaskConfig) {
        self.tasks = config.tasks;
    }

    pub fn run_task(&self, label: &str) -> Result<()> {
        if let Some(task) = self.tasks.iter().find(|t| t.label == label) {
            // Placeholder: In a real app, this would spawn a terminal process
            // and pipe output to a panel.
            tracing::info!("Running task: {}", task.label);

            // Just spawn a detached process for now
            // Splitting command string is naive but sufficient for placeholder
            let parts: Vec<&str> = task.command.split_whitespace().collect();
            if let Some((cmd, args)) = parts.split_first() {
                Command::new(cmd).args(args).spawn()?;
            }
        }
        Ok(())
    }
}
