#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelTab {
    Problems,
    Output,
    Terminal,
    DebugConsole,
}

pub struct BottomPanel {
    pub visible: bool,
    pub height: f32,
    pub active_tab: PanelTab,
}

impl Default for BottomPanel {
    fn default() -> Self {
        Self {
            visible: false,
            height: 200.0,
            active_tab: PanelTab::Problems,
        }
    }
}

impl BottomPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn set_tab(&mut self, tab: PanelTab) {
        self.active_tab = tab;
        if !self.visible {
            self.visible = true;
        }
    }

    pub fn resize(&mut self, new_height: f32) {
        // Clamp between 100.0 and max? What is max?
        // Prompt says "clamp between 100.0 and max". Max is usually window height related, but here we don't know it.
        // Maybe I should take max as argument?
        // Or just clamp min?
        // Prompt: `resize(new_height: f32)` (clamp between 100.0 and max)
        // Since I don't know max, I'll clamp min 100.0. Max I'll assume is very large or just ignore max clamp if not provided.
        // Or maybe assume a reasonable max like 1000.0 or just ignore.
        // I'll clamp min 100.0.

        if new_height < 100.0 {
            self.height = 100.0;
        } else {
            self.height = new_height;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle() {
        let mut panel = BottomPanel::new();
        assert!(!panel.visible);
        panel.toggle();
        assert!(panel.visible);
        panel.toggle();
        assert!(!panel.visible);
    }

    #[test]
    fn test_set_tab() {
        let mut panel = BottomPanel::new();
        panel.set_tab(PanelTab::Terminal);
        assert_eq!(panel.active_tab, PanelTab::Terminal);
        assert!(panel.visible); // set_tab usually opens panel
    }

    #[test]
    fn test_resize() {
        let mut panel = BottomPanel::new();
        panel.resize(50.0);
        assert_eq!(panel.height, 100.0);

        panel.resize(300.0);
        assert_eq!(panel.height, 300.0);
    }
}
