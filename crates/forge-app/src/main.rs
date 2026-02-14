//! Forge Editor — main entry point
//!
//! Opens a GPU-accelerated native window with a text editor.

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod application;
mod editor;
mod extensions;
pub mod file_explorer;
pub mod file_tree_ui;
mod gpu;
mod modes;
pub mod selection_render;
pub mod tab_manager;

mod accessibility;
mod emmet;
mod image_preview;
mod markdown_preview;
mod terminal_tabs;

// UI components
mod activity_bar;
mod autocomplete;
mod breadcrumb;
mod cursor;
mod debug_ui;
mod drag_drop;
mod editor_groups;
mod extensions_panel;
mod formatter;
mod guard;
mod gutter;
mod hover_info;
mod organism;
mod param_hints;
mod rect_renderer;
mod rename_symbol;
mod scrollbar;
mod settings_ui;
mod snippets;
mod status_bar;
mod tab_bar;
mod task_runner;
mod ui;
mod zen_mode;

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

// Session 3 - Terminal + Git + Search
pub mod diff_view;
pub mod git_blame;
pub mod git_branch;
pub mod git_gutter;
pub mod git_panel;
pub mod go_to_def;
pub mod outline_panel;
pub mod references;
pub mod search_panel;
pub mod terminal_ui;
pub mod workspace_symbols;

use anyhow::{anyhow, Result};
use tracing::info;
use winit::event_loop::EventLoop;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("🔥 Forge Editor starting...");

    // Determine file to open from command line args
    let mut args = std::env::args().skip(1).peekable();
    let mut file_path = None;
    let mut screenshot_path = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("Usage: forge [FILE] [--screenshot [PATH]]");
                println!("  FILE                    Optional file path to open");
                println!("  --screenshot [PATH]     Render one frame and save as PNG");
                return Ok(());
            }
            "--screenshot" => {
                if let Some(next) = args.peek() {
                    if next.starts_with("--") {
                        screenshot_path = Some("screenshot.png".to_string());
                    } else {
                        screenshot_path = args.next();
                    }
                } else {
                    screenshot_path = Some("screenshot.png".to_string());
                }
            }
            _ if arg.starts_with("--") => {
                return Err(anyhow!("Unknown argument: {}", arg));
            }
            _ => {
                if file_path.is_none() {
                    file_path = Some(arg);
                } else {
                    return Err(anyhow!("Only one input file is supported"));
                }
            }
        }
    }

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = application::Application::new(file_path, screenshot_path);

    event_loop.run_app(&mut app)?;

    Ok(())
}
