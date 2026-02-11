//! Forge Terminal — embedded terminal emulator with PTY support.

pub mod ansi;
pub mod grid;
pub mod pty;
pub mod shell;

use anyhow::Result;
use ansi::{AnsiParser, TermEvent};
use grid::TerminalGrid;
use pty::Pty;
use shell::detect_shell;

pub struct Terminal {
    pub pty: Pty,
    pub parser: AnsiParser,
    pub grid: TerminalGrid,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let shell = detect_shell();
        let cols = 80;
        let rows = 24;
        let pty = Pty::spawn(&shell, cols, rows)?;
        let parser = AnsiParser::new();
        let grid = TerminalGrid::new(cols, rows);

        Ok(Self {
            pty,
            parser,
            grid,
        })
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.pty.resize(cols, rows)?;
        self.grid.resize(cols, rows);
        Ok(())
    }

    pub fn send_input(&mut self, text: &str) -> Result<()> {
        self.pty.write(text.as_bytes())
    }

    pub fn tick(&mut self) -> Vec<TermEvent> {
        let data = self.pty.read();
        if data.is_empty() {
            return Vec::new();
        }

        let events = self.parser.parse(&data);
        for event in &events {
            self.apply_event(event);
        }
        events
    }

    fn apply_event(&mut self, event: &TermEvent) {
        match event {
            TermEvent::Print(c) => self.grid.write_char(*c),
            TermEvent::SetFg(_color) => {
                // Map color to RGB
                // For now, simplify or map properly
                // self.grid.current_fg = ...
            }
            TermEvent::SetBg(_color) => {
                // self.grid.current_bg = ...
            }
            TermEvent::MoveCursor(row, col) => {
                self.grid.cursor_row = (*row as u16).saturating_sub(1).min(self.grid.rows - 1);
                self.grid.cursor_col = (*col as u16).saturating_sub(1).min(self.grid.cols - 1);
            }
            TermEvent::CursorUp(n) => {
                self.grid.cursor_row = self.grid.cursor_row.saturating_sub(*n as u16);
            }
            TermEvent::CursorDown(n) => {
                self.grid.cursor_row = (self.grid.cursor_row + *n as u16).min(self.grid.rows - 1);
            }
            TermEvent::CursorForward(n) => {
                self.grid.cursor_col = (self.grid.cursor_col + *n as u16).min(self.grid.cols - 1);
            }
            TermEvent::CursorBack(n) => {
                self.grid.cursor_col = self.grid.cursor_col.saturating_sub(*n as u16);
            }
            TermEvent::ClearLine(mode) => self.grid.clear_line(*mode),
            TermEvent::ClearScreen(_) => self.grid.clear_screen(),
            TermEvent::ScrollUp(n) => {
                for _ in 0..*n {
                    self.grid.scroll_up();
                }
            }
            TermEvent::ScrollDown(_) => {
                // TODO: Scroll down (reverse scroll)
            }
            TermEvent::Bell => {}
            TermEvent::Reset => {
                // Reset colors
            }
        }
    }

    pub fn render_grid(&self) -> &TerminalGrid {
        &self.grid
    }
}
