
impl crate::dock::DockTree {
    pub fn handle_resize(&mut self, _mx: f32, _my: f32) -> bool {
        // Basic stub for resizing.
        // Full implementation requires hit testing separators which isn't currently output by compute_layout.
        // For Epic 1.1, we rely on SizePolicy logic being correct.
        // Future task: Add `separators` to `compute_layout` output and check drag.
        false
    }
}
