use anyhow::{anyhow, Result};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::Mutex;
use tracing::debug;

pub struct Transport {
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
}

impl Transport {
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

        debug!("LSP Sent: {}", json_str);
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
                return Err(anyhow!("LSP stream closed"));
            }

            // Check for empty line (end of headers)
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
        debug!("LSP Received: {}", json_str);

        let value: JsonValue = serde_json::from_str(&json_str)?;
        Ok(value)
    }
}
