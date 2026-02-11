//! Terminal Grid Buffer.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: [255, 255, 255], // White
            bg: [0, 0, 0],       // Black
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

pub struct TerminalGrid {
    pub cells: Vec<Vec<Cell>>,
    pub cols: u16,
    pub rows: u16,
    pub cursor_row: u16, // 0-based
    pub cursor_col: u16, // 0-based
    pub scrollback: Vec<Vec<Cell>>,
    pub current_fg: [u8; 3],
    pub current_bg: [u8; 3],
}

impl TerminalGrid {
    pub fn new(cols: u16, rows: u16) -> Self {
        let mut grid = Self {
            cells: Vec::new(),
            cols,
            rows,
            cursor_row: 0,
            cursor_col: 0,
            scrollback: Vec::with_capacity(10000),
            current_fg: [255, 255, 255],
            current_bg: [0, 0, 0],
        };
        grid.resize(cols, rows);
        grid
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;

        // Resize rows
        if self.cells.len() < rows as usize {
            for _ in self.cells.len()..rows as usize {
                self.cells.push(vec![Cell::default(); cols as usize]);
            }
        } else {
            self.cells.truncate(rows as usize);
        }

        // Resize columns
        for row in &mut self.cells {
            if row.len() < cols as usize {
                row.resize(cols as usize, Cell::default());
            } else {
                row.truncate(cols as usize);
            }
        }

        // Clamp cursor
        if self.cursor_row >= rows {
            self.cursor_row = rows - 1;
        }
        if self.cursor_col >= cols {
            self.cursor_col = cols - 1;
        }
    }

    pub fn write_char(&mut self, c: char) {
        if c == '\n' {
            self.newline();
            return;
        }
        if c == '\r' {
            self.cursor_col = 0;
            return;
        }
        if c == '\t' {
            // Simple tab expansion to 4 spaces
            let spaces = 4 - (self.cursor_col % 4);
            for _ in 0..spaces {
                self.write_char(' ');
            }
            return;
        }

        // Wrap if at end of line
        if self.cursor_col >= self.cols {
            self.newline();
        }

        if (self.cursor_row as usize) < self.cells.len()
            && (self.cursor_col as usize) < self.cells[self.cursor_row as usize].len()
        {
            self.cells[self.cursor_row as usize][self.cursor_col as usize] = Cell {
                ch: c,
                fg: self.current_fg,
                bg: self.current_bg,
                bold: false,      // TODO
                italic: false,    // TODO
                underline: false, // TODO
            };
        }

        self.cursor_col += 1;
    }

    pub fn newline(&mut self) {
        self.cursor_col = 0;
        if self.cursor_row < self.rows - 1 {
            self.cursor_row += 1;
        } else {
            self.scroll_up();
        }
    }

    pub fn scroll_up(&mut self) {
        if !self.cells.is_empty() {
            let removed_row = self.cells.remove(0);
            self.scrollback.push(removed_row);
            if self.scrollback.len() > 10000 {
                self.scrollback.remove(0);
            }
            self.cells.push(vec![Cell::default(); self.cols as usize]);
        }
    }

    pub fn clear_line(&mut self, mode: u8) {
        let row = self.cursor_row as usize;
        if row >= self.cells.len() {
            return;
        }

        let start = match mode {
            0 => self.cursor_col as usize, // Cursor to end
            1 => 0,                        // Start to cursor
            2 => 0,                        // Entire line
            _ => 0,
        };
        let end = match mode {
            0 => self.cols as usize,
            1 => self.cursor_col as usize + 1,
            2 => self.cols as usize,
            _ => self.cols as usize,
        };

        for i in start..end {
            if i < self.cells[row].len() {
                self.cells[row][i] = Cell::default();
            }
        }
    }

    pub fn clear_screen(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = Cell::default();
            }
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
    }
}
