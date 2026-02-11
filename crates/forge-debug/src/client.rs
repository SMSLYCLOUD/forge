use anyhow::{anyhow, Result};
use serde_json::{json, Value as JsonValue};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tracing::debug;

pub struct DapTransport {
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
}

impl DapTransport {
    pub fn new(stdin: ChildStdin, stdout: BufReader<ChildStdout>) -> Self {
        Self {
            stdin: Arc::new(Mutex::new(stdin)),
            stdout: Arc::new(Mutex::new(stdout)),
        }
    }

    pub async fn send(&self, msg: &JsonValue) -> Result<()> {
        let json_str = serde_json::to_string(msg)?;
        let content_length = json_str.len();
        let header = format!("Content-Length: {}\r\n\r\n", content_length);

        let mut stdin = self.stdin.lock().await;
        stdin.write_all(header.as_bytes()).await?;
        stdin.write_all(json_str.as_bytes()).await?;
        stdin.flush().await?;

        debug!("DAP Sent: {}", json_str);
        Ok(())
    }

    pub async fn receive(&self) -> Result<JsonValue> {
        let mut stdout = self.stdout.lock().await;

        let mut size = None;
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = stdout.read_line(&mut line).await?;
            if bytes_read == 0 {
                return Err(anyhow!("DAP stream closed"));
            }

            if line.trim().is_empty() {
                break;
            }

            let lower = line.to_lowercase();
            if lower.starts_with("content-length:") {
                if let Some(val) = line.split(':').nth(1) {
                    if let Ok(len) = val.trim().parse::<usize>() {
                        size = Some(len);
                    }
                }
            }
        }

        let size = size.ok_or_else(|| anyhow!("Missing Content-Length header"))?;

        let mut body = vec![0u8; size];
        stdout.read_exact(&mut body).await?;

        let json_str = String::from_utf8(body)?;
        debug!("DAP Received: {}", json_str);

        let value: JsonValue = serde_json::from_str(&json_str)?;
        Ok(value)
    }
}

pub struct DebugClient {
    process: Option<Child>,
    transport: Option<DapTransport>,
    seq: i64,
}

impl DebugClient {
    pub fn new() -> Self {
        Self {
            process: None,
            transport: None,
            seq: 1,
        }
    }

    pub fn launch(&mut self, program: &str, args: &[&str]) -> Result<()> {
        // This usually spawns the debug ADAPTER (e.g. mads, codelldb), not the program directly.
        // The adapter then launches the program.
        // Assuming `program` here is the adapter executable.
        let mut process = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = process.stdin.take().ok_or_else(|| anyhow!("No stdin"))?;
        let stdout = BufReader::new(process.stdout.take().ok_or_else(|| anyhow!("No stdout"))?);

        self.transport = Some(DapTransport::new(stdin, stdout));
        self.process = Some(process);

        Ok(())
    }

    pub async fn send_request(&mut self, command: &str, arguments: Option<JsonValue>) -> Result<JsonValue> {
        let transport = self.transport.as_ref().ok_or_else(|| anyhow!("Not connected"))?;

        let req = json!({
            "seq": self.seq,
            "type": "request",
            "command": command,
            "arguments": arguments
        });
        self.seq += 1;

        transport.send(&req).await?;

        // Wait for response (simplified)
        loop {
            let resp = transport.receive().await?;
            if resp["type"] == "response" && resp["request_seq"] == req["seq"] {
                if resp["success"].as_bool() == Some(false) {
                    return Err(anyhow!("DAP Error: {:?}", resp["message"]));
                }
                return Ok(resp["body"].clone());
            }
            // Handle events
        }
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            process.kill().await?;
        }
        self.transport = None;
        Ok(())
    }
}
