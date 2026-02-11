//! ANSI Escape Sequence Parser.

#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Reset,
    Named(u8),       // 0-15
    Rgb(u8, u8, u8), // TrueColor
    Indexed(u8),     // 256-color
}

#[derive(Debug, Clone, PartialEq)]
pub enum TermEvent {
    Print(char),
    SetFg(Color),
    SetBg(Color),
    MoveCursor(usize, usize), // row, col (1-based)
    CursorUp(usize),
    CursorDown(usize),
    CursorForward(usize),
    CursorBack(usize),
    ClearLine(u8),   // 0=end, 1=start, 2=all
    ClearScreen(u8), // 0=end, 1=start, 2=all
    ScrollUp(usize),
    ScrollDown(usize),
    Bell,
    Reset,
}

pub struct AnsiParser {
    state: State,
    buffer: Vec<u8>,
    params: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Ground,
    Escape,
    Csi,     // [
    Osc,     // ]
}

impl Default for AnsiParser {
    fn default() -> Self {
        Self {
            state: State::Ground,
            buffer: Vec::with_capacity(64),
            params: Vec::with_capacity(4),
        }
    }
}

impl AnsiParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&mut self, input: &[u8]) -> Vec<TermEvent> {
        let mut events = Vec::new();

        for &byte in input {
            match self.state {
                State::Ground => match byte {
                    0x1b => self.state = State::Escape,
                    0x07 => events.push(TermEvent::Bell),
                    0x08 => events.push(TermEvent::CursorBack(1)),
                    0x0a => events.push(TermEvent::Print('\n')),
                    0x0d => events.push(TermEvent::Print('\r')),
                    _ => {
                        // Handle UTF-8? For simplicity, treat as separate bytes or basic ASCII for now.
                        // A real terminal handles UTF-8 streaming.
                        // Here we assume simple ASCII or full UTF-8 chunks if possible.
                        // For this task, we'll cast to char (lossy) if it's ASCII, or handle UTF-8 properly?
                        // Let's assume input is valid UTF-8 for now or handle single bytes.
                        // If byte < 128, it's ASCII.
                        if byte < 128 {
                            events.push(TermEvent::Print(byte as char));
                        } else {
                            // TODO: UTF-8 continuation handling.
                            // For now, print a replacement char or ignore.
                            events.push(TermEvent::Print('\u{FFFD}'));
                        }
                    }
                },
                State::Escape => match byte {
                    b'[' => {
                        self.state = State::Csi;
                        self.params.clear();
                        self.buffer.clear();
                    }
                    b']' => {
                        self.state = State::Osc;
                        self.buffer.clear();
                    }
                    _ => {
                        self.state = State::Ground; // Cancel escape
                    }
                },
                State::Csi => {
                    if byte.is_ascii_digit() {
                        self.buffer.push(byte);
                    } else if byte == b';' {
                        self.push_param();
                    } else {
                        self.push_param();
                        self.handle_csi(byte, &mut events);
                        self.state = State::Ground;
                    }
                }
                State::Osc => {
                    // Operating System Command - usually ends with BEL or ST (ESC \)
                    if byte == 0x07 || (byte == b'\\' && self.buffer.last() == Some(&0x1b)) {
                        // End of OSC
                        self.state = State::Ground;
                    } else {
                        self.buffer.push(byte);
                    }
                }
            }
        }

        events
    }

    fn push_param(&mut self) {
        if self.buffer.is_empty() {
            if !self.params.is_empty() {
                // If checking for empty param, usually defaults to 0
                // But if it's the first param and buffer empty, it might be 0 or default.
            }
            // If buffer is empty but we hit ';', it implies a missing param (default 0 or 1 depending on command)
            // We'll push 0 for now.
            // But if params is empty and buffer is empty, it means no params yet.
        } else {
            let s = std::str::from_utf8(&self.buffer).unwrap_or("0");
            let val = s.parse::<usize>().unwrap_or(0);
            self.params.push(val);
            self.buffer.clear();
        }
    }

    fn param(&self, idx: usize, default: usize) -> usize {
        if idx < self.params.len() {
            let val = self.params[idx];
            if val == 0 { default } else { val }
        } else {
            default
        }
    }

    fn handle_csi(&mut self, final_byte: u8, events: &mut Vec<TermEvent>) {
        match final_byte {
            b'm' => self.handle_sgr(events),
            b'A' => events.push(TermEvent::CursorUp(self.param(0, 1))),
            b'B' => events.push(TermEvent::CursorDown(self.param(0, 1))),
            b'C' => events.push(TermEvent::CursorForward(self.param(0, 1))),
            b'D' => events.push(TermEvent::CursorBack(self.param(0, 1))),
            b'H' | b'f' => {
                let row = self.param(0, 1);
                let col = self.param(1, 1);
                events.push(TermEvent::MoveCursor(row, col));
            }
            b'J' => events.push(TermEvent::ClearScreen(self.param(0, 0) as u8)),
            b'K' => events.push(TermEvent::ClearLine(self.param(0, 0) as u8)),
            b'S' => events.push(TermEvent::ScrollUp(self.param(0, 1))),
            b'T' => events.push(TermEvent::ScrollDown(self.param(0, 1))),
            _ => {}
        }
    }

    fn handle_sgr(&self, events: &mut Vec<TermEvent>) {
        if self.params.is_empty() {
            events.push(TermEvent::Reset);
            return;
        }

        let mut i = 0;
        while i < self.params.len() {
            let code = self.params[i];
            match code {
                0 => events.push(TermEvent::Reset),
                30..=37 => events.push(TermEvent::SetFg(Color::Named((code - 30) as u8))),
                90..=97 => events.push(TermEvent::SetFg(Color::Named((code - 90 + 8) as u8))),
                40..=47 => events.push(TermEvent::SetBg(Color::Named((code - 40) as u8))),
                100..=107 => events.push(TermEvent::SetBg(Color::Named((code - 100 + 8) as u8))),
                38 => {
                    // Extended FG
                    if i + 1 < self.params.len() {
                        match self.params[i + 1] {
                            5 => { // 256 color
                                if i + 2 < self.params.len() {
                                    events.push(TermEvent::SetFg(Color::Indexed(self.params[i + 2] as u8)));
                                    i += 2;
                                }
                            }
                            2 => { // RGB
                                if i + 4 < self.params.len() {
                                    events.push(TermEvent::SetFg(Color::Rgb(
                                        self.params[i + 2] as u8,
                                        self.params[i + 3] as u8,
                                        self.params[i + 4] as u8,
                                    )));
                                    i += 4;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                48 => {
                    // Extended BG
                    if i + 1 < self.params.len() {
                        match self.params[i + 1] {
                            5 => { // 256 color
                                if i + 2 < self.params.len() {
                                    events.push(TermEvent::SetBg(Color::Indexed(self.params[i + 2] as u8)));
                                    i += 2;
                                }
                            }
                            2 => { // RGB
                                if i + 4 < self.params.len() {
                                    events.push(TermEvent::SetBg(Color::Rgb(
                                        self.params[i + 2] as u8,
                                        self.params[i + 3] as u8,
                                        self.params[i + 4] as u8,
                                    )));
                                    i += 4;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
}
