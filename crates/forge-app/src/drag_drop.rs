use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum DragPayload {
    TabDrag { group_id: u32, tab_idx: usize },
    FileDrag(PathBuf),
}

#[derive(Clone, Debug)]
pub struct DropZone {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub struct DragDrop {
    pub dragging: Option<DragPayload>,
    pub drop_indicator: Option<DropZone>,
    pub start_pos: Option<(f32, f32)>,
}

impl DragDrop {
    pub fn new() -> Self {
        Self {
            dragging: None,
            drop_indicator: None,
            start_pos: None,
        }
    }

    pub fn start_drag(&mut self, payload: DragPayload, pos: (f32, f32)) {
        self.dragging = Some(payload);
        self.start_pos = Some(pos);
    }

    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        if self.dragging.is_some() {
            // Update drop indicator based on mouse position
            // This is a placeholder logic
            self.drop_indicator = Some(DropZone {
                x,
                y,
                width: 100.0,
                height: 4.0,
            });
        }
    }

    pub fn handle_mouse_up(&mut self) -> Option<DragPayload> {
        let payload = self.dragging.take();
        self.drop_indicator = None;
        self.start_pos = None;
        payload
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging.is_some()
    }
}
