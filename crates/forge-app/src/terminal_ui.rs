use crate::rect_renderer::Rect;
use crate::ui::Zone;
use forge_terminal::grid::TerminalGrid;
use forge_terminal::Terminal;

pub struct TerminalUi {
    pub visible: bool,
    pub height: f32,
}

impl TerminalUi {
    pub fn new() -> Self {
        Self {
            visible: false,
            height: 200.0,
        }
    }

    pub fn render_rects(&self, terminal: &Terminal, zone: &Zone) -> Vec<Rect> {
        // Render background
        let mut rects = vec![Rect {
            x: zone.x,
            y: zone.y,
            width: zone.width,
            height: zone.height,
            color: [0.0, 0.0, 0.0, 1.0], // Black background
        }];

        // Render cursor
        let grid = terminal.render_grid();
        let cursor_x = zone.x + (grid.cursor_col as f32 * 8.0); // Assuming 8px char width
        let cursor_y = zone.y + (grid.cursor_row as f32 * 20.0); // Assuming 20px line height

        if cursor_x < zone.x + zone.width && cursor_y < zone.y + zone.height {
            rects.push(Rect {
                x: cursor_x,
                y: cursor_y,
                width: 8.0,
                height: 20.0,
                color: [0.8, 0.8, 0.8, 0.5], // Semi-transparent cursor
            });
        }

        rects
    }

    // Text rendering is handled by glyphon in Application::render
}

impl Default for TerminalUi {
    fn default() -> Self {
        Self::new()
    }
}
