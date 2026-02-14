use forge_debug::{DebugSession, Variable};
use crate::rect_renderer::Rect;
use crate::ui::Zone;

pub struct VariablesTree;

impl VariablesTree {
    pub fn render_rects(session: &DebugSession, zone: &Zone, theme: &forge_theme::Theme) -> Vec<Rect> {
        let mut rects = Vec::new();
        // Placeholder for background if needed
        // Actual rendering would calculate text positions, but this function returns background highlights
        // We'll leave it empty for now, or add selection highlight
        rects
    }

    pub fn render_text(session: &DebugSession) -> String {
        let mut text = String::new();
        text.push_str("VARIABLES\n");
        for scope in &session.scopes {
            text.push_str(&format!("> {}\n", scope.name));
            if scope.expanded {
                for var in &scope.variables {
                    Self::render_var(&mut text, var, 1);
                }
            }
        }
        text
    }

    fn render_var(text: &mut String, var: &Variable, depth: usize) {
        let indent = "  ".repeat(depth);
        let icon = if !var.children.is_empty() {
            if var.expanded { "v " } else { "> " }
        } else {
            "  "
        };
        text.push_str(&format!("{}{}{}: {}\n", indent, icon, var.name, var.value));
        if var.expanded {
            for child in &var.children {
                Self::render_var(text, child, depth + 1);
            }
        }
    }
}

pub struct CallStackList;

impl CallStackList {
    pub fn render_text(session: &DebugSession) -> String {
        let mut text = String::new();
        text.push_str("CALL STACK\n");
        for frame in &session.stack_frames {
            text.push_str(&format!("  {} ({}:{})\n", frame.name, frame.file, frame.line));
        }
        text
    }
}
