#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoldRange {
    pub start_line: usize,
    pub end_line: usize,
    pub folded: bool,
}

pub struct FoldingManager {
    pub ranges: Vec<FoldRange>,
}

impl Default for FoldingManager {
    fn default() -> Self {
        Self { ranges: Vec::new() }
    }
}

impl FoldingManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compute_ranges(&mut self, text: &str) -> Vec<FoldRange> {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            self.ranges.clear();
            return Vec::new();
        }

        let mut indents: Vec<usize> = Vec::with_capacity(lines.len());
        // Simple heuristic: 1 char = 1 indent unit. Tabs? Assume 4 spaces.
        // Or just count leading whitespace.
        // Prompt says "block starts when indent increases".

        for line in &lines {
            let mut width = 0;
            for c in line.chars() {
                if c == ' ' {
                    width += 1;
                } else if c == '\t' {
                    width += 4; // Assume 4 for simplicity unless passed
                } else {
                    break;
                }
            }
            indents.push(width);
        }

        // Handle empty lines: they interrupt the block?
        // Usually empty lines inherit indent from previous line for folding purposes?
        // Or we ignore them during indent calculation but count them in ranges?
        // Let's refine: if line is empty, treat indent as "same as next non-empty" or "max of prev/next"?
        // Simpler: treat empty lines as having -1 indent, and skip them in logic,
        // but include them in the range if they are inside.

        // Revised loop:
        let mut computed_ranges = Vec::new();
        let mut stack: Vec<(usize, usize)> = Vec::new(); // (start_line, base_indent)

        // We need to be careful with empty lines.
        // If we have:
        // header
        //   content
        //
        //   more content
        //
        // The empty line should be part of the block.
        // So we should effectively "fill in" indents for empty lines based on context.
        // Forward pass to fill empty lines with next non-empty indent?

        let mut effective_indents = indents.clone();
        let mut next_valid_indent = 0;
        for i in (0..lines.len()).rev() {
            if lines[i].trim().is_empty() {
                effective_indents[i] = next_valid_indent;
            } else {
                next_valid_indent = effective_indents[i];
            }
        }

        // Actually, if file ends with empty lines, they get 0 indent (default next_valid_indent).
        // If block ends with empty lines, they might be excluded or included.
        // Let's stick to the stack logic with effective_indents.

        for i in 0..lines.len() {
            let indent = effective_indents[i];

            // Check if we need to pop
            while let Some(&(start, base)) = stack.last() {
                if indent <= base {
                    // Block ended
                    stack.pop();
                    computed_ranges.push(FoldRange {
                        start_line: start,
                        end_line: i - 1,
                        folded: false, // Default unfolded
                    });
                } else {
                    break;
                }
            }

            // Check if we need to push
            // We start a block if next line has higher indent.
            // Wait, looking ahead is easier.
            if i + 1 < lines.len() {
                let next_indent = effective_indents[i + 1];
                if next_indent > indent {
                    // Start of block
                    stack.push((i, indent));
                }
            }
        }

        // Close remaining blocks
        while let Some((start, _)) = stack.pop() {
            computed_ranges.push(FoldRange {
                start_line: start,
                end_line: lines.len() - 1,
                folded: false,
            });
        }

        // Sort ranges? Usually good to have them sorted by start_line.
        computed_ranges.sort_by_key(|r| r.start_line);

        // Merge with existing ranges to preserve folded state?
        // The prompt implies `compute_ranges` just returns ranges.
        // But `FoldingManager` has `ranges` field.
        // The method signature `compute_ranges(text)` returns `Vec<FoldRange>`.
        // Usually this method updates `self.ranges` or returns new ones.
        // If it updates `self.ranges`, it should try to preserve `folded` status of matching ranges.

        // I will implement it to update self.ranges AND return them.
        // Preserving state:
        let old_ranges = std::mem::take(&mut self.ranges);

        for new_range in &mut computed_ranges {
            if let Some(old) = old_ranges
                .iter()
                .find(|r| r.start_line == new_range.start_line && r.end_line == new_range.end_line)
            {
                new_range.folded = old.folded;
            }
        }

        self.ranges = computed_ranges.clone();
        computed_ranges
    }

    pub fn toggle_fold(&mut self, line: usize) {
        // Find range starting at line
        if let Some(range) = self.ranges.iter_mut().find(|r| r.start_line == line) {
            range.folded = !range.folded;
        }
    }

    pub fn fold_all(&mut self) {
        for range in &mut self.ranges {
            range.folded = true;
        }
    }

    pub fn unfold_all(&mut self) {
        for range in &mut self.ranges {
            range.folded = false;
        }
    }

    pub fn is_line_visible(&self, line: usize) -> bool {
        // A line is hidden if it is inside a folded range.
        // Range starts at `start_line` (visible header) and covers `start_line + 1 ..= end_line`.

        for range in &self.ranges {
            if range.folded {
                if line > range.start_line && line <= range.end_line {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_ranges() {
        let mut fm = FoldingManager::new();
        let text = "
fn foo() {
    let x = 1;
    if x > 0 {
        print(x);
    }
}
";
        // 0: empty -> indent 0
        // 1: fn foo -> indent 0
        // 2:     let -> indent 4
        // 3:     if -> indent 4
        // 4:         print -> indent 8
        // 5:     } -> indent 4
        // 6: } -> indent 0
        // 7: empty -> indent 0

        // Expected ranges:
        // 1. start: 1 (fn foo), end: 5 (}). Base indent 0. Next indent 4.
        // 2. start: 3 (if), end: 4 (print). Base indent 4. Next indent 8.

        let ranges = fm.compute_ranges(text.trim());

        // text.trim() removes first newline.
        // 0: fn foo (0)
        // 1:     let (4)
        // 2:     if (4)
        // 3:         print (8)
        // 4:     } (4)
        // 5: } (0)

        assert_eq!(ranges.len(), 2);

        let r1 = ranges.iter().find(|r| r.start_line == 0).unwrap();
        assert_eq!(r1.end_line, 4); // } is at 4. Block covers 1..4.

        let r2 = ranges.iter().find(|r| r.start_line == 2).unwrap();
        assert_eq!(r2.end_line, 3); // print is at 3. Block covers 3..3.
    }

    #[test]
    fn test_toggle_fold() {
        let mut fm = FoldingManager::new();
        let text = "
block
    content
";
        fm.compute_ranges(text.trim());
        // Range start 0, end 1.

        assert!(fm.is_line_visible(1));

        fm.toggle_fold(0);
        assert!(!fm.is_line_visible(1));
        assert!(fm.is_line_visible(0)); // Header always visible

        fm.toggle_fold(0);
        assert!(fm.is_line_visible(1));
    }

    #[test]
    fn test_fold_unfold_all() {
        let mut fm = FoldingManager::new();
        let text = "
a
    b
    c
        d
";
        // Ranges:
        // 0 -> 3 (a covers b, c, d)
        // 2 -> 3 (c covers d)

        fm.compute_ranges(text.trim());
        assert_eq!(fm.ranges.len(), 2);

        fm.fold_all();
        assert!(!fm.is_line_visible(1));
        assert!(!fm.is_line_visible(3));

        fm.unfold_all();
        assert!(fm.is_line_visible(1));
        assert!(fm.is_line_visible(3));
    }
}
