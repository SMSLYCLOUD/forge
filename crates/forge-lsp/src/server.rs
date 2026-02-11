use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tracing::{error, info};

pub struct LspServer {
    process: Child,
    pub stdin: Option<ChildStdin>,
    pub stdout: Option<BufReader<ChildStdout>>,
}

impl LspServer {
    pub fn spawn(command: &str, args: &[&str]) -> Result<Self> {
        info!("Spawning LSP server: {} {:?}", command, args);
        let mut process = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()) // Log stderr?
            .spawn()
            .with_context(|| format!("Failed to spawn LSP server: {}", command))?;

        let stdin = process.stdin.take();
        let stdout = process.stdout.take().map(BufReader::new);

        Ok(Self {
            process,
            stdin,
            stdout,
        })
    }

    pub fn kill(&mut self) -> Result<()> {
        self.process.start_kill()?;
        Ok(())
    }

    pub async fn is_alive(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(_) => false,
        }
    }
}
