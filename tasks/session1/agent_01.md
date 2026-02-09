# Agent 01 — Fix All Compiler Warnings + Add forge-types Crate

> **Read `tasks/GLOBAL_RULES.md` first.**

## Task A: Fix All Compiler Warnings

Audit every crate for warnings. Run `cargo check --workspace 2>&1` and fix ALL:
- Remove dead code or add `#[allow(dead_code)] // TODO: used in Phase X`
- Add `_` prefix to intentionally unused variables
- Fix unused imports
- Fix missing docs on public items

**Acceptance**: `cargo check --workspace 2>&1 | Select-String "warning"` returns ZERO lines.

## Task B: Add `forge-types` Crate

Extract shared types into a new crate so other crates don't duplicate them.

### Create `crates/forge-types/Cargo.toml`
```toml
[package]
name = "forge-types"
version.workspace = true
edition.workspace = true
[dependencies]
serde = { workspace = true }
```

### Create `crates/forge-types/src/lib.rs`
```rust
//! Shared types used across all Forge crates.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self { Self { r, g, b, a } }
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self { Self { r, g, b, a: 1.0 } }
    pub fn to_u8_array(&self) -> [u8; 4] {
        [(self.r * 255.0) as u8, (self.g * 255.0) as u8,
         (self.b * 255.0) as u8, (self.a * 255.0) as u8]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect { pub x: f32, pub y: f32, pub width: f32, pub height: f32 }

impl Rect {
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position { pub line: usize, pub col: usize }

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size { pub width: f32, pub height: f32 }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn color_to_u8() {
        let c = Color::rgb(1.0, 0.5, 0.0);
        assert_eq!(c.to_u8_array(), [255, 127, 0, 255]);
    }
    #[test]
    fn rect_contains() {
        let r = Rect { x: 10.0, y: 10.0, width: 100.0, height: 50.0 };
        assert!(r.contains(50.0, 30.0));
        assert!(!r.contains(5.0, 5.0));
    }
}
```

### Add to root `Cargo.toml`
Add `"crates/forge-types"` to `[workspace] members` and `forge-types = { path = "crates/forge-types" }` to `[workspace.dependencies]`.

**Acceptance**: `cargo check --workspace && cargo test -p forge-types` passes with 0 errors, 0 warnings.
