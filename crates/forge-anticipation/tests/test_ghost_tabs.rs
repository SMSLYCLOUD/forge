use forge_anticipation::GhostTabsEngine;
use std::fs;

#[test]
fn test_ghost_tabs_persistence() {
    let mut engine = GhostTabsEngine::new();
    engine.on_file_open("src/main.rs");
    engine.on_file_open("src/lib.rs");
    engine.on_file_open("src/main.rs");

    // P(lib.rs | main.rs) = 0.5 (initial open) -> No wait.
    // main -> lib -> main
    // transitions from main: lib (1)
    // transitions from lib: main (1)

    let path = "test_ghost_tabs.json";
    engine.save_to_file(path).unwrap();

    let loaded = GhostTabsEngine::load_from_file(path).unwrap();
    // Verify loaded state logic (can't access internal fields directly unless pub)
    // But we can verify behavior.
    // If I call predict on loaded engine for "src/lib.rs", it should suggest "src/main.rs"
    // However, `on_file_open` sets current file.
    // `loaded` has `current_file` from saved state (which was "src/main.rs").
    // So `loaded.get_suggestions()` should return "src/lib.rs" if probability > 0.3.
    // Wait.
    // main -> lib -> main.
    // Current is main.
    // Transitions from main: lib (1). Total 1.
    // P(lib|main) = 1.0 > 0.3.
    // So get_suggestions should return ["src/lib.rs"].

    let suggestions = loaded.get_suggestions();
    assert_eq!(suggestions, vec!["src/lib.rs"]);

    fs::remove_file(path).unwrap();
}
