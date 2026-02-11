//! Forge Editor — main entry point
//!
//! Opens a GPU-accelerated native window with a text editor.

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod application;
mod editor;
mod extensions;
pub mod file_tree_ui;
pub mod tab_manager;
pub mod selection_render;
mod gpu;
mod modes;

// UI components
mod activity_bar;
mod breadcrumb;
mod cursor;
mod guard;
mod gutter;
mod organism;
mod rect_renderer;
mod scrollbar;
mod status_bar;
mod tab_bar;
mod ui;

// Session 2 - New UI Components
pub mod bottom_panel;
pub mod bracket_match;
pub mod breadcrumb_dropdown;
pub mod code_folding;
pub mod command_palette;
pub mod comment_toggle;
pub mod context_menu;
pub mod file_picker;
pub mod find_bar;
pub mod go_to_line;
pub mod indent_guides;
pub mod minimap;
pub mod multicursor;
pub mod notifications;
pub mod output_panel;
pub mod problems_panel;
pub mod replace_bar;
pub mod status_segments;
pub mod title_bar;
pub mod word_wrap;

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
