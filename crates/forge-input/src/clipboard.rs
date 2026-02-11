use anyhow::Result;

pub struct Clipboard {
    board: arboard::Clipboard,
}

impl Clipboard {
    pub fn new() -> Result<Self> {
        Ok(Self { board: arboard::Clipboard::new().map_err(|e| anyhow::anyhow!("{}", e))? })
    }
    pub fn copy(&mut self, text: &str) -> Result<()> {
        self.board.set_text(text).map_err(|e| anyhow::anyhow!("{}", e))
    }
    pub fn paste(&mut self) -> Result<String> {
        self.board.get_text().map_err(|e| anyhow::anyhow!("{}", e))
    }
}
