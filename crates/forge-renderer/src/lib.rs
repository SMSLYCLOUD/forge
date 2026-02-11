//! forge-renderer: GPU-accelerated text rendering engine
//!
//! Uses wgpu for cross-platform GPU acceleration and cosmic-text for text shaping.

mod atlas;
pub mod decorations;
mod pipeline;
mod text;
mod theme;
mod viewport;

pub use atlas::GlyphAtlas;
pub use pipeline::RenderPipeline;
pub use text::TextRenderer;
pub use theme::{Color, Theme};
pub use viewport::Viewport;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
