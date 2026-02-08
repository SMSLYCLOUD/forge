#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelState {
    Visible,
    Hidden,
}

#[derive(Debug, Clone)]
pub struct Layout {
    /// Activity Bar (left edge)
    pub activity_bar_visible: bool,
    /// Side Panel (left of editor)
    pub side_panel_state: PanelState,
    pub active_side_panel: SidePanelContent,

    /// Editor Area (center)
    /// In a real app, this would be a tree of split panes.
    /// For now, we assume a single editor group.

    /// Bottom Panel (below editor)
    pub bottom_panel_state: PanelState,
    pub active_bottom_panel: BottomPanelContent,

    /// Status Bar (bottom edge)
    pub status_bar_visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidePanelContent {
    FileTree,
    Search,
    SourceControl,
    Extensions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomPanelContent {
    Terminal,
    Output,
    Problems,
    DebugConsole,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            activity_bar_visible: true,
            side_panel_state: PanelState::Visible,
            active_side_panel: SidePanelContent::FileTree,
            bottom_panel_state: PanelState::Hidden, // Hidden by default per spec (progressive disclosure)
            active_bottom_panel: BottomPanelContent::Terminal,
            status_bar_visible: true,
        }
    }
}

impl Layout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_side_panel(&mut self) {
        self.side_panel_state = match self.side_panel_state {
            PanelState::Visible => PanelState::Hidden,
            PanelState::Hidden => PanelState::Visible,
        };
    }

    pub fn toggle_bottom_panel(&mut self) {
        self.bottom_panel_state = match self.bottom_panel_state {
            PanelState::Visible => PanelState::Hidden,
            PanelState::Hidden => PanelState::Visible,
        };
    }
}
