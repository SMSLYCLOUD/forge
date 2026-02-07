use forge_core::*;

fn main() {
    // Test basic buffer undoredo
    let mut buffer = Buffer::from_str("original");
    println!("Initial: '{}'", buffer.text());

    let change = Change::insert(Position::new(8), " text".to_string());
    let tx = Transaction::new(ChangeSet::with_change(change), None);

    buffer.apply(tx);
    println!("After apply: '{}'", buffer.text());

    buffer.undo();
    println!("After undo: '{}'", buffer.text());

    buffer.redo();
    println!("After redo: '{}'", buffer.text());
}
