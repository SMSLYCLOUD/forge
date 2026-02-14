
use crate::application::AppState;

impl forge_test_tools::zone_debug::ZoneDebugContext for AppState {
    fn add_debug_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        self.render_batch.push(crate::rect_renderer::Rect {
            x,
            y,
            width: w,
            height: h,
            color,
        });
    }

    fn get_zones(&self) -> Vec<(f32, f32, f32, f32, String)> {
        let mut zones = Vec::new();
        let l = &self.layout;
        zones.push((l.activity_bar.x, l.activity_bar.y, l.activity_bar.width, l.activity_bar.height, "activity_bar".into()));
        zones.push((l.tab_bar.x, l.tab_bar.y, l.tab_bar.width, l.tab_bar.height, "tab_bar".into()));
        zones.push((l.gutter.x, l.gutter.y, l.gutter.width, l.gutter.height, "gutter".into()));
        zones.push((l.editor.x, l.editor.y, l.editor.width, l.editor.height, "editor".into()));
        zones.push((l.status_bar.x, l.status_bar.y, l.status_bar.width, l.status_bar.height, "status_bar".into()));
        if let Some(sb) = &l.sidebar {
            zones.push((sb.x, sb.y, sb.width, sb.height, "sidebar".into()));
        }
        zones
    }
}
