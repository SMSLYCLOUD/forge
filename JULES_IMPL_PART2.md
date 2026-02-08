# JULES IMPLEMENTATION — PART 2 OF 4
# Tasks 5-8: Editor Gutter, Status Bar, Cursor Rendering, Breadcrumb Bar

> **CRITICAL**: Complete Part 1 first. Run `cargo check --package forge-app` after every file.

---

## TASK 5: Editor Gutter (Line Numbers)

### Create file: `crates/forge-app/src/gutter.rs`

```rust
use crate::rect_renderer::Rect;
use crate::ui::{colors, LayoutConstants, Zone};

/// Editor gutter — line numbers + diagnostics indicators
pub struct Gutter {
    /// First visible line (0-indexed)
    pub scroll_top: usize,
    /// Total lines in the file
    pub total_lines: usize,
    /// Current cursor line (0-indexed)
    pub cursor_line: usize,
    /// Lines with errors (line number → severity)
    pub diagnostics: Vec<(usize, DiagnosticSeverity)>,
    /// Lines with breakpoints
    pub breakpoints: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

impl Gutter {
    pub fn new() -> Self {
        Self {
            scroll_top: 0,
            total_lines: 1,
            cursor_line: 0,
            diagnostics: Vec::new(),
            breakpoints: Vec::new(),
        }
    }

    /// Calculate how many lines fit in the visible area
    pub fn visible_lines(zone: &Zone) -> usize {
        (zone.height / LayoutConstants::LINE_HEIGHT).floor() as usize
    }

    /// Generate rectangles for gutter decorations
    pub fn render_rects(&self, zone: &Zone) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(32);
        let visible = Self::visible_lines(zone);

        for i in 0..visible {
            let line = self.scroll_top + i;
            if line >= self.total_lines {
                break;
            }
            let y = zone.y + (i as f32 * LayoutConstants::LINE_HEIGHT);

            // Current line highlight
            if line == self.cursor_line {
                rects.push(Rect {
                    x: zone.x,
                    y,
                    width: zone.width,
                    height: LayoutConstants::LINE_HEIGHT,
                    color: colors::CURRENT_LINE,
                });
            }

            // Breakpoint indicator (red circle area)
            if self.breakpoints.contains(&line) {
                rects.push(Rect {
                    x: zone.x + 4.0,
                    y: y + 3.0,
                    width: 14.0,
                    height: 14.0,
                    color: colors::ERROR,
                });
            }

            // Diagnostic indicator (colored bar on the right edge of gutter)
            if let Some((_, severity)) = self.diagnostics.iter().find(|(l, _)| *l == line) {
                let color = match severity {
                    DiagnosticSeverity::Error => colors::ERROR,
                    DiagnosticSeverity::Warning => colors::WARNING,
                    DiagnosticSeverity::Info => colors::STATUS_BAR,
                    DiagnosticSeverity::Hint => colors::TEXT_DIM,
                };
                rects.push(Rect {
                    x: zone.x + zone.width - 3.0,
                    y,
                    width: 3.0,
                    height: LayoutConstants::LINE_HEIGHT,
                    color,
                });
            }
        }

        rects
    }

    /// Get line number text positions (returns (text, x, y, is_current_line))
    pub fn text_positions(&self, zone: &Zone) -> Vec<(String, f32, f32, bool)> {
        let visible = Self::visible_lines(zone);
        let mut result = Vec::with_capacity(visible);
        let line_num_width = format!("{}", self.total_lines).len();

        for i in 0..visible {
            let line = self.scroll_top + i;
            if line >= self.total_lines {
                break;
            }
            let text = format!("{:>width$}", line + 1, width = line_num_width);
            let x = zone.x + 20.0; // After breakpoint area
            let y = zone.y + (i as f32 * LayoutConstants::LINE_HEIGHT) + 2.0;
            let is_current = line == self.cursor_line;
            result.push((text, x, y, is_current));
        }

        result
    }

    /// Toggle breakpoint on a line
    pub fn toggle_breakpoint(&mut self, line: usize) {
        if let Some(idx) = self.breakpoints.iter().position(|l| *l == line) {
            self.breakpoints.remove(idx);
        } else {
            self.breakpoints.push(line);
        }
    }

    /// Handle click in gutter (toggle breakpoint)
    pub fn handle_click(&mut self, click_y: f32, zone: &Zone) -> Option<usize> {
        let relative_y = click_y - zone.y;
        if relative_y < 0.0 {
            return None;
        }
        let line_index = (relative_y / LayoutConstants::LINE_HEIGHT) as usize;
        let line = self.scroll_top + line_index;
        if line < self.total_lines {
            self.toggle_breakpoint(line);
            Some(line)
        } else {
            None
        }
    }
}

impl Default for Gutter {
    fn default() -> Self {
        Self::new()
    }
}
```

