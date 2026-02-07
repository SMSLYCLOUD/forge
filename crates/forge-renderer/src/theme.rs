/// RGBA color with 8-bit components
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    /// Convert to normalized float array [0.0, 1.0]
    pub fn as_rgba_f32(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }
}

/// Color theme for the editor
#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub cursor: Color,
    pub selection: Color,
    pub line_number: Color,
    pub line_number_active: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::forge_dark()
    }
}

impl Theme {
    /// Forge's default dark theme
    pub fn forge_dark() -> Self {
        Self {
            background: Color::rgb(26, 27, 38),            // Dark navy
            foreground: Color::rgb(224, 227, 236),         // Light gray
            cursor: Color::rgb(255, 121, 198),             // Pink
            selection: Color::new(68, 71, 90, 128),        // Semi-transparent gray
            line_number: Color::rgb(98, 114, 164),         // Muted blue
            line_number_active: Color::rgb(255, 121, 198), // Pink (same as cursor)
        }
    }

    /// Light theme
    pub fn forge_light() -> Self {
        Self {
            background: Color::rgb(250, 250, 250),
            foreground: Color::rgb(40, 40, 40),
            cursor: Color::rgb(0, 100, 200),
            selection: Color::new(200, 220, 255, 128),
            line_number: Color::rgb(150, 150, 150),
            line_number_active: Color::rgb(0, 100, 200),
        }
    }
}
