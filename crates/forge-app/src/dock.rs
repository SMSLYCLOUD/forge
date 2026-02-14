use crate::rect_renderer::Rect;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PanelId {
    ActivityBar,
    Sidebar,
    Editor,
    BottomPanel,
    AiPanel,
    TitleBar,
    StatusBar,
    // Future expansion
    Terminal,
    Output,
    Debug,
}

#[derive(Clone, Debug)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
pub enum SizePolicy {
    Fixed(f32),
    Flexible(f32), // Flex grow factor
}

#[derive(Clone, Debug)]
pub struct DockNode {
    pub id: Option<PanelId>, // None if it's a container
    pub children: Vec<DockNode>,
    pub direction: SplitDirection,
    pub size: SizePolicy,
}

impl DockNode {
    pub fn leaf(id: PanelId, size: SizePolicy) -> Self {
        Self {
            id: Some(id),
            children: Vec::new(),
            direction: SplitDirection::Horizontal,
            size,
        }
    }

    pub fn container(direction: SplitDirection, size: SizePolicy, children: Vec<DockNode>) -> Self {
        Self {
            id: None,
            children,
            direction,
            size,
        }
    }
}

pub struct DockTree {
    pub root: DockNode,
}

impl DockTree {
    pub fn default_layout() -> Self {
        use PanelId::*;
        use SizePolicy::*;
        use SplitDirection::*;

        // Middle area: ActivityBar | Sidebar | Content | AI
        let center_content = DockNode::container(
            Vertical,
            Flexible(1.0),
            vec![
                DockNode::leaf(Editor, Flexible(1.0)),
                DockNode::leaf(BottomPanel, Fixed(0.0)), // Hidden by default
            ],
        );

        let middle_row = DockNode::container(
            Horizontal,
            Flexible(1.0),
            vec![
                DockNode::leaf(ActivityBar, Fixed(48.0)),
                DockNode::leaf(Sidebar, Fixed(0.0)), // Start closed? Or 250.0
                center_content,
                DockNode::leaf(AiPanel, Fixed(0.0)), // Hidden by default
            ],
        );

        let root = DockNode::container(
            Vertical,
            Flexible(1.0),
            vec![
                DockNode::leaf(TitleBar, Fixed(30.0)),
                middle_row,
                DockNode::leaf(StatusBar, Fixed(22.0)),
            ],
        );

        Self { root }
    }

    pub fn compute_layout(&self, area: Rect) -> HashMap<PanelId, Rect> {
        let mut map = HashMap::new();
        Self::compute_node(&self.root, area, &mut map);
        map
    }

    fn compute_node(node: &DockNode, area: Rect, map: &mut HashMap<PanelId, Rect>) {
        if let Some(id) = &node.id {
            map.insert(id.clone(), area);
            return;
        }

        if node.children.is_empty() {
            return;
        }

        // Calculate total fixed size and total flexible weight
        let mut total_fixed = 0.0;
        let mut total_flex = 0.0;

        for child in &node.children {
            match child.size {
                SizePolicy::Fixed(s) => total_fixed += s,
                SizePolicy::Flexible(w) => total_flex += w,
            }
        }

        let available_space = match node.direction {
            SplitDirection::Horizontal => area.width,
            SplitDirection::Vertical => area.height,
        };

        let remaining_space = (available_space - total_fixed).max(0.0);
        let flex_unit = if total_flex > 0.0 {
            remaining_space / total_flex
        } else {
            0.0
        };

        let mut current_pos = match node.direction {
            SplitDirection::Horizontal => area.x,
            SplitDirection::Vertical => area.y,
        };

        for child in &node.children {
            let size = match child.size {
                SizePolicy::Fixed(s) => s,
                SizePolicy::Flexible(w) => w * flex_unit,
            };

            let child_rect = match node.direction {
                SplitDirection::Horizontal => Rect {
                    x: current_pos,
                    y: area.y,
                    width: size,
                    height: area.height,
                    color: [0.0; 4],
                },
                SplitDirection::Vertical => Rect {
                    x: area.x,
                    y: current_pos,
                    width: area.width,
                    height: size,
                    color: [0.0; 4],
                },
            };

            Self::compute_node(child, child_rect, map);
            current_pos += size;
        }
    }

    // Helper to update panel visibility/size dynamically
    pub fn set_panel_size(&mut self, id: PanelId, size: f32) {
        Self::update_node_size(&mut self.root, &id, size);
    }

    fn update_node_size(node: &mut DockNode, target_id: &PanelId, new_size: f32) -> bool {
        if let Some(node_id) = &node.id {
            if node_id == target_id {
                node.size = SizePolicy::Fixed(new_size);
                return true;
            }
        }
        for child in &mut node.children {
            if Self::update_node_size(child, target_id, new_size) {
                return true;
            }
        }
        false
    }
}

#[path = "dock_input.rs"]
mod dock_input;
