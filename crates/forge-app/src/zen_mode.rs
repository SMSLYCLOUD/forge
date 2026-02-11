use crate::ui::LayoutZones;

#[derive(Default)]
pub struct ZenMode {
    pub active: bool,
    pub saved_layout: Option<LayoutZones>,
}

impl ZenMode {
    pub fn new() -> Self {
        Self {
            active: false,
            saved_layout: None,
        }
    }

    pub fn enter(&mut self, current_layout: LayoutZones) {
        if self.active {
            return;
        }
        self.saved_layout = Some(current_layout);
        self.active = true;
    }

    pub fn exit(&mut self) -> Option<LayoutZones> {
        if !self.active {
            return None;
        }
        self.active = false;
        self.saved_layout.take()
    }

    pub fn toggle(&mut self, current_layout: LayoutZones) -> Option<LayoutZones> {
        if self.active {
            self.exit()
        } else {
            self.enter(current_layout);
            None // Return None to signal we entered zen mode and layout should be recomputed externally
        }
    }
}
