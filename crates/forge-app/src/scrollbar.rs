use crate::rect_renderer::Rect;
use crate::ui::{colors, LayoutConstants, Zone};

/// Vertical scrollbar state and rendering
pub struct Scrollbar {
    /// Whether mouse is over scrollbar
    pub hovered: bool,
    /// Whether scrollbar is being dragged
    pub dragging: bool,
    /// Drag start Y position
    drag_start_y: f32,
    drag_start_scroll: usize,
}

impl Scrollbar {
    pub fn new() -> Self {
        Self {
            hovered: false,
            dragging: false,
            drag_start_y: 0.0,
            drag_start_scroll: 0,
        }
    }

    /// Calculate scrollbar thumb dimensions
    fn thumb_geometry(
        &self,
        zone: &Zone,
        total_lines: usize,
        visible_lines: usize,
        scroll_top: usize,
    ) -> (f32, f32) {
        if total_lines <= visible_lines {
            return (zone.y, zone.height);
        }

        let ratio = visible_lines as f32 / total_lines as f32;
        let thumb_height = (zone.height * ratio).max(30.0); // Minimum 30px
        let scroll_ratio = scroll_top as f32 / (total_lines - visible_lines) as f32;
        let thumb_y = zone.y + scroll_ratio * (zone.height - thumb_height);

        (thumb_y, thumb_height)
    }

    /// Generate scrollbar rectangle
    pub fn render_rect(
        &self,
        zone: &Zone,
        total_lines: usize,
        visible_lines: usize,
        scroll_top: usize,
    ) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(2);

        // Track background
        rects.push(Rect {
            x: zone.x,
            y: zone.y,
            width: zone.width,
            height: zone.height,
            color: [0.15, 0.15, 0.15, 0.3],
        });

        // Thumb
        let (thumb_y, thumb_height) =
            self.thumb_geometry(zone, total_lines, visible_lines, scroll_top);
        let thumb_color = if self.dragging {
            [0.5, 0.5, 0.5, 0.8]
        } else if self.hovered {
            [0.4, 0.4, 0.4, 0.7]
        } else {
            colors::SCROLLBAR
        };

        rects.push(Rect {
            x: zone.x + 2.0,
            y: thumb_y,
            width: zone.width - 4.0,
            height: thumb_height,
            color: thumb_color,
        });

        rects
    }

    /// Start dragging
    pub fn start_drag(&mut self, mouse_y: f32, scroll_top: usize) {
        self.dragging = true;
        self.drag_start_y = mouse_y;
        self.drag_start_scroll = scroll_top;
    }

    /// Update during drag, returns new scroll_top
    pub fn update_drag(
        &self,
        mouse_y: f32,
        zone: &Zone,
        total_lines: usize,
        visible_lines: usize,
    ) -> usize {
        if total_lines <= visible_lines {
            return 0;
        }

        let (_, thumb_height) =
            self.thumb_geometry(zone, total_lines, visible_lines, self.drag_start_scroll);
        let delta_y = mouse_y - self.drag_start_y;
        let scroll_range = zone.height - thumb_height;
        if scroll_range <= 0.0 {
            return 0;
        }

        let delta_scroll = (delta_y / scroll_range * (total_lines - visible_lines) as f32) as isize;
        let new_scroll = (self.drag_start_scroll as isize + delta_scroll).max(0) as usize;
        new_scroll.min(total_lines.saturating_sub(visible_lines))
    }

    /// Stop dragging
    pub fn stop_drag(&mut self) {
        self.dragging = false;
    }
}

impl Default for Scrollbar {
    fn default() -> Self {
        Self::new()
    }
}
