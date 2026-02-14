pub trait ZoneDebugContext {
    fn add_debug_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]);
    fn get_zones(&self) -> Vec<(f32, f32, f32, f32, String)>; // x, y, w, h, name
}

pub fn enable_zone_overlay<C: ZoneDebugContext>(context: &mut C) {
    let zones = context.get_zones();
    // Colors for rotation
    let colors = [
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0, 1.0],
        [1.0, 0.0, 1.0, 1.0],
    ];

    for (i, (x, y, w, h, _name)) in zones.iter().enumerate() {
        let color = colors[i % colors.len()];

        // Draw border (hollow rect)
        // Top
        context.add_debug_rect(*x, *y, *w, 2.0, color);
        // Bottom
        context.add_debug_rect(*x, *y + *h - 2.0, *w, 2.0, color);
        // Left
        context.add_debug_rect(*x, *y, 2.0, *h, color);
        // Right
        context.add_debug_rect(*x + *w - 2.0, *y, 2.0, *h, color);
    }
}
