#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputChannel {
    pub name: String,
    pub lines: Vec<String>,
}

pub struct OutputPanel {
    pub channels: Vec<OutputChannel>,
    pub active: usize,
}

impl Default for OutputPanel {
    fn default() -> Self {
        Self {
            channels: Vec::new(),
            active: 0,
        }
    }
}

impl OutputPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_channel(&mut self, name: &str) {
        self.channels.push(OutputChannel {
            name: name.to_string(),
            lines: Vec::new(),
        });
    }

    pub fn append(&mut self, channel_idx: usize, text: &str) {
        if let Some(channel) = self.channels.get_mut(channel_idx) {
            channel.lines.push(text.to_string());
        }
    }

    pub fn clear(&mut self, channel_idx: usize) {
        if let Some(channel) = self.channels.get_mut(channel_idx) {
            channel.lines.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_append() {
        let mut panel = OutputPanel::new();
        panel.create_channel("Build");
        assert_eq!(panel.channels.len(), 1);

        panel.append(0, "Compiling...");
        assert_eq!(panel.channels[0].lines.len(), 1);
        assert_eq!(panel.channels[0].lines[0], "Compiling...");
    }

    #[test]
    fn test_clear() {
        let mut panel = OutputPanel::new();
        panel.create_channel("Log");
        panel.append(0, "Log 1");
        panel.clear(0);
        assert!(panel.channels[0].lines.is_empty());
    }
}
