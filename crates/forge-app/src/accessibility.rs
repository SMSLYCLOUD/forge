use accesskit::{
    NodeBuilder, NodeId, Role, Tree, TreeUpdate,
};
use accesskit_winit::Adapter;
use winit::event::WindowEvent;
use winit::window::Window;

pub struct AccessibilityState {
    pub editor_text: String,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub sidebar_visible: bool,
}

pub struct AccessibilityManager {
    adapter: Adapter,
}

impl AccessibilityManager {
    pub fn new(window: &Window, proxy: winit::event_loop::EventLoopProxy<super::UserEvent>) -> Self {
        let adapter = Adapter::with_event_loop_proxy(window, proxy);

        let mut manager = Self { adapter };

        // Initial update
        manager.update(AccessibilityState {
            editor_text: "".to_string(),
            cursor_line: 0,
            cursor_col: 0,
            sidebar_visible: false,
        });

        manager
    }

    pub fn on_event(&mut self, window: &Window, event: &WindowEvent) {
        self.adapter.process_event(window, event);
    }

    pub fn update(&mut self, state: AccessibilityState) {
        self.adapter.update_if_active(|| {
            let root_id = NodeId(1);
            let editor_id = NodeId(2);

            let mut root_builder = NodeBuilder::new(Role::Window);
            root_builder.set_name("Forge IDE");
            root_builder.set_children(vec![editor_id]);
            let root = root_builder.build();

            let mut editor_builder = NodeBuilder::new(Role::TextInput);
            editor_builder.set_name("Editor");
            editor_builder.set_value(state.editor_text);
            let editor = editor_builder.build();

            TreeUpdate {
                nodes: vec![
                    (root_id, root),
                    (editor_id, editor),
                ],
                tree: Some(Tree::new(root_id)),
                focus: editor_id,
            }
        });
    }
}
