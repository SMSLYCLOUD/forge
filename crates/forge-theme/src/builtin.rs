use crate::{Theme, ThemeKind, TokenColor};
use crate::token::TokenSettings;
use std::collections::HashMap;

pub fn forge_dark() -> Theme {
    let mut colors = HashMap::new();
    // Editor
    colors.insert("editor.background".into(), "#1e1e1e".into());
    colors.insert("editor.foreground".into(), "#d4d4d4".into());
    colors.insert("editor.lineHighlightBackground".into(), "#2f3337".into());
    colors.insert("editor.selectionBackground".into(), "#264f78".into());
    colors.insert("editorCursor.foreground".into(), "#aeafad".into());
    colors.insert("editorWhitespace.foreground".into(), "#3e3e42".into());
    colors.insert("editorIndentGuide.background".into(), "#404040".into());
    colors.insert("editorLineNumber.foreground".into(), "#858585".into());
    colors.insert("editorLineNumber.activeForeground".into(), "#c6c6c6".into());

    // UI
    colors.insert("activityBar.background".into(), "#333333".into());
    colors.insert("activityBar.foreground".into(), "#ffffff".into());
    colors.insert("sideBar.background".into(), "#252526".into());
    colors.insert("sideBar.foreground".into(), "#cccccc".into());
    colors.insert("sideBarTitle.foreground".into(), "#bbbbbb".into());
    colors.insert("sideBarSectionHeader.background".into(), "#00000000".into());
    colors.insert("statusBar.background".into(), "#007acc".into());
    colors.insert("statusBar.foreground".into(), "#ffffff".into());
    colors.insert("tab.activeBackground".into(), "#1e1e1e".into());
    colors.insert("tab.inactiveBackground".into(), "#2d2d2d".into());
    colors.insert("tab.activeForeground".into(), "#ffffff".into());
    colors.insert("tab.inactiveForeground".into(), "#969696".into());
    colors.insert("titleBar.activeBackground".into(), "#3c3c3c".into());
    colors.insert("titleBar.activeForeground".into(), "#cccccc".into());

    // Lists & Trees
    colors.insert("list.activeSelectionBackground".into(), "#094771".into());
    colors.insert("list.activeSelectionForeground".into(), "#ffffff".into());
    colors.insert("list.hoverBackground".into(), "#2a2d2e".into());

    // Input
    colors.insert("input.background".into(), "#3c3c3c".into());
    colors.insert("input.foreground".into(), "#cccccc".into());
    colors.insert("input.placeholderForeground".into(), "#a6a6a6".into());

    // Scrollbar
    colors.insert("scrollbarSlider.background".into(), "#79797966".into());
    colors.insert("scrollbarSlider.hoverBackground".into(), "#646464b3".into());
    colors.insert("scrollbarSlider.activeBackground".into(), "#bfbfbf66".into());

    // Git
    colors.insert("gitDecoration.addedResourceForeground".into(), "#81b88b".into());
    colors.insert("gitDecoration.modifiedResourceForeground".into(), "#e2c08d".into());
    colors.insert("gitDecoration.deletedResourceForeground".into(), "#c74e39".into());
    colors.insert("gitDecoration.untrackedResourceForeground".into(), "#73c991".into());

    // Diagnostics
    colors.insert("editorError.foreground".into(), "#f48771".into());
    colors.insert("editorWarning.foreground".into(), "#cca700".into());
    colors.insert("editorInfo.foreground".into(), "#75beff".into());
    colors.insert("editorHint.foreground".into(), "#eeeeeeb3".into());

    // More keys
    colors.insert("button.background".into(), "#0e639c".into());
    colors.insert("button.foreground".into(), "#ffffff".into());
    colors.insert("button.hoverBackground".into(), "#1177bb".into());
    colors.insert("dropdown.background".into(), "#3c3c3c".into());
    colors.insert("dropdown.foreground".into(), "#f0f0f0".into());
    colors.insert("editorWidget.background".into(), "#252526".into());
    colors.insert("editorWidget.border".into(), "#454545".into());
    colors.insert("menu.background".into(), "#3c3c3c".into());
    colors.insert("menu.foreground".into(), "#f0f0f0".into());
    colors.insert("menu.selectionBackground".into(), "#094771".into());
    colors.insert("menu.selectionForeground".into(), "#ffffff".into());
    colors.insert("menu.separatorBackground".into(), "#bbbbbb".into());
    colors.insert("panel.background".into(), "#1e1e1e".into());
    colors.insert("panel.border".into(), "#80808059".into());
    colors.insert("panelTitle.activeBorder".into(), "#e7e7e7".into());
    colors.insert("panelTitle.activeForeground".into(), "#e7e7e7".into());
    colors.insert("panelTitle.inactiveForeground".into(), "#e7e7e799".into());
    colors.insert("peekView.border".into(), "#007acc".into());
    colors.insert("peekViewEditor.background".into(), "#001f33".into());
    colors.insert("peekViewResult.background".into(), "#252526".into());
    colors.insert("pickerGroup.border".into(), "#3f3f46".into());
    colors.insert("pickerGroup.foreground".into(), "#3794ff".into());
    colors.insert("progressBar.background".into(), "#0e70c0".into());
    colors.insert("settings.headerForeground".into(), "#e7e7e7".into());
    colors.insert("settings.modifiedItemIndicator".into(), "#0c7d9d".into());
    colors.insert("textLink.activeForeground".into(), "#3794ff".into());
    colors.insert("textLink.foreground".into(), "#3794ff".into());

    let token_colors = vec![
        token("keyword", "#569cd6"),
        token("control", "#c586c0"),
        token("function", "#dcdcaa"),
        token("type", "#4ec9b0"),
        token("string", "#ce9178"),
        token("number", "#b5cea8"),
        token("comment", "#6a9955"),
        token("operator", "#d4d4d4"),
        token("variable", "#9cdcfe"),
        token("constant", "#4fc1ff"),
        token("class", "#4ec9b0"),
        token("interface", "#9cdcfe"),
        token("parameter", "#9cdcfe"),
        token("property", "#9cdcfe"),
        token("enum", "#4ec9b0"),
    ];

    Theme {
        name: "Forge Dark".into(),
        kind: ThemeKind::Dark,
        colors,
        token_colors,
    }
}

