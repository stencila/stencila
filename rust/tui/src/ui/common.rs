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

/// Pulsating frames for in-progress thinking: · + ∗ ✱ ∗ + (grow then shrink).
pub(super) const THINKING_FRAMES: [char; 6] =
    ['\u{00b7}', '+', '\u{2217}', '\u{2731}', '\u{2217}', '+'];

/// Braille spinner frames for the input prompt when the active agent is busy.
pub(super) const BRAILLE_SPINNER_FRAMES: [char; 10] =
    ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

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

/// Compute char-offset break points for word-wrapping a single logical line
/// (no embedded newlines). Each returned offset is where a new visual line
/// begins. Falls back to hard breaks for words longer than `width`.
fn line_wrap_breaks(line: &str, width: usize) -> Vec<usize> {
    if width == 0 {
        return vec![];
    }

    let chars: Vec<char> = line.chars().collect();
    let mut breaks = Vec::new();
    let mut line_start = 0;

    while line_start < chars.len() {
        if line_start + width >= chars.len() {
            break;
        }

        let line_end = line_start + width;
        let break_at = chars[line_start..line_end]
            .iter()
            .rposition(|&c| c.is_whitespace() && c != '\n')
            .map(|p| line_start + p + 1)
            .filter(|&p| p > line_start)
            .unwrap_or(line_end);

        breaks.push(break_at);
        line_start = break_at;
    }

    breaks
}

/// Split text into chunks that fit within `width` characters using word
/// wrapping. Falls back to character-level breaking for words longer than
/// `width`.
pub(super) fn wrap_content(text: &str, width: usize) -> Vec<String> {
    if width == 0 || text.is_empty() {
        return vec![text.to_string()];
    }

    let chars: Vec<char> = text.chars().collect();
    let breaks = line_wrap_breaks(text, width);

    let mut result = Vec::new();
    let mut start = 0;
    for brk in breaks {
        result.push(chars[start..brk].iter().collect());
        start = brk;
    }
    result.push(chars[start..].iter().collect());

    result
}

/// Count the number of visual lines the text occupies, accounting for word
/// wrapping.
pub(super) fn visual_line_count(text: &str, wrap_width: usize) -> usize {
    if text.is_empty() {
        return 1;
    }

    text.split('\n')
        .map(|line| 1 + line_wrap_breaks(line, wrap_width).len())
        .sum()
}

/// Calculate the visual (column, row) of the cursor, accounting for word
/// wrapping.
///
/// `wrap_width` is the number of character columns available (inner widget
/// width).
pub(super) fn cursor_position_wrapped(
    text: &str,
    byte_offset: usize,
    wrap_width: usize,
) -> (usize, usize) {
    let mut visual_row = 0;
    let mut line_byte_start = 0;

    for line in text.split('\n') {
        let line_byte_end = line_byte_start + line.len();

        if byte_offset <= line_byte_end {
            let cursor_char = text[line_byte_start..byte_offset].chars().count();
            let breaks = line_wrap_breaks(line, wrap_width);

            let mut segment_start = 0;
            for &brk in &breaks {
                if cursor_char < brk {
                    break;
                }
                visual_row += 1;
                segment_start = brk;
            }

            return (cursor_char - segment_start, visual_row);
        }

        visual_row += 1 + line_wrap_breaks(line, wrap_width).len();
        line_byte_start = line_byte_end + 1;
    }

    (0, visual_row)
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

    #[test]
    fn word_wrap_breaks_at_space() {
        // "hello world" width 8 -> "hello " / "world"
        assert_eq!(wrap_content("hello world", 8), vec!["hello ", "world"]);
    }

    #[test]
    fn word_wrap_long_word_hard_breaks() {
        // No spaces: falls back to character-level breaking
        assert_eq!(wrap_content("abcdefgh", 5), vec!["abcde", "fgh"]);
    }

    #[test]
    fn word_wrap_multiple_words() {
        // "aa bb cc dd" width 6 -> "aa bb " / "cc dd"
        assert_eq!(
            wrap_content("aa bb cc dd", 6),
            vec!["aa bb ", "cc dd"]
        );
    }

    #[test]
    fn word_wrap_fits_exactly() {
        assert_eq!(wrap_content("hello", 5), vec!["hello"]);
        assert_eq!(wrap_content("hello", 10), vec!["hello"]);
    }

    #[test]
    fn cursor_word_wrap() {
        // "hello world" width 8 -> "hello " / "world"
        let text = "hello world";
        // cursor at 'w' (byte 6) -> col 0, row 1
        assert_eq!(cursor_position_wrapped(text, 6, 8), (0, 1));
        // cursor at 'r' (byte 8) -> col 2, row 1
        assert_eq!(cursor_position_wrapped(text, 8, 8), (2, 1));
        // cursor at end of "hello " (byte 5) -> col 5, row 0
        assert_eq!(cursor_position_wrapped(text, 5, 8), (5, 0));
    }

    #[test]
    fn visual_lines_word_wrap() {
        // "hello world" width 8 -> 2 lines
        assert_eq!(visual_line_count("hello world", 8), 2);
        // "aa bb cc" width 4 -> "aa " / "bb " / "cc" = 3 lines
        assert_eq!(visual_line_count("aa bb cc", 4), 3);
    }

    #[test]
    fn cursor_word_wrap_with_newlines() {
        // "hi there\nfoo bar" width 6 -> "hi " / "there" / "foo " / "bar"
        let text = "hi there\nfoo bar";
        // cursor at 't' (byte 3) -> col 0, row 1 (word wrapped)
        assert_eq!(cursor_position_wrapped(text, 3, 6), (0, 1));
        // cursor at 'f' (byte 9) -> col 0, row 2 (after newline)
        assert_eq!(cursor_position_wrapped(text, 9, 6), (0, 2));
        // cursor at 'b' (byte 13) -> col 0, row 3 (word wrapped)
        assert_eq!(cursor_position_wrapped(text, 13, 6), (0, 3));
    }
}
