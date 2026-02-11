use crate::editor::Editor;
use crate::ui::Zone;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SplitDir {
    Horizontal,
    Vertical,
}

pub struct EditorGroup {
    pub id: u32,
    pub tabs: crate::tab_bar::TabBar,
    pub zone: Zone,
    pub editor: Editor, // Each group has its own editor instance or at least a buffer view
}

impl EditorGroup {
    pub fn new(id: u32, zone: Zone) -> Self {
        Self {
            id,
            tabs: crate::tab_bar::TabBar::new(),
            zone,
            editor: Editor::new(),
        }
    }

    pub fn focus(&mut self) {
        // Implementation for focusing this group
    }
}

pub struct EditorLayout {
    pub groups: Vec<EditorGroup>,
    pub active_group_id: u32,
    pub split_direction: SplitDir,
    next_group_id: u32,
}

impl EditorLayout {
    pub fn new(width: f32, height: f32) -> Self {
        let initial_group = EditorGroup::new(
            0,
            Zone {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
        );
        Self {
            groups: vec![initial_group],
            active_group_id: 0,
            split_direction: SplitDir::Vertical,
            next_group_id: 1,
        }
    }

    pub fn active_group(&mut self) -> Option<&mut EditorGroup> {
        self.groups
            .iter_mut()
            .find(|g| g.id == self.active_group_id)
    }

    pub fn active_group_ref(&self) -> Option<&EditorGroup> {
        self.groups.iter().find(|g| g.id == self.active_group_id)
    }

    pub fn split(&mut self, direction: SplitDir) {
        // Logic to split the active group
        // This is a simplified placeholder logic
        if let Some(active_idx) = self
            .groups
            .iter()
            .position(|g| g.id == self.active_group_id)
        {
            let active_zone = self.groups[active_idx].zone.clone();
            let (new_zone1, new_zone2) = match direction {
                SplitDir::Horizontal => {
                    let h = active_zone.height / 2.0;
                    (
                        Zone {
                            height: h,
                            ..active_zone
                        },
                        Zone {
                            y: active_zone.y + h,
                            height: h,
                            ..active_zone
                        },
                    )
                }
                SplitDir::Vertical => {
                    let w = active_zone.width / 2.0;
                    (
                        Zone {
                            width: w,
                            ..active_zone
                        },
                        Zone {
                            x: active_zone.x + w,
                            width: w,
                            ..active_zone
                        },
                    )
                }
            };

            self.groups[active_idx].zone = new_zone1;

            let new_group = EditorGroup::new(self.next_group_id, new_zone2);
            self.active_group_id = self.next_group_id;
            self.next_group_id += 1;
            self.groups.push(new_group);
        }
    }

    pub fn focus_group(&mut self, id: u32) {
        if self.groups.iter().any(|g| g.id == id) {
            self.active_group_id = id;
        }
    }

    pub fn close_group(&mut self, id: u32) {
        if self.groups.len() <= 1 {
            return;
        }
        if let Some(idx) = self.groups.iter().position(|g| g.id == id) {
            self.groups.remove(idx);
            // Re-assign active group if needed
            if self.active_group_id == id {
                self.active_group_id = self.groups.first().map(|g| g.id).unwrap_or(0);
            }
            // Re-calculate layout (simplified here)
            // Ideally, we would expand adjacent groups to fill the space
        }
    }
}
