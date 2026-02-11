use forge_core::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum CompletionKind {
    Function,
    Variable,
    Keyword,
    Snippet,
    Type,
    Module,
}

#[derive(Clone, Debug)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
    pub insert_text: String,
}

pub struct Autocomplete {
    pub visible: bool,
    pub items: Vec<CompletionItem>,
    pub selected: usize,
    pub trigger_pos: Position,
}

impl Autocomplete {
    pub fn new() -> Self {
        Self {
            visible: false,
            items: Vec::new(),
            selected: 0,
            trigger_pos: Position::zero(),
        }
    }

    pub fn show(&mut self, pos: Position, items: Vec<CompletionItem>) {
        if items.is_empty() {
            self.visible = false;
            return;
        }
        self.visible = true;
        self.items = items;
        self.selected = 0;
        self.trigger_pos = pos;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.items.clear();
    }

    pub fn next(&mut self) {
        if self.visible && !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }

    pub fn prev(&mut self) {
        if self.visible && !self.items.is_empty() {
            if self.selected == 0 {
                self.selected = self.items.len() - 1;
            } else {
                self.selected -= 1;
            }
        }
    }

    pub fn get_selected(&self) -> Option<&CompletionItem> {
        if self.visible {
            self.items.get(self.selected)
        } else {
            None
        }
    }
}
