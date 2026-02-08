use crate::rect_renderer::Rect;

/// Stub for UI constants and structures needed for Part 2 compilation.

pub mod colors {
    pub const CURRENT_LINE: [f32; 4] = [0.165, 0.176, 0.18, 1.0];
    pub const ERROR: [f32; 4] = [0.937, 0.325, 0.314, 1.0];
    pub const WARNING: [f32; 4] = [0.804, 0.682, 0.263, 1.0];
    pub const STATUS_BAR: [f32; 4] = [0.0, 0.478, 0.8, 1.0];
    pub const TEXT_DIM: [f32; 4] = [0.522, 0.522, 0.522, 1.0];
    pub const TEXT_WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    pub const SUCCESS: [f32; 4] = [0.345, 0.663, 0.369, 1.0];
    pub const CURSOR: [f32; 4] = [0.682, 0.686, 0.678, 1.0];
    pub const SELECTION: [f32; 4] = [0.149, 0.31, 0.471, 0.5];
    pub const TEXT_FG: [f32; 4] = [0.8, 0.8, 0.8, 1.0];
}

pub struct LayoutConstants;

impl LayoutConstants {
    pub const LINE_HEIGHT: f32 = 20.0;
    pub const CHAR_WIDTH: f32 = 8.4;
    pub const SMALL_FONT_SIZE: f32 = 12.0;
}

#[derive(Clone, Debug, Default)]
pub struct Zone {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Zone {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}
