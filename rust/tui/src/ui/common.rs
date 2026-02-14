use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// The sidebar character (U+258C, left half block).
pub(super) const SIDEBAR_CHAR: &str = "\u{258c}";

/// Width of the exchange number gutter (2-digit number + space).
pub(super) const NUM_GUTTER: u16 = 3;

/// Background color for the input area.
pub(super) const INPUT_BG: Color = Color::Rgb(40, 40, 40);

/// Rotating half-circle spinner frames for in-progress tool calls.
pub(super) const TOOL_CALL_FRAMES: [char; 4] = ['\u{25d0}', '\u{25d3}', '\u{25d1}', '\u{25d2}'];

/// Pulsating frames for in-progress thinking: · ∗ ✱ (small → medium → large).
pub(super) const THINKING_FRAMES: [char; 3] = ['\u{00b7}', '\u{2217}', '\u{2731}'];

/// Dim style used for hint descriptions.
pub(super) const fn dim() -> Style {
    Style::new().fg(Color::DarkGray)
}

/// Style for unselected row primary text.
pub(super) const fn unselected_style() -> Style {
    Style::new().fg(Color::White)
}

/// Style for the selected row's primary text in autocomplete popups.
pub(super) const fn selected_style() -> Style {
    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
}

/// Style for the selected row's secondary text (e.g. descriptions, paths).
pub(super) const fn selected_secondary_style() -> Style {
    Style::new().fg(Color::White)
}

/// Split text into chunks that fit within `width` characters, for manual
/// line wrapping that preserves the gutter/sidebar prefix on each visual line.
pub(super) fn wrap_content(text: &str, width: usize) -> Vec<String> {
    if width == 0 || text.is_empty() {
        return vec![text.to_string()];
    }

    let chars: Vec<char> = text.chars().collect();
    let mut result = Vec::new();
    let mut start = 0;

    while start < chars.len() {
        let end = (start + width).min(chars.len());
        result.push(chars[start..end].iter().collect());
        start = end;
    }

    if result.is_empty() {
        result.push(String::new());
    }

    result
}

/// Count the number of visual lines the text occupies, accounting for wrapping.
pub(super) fn visual_line_count(text: &str, wrap_width: usize) -> usize {
    if text.is_empty() {
        return 1;
    }

    let mut lines = 1;
    let mut col = 0;

    for c in text.chars() {
        if c == '\n' {
            lines += 1;
            col = 0;
        } else {
            if col >= wrap_width {
                lines += 1;
                col = 0;
            }
            col += 1;
        }
    }

    lines
}

/// Calculate the visual (column, row) of the cursor, accounting for line wrapping.
///
/// `wrap_width` is the number of character columns available (inner widget width).
pub(super) fn cursor_position_wrapped(
    text: &str,
    byte_offset: usize,
    wrap_width: usize,
) -> (usize, usize) {
    let before_cursor = &text[..byte_offset];
    let mut visual_row = 0;
    let mut visual_col = 0;

    for c in before_cursor.chars() {
        if c == '\n' {
            // Explicit newline: move to start of next row
            visual_row += 1;
            visual_col = 0;
        } else {
            visual_col += 1;
            if visual_col >= wrap_width {
                // Reached end of visual line — next char goes to next row
                visual_row += 1;
                visual_col = 0;
            }
        }
    }

    (visual_col, visual_row)
}

/// Compute the popup area above the input area for the given number of items.
///
/// Returns `None` if there isn't enough space to render a meaningful popup.
pub(super) fn popup_area(input_area: Rect, item_count: usize) -> Option<Rect> {
    let popup_width = input_area.width;
    #[allow(clippy::cast_possible_truncation)]
    let popup_height = (item_count as u16 + 2).min(input_area.y); // +2 for borders

    if popup_height < 3 || popup_width < 10 {
        return None;
    }

    Some(Rect {
        x: input_area.x,
        y: input_area.y.saturating_sub(popup_height),
        width: popup_width,
        height: popup_height,
    })
}

/// Render an autocomplete popup with the given lines and optional title.
pub(super) fn render_popup(frame: &mut Frame, area: Rect, lines: Vec<Line>, title: Option<&str>) {
    frame.render_widget(Clear, area);

    let mut block = Block::default().borders(Borders::ALL).border_style(dim());
    if let Some(t) = title {
        block = block.title(t);
    }

    let popup = Paragraph::new(Text::from(lines)).block(block);
    frame.render_widget(popup, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_no_wrap() {
        // Width of 80: no wrapping needed
        assert_eq!(cursor_position_wrapped("", 0, 80), (0, 0));
        assert_eq!(cursor_position_wrapped("hello", 3, 80), (3, 0));
        assert_eq!(cursor_position_wrapped("hello", 5, 80), (5, 0));
    }

    #[test]
    fn cursor_explicit_newlines() {
        let text = "abc\ndef";
        assert_eq!(cursor_position_wrapped(text, 4, 80), (0, 1));
        assert_eq!(cursor_position_wrapped(text, 6, 80), (2, 1));
    }

    #[test]
    fn cursor_wraps_long_line() {
        // Width of 5: "abcdefgh" wraps after 5 chars
        let text = "abcdefgh";
        // After 5 chars: visual row 1, col 0
        assert_eq!(cursor_position_wrapped(text, 5, 5), (0, 1));
        // After 7 chars: visual row 1, col 2
        assert_eq!(cursor_position_wrapped(text, 7, 5), (2, 1));
        // After all 8 chars: visual row 1, col 3
        assert_eq!(cursor_position_wrapped(text, 8, 5), (3, 1));
    }

    #[test]
    fn cursor_wraps_multiple_times() {
        // Width of 3: "abcdefghi" -> "abc" / "def" / "ghi"
        let text = "abcdefghi";
        assert_eq!(cursor_position_wrapped(text, 3, 3), (0, 1));
        assert_eq!(cursor_position_wrapped(text, 6, 3), (0, 2));
        assert_eq!(cursor_position_wrapped(text, 8, 3), (2, 2));
    }

    #[test]
    fn cursor_wrap_with_newlines() {
        // Width of 4: "abcdef\ngh" -> "abcd" / "ef" / "gh"
        let text = "abcdef\ngh";
        assert_eq!(cursor_position_wrapped(text, 4, 4), (0, 1)); // soft wrap
        assert_eq!(cursor_position_wrapped(text, 7, 4), (0, 2)); // after \n
        assert_eq!(cursor_position_wrapped(text, 9, 4), (2, 2)); // "gh"
    }

    #[test]
    fn visual_lines_empty() {
        assert_eq!(visual_line_count("", 80), 1);
    }

    #[test]
    fn visual_lines_no_wrap() {
        assert_eq!(visual_line_count("hello", 80), 1);
        assert_eq!(visual_line_count("a\nb\nc", 80), 3);
    }

    #[test]
    fn visual_lines_with_wrap() {
        // Width 5: "abcdefgh" -> 2 visual lines
        assert_eq!(visual_line_count("abcdefgh", 5), 2);
        // Width 3: "abcdefghi" -> 3 visual lines
        assert_eq!(visual_line_count("abcdefghi", 3), 3);
    }

    #[test]
    fn visual_lines_wrap_and_newlines() {
        // Width 4: "abcdef\ngh" -> "abcd" / "ef" / "gh" = 3 lines
        assert_eq!(visual_line_count("abcdef\ngh", 4), 3);
    }
}
