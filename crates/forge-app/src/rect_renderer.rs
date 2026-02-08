/// Stub for RectRenderer needed for Part 2 compilation.
#[derive(Clone, Debug, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 4],
}

pub struct RectRenderer;

impl RectRenderer {
    pub fn new(_device: &wgpu::Device, _format: wgpu::TextureFormat) -> Self {
        Self
    }
}
