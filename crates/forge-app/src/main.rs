//! Forge Editor — main entry point
//!
//! Opens a GPU-accelerated native window with a text editor.

mod application;
mod editor;
mod gpu;
mod extensions;
mod modes;

// Part 1
mod rect_renderer;
mod ui;
mod tab_bar;
mod activity_bar;

// Part 2
mod rect_renderer;
mod ui;
mod gutter;
mod status_bar;
mod cursor;
mod breadcrumb;

// Part 3
mod guard;
mod scrollbar;
mod organism;
mod tab_bar;
mod activity_bar;

use anyhow::Result;
use tracing::info;
use winit::event_loop::EventLoop;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("🔥 Forge Editor starting...");

    // Determine file to open from command line args
    let file_path = std::env::args().nth(1);

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = application::Application::new(file_path);

    event_loop.run_app(&mut app)?;

    Ok(())
}
