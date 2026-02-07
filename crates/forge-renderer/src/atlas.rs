use std::collections::HashMap;

/// Glyph atlas for caching rendered glyphs in a texture
#[derive(Debug)]
pub struct GlyphAtlas {
    /// Map from (font_id, glyph_id) to atlas position
    glyph_positions: HashMap<(u32, u32), AtlasPosition>,
    /// Current atlas dimensions
    width: u32,
    height: u32,
    /// Next free position for a new glyph
    next_x: u32,
    next_y: u32,
    /// Current row height
    row_height: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct AtlasPosition {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl GlyphAtlas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            glyph_positions: HashMap::new(),
            width,
            height,
            next_x: 0,
            next_y: 0,
            row_height: 0,
        }
    }

    /// Try to allocate space for a new glyph
    /// Returns the position in the atlas, or None if the atlas is full
    pub fn allocate(
        &mut self,
        font_id: u32,
        glyph_id: u32,
        width: u32,
        height: u32,
    ) -> Option<AtlasPosition> {
        // Check if already cached
        if let Some(pos) = self.glyph_positions.get(&(font_id, glyph_id)) {
            return Some(*pos);
        }

        // Try to fit in current row
        if self.next_x + width > self.width {
            // Move to next row
            self.next_x = 0;
            self.next_y += self.row_height;
            self.row_height = 0;
        }

        // Check if we have vertical space
        if self.next_y + height > self.height {
            return None; // Atlas is full
        }

        let pos = AtlasPosition {
            x: self.next_x,
            y: self.next_y,
            width,
            height,
        };

        self.glyph_positions.insert((font_id, glyph_id), pos);
        self.next_x += width;
        self.row_height = self.row_height.max(height);

        Some(pos)
    }

    /// Get the position of a cached glyph
    pub fn get(&self, font_id: u32, glyph_id: u32) -> Option<AtlasPosition> {
        self.glyph_positions.get(&(font_id, glyph_id)).copied()
    }

    /// Clear the atlas
    pub fn clear(&mut self) {
        self.glyph_positions.clear();
        self.next_x = 0;
        self.next_y = 0;
        self.row_height = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_allocation() {
        let mut atlas = GlyphAtlas::new(256, 256);

        let pos1 = atlas.allocate(0, 'a' as u32, 10, 10).unwrap();
        assert_eq!(pos1.x, 0);
        assert_eq!(pos1.y, 0);

        let pos2 = atlas.allocate(0, 'b' as u32, 10, 10).unwrap();
        assert_eq!(pos2.x, 10);
        assert_eq!(pos2.y, 0);

        // Allocate same glyph again - should return cached position
        let pos_cached = atlas.allocate(0, 'a' as u32, 10, 10).unwrap();
        assert_eq!(pos_cached.x, pos1.x);
        assert_eq!(pos_cached.y, pos1.y);
    }
}