### Update module declarations

```rust
mod gutter;
```

### Run `cargo check --package forge-app` — fix ALL errors.

---

## TASK 6: Status Bar

### Create file: `crates/forge-app/src/status_bar.rs`

```rust
use crate::rect_renderer::Rect;
use crate::ui::{colors, LayoutConstants, Zone};

/// A single item displayed in the status bar
#[derive(Clone, Debug)]
pub struct StatusItem {
    pub text: String,
    pub tooltip: String,
    pub color: Option<[f32; 4]>,
    pub alignment: StatusAlignment,
    pub priority: i32, // Higher = more important, gets rendered first
    pub click_action: Option<StatusAction>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatusAlignment {
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub enum StatusAction {
    ToggleAiPanel,
    ToggleSidebar,
    CycleMode,
    OpenCommandPalette,
    ShowNotifications,
    SelectLanguage,
    SelectEncoding,
    SelectLineEnding,
}

/// Status bar state and rendering
pub struct StatusBar {
    pub items: Vec<StatusItem>,
    /// Current cursor line (1-indexed for display)
    pub cursor_line: usize,
    /// Current cursor column (1-indexed for display)
    pub cursor_col: usize,
    /// File encoding
    pub encoding: String,
    /// File language
    pub language: String,
    /// Line ending style
    pub line_ending: String,
    /// Git branch name
    pub git_branch: Option<String>,
    /// Frame time in ms
    pub frame_time_ms: f32,
    /// Confidence score (0-100)
    pub confidence_score: Option<f32>,
    /// AI agent status
    pub ai_status: String,
    /// Network status
    pub network_status: String,
    /// Current UI mode
    pub mode_indicator: String,
    /// Error count
    pub error_count: usize,
    /// Warning count
    pub warning_count: usize,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            cursor_line: 1,
            cursor_col: 1,
            encoding: String::from("UTF-8"),
            language: String::from("Plain Text"),
            line_ending: String::from("LF"),
            git_branch: None,
            frame_time_ms: 0.0,
            confidence_score: None,
            ai_status: String::from("Ready"),
            network_status: String::from("🌐 Online"),
            mode_indicator: String::from("🖥️ Standard"),
            error_count: 0,
            warning_count: 0,
        }
    }

    /// Build the ordered list of status items
    pub fn build_items(&self) -> Vec<StatusItem> {
        let mut items = Vec::with_capacity(16);

        // LEFT SIDE items

        // Git branch
        if let Some(ref branch) = self.git_branch {
            items.push(StatusItem {
                text: format!("⎇ {}", branch),
                tooltip: format!("Git Branch: {}", branch),
                color: None,
                alignment: StatusAlignment::Left,
                priority: 100,
                click_action: None,
            });
        }

        // Errors and warnings
        if self.error_count > 0 || self.warning_count > 0 {
            items.push(StatusItem {
                text: format!("✕ {}  ⚠ {}", self.error_count, self.warning_count),
                tooltip: format!("{} errors, {} warnings", self.error_count, self.warning_count),
                color: if self.error_count > 0 { Some(colors::ERROR) } else { None },
                alignment: StatusAlignment::Left,
                priority: 90,
                click_action: Some(StatusAction::ShowNotifications),
            });
        }

        // Mode indicator
        items.push(StatusItem {
            text: self.mode_indicator.clone(),
            tooltip: String::from("Click to change UI mode"),
            color: None,
            alignment: StatusAlignment::Left,
            priority: 80,
            click_action: Some(StatusAction::CycleMode),
        });

        // Network status
        items.push(StatusItem {
            text: self.network_status.clone(),
            tooltip: String::from("Network connection status"),
            color: None,
            alignment: StatusAlignment::Left,
            priority: 70,
            click_action: None,
        });

        // RIGHT SIDE items

        // Cursor position
        items.push(StatusItem {
            text: format!("Ln {}, Col {}", self.cursor_line, self.cursor_col),
            tooltip: String::from("Go to Line"),
            color: None,
            alignment: StatusAlignment::Right,
            priority: 100,
            click_action: None,
        });

        // Encoding
        items.push(StatusItem {
            text: self.encoding.clone(),
            tooltip: String::from("Select Encoding"),
            color: None,
            alignment: StatusAlignment::Right,
            priority: 70,
            click_action: Some(StatusAction::SelectEncoding),
        });

        // Line ending
        items.push(StatusItem {
            text: self.line_ending.clone(),
            tooltip: String::from("Select End of Line Sequence"),
            color: None,
            alignment: StatusAlignment::Right,
            priority: 60,
            click_action: Some(StatusAction::SelectLineEnding),
        });

        // Language
        items.push(StatusItem {
            text: self.language.clone(),
            tooltip: String::from("Select Language Mode"),
            color: None,
            alignment: StatusAlignment::Right,
            priority: 50,
            click_action: Some(StatusAction::SelectLanguage),
        });

        // Confidence score
        if let Some(score) = self.confidence_score {
            items.push(StatusItem {
                text: format!("⚡ {:.1}%", score),
                tooltip: format!("Confidence Score: {:.1}%", score),
                color: Some(if score > 80.0 {
                    colors::SUCCESS
                } else if score > 50.0 {
                    colors::WARNING
                } else {
                    colors::ERROR
                }),
                alignment: StatusAlignment::Right,
                priority: 40,
                click_action: None,
            });
        }

        // AI status
        items.push(StatusItem {
            text: format!("🤖 {}", self.ai_status),
            tooltip: String::from("AI Agent Status — click to toggle"),
            color: None,
            alignment: StatusAlignment::Right,
            priority: 30,
            click_action: Some(StatusAction::ToggleAiPanel),
        });

        // Frame time
        items.push(StatusItem {
            text: format!("{:.1}ms", self.frame_time_ms),
            tooltip: String::from("Frame render time"),
            color: Some(if self.frame_time_ms < 7.0 {
                colors::SUCCESS
            } else if self.frame_time_ms < 16.0 {
                colors::WARNING
            } else {
                colors::ERROR
            }),
            alignment: StatusAlignment::Right,
            priority: 10,
            click_action: None,
        });

        items
    }

    /// Get text positions for rendering
    /// Returns (text, x, y, color) tuples
    pub fn text_positions(&self, zone: &Zone) -> Vec<(String, f32, f32, [f32; 4])> {
        let items = self.build_items();
        let mut result = Vec::with_capacity(items.len());
        let text_y = zone.y + (zone.height - LayoutConstants::SMALL_FONT_SIZE) / 2.0;
        let char_width = LayoutConstants::CHAR_WIDTH;
        let padding = 12.0;

        // Left items
        let mut left_x = zone.x + padding;
        let mut left_items: Vec<&StatusItem> = items.iter()
            .filter(|i| i.alignment == StatusAlignment::Left)
            .collect();
        left_items.sort_by(|a, b| b.priority.cmp(&a.priority));

        for item in &left_items {
            let color = item.color.unwrap_or(colors::TEXT_WHITE);
            result.push((item.text.clone(), left_x, text_y, color));
            left_x += item.text.len() as f32 * char_width + padding;
        }

        // Right items (render from right edge leftward)
        let mut right_x = zone.x + zone.width - padding;
        let mut right_items: Vec<&StatusItem> = items.iter()
            .filter(|i| i.alignment == StatusAlignment::Right)
            .collect();
        right_items.sort_by(|a, b| a.priority.cmp(&b.priority)); // Lowest priority = rightmost

        for item in &right_items {
            let text_width = item.text.len() as f32 * char_width;
            right_x -= text_width;
            let color = item.color.unwrap_or(colors::TEXT_WHITE);
            result.push((item.text.clone(), right_x, text_y, color));
            right_x -= padding;
        }

        result
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}
```

