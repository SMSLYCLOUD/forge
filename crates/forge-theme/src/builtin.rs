use crate::token::TokenSettings;
use crate::{Theme, ThemeKind, TokenColor};
use std::collections::HashMap;

pub fn forge_dark() -> Theme {
    let mut colors = HashMap::new();

    // ─── Base Colors ───
    colors.insert("focusBorder".into(), "#007fd4".into());
    colors.insert("foreground".into(), "#cccccc".into());
    colors.insert("widget.shadow".into(), "#000000".into());
    colors.insert("selection.background".into(), "#3399ff".into());
    colors.insert("descriptionForeground".into(), "#ccccccb3".into());
    colors.insert("errorForeground".into(), "#f48771".into());

    // ─── Title Bar ───
    colors.insert("titleBar.activeBackground".into(), "#3c3c3c".into());
    colors.insert("titleBar.activeForeground".into(), "#cccccc".into());
    colors.insert("titleBar.inactiveBackground".into(), "#3c3c3c99".into());
    colors.insert("titleBar.inactiveForeground".into(), "#cccccc99".into());
    colors.insert("titleBar.border".into(), "#3c3c3c".into());

    // ─── Text / Editor ───
    colors.insert("editor.background".into(), "#1e1e1e".into());
    colors.insert("editor.foreground".into(), "#cccccc".into());
    colors.insert("editor.lineHighlightBackground".into(), "#2f3337".into());
    colors.insert("editor.selectionBackground".into(), "#264f78".into());
    colors.insert(
        "editor.inactiveSelectionBackground".into(),
        "#3a3d41".into(),
    );
    colors.insert("editorCursor.foreground".into(), "#aeafad".into());
    colors.insert("editorWhitespace.foreground".into(), "#3e3e42".into());
    colors.insert("editorIndentGuide.background".into(), "#404040".into());
    colors.insert(
        "editorIndentGuide.activeBackground".into(),
        "#707070".into(),
    );
    colors.insert("editorLineNumber.foreground".into(), "#858585".into());
    colors.insert("editorLineNumber.activeForeground".into(), "#c6c6c6".into());
    colors.insert("editorRuler.foreground".into(), "#5a5a5a".into());
    colors.insert("editorCodeLens.foreground".into(), "#999999".into());
    colors.insert("editorBracketMatch.background".into(), "#0064001a".into());
    colors.insert("editorBracketMatch.border".into(), "#888888".into());

    // ─── Activity Bar ───
    colors.insert("activityBar.background".into(), "#333333".into());
    colors.insert("activityBar.foreground".into(), "#ffffff".into());
    colors.insert("activityBar.inactiveForeground".into(), "#ffffff66".into());
    colors.insert("activityBar.border".into(), "#252526".into());
    colors.insert("activityBarBadge.background".into(), "#007acc".into());
    colors.insert("activityBarBadge.foreground".into(), "#ffffff".into());
    colors.insert("activityBar.activeBorder".into(), "#ffffff".into());
    colors.insert("activityBar.activeBackground".into(), "#ffffff14".into());

    // ─── Side Bar ───
    colors.insert("sideBar.background".into(), "#252526".into());
    colors.insert("sideBar.foreground".into(), "#cccccc".into());
    colors.insert("sideBar.border".into(), "#252526".into());
    colors.insert("sideBarTitle.foreground".into(), "#bbbbbb".into());
    colors.insert("sideBarSectionHeader.background".into(), "#00000000".into()); // often transparent in dark+
    colors.insert("sideBarSectionHeader.foreground".into(), "#cccccc".into());
    colors.insert("sideBarSectionHeader.border".into(), "#cccccc33".into());

    // ─── Status Bar ───
    colors.insert("statusBar.background".into(), "#007acc".into());
    colors.insert("statusBar.foreground".into(), "#ffffff".into());
    colors.insert("statusBar.noFolderBackground".into(), "#68217a".into());
    colors.insert("statusBar.debuggingBackground".into(), "#cc6633".into());
    colors.insert("statusBarItem.hoverBackground".into(), "#ffffff2e".into());
    colors.insert("statusBar.border".into(), "#007acc".into());

    // ─── Tabs / Editor Groups ───
    colors.insert("editorGroup.border".into(), "#444444".into());
    colors.insert("editorGroupHeader.tabsBackground".into(), "#252526".into());
    colors.insert("tab.activeBackground".into(), "#1e1e1e".into());
    colors.insert("tab.inactiveBackground".into(), "#2d2d2d".into());
    colors.insert("tab.activeForeground".into(), "#ffffff".into());
    colors.insert("tab.inactiveForeground".into(), "#969696".into());
    colors.insert("tab.border".into(), "#252526".into());
    colors.insert("tab.activeBorder".into(), "#1e1e1e".into()); // Usually no border or top border
    colors.insert("tab.activeBorderTop".into(), "#007acc".into()); // VS Code top border

    // ─── Breadcrumbs ───
    colors.insert("breadcrumb.background".into(), "#1e1e1e".into());
    colors.insert("breadcrumb.foreground".into(), "#a9a9a9".into());
    colors.insert("breadcrumb.focusForeground".into(), "#e0e0e0".into());

    // ─── Lists & Trees ───
    colors.insert("list.activeSelectionBackground".into(), "#094771".into());
    colors.insert("list.activeSelectionForeground".into(), "#ffffff".into());
    colors.insert("list.inactiveSelectionBackground".into(), "#37373d".into());
    colors.insert("list.hoverBackground".into(), "#2a2d2e".into());
    colors.insert("list.dropBackground".into(), "#062f4a".into());
    colors.insert("list.highlightForeground".into(), "#18a3ff".into());
    colors.insert("list.focusBackground".into(), "#062f4a".into());

    // ─── Inputs ───
    colors.insert("input.background".into(), "#3c3c3c".into());
    colors.insert("input.foreground".into(), "#cccccc".into());
    colors.insert("input.placeholderForeground".into(), "#a6a6a6".into());
    colors.insert("inputOption.activeBorder".into(), "#007acc".into());
    colors.insert("inputValidation.errorBackground".into(), "#5a1d1d".into());
    colors.insert("inputValidation.errorBorder".into(), "#be1100".into());

    // ─── Scrollbar ───
    colors.insert("scrollbar.shadow".into(), "#000000".into());
    colors.insert("scrollbarSlider.background".into(), "#79797966".into());
    colors.insert("scrollbarSlider.hoverBackground".into(), "#646464b3".into());
    colors.insert(
        "scrollbarSlider.activeBackground".into(),
        "#bfbfbf66".into(),
    );

    // ─── Git ───
    colors.insert(
        "gitDecoration.addedResourceForeground".into(),
        "#81b88b".into(),
    );
    colors.insert(
        "gitDecoration.modifiedResourceForeground".into(),
        "#e2c08d".into(),
    );
    colors.insert(
        "gitDecoration.deletedResourceForeground".into(),
        "#c74e39".into(),
    );
    colors.insert(
        "gitDecoration.untrackedResourceForeground".into(),
        "#73c991".into(),
    );
    colors.insert(
        "gitDecoration.ignoredResourceForeground".into(),
        "#8c8c8c".into(),
    );

    // ─── Diagnostics ───
    colors.insert("editorError.foreground".into(), "#f48771".into());
    colors.insert("editorWarning.foreground".into(), "#cca700".into());
    colors.insert("editorInfo.foreground".into(), "#75beff".into());
    colors.insert("editorHint.foreground".into(), "#eeeeeeb3".into());

    // ─── Widgets & Overlays ───
    colors.insert("editorWidget.background".into(), "#252526".into());
    colors.insert("editorWidget.border".into(), "#454545".into());
    colors.insert("editorSuggestWidget.background".into(), "#252526".into());
    colors.insert("editorSuggestWidget.border".into(), "#454545".into());
    colors.insert("editorSuggestWidget.foreground".into(), "#d4d4d4".into());
    colors.insert(
        "editorSuggestWidget.selectedBackground".into(),
        "#062f4a".into(),
    );
    colors.insert("editorHoverWidget.background".into(), "#252526".into());
    colors.insert("editorHoverWidget.border".into(), "#454545".into());
    colors.insert("debugToolBar.background".into(), "#333333".into());

    // ─── Menus ───
    colors.insert("menu.background".into(), "#3c3c3c".into());
    colors.insert("menu.foreground".into(), "#f0f0f0".into());
    colors.insert("menu.selectionBackground".into(), "#094771".into());
    colors.insert("menu.selectionForeground".into(), "#ffffff".into());
    colors.insert("menu.separatorBackground".into(), "#bbbbbb".into());
    colors.insert("menubar.selectionBackground".into(), "#ffffff1a".into());
    colors.insert("menubar.selectionForeground".into(), "#cccccc".into());

    // ─── Panels ───
    colors.insert("panel.background".into(), "#1e1e1e".into());
    colors.insert("panel.border".into(), "#80808059".into());
    colors.insert("panelTitle.activeBorder".into(), "#e7e7e7".into());
    colors.insert("panelTitle.activeForeground".into(), "#e7e7e7".into());
    colors.insert("panelTitle.inactiveForeground".into(), "#e7e7e799".into());

    // ─── Peek View ───
    colors.insert("peekView.border".into(), "#007acc".into());
    colors.insert("peekViewEditor.background".into(), "#001f33".into());
    colors.insert(
        "peekViewEditor.matchHighlightBackground".into(),
        "#ff8f0099".into(),
    );
    colors.insert("peekViewResult.background".into(), "#252526".into());
    colors.insert("peekViewResult.fileForeground".into(), "#ffffff".into());
    colors.insert("peekViewResult.lineForeground".into(), "#bbbbbb".into());
    colors.insert(
        "peekViewResult.matchHighlightBackground".into(),
        "#ea5c004d".into(),
    );
    colors.insert(
        "peekViewResult.selectionBackground".into(),
        "#3399ff33".into(),
    );
    colors.insert(
        "peekViewResult.selectionForeground".into(),
        "#ffffff".into(),
    );
    colors.insert("peekViewTitle.background".into(), "#1e1e1e".into());
    colors.insert(
        "peekViewTitleDescription.foreground".into(),
        "#ccccccb3".into(),
    );
    colors.insert("peekViewTitleLabel.foreground".into(), "#ffffff".into());

    // ─── Other ───
    colors.insert("pickerGroup.border".into(), "#3f3f46".into());
    colors.insert("pickerGroup.foreground".into(), "#3794ff".into());
    colors.insert("progressBar.background".into(), "#0e70c0".into());
    colors.insert("settings.headerForeground".into(), "#e7e7e7".into());
    colors.insert("settings.modifiedItemIndicator".into(), "#0c7d9d".into());
    colors.insert("textLink.activeForeground".into(), "#3794ff".into());
    colors.insert("textLink.foreground".into(), "#3794ff".into());
    colors.insert("tree.indentGuidesStroke".into(), "#585858".into());

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
