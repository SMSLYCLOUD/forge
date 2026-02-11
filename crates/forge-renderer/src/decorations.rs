//! Line decoration framework: underlines, highlights, inline annotations.

#[derive(Debug, Clone)]
pub enum Decoration {
    Underline { line: usize, start_col: usize, end_col: usize, color: [u8; 4], style: UnderlineStyle },
    LineBackground { line: usize, color: [u8; 4] },
    InlineText { line: usize, col: usize, text: String, color: [u8; 4] },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnderlineStyle { Solid, Wavy, Dashed, Dotted }

#[derive(Debug, Default)]
pub struct DecorationLayer {
    decorations: Vec<Decoration>,
}

impl DecorationLayer {
    pub fn new() -> Self { Self::default() }

    pub fn add(&mut self, dec: Decoration) { self.decorations.push(dec); }
    pub fn clear(&mut self) { self.decorations.clear(); }

    pub fn get_line_decorations(&self, line: usize) -> Vec<&Decoration> {
        self.decorations.iter().filter(|d| match d {
            Decoration::Underline { line: l, .. } => *l == line,
            Decoration::LineBackground { line: l, .. } => *l == line,
            Decoration::InlineText { line: l, .. } => *l == line,
        }).collect()
    }

    pub fn count(&self) -> usize { self.decorations.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn add_and_query() {
        let mut layer = DecorationLayer::new();
        layer.add(Decoration::LineBackground { line: 5, color: [255, 0, 0, 50] });
        layer.add(Decoration::LineBackground { line: 10, color: [0, 255, 0, 50] });
        assert_eq!(layer.get_line_decorations(5).len(), 1);
        assert_eq!(layer.get_line_decorations(7).len(), 0);
    }
    #[test] fn clear_works() {
        let mut layer = DecorationLayer::new();
        layer.add(Decoration::LineBackground { line: 1, color: [0, 0, 0, 0] });
        assert_eq!(layer.count(), 1);
        layer.clear();
        assert_eq!(layer.count(), 0);
    }
}