### Update module declarations

```rust
mod status_bar;
```

### Run `cargo check --package forge-app` — fix ALL errors.

---

## TASK 7: Cursor Rendering

### Create file: `crates/forge-app/src/cursor.rs`

```rust
use crate::rect_renderer::Rect;
use crate::ui::{colors, LayoutConstants, Zone};
use std::time::Instant;

/// Cursor rendering with blink support
pub struct CursorRenderer {
    /// Whether cursor is visible (for blinking)
    visible: bool,
    /// Last toggle time
    last_toggle: Instant,
    /// Blink interval
    blink_interval_ms: u64,
    /// Whether blinking is enabled
    blink_enabled: bool,
    /// Cursor style
    style: CursorStyle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CursorStyle {
    /// Thin vertical line (default, like VS Code)
    Line,
    /// Full character block
    Block,
    /// Underline
    Underline,
}

impl CursorRenderer {
    pub fn new() -> Self {
        Self {
            visible: true,
            last_toggle: Instant::now(),
            blink_interval_ms: 530,
            blink_enabled: true,
            style: CursorStyle::Line,
        }
    }

    /// Update blink state (call every frame)
    pub fn update(&mut self) {
        if !self.blink_enabled {
            self.visible = true;
            return;
        }

        let elapsed = self.last_toggle.elapsed().as_millis() as u64;
        if elapsed >= self.blink_interval_ms {
            self.visible = !self.visible;
            self.last_toggle = Instant::now();
        }
    }

    /// Reset blink (make cursor visible immediately — call on any keypress)
    pub fn reset_blink(&mut self) {
        self.visible = true;
        self.last_toggle = Instant::now();
    }

    /// Set cursor style
    pub fn set_style(&mut self, style: CursorStyle) {
        self.style = style;
    }

    /// Enable/disable blinking
    pub fn set_blink_enabled(&mut self, enabled: bool) {
        self.blink_enabled = enabled;
        if !enabled {
            self.visible = true;
        }
    }

    /// Generate cursor rectangle
    /// `cursor_line` and `cursor_col` are 0-indexed
    /// `scroll_top` is the first visible line (0-indexed)
    pub fn render_rect(
        &self,
        cursor_line: usize,
        cursor_col: usize,
        scroll_top: usize,
        editor_zone: &Zone,
    ) -> Option<Rect> {
        if !self.visible {
            return None;
        }

        // Check if cursor is in visible area
        if cursor_line < scroll_top {
            return None;
        }
        let visible_line = cursor_line - scroll_top;
        let visible_lines = (editor_zone.height / LayoutConstants::LINE_HEIGHT) as usize;
        if visible_line >= visible_lines {
            return None;
        }

        let x = editor_zone.x + (cursor_col as f32 * LayoutConstants::CHAR_WIDTH);
        let y = editor_zone.y + (visible_line as f32 * LayoutConstants::LINE_HEIGHT);

        let (width, height) = match self.style {
            CursorStyle::Line => (2.0, LayoutConstants::LINE_HEIGHT),
            CursorStyle::Block => (LayoutConstants::CHAR_WIDTH, LayoutConstants::LINE_HEIGHT),
            CursorStyle::Underline => (LayoutConstants::CHAR_WIDTH, 2.0),
        };

        let adjusted_y = match self.style {
            CursorStyle::Underline => y + LayoutConstants::LINE_HEIGHT - 2.0,
            _ => y,
        };

        Some(Rect {
            x,
            y: adjusted_y,
            width,
            height,
            color: colors::CURSOR,
        })
    }

    /// Generate current line highlight rectangle
    pub fn current_line_rect(
        &self,
        cursor_line: usize,
        scroll_top: usize,
        editor_zone: &Zone,
    ) -> Option<Rect> {
        if cursor_line < scroll_top {
            return None;
        }
        let visible_line = cursor_line - scroll_top;
        let visible_lines = (editor_zone.height / LayoutConstants::LINE_HEIGHT) as usize;
        if visible_line >= visible_lines {
            return None;
        }

        let y = editor_zone.y + (visible_line as f32 * LayoutConstants::LINE_HEIGHT);

        Some(Rect {
            x: editor_zone.x,
            y,
            width: editor_zone.width,
            height: LayoutConstants::LINE_HEIGHT,
            color: colors::CURRENT_LINE,
        })
    }

    /// Generate selection rectangles
    pub fn selection_rects(
        &self,
        sel_start_line: usize,
        sel_start_col: usize,
        sel_end_line: usize,
        sel_end_col: usize,
        scroll_top: usize,
        line_lengths: &[usize], // length of each line in characters
        editor_zone: &Zone,
    ) -> Vec<Rect> {
        let mut rects = Vec::new();
        let visible_lines = (editor_zone.height / LayoutConstants::LINE_HEIGHT) as usize;

        let start_line = sel_start_line.min(sel_end_line);
        let end_line = sel_start_line.max(sel_end_line);
        let (start_col, end_col) = if sel_start_line <= sel_end_line {
            (sel_start_col, sel_end_col)
        } else {
            (sel_end_col, sel_start_col)
        };

        for line in start_line..=end_line {
            if line < scroll_top || line >= scroll_top + visible_lines {
                continue;
            }
            let visible_line = line - scroll_top;
            let y = editor_zone.y + (visible_line as f32 * LayoutConstants::LINE_HEIGHT);

            let line_len = line_lengths.get(line).copied().unwrap_or(0);

            let col_start = if line == start_line { start_col } else { 0 };
            let col_end = if line == end_line { end_col } else { line_len };

            if col_start >= col_end && line != end_line {
                continue;
            }

            let x = editor_zone.x + (col_start as f32 * LayoutConstants::CHAR_WIDTH);
            let width = ((col_end - col_start) as f32 * LayoutConstants::CHAR_WIDTH).max(LayoutConstants::CHAR_WIDTH);

            rects.push(Rect {
                x,
                y,
                width,
                height: LayoutConstants::LINE_HEIGHT,
                color: colors::SELECTION,
            });
        }

        rects
    }
}

impl Default for CursorRenderer {
    fn default() -> Self {
        Self::new()
    }
}
```

