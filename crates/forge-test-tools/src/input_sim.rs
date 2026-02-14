use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{Key, ModifiersState};

pub fn click(_x: f32, _y: f32) -> WindowEvent {
    // WindowEvent construction is difficult in winit 0.30 due to private fields and opaque types.
    // Stubbing this for now to allow compilation.
    unimplemented!("Input simulation not fully supported in winit 0.30 yet")
}

pub fn type_text(_text: &str) -> Vec<WindowEvent> {
    unimplemented!("Input simulation not fully supported in winit 0.30 yet")
}

pub fn key_combo(_modifiers: ModifiersState, _key: Key) -> WindowEvent {
    unimplemented!("Input simulation not fully supported in winit 0.30 yet")
}
