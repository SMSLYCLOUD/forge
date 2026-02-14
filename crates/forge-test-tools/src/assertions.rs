pub trait AppStateAssertion {
    fn get_active_file(&self) -> Option<String>;
    fn get_editor_text(&self) -> String;
}

pub fn assert_file_open<A: AppStateAssertion>(app: &A, expected_path: &str) {
    let path = app.get_active_file().expect("No file open");
    if !path.ends_with(expected_path) {
        panic!("Expected file ending with {}, got {}", expected_path, path);
    }
}

pub fn assert_text_contains<A: AppStateAssertion>(app: &A, needle: &str) {
    let text = app.get_editor_text();
    if !text.contains(needle) {
        panic!("Text does not contain '{}'", needle);
    }
}
