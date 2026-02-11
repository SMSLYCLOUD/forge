//! PTY (Pseudo-Terminal) abstraction using `portable-pty`.

use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, PtySize, native_pty_system, PtyPair, Child};
use std::io::{Read, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, mpsc,
};
use std::thread;

/// Represents a running PTY process.
pub struct Pty {
    pub pair: PtyPair,
    pub writer: Box<dyn Write + Send>,
    pub rx: mpsc::Receiver<Vec<u8>>,
    pub alive: Arc<AtomicBool>,
    pub child: Box<dyn Child + Send + Sync>,
}

impl Pty {
    /// Spawns a new PTY with the given command and size.
    pub fn spawn(command: &str, cols: u16, rows: u16) -> Result<Self> {
        let system = native_pty_system();

        let pair = system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        }).context("Failed to open PTY")?;

        let cmd = CommandBuilder::new(command);
        let child = pair.slave.spawn_command(cmd).context("Failed to spawn command")?;

        let writer = pair.master.take_writer().context("Failed to take writer")?;
        let mut reader = pair.master.try_clone_reader().context("Failed to clone reader")?;

        let (tx, rx) = mpsc::channel();
        let alive = Arc::new(AtomicBool::new(true));
        let alive_clone = alive.clone();

        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        // EOF
                        alive_clone.store(false, Ordering::Relaxed);
                        break;
                    }
                    Ok(n) => {
                        let data = buf[0..n].to_vec();
                        if tx.send(data).is_err() {
                            break;
                        }
                    }
                    Err(_) => {
                        alive_clone.store(false, Ordering::Relaxed);
                        break;
                    }
                }
            }
        });

        Ok(Self {
            pair,
            writer,
            rx,
            alive,
            child,
        })
    }

    /// Writes data to the PTY.
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.writer.write_all(data).context("Failed to write to PTY")
    }

    /// Reads pending data from the PTY (non-blocking).
    pub fn read(&self) -> Vec<u8> {
        let mut data = Vec::new();
        while let Ok(chunk) = self.rx.try_recv() {
            data.extend_from_slice(&chunk);
        }
        data
    }

    /// Resizes the PTY.
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.pair
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to resize PTY")
    }

    /// Checks if the PTY process is still alive.
    pub fn is_alive(&mut self) -> bool {
        // If the reader thread signaled exit, return false
        if !self.alive.load(Ordering::Relaxed) {
            return false;
        }

        // Also check process status if possible
        if let Ok(Some(_)) = self.child.try_wait() {
            self.alive.store(false, Ordering::Relaxed);
            return false;
        }

        true
    }

    /// Kills the PTY process.
    pub fn kill(&mut self) -> Result<()> {
        self.child.kill().context("Failed to kill PTY process")
    }
}