pub fn forge_light() -> Theme {
    let mut colors = HashMap::new();
    // Editor
    colors.insert("editor.background".into(), "#ffffff".into());
    colors.insert("editor.foreground".into(), "#000000".into());
    colors.insert("editor.lineHighlightBackground".into(), "#eeeeee".into());
    colors.insert("editor.selectionBackground".into(), "#add6ff".into());
    colors.insert("editorCursor.foreground".into(), "#000000".into());
    colors.insert("editorWhitespace.foreground".into(), "#e3e4e229".into());
    colors.insert("editorIndentGuide.background".into(), "#e3e4e229".into());
    colors.insert("editorLineNumber.foreground".into(), "#2b91af".into());
    colors.insert("editorLineNumber.activeForeground".into(), "#000000".into());

    // UI
    colors.insert("activityBar.background".into(), "#2c2c2c".into());
    colors.insert("activityBar.foreground".into(), "#ffffff".into());
    colors.insert("sideBar.background".into(), "#f3f3f3".into());
    colors.insert("sideBar.foreground".into(), "#616161".into());
    colors.insert("statusBar.background".into(), "#007acc".into());
    colors.insert("statusBar.foreground".into(), "#ffffff".into());
    colors.insert("tab.activeBackground".into(), "#ffffff".into());
    colors.insert("tab.inactiveBackground".into(), "#ececec".into());
    colors.insert("tab.activeForeground".into(), "#333333".into());
    colors.insert("tab.inactiveForeground".into(), "#333333b3".into());
    colors.insert("titleBar.activeBackground".into(), "#dddddd".into());
    colors.insert("titleBar.activeForeground".into(), "#333333".into());

    // Lists
    colors.insert("list.activeSelectionBackground".into(), "#0060c0".into());
    colors.insert("list.activeSelectionForeground".into(), "#ffffff".into());
    colors.insert("list.hoverBackground".into(), "#e8e8e8".into());

    let token_colors = vec![
        token("keyword", "#0000ff"),
        token("control", "#af00db"),
        token("function", "#795e26"),
        token("type", "#267f99"),
        token("string", "#a31515"),
        token("number", "#098658"),
        token("comment", "#008000"),
        token("operator", "#000000"),
        token("variable", "#001080"),
        token("constant", "#0000ff"),
    ];

    Theme {
        name: "Forge Light".into(),
        kind: ThemeKind::Light,
        colors,
        token_colors,
    }
}

fn token(scope: &str, color: &str) -> TokenColor {
    TokenColor {
        scope: vec![scope.into()],
        settings: TokenSettings {
            foreground: Some(color.into()),
            font_style: None,
        },
    }
}
