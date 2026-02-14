use crate::rect_renderer::Rect;
use crate::ui::Zone;
use forge_icons::UiIcon;

pub struct DebugToolbar {
    pub visible: bool,
    pub x: f32,
    pub y: f32,
}

impl DebugToolbar {
    pub fn new() -> Self {
        Self {
            visible: false,
            x: 0.0,
            y: 0.0,
        }
    }

    pub fn render_rects(&self, width: f32) -> Vec<Rect> {
        let mut rects = Vec::new();
        // Centered floating toolbar
        let tb_width = 200.0;
        let tb_height = 36.0;
        let x = (width - tb_width) / 2.0;
        let y = 40.0; // Top margin

        // Background
        rects.push(Rect {
            x,
            y,
            width: tb_width,
            height: tb_height,
            color: [0.18, 0.18, 0.18, 1.0], // Dark gray
        });

        // Button placeholders (hover effects would be added here)
        // 5 buttons: Continue, StepOver, StepInto, StepOut, Stop
        // Width 32px each, spacing

        rects
    }

    pub fn render_text(&self, width: f32, theme: &forge_theme::Theme) -> Vec<(String, f32, f32, [f32; 4])> {
        let mut items = Vec::new();
        let tb_width = 200.0;
        let x_start = (width - tb_width) / 2.0;
        let y = 48.0;

        let icons = ["▷", "↷", "↘", "↗", "□"]; // Unicode fallbacks or Codicons if available
        // Continue, StepOver, StepInto, StepOut, Stop

        let color = [0.8, 0.8, 0.8, 1.0];

        for (i, icon) in icons.iter().enumerate() {
            let x = x_start + 20.0 + (i as f32 * 35.0);
            items.push((icon.to_string(), x, y, color));
        }

        items
    }
}
