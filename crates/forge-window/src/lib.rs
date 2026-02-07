//! forge-window: Platform abstraction for windowing and events
//!
//! Wraps winit for cross-platform window management and input handling.

mod event_loop;
mod input;
mod window;

pub use event_loop::EventLoop;
pub use input::InputHandler;
pub use window::ForgeWindow;