### Update module declarations

```rust
mod cursor;
```

### Run `cargo check --package forge-app` — fix ALL errors.

---

## TASK 8: Breadcrumb Bar

### Create file: `crates/forge-app/src/breadcrumb.rs`

```rust
use crate::rect_renderer::Rect;
use crate::ui::{colors, LayoutConstants, Zone};

/// A single breadcrumb segment
#[derive(Clone, Debug)]
pub struct BreadcrumbSegment {
    pub text: String,
    pub kind: SegmentKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SegmentKind {
    Folder,
    File,
    Symbol,
}

/// Breadcrumb bar state and rendering
pub struct BreadcrumbBar {
    pub segments: Vec<BreadcrumbSegment>,
}

impl BreadcrumbBar {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Update breadcrumbs from file path and cursor position
    pub fn update_from_path(&mut self, file_path: &str) {
        self.segments.clear();

        // Split path into components
        let parts: Vec<&str> = file_path.split(['/', '\\']).collect();

        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            let kind = if i == parts.len() - 1 {
                SegmentKind::File
            } else {
                SegmentKind::Folder
            };
            self.segments.push(BreadcrumbSegment {
                text: part.to_string(),
                kind,
            });
        }
    }

    /// Add a symbol segment (e.g., current function name)
    pub fn set_symbol(&mut self, symbol: Option<String>) {
        // Remove existing symbol segments
        self.segments.retain(|s| s.kind != SegmentKind::Symbol);

        if let Some(sym) = symbol {
            self.segments.push(BreadcrumbSegment {
                text: sym,
                kind: SegmentKind::Symbol,
            });
        }
    }

    /// Render separator rectangles (small chevrons between segments)
    pub fn render_rects(&self, zone: &Zone) -> Vec<Rect> {
        // Breadcrumb bar is mostly text; separators are rendered as small rects
        let mut rects = Vec::new();
        let char_width = LayoutConstants::CHAR_WIDTH;
        let padding = 6.0;
        let text_y = zone.y + (zone.height - LayoutConstants::SMALL_FONT_SIZE) / 2.0;

        let mut x = zone.x + padding;
        for (i, segment) in self.segments.iter().enumerate() {
            x += segment.text.len() as f32 * char_width + padding;

            // Separator chevron (small triangle/rect)
            if i < self.segments.len() - 1 {
                rects.push(Rect {
                    x: x + 2.0,
                    y: text_y + 2.0,
                    width: 6.0,
                    height: LayoutConstants::SMALL_FONT_SIZE - 4.0,
                    color: colors::TEXT_DIM,
                });
                x += 14.0; // Space for separator
            }
        }

        rects
    }

    /// Get text positions for rendering
    /// Returns (text, x, y, color) tuples
    pub fn text_positions(&self, zone: &Zone) -> Vec<(String, f32, f32, [f32; 4])> {
        let mut result = Vec::with_capacity(self.segments.len() * 2);
        let char_width = LayoutConstants::CHAR_WIDTH;
        let padding = 6.0;
        let text_y = zone.y + (zone.height - LayoutConstants::SMALL_FONT_SIZE) / 2.0;

        let mut x = zone.x + padding;
        for (i, segment) in self.segments.iter().enumerate() {
            let color = match segment.kind {
                SegmentKind::Folder => colors::TEXT_DIM,
                SegmentKind::File => colors::TEXT_FG,
                SegmentKind::Symbol => colors::TEXT_FG,
            };
            result.push((segment.text.clone(), x, text_y, color));
            x += segment.text.len() as f32 * char_width + padding;

            // Separator text
            if i < self.segments.len() - 1 {
                result.push((String::from(">"), x + 2.0, text_y, colors::TEXT_DIM));
                x += 14.0;
            }
        }

        result
    }
}

impl Default for BreadcrumbBar {
    fn default() -> Self {
        Self::new()
    }
}
```

### Update module declarations

```rust
mod breadcrumb;
```

### Run `cargo check --package forge-app` — fix ALL errors.
### Run `cargo test --workspace` — fix ALL failures.
