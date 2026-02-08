use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaColor {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

pub fn color_from_confidence(c: f64) -> RgbaColor {
    let c = c.clamp(0.0, 1.0);

    // HSL gradient:
    // 0.0 -> Red (Hue 0)
    // 0.5 -> Yellow (Hue 60)
    // 1.0 -> Green (Hue 120)

    // Simple linear interpolation in RGB space for speed
    // Red:   (255, 0, 0)
    // Yellow:(255, 255, 0)
    // Green: (0, 255, 0)

    let r: u8;
    let g: u8;

    if c < 0.5 {
        // Red to Yellow
        // c: 0.0 -> 0.5
        // factor: 0.0 -> 1.0
        let factor = c * 2.0;
        r = 255;
        g = (255.0 * factor) as u8;
    } else {
        // Yellow to Green
        // c: 0.5 -> 1.0
        // factor: 0.0 -> 1.0
        let factor = (c - 0.5) * 2.0;
        r = (255.0 * (1.0 - factor)) as u8;
        g = 255;
    }

    RgbaColor::new(r, g, 0, 255)
}
