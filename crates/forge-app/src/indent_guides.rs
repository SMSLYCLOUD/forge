#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuideLine {
    pub col: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub active: bool,
}

pub struct IndentGuides;

impl IndentGuides {
    pub fn compute(text: &str, tab_size: u32, cursor_line: usize) -> Vec<GuideLine> {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return Vec::new();
        }

        let mut indents: Vec<Option<usize>> = Vec::with_capacity(lines.len());
        let mut max_indent = 0;

        for line in &lines {
            if line.trim().is_empty() {
                indents.push(None);
            } else {
                let mut width = 0;
                for c in line.chars() {
                    if c == ' ' {
                        width += 1;
                    } else if c == '\t' {
                        width += tab_size as usize;
                    } else {
                        break;
                    }
                }
                let level = width / (tab_size as usize);
                indents.push(Some(level));
                if level > max_indent {
                    max_indent = level;
                }
            }
        }

        let mut guides = Vec::new();

        // Level 0 doesn't have a guide (it's the left margin). Guides start at level 1?
        // Usually vertical lines are at indentation steps.
        // If code is indented at level 1, there is a line at level 0 (left edge)? No.
        // Usually guides appear AT the indentation column.
        // Level 1 indentation (e.g. 4 spaces) -> guide at column 0? Or column 4?
        // VS Code shows guide at indent level boundaries.
        // Guide 1 is at indent 1? No, guide 1 connects lines with indent >= 1.
        // It is drawn at the *start* of the indent.
        // So for level 1 (4 spaces), the guide is at x=0 (relative to block content) or x=4?
        // Actually, guide is at `(level - 1) * tab_size`?
        // If I have:
        // def foo():
        //     print("hi")
        //
        // Indent of `print` is 1. We draw a guide connecting `print` to... nothing if it's one line.
        // If we have:
        // if x:
        //     a
        //     b
        // Indent is 1. We draw a vertical line at indentation 0? No, at indentation 1.
        // But the line is AT the indentation.
        // Let's assume `col` in `GuideLine` is the character column index.
        // If `tab_size`=4. Level 1 starts at col 4.
        // Guide for level 1 usually appears at col 0? Or col 4?
        // VS Code: Indent guide for level 1 is drawn at 0 indent width? No, it's drawn at the indentation of the parent.
        // For a block at level 1, the guide is at level 0 position (left).
        // For a block at level 2, the guide is at level 1 position.
        // So for level `l`, the guide is at `(l-1) * tab_size`.

        for level in 1..=max_indent {
            let guide_col = (level - 1) * (tab_size as usize); // 0-based index of the guide line

            let mut start_line: Option<usize> = None;

            for (i, indent_opt) in indents.iter().enumerate() {
                let has_indent = match indent_opt {
                    Some(indent) => *indent >= level,
                    None => {
                        // Empty line. Check context?
                        // If we are tracking a range, we assume it continues.
                        // If we are NOT tracking, we don't start one on empty line.
                        start_line.is_some()
                    }
                };

                if has_indent {
                    if start_line.is_none() {
                        start_line = Some(i);
                    }
                } else {
                    if let Some(start) = start_line {
                        // Range ended at i-1
                        // But wait, indentation guides usually don't include the parent line (which has lower indent).
                        // They start at the first line with higher indent.
                        // And usually go up to the last line with higher indent.
                        // Filter out single-line blocks? Usually yes, or maybe not.

                        guides.push(GuideLine {
                            col: guide_col,
                            start_line: start,
                            end_line: i, // exclusive end? Prompt says "start_line, end_line". usually inclusive or range.
                            // Let's assume inclusive range of the guide visibility?
                            // If I return `GuideLine` for rendering, usually top to bottom.
                            // If I say 0 to 1, does it cover line 0 and 1?
                            // Let's assume `end_line` is exclusive like range 0..2
                            // But prompt struct doesn't specify.
                            // Renderers usually want y_start, y_end.
                            // "start_line: usize, end_line: usize".
                            // I'll make it exclusive (standard Rust range).
                            active: false,
                        });
                        start_line = None;
                    }
                }
            }

            if let Some(start) = start_line {
                guides.push(GuideLine {
                    col: guide_col,
                    start_line: start,
                    end_line: lines.len(),
                    active: false,
                });
            }
        }

        // Determine active guide
        // Find deepest guide containing cursor_line
        let mut best_idx = None;
        let mut max_col = -1isize;

        for (i, guide) in guides.iter().enumerate() {
            if cursor_line >= guide.start_line && cursor_line < guide.end_line {
                // Determine if this guide is "active".
                // Usually the active guide is the one corresponding to the current indentation scope.
                // If cursor is at indent 2, guides 0 and 1 are visible. 1 is "closest".
                if (guide.col as isize) > max_col {
                    max_col = guide.col as isize;
                    best_idx = Some(i);
                }
            }
        }

        if let Some(idx) = best_idx {
            guides[idx].active = true;
        }

        guides
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_guides() {
        let text = "
fn foo() {
    let x = 1;
    if x > 0 {
        print(x);
    }
}
";
        // Line 0: empty (None)
        // Line 1: fn foo... (indent 0)
        // Line 2:     let x... (indent 1)
        // Line 3:     if x... (indent 1)
        // Line 4:         print... (indent 2)
        // Line 5:     } (indent 1)
        // Line 6: } (indent 0)
        // Line 7: empty

        // Trimmed text lines:
        // 0: "" -> None
        // 1: "fn..." -> 0
        // 2: "    let..." -> 1
        // 3: "    if..." -> 1
        // 4: "        print..." -> 2
        // 5: "    }" -> 1
        // 6: "}" -> 0
        // 7: "" -> None

        // Guides:
        // Level 1 (col 0): Lines 2-5 (inclusive). Range 2..6.
        // Level 2 (col 4): Line 4. Range 4..5.

        let guides = IndentGuides::compute(text.trim(), 4, 4);
        // cursor at line 4 ("print(x)"). Should activate level 2 guide?
        // Or level 1?
        // At line 4, we are inside level 2 block.
        // So guide at level 2 (col 4) should be active.

        // Wait, lines are 0-indexed relative to provided text.
        // text.trim() removes initial newline.
        // 0: fn foo...
        // 1: let x...
        // 2: if x...
        // 3:     print...
        // 4: }
        // 5: }

        // Indents: 0, 1, 1, 2, 1, 0.

        // Level 1 guide: lines 1, 2, 3, 4. (indices).
        // Start line 1. End line 5 (exclusive).
        // Level 2 guide: line 3. Start 3, End 4.

        // Cursor at 3 (print).
        // Both guides contain 3.
        // Level 2 guide is deeper (col 4). It should be active.

        let guides = IndentGuides::compute(text.trim(), 4, 3);

        assert_eq!(guides.len(), 2);

        // Level 1 guide
        let g1 = guides.iter().find(|g| g.col == 0).unwrap();
        assert_eq!(g1.start_line, 1);
        assert_eq!(g1.end_line, 5);

        // Level 2 guide
        let g2 = guides.iter().find(|g| g.col == 4).unwrap();
        assert_eq!(g2.start_line, 3);
        assert_eq!(g2.end_line, 4);

        // Check active
        assert!(!g1.active);
        assert!(g2.active);
    }
}
