use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
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
///
/// ANSI escape sequences (e.g. `\x1b[1m`) are treated as zero-width so they
/// do not inflate the measured line length.
fn line_wrap_breaks(line: &str, width: usize) -> Vec<usize> {
    if width == 0 {
        return vec![];
    }

    let chars: Vec<char> = line.chars().collect();

    // Build a parallel array: `true` if the char at that index is part of an
    // ANSI escape sequence and should not count towards visual width.
    let invisible = ansi_invisible_mask(&chars);

    let mut breaks = Vec::new();
    let mut idx = 0; // current char index
    let mut visual_col = 0; // visual columns consumed on the current line
    let mut last_ws_idx: Option<usize> = None; // last breakable whitespace char index

    while idx < chars.len() {
        if invisible[idx] {
            idx += 1;
            continue;
        }

        visual_col += 1;

        if chars[idx].is_whitespace() && chars[idx] != '\n' {
            last_ws_idx = Some(idx);
        }

        if visual_col > width && idx < chars.len() {
            // Need to break: prefer the last whitespace position
            let break_at = if let Some(ws) = last_ws_idx {
                ws + 1
            } else {
                idx // hard break at current position
            };

            breaks.push(break_at);

            // Restart measurement from break_at
            visual_col = 0;
            last_ws_idx = None;
            idx = break_at;
            continue;
        }

        idx += 1;
    }

    breaks
}

/// Return a mask indicating which char indices belong to ANSI escape sequences.
///
/// An ANSI escape sequence starts with `ESC [` and ends at the first byte in
/// the range `0x40..=0x7E` (the "final byte"). For SGR sequences the final
/// byte is `m`, but we handle all CSI sequences so that cursor-movement and
/// other codes are also treated as invisible.
fn ansi_invisible_mask(chars: &[char]) -> Vec<bool> {
    let mut mask = vec![false; chars.len()];
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\x1b' && i + 1 < chars.len() && chars[i + 1] == '[' {
            mask[i] = true; // ESC
            i += 1;
            mask[i] = true; // [
            i += 1;
            // Consume parameter bytes and the final byte
            while i < chars.len() {
                mask[i] = true;
                if chars[i] as u32 >= 0x40 && chars[i] as u32 <= 0x7E {
                    // final byte — sequence complete
                    i += 1;
                    break;
                }
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    mask
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

// ---------------------------------------------------------------------------
// Inline markdown styling
// ---------------------------------------------------------------------------

/// Controls the color intensity for inline markdown styling.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub(super) enum InlineStyleMode {
    /// Vibrant colors matching the full markdown renderer.
    Normal,
    /// Dimmer colors for contexts that are already subdued (e.g. thinking).
    Muted,
}

/// Controls whether markdown delimiters (`**`, `*`, `` ` ``, `$`, `~~`) are
/// kept in the output or stripped.
#[derive(Clone, Copy)]
pub(super) enum DelimiterDisplay {
    /// Retain delimiters as dimmed spans (required when output must match the
    /// source text, e.g. the input buffer).
    Show,
    /// Strip delimiters so only the styled body is emitted.
    Hide,
}

/// Apply lightweight inline markdown styling to `text`, returning styled spans
/// whose concatenated content is **identical** to `text` (delimiters are
/// preserved and dimmed, not consumed). Safe to use on text that feeds into
/// cursor-position / wrapping calculations.
///
/// Recognised patterns (outermost wins, code/math suppress inner parsing):
/// - `` `code` ``  — inline code
/// - `$math$`      — inline math
/// - `**bold**`    — strong
/// - `*italic*`    — emphasis
/// - `~~strike~~`  — strikethrough
#[allow(clippy::too_many_lines)]
pub(super) fn style_inline_markdown(
    text: &str,
    mode: InlineStyleMode,
    delimiters: DelimiterDisplay,
) -> Vec<Span<'static>> {
    let (delim_style, code_style, math_style, bold_mod, italic_mod, strike_mod) = match mode {
        InlineStyleMode::Normal => (
            Style::new().fg(Color::DarkGray),
            Style::new().fg(Color::Rgb(200, 160, 255)),
            Style::new().fg(Color::Cyan),
            Style::new().add_modifier(Modifier::BOLD),
            Style::new().add_modifier(Modifier::ITALIC),
            Style::new().add_modifier(Modifier::CROSSED_OUT),
        ),
        InlineStyleMode::Muted => (
            Style::new().fg(Color::DarkGray).add_modifier(Modifier::DIM),
            Style::new().fg(Color::Rgb(140, 112, 175)),
            Style::new()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
            Style::new()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            Style::new()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
            Style::new()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::CROSSED_OUT),
        ),
    };

    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut plain_start = 0;
    let mut i = 0;

    while i < len {
        // Skip escaped characters — the backslash and the following char stay
        // in the current plain-text run and are never treated as delimiters.
        if chars[i] == '\\' && i + 1 < len {
            i += 2;
            continue;
        }

        // Try each pattern; on match emit_match advances i and continues.
        if chars[i] == '`'
            && let Some(close) = find_closing(&chars, i + 1, &['`'])
            && close > i + 1
        {
            let m = InlineMatch {
                open: i,
                open_len: 1,
                close,
                close_len: 1,
                body_style: code_style,
            };
            emit_match(
                &chars,
                &mut spans,
                &mut plain_start,
                delim_style,
                delimiters,
                &m,
            );
            i = close + 1;
            continue;
        }
        if chars[i] == '$'
            && (i + 1 < len && chars[i + 1] != '$')
            && let Some(close) = find_closing(&chars, i + 1, &['$'])
            && close > i + 1
        {
            let m = InlineMatch {
                open: i,
                open_len: 1,
                close,
                close_len: 1,
                body_style: math_style,
            };
            emit_match(
                &chars,
                &mut spans,
                &mut plain_start,
                delim_style,
                delimiters,
                &m,
            );
            i = close + 1;
            continue;
        }
        if chars[i] == '~'
            && i + 1 < len
            && chars[i + 1] == '~'
            && let Some(close) = find_closing_double(&chars, i + 2, '~')
            && close > i + 2
        {
            let m = InlineMatch {
                open: i,
                open_len: 2,
                close,
                close_len: 2,
                body_style: strike_mod,
            };
            emit_match(
                &chars,
                &mut spans,
                &mut plain_start,
                delim_style,
                delimiters,
                &m,
            );
            i = close + 2;
            continue;
        }
        if chars[i] == '*'
            && i + 1 < len
            && chars[i + 1] == '*'
            && let Some(close) = find_closing_double(&chars, i + 2, '*')
            && close > i + 2
        {
            let m = InlineMatch {
                open: i,
                open_len: 2,
                close,
                close_len: 2,
                body_style: bold_mod,
            };
            emit_match(
                &chars,
                &mut spans,
                &mut plain_start,
                delim_style,
                delimiters,
                &m,
            );
            i = close + 2;
            continue;
        }
        if chars[i] == '*'
            && !(i + 1 < len && chars[i + 1] == '*')
            && let Some(close) = find_closing_single(&chars, i + 1, '*')
            && close > i + 1
        {
            let m = InlineMatch {
                open: i,
                open_len: 1,
                close,
                close_len: 1,
                body_style: italic_mod,
            };
            emit_match(
                &chars,
                &mut spans,
                &mut plain_start,
                delim_style,
                delimiters,
                &m,
            );
            i = close + 1;
            continue;
        }
        i += 1;
    }

    if plain_start < len {
        let s: String = chars[plain_start..].iter().collect();
        spans.push(Span::raw(s));
    }
    if spans.is_empty() {
        spans.push(Span::raw(text.to_string()));
    }
    spans
}

/// Parameters for emitting a matched inline pattern.
struct InlineMatch {
    open: usize,
    open_len: usize,
    close: usize,
    close_len: usize,
    body_style: Style,
}

/// Emit three styled spans for a matched inline pattern: open delimiter, body,
/// close delimiter. Flushes any preceding plain text first.
fn emit_match(
    chars: &[char],
    spans: &mut Vec<Span<'static>>,
    plain_start: &mut usize,
    delim_style: Style,
    delimiters: DelimiterDisplay,
    m: &InlineMatch,
) {
    if *plain_start < m.open {
        let s: String = chars[*plain_start..m.open].iter().collect();
        spans.push(Span::raw(s));
    }
    let body: String = chars[m.open + m.open_len..m.close].iter().collect();
    match delimiters {
        DelimiterDisplay::Show => {
            let od: String = chars[m.open..m.open + m.open_len].iter().collect();
            let cd: String = chars[m.close..m.close + m.close_len].iter().collect();
            spans.push(Span::styled(od, delim_style));
            spans.push(Span::styled(body, m.body_style));
            spans.push(Span::styled(cd, delim_style));
        }
        DelimiterDisplay::Hide => {
            spans.push(Span::styled(body, m.body_style));
        }
    }
    *plain_start = m.close + m.close_len;
}

/// Find the char-index of a single closing delimiter, skipping escaped chars.
fn find_closing(chars: &[char], start: usize, closing: &[char]) -> Option<usize> {
    let mut j = start;
    while j < chars.len() {
        if chars[j] == '\n' {
            return None;
        }
        if chars[j] == '\\' {
            j += 2;
            continue;
        }
        if closing.contains(&chars[j]) {
            return Some(j);
        }
        j += 1;
    }
    None
}

/// Find a closing double-char delimiter (e.g. `**`, `~~`).
fn find_closing_double(chars: &[char], start: usize, ch: char) -> Option<usize> {
    let mut j = start;
    while j + 1 < chars.len() {
        if chars[j] == '\n' {
            return None;
        }
        if chars[j] == '\\' {
            j += 2;
            continue;
        }
        if chars[j] == ch && chars[j + 1] == ch {
            return Some(j);
        }
        j += 1;
    }
    None
}

/// Find a closing single-char delimiter, making sure we don't match a double.
fn find_closing_single(chars: &[char], start: usize, ch: char) -> Option<usize> {
    let mut j = start;
    while j < chars.len() {
        if chars[j] == '\n' {
            return None;
        }
        if chars[j] == '\\' {
            j += 2;
            continue;
        }
        if chars[j] == ch {
            // Don't match if next char is the same (that would be a double delimiter)
            if j + 1 < chars.len() && chars[j + 1] == ch {
                j += 2;
                continue;
            }
            return Some(j);
        }
        j += 1;
    }
    None
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

/// Parse a string containing ANSI SGR escape sequences into styled ratatui spans.
///
/// Handles the escape codes emitted by the CLI's `comfy-table` / `yansi` output:
/// - `\x1b[1m` bold, `\x1b[0m` reset
/// - `\x1b[38;5;Nm` 256-color foreground
/// - `\x1b[39m` default foreground
/// - Basic foreground colors `\x1b[30m`–`\x1b[37m` and bright `\x1b[90m`–`\x1b[97m`
///
/// Accepts an initial `Style` to carry over from a previous chunk (e.g. when
/// text is word-wrapped into multiple lines) and returns the final active style
/// alongside the spans.
///
/// Unrecognized sequences are silently consumed (their text content is not lost).
pub(super) fn parse_ansi_spans(input: &str, initial_style: Style) -> (Vec<Span<'static>>, Style) {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut style = initial_style;
    let mut buf = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            // Flush accumulated text
            if !buf.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut buf), style));
            }
            chars.next(); // consume '['

            // Collect the parameter bytes (digits and semicolons)
            let mut params = String::new();
            while let Some(&pc) = chars.peek() {
                if pc.is_ascii_digit() || pc == ';' {
                    params.push(pc);
                    chars.next();
                } else {
                    break;
                }
            }
            // Consume the final byte (should be 'm' for SGR)
            let final_byte = chars.next();
            if final_byte == Some('m') {
                apply_sgr(&mut style, &params);
            }
            // Non-SGR sequences are silently dropped
        } else {
            buf.push(c);
        }
    }
    if !buf.is_empty() {
        spans.push(Span::styled(buf, style));
    }
    (spans, style)
}

/// Apply SGR (Select Graphic Rendition) parameters to a style.
fn apply_sgr(style: &mut Style, params: &str) {
    if params.is_empty() || params == "0" {
        *style = Style::default();
        return;
    }

    let codes: Vec<u8> = params.split(';').filter_map(|s| s.parse().ok()).collect();
    let mut i = 0;
    while i < codes.len() {
        match codes[i] {
            0 => *style = Style::default(),
            1 => *style = style.add_modifier(Modifier::BOLD),
            2 => *style = style.add_modifier(Modifier::DIM),
            3 => *style = style.add_modifier(Modifier::ITALIC),
            4 => *style = style.add_modifier(Modifier::UNDERLINED),
            22 => *style = style.remove_modifier(Modifier::BOLD | Modifier::DIM),
            23 => *style = style.remove_modifier(Modifier::ITALIC),
            24 => *style = style.remove_modifier(Modifier::UNDERLINED),
            // Basic foreground colors
            30 => style.fg = Some(Color::Black),
            31 => style.fg = Some(Color::Red),
            32 => style.fg = Some(Color::Green),
            33 => style.fg = Some(Color::Yellow),
            34 => style.fg = Some(Color::Blue),
            35 => style.fg = Some(Color::Magenta),
            36 => style.fg = Some(Color::Cyan),
            37 => style.fg = Some(Color::White),
            39 => style.fg = None,
            // Bright foreground colors
            90 => style.fg = Some(Color::DarkGray),
            91 => style.fg = Some(Color::LightRed),
            92 => style.fg = Some(Color::LightGreen),
            93 => style.fg = Some(Color::LightYellow),
            94 => style.fg = Some(Color::LightBlue),
            95 => style.fg = Some(Color::LightMagenta),
            96 => style.fg = Some(Color::LightCyan),
            97 => style.fg = Some(Color::Gray),
            // 256-color foreground: 38;5;N
            38 if i + 2 < codes.len() && codes[i + 1] == 5 => {
                style.fg = Some(Color::Indexed(codes[i + 2]));
                i += 2;
            }
            // 256-color background: 48;5;N
            48 if i + 2 < codes.len() && codes[i + 1] == 5 => {
                style.bg = Some(Color::Indexed(codes[i + 2]));
                i += 2;
            }
            _ => {}
        }
        i += 1;
    }
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
        assert_eq!(wrap_content("aa bb cc dd", 6), vec!["aa bb ", "cc dd"]);
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

    // -- inline markdown styling tests --

    fn spans_text(spans: &[Span]) -> String {
        spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn inline_md_preserves_full_text() {
        // The concatenated span content must equal the original text exactly
        for input in [
            "plain text",
            "**bold**",
            "*italic*",
            "`code`",
            "$math$",
            "~~strike~~",
            "**bold** and *italic* with `code`",
            "unmatched * star",
            "empty `` backticks",
            "",
        ] {
            let spans =
                style_inline_markdown(input, InlineStyleMode::Normal, DelimiterDisplay::Show);
            assert_eq!(
                spans_text(&spans),
                input,
                "text preservation failed for: {input:?}"
            );
        }
    }

    #[test]
    fn inline_md_bold() {
        let spans = style_inline_markdown(
            "a **bold** b",
            InlineStyleMode::Normal,
            DelimiterDisplay::Show,
        );
        assert_eq!(spans.len(), 5); // "a " + "**" + "bold" + "**" + " b"
        assert!(spans[2].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(spans[2].content.as_ref(), "bold");
    }

    #[test]
    fn inline_md_italic() {
        let spans = style_inline_markdown(
            "a *italic* b",
            InlineStyleMode::Normal,
            DelimiterDisplay::Show,
        );
        assert_eq!(spans.len(), 5);
        assert!(spans[2].style.add_modifier.contains(Modifier::ITALIC));
        assert_eq!(spans[2].content.as_ref(), "italic");
    }

    #[test]
    fn inline_md_code() {
        let spans = style_inline_markdown(
            "a `code` b",
            InlineStyleMode::Normal,
            DelimiterDisplay::Show,
        );
        assert_eq!(spans.len(), 5);
        assert_eq!(spans[2].content.as_ref(), "code");
        assert_eq!(spans[2].style.fg, Some(Color::Rgb(200, 160, 255)));
    }

    #[test]
    fn inline_md_math() {
        let spans =
            style_inline_markdown("a $x+1$ b", InlineStyleMode::Normal, DelimiterDisplay::Show);
        assert_eq!(spans.len(), 5);
        assert_eq!(spans[2].content.as_ref(), "x+1");
        assert_eq!(spans[2].style.fg, Some(Color::Cyan));
    }

    #[test]
    fn inline_md_strikethrough() {
        let spans = style_inline_markdown(
            "a ~~old~~ b",
            InlineStyleMode::Normal,
            DelimiterDisplay::Show,
        );
        assert_eq!(spans.len(), 5);
        assert!(spans[2].style.add_modifier.contains(Modifier::CROSSED_OUT));
        assert_eq!(spans[2].content.as_ref(), "old");
    }

    #[test]
    fn inline_md_unmatched_delimiters() {
        // Unmatched delimiters should appear as plain text
        let spans = style_inline_markdown("a * b", InlineStyleMode::Normal, DelimiterDisplay::Show);
        assert_eq!(spans_text(&spans), "a * b");
        assert_eq!(spans.len(), 1); // all plain
    }

    #[test]
    fn inline_md_empty_delimiters() {
        // Empty content between delimiters should not match
        let spans =
            style_inline_markdown("a ** ** b", InlineStyleMode::Normal, DelimiterDisplay::Show);
        assert_eq!(spans_text(&spans), "a ** ** b");
    }

    #[test]
    fn inline_md_newline_blocks_match() {
        // Delimiters don't span across newlines
        let spans = style_inline_markdown(
            "**bold\ntext**",
            InlineStyleMode::Normal,
            DelimiterDisplay::Show,
        );
        assert_eq!(spans_text(&spans), "**bold\ntext**");
        assert_eq!(spans.len(), 1); // all plain, no match
    }

    #[test]
    fn inline_md_multiple_patterns() {
        let spans = style_inline_markdown(
            "**b** *i* `c`",
            InlineStyleMode::Normal,
            DelimiterDisplay::Show,
        );
        assert_eq!(spans_text(&spans), "**b** *i* `c`");
        // Should have styled segments for each
        assert!(spans.iter().any(|s| s.content.as_ref() == "b" && s.style.add_modifier.contains(Modifier::BOLD)));
        assert!(
            spans
                .iter()
                .any(|s| s.content.as_ref() == "i"
                    && s.style.add_modifier.contains(Modifier::ITALIC))
        );
        assert!(spans.iter().any(|s| s.content.as_ref() == "c" && s.style.fg == Some(Color::Rgb(200, 160, 255))));
    }

    #[test]
    fn inline_md_muted_mode() {
        let spans = style_inline_markdown("`code`", InlineStyleMode::Muted, DelimiterDisplay::Show);
        assert_eq!(spans_text(&spans), "`code`");
        // Muted code uses a dimmer purple
        assert_eq!(spans[1].style.fg, Some(Color::Rgb(140, 112, 175)));
    }

    #[test]
    fn inline_md_code_suppresses_inner() {
        // Bold delimiters inside code should not be parsed
        let spans = style_inline_markdown(
            "`**not bold**`",
            InlineStyleMode::Normal,
            DelimiterDisplay::Show,
        );
        assert_eq!(spans_text(&spans), "`**not bold**`");
        assert_eq!(spans[1].content.as_ref(), "**not bold**");
        // Should be styled as code, not bold
        assert_eq!(spans[1].style.fg, Some(Color::Rgb(200, 160, 255)));
    }

    #[test]
    fn inline_md_bold_not_confused_with_italic() {
        // "**bold**" should not be parsed as two empty italic spans
        let spans =
            style_inline_markdown("**bold**", InlineStyleMode::Normal, DelimiterDisplay::Show);
        assert_eq!(spans_text(&spans), "**bold**");
        assert!(
            spans
                .iter()
                .any(|s| s.content.as_ref() == "bold"
                    && s.style.add_modifier.contains(Modifier::BOLD))
        );
        assert!(
            !spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::ITALIC))
        );
    }

    #[test]
    fn inline_md_escaped_opener_not_styled() {
        // A backslash before the opening delimiter should suppress matching
        let spans = style_inline_markdown(
            r"\*literal*",
            InlineStyleMode::Normal,
            DelimiterDisplay::Show,
        );
        assert_eq!(spans_text(&spans), r"\*literal*");
        assert!(
            !spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::ITALIC))
        );
    }

    #[test]
    fn inline_md_escaped_double_opener() {
        let spans = style_inline_markdown(
            r"\**not bold**",
            InlineStyleMode::Normal,
            DelimiterDisplay::Show,
        );
        assert_eq!(spans_text(&spans), r"\**not bold**");
        assert!(
            !spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::BOLD))
        );
    }

    #[test]
    fn inline_md_escaped_backtick() {
        let spans = style_inline_markdown(
            r"\`not code`",
            InlineStyleMode::Normal,
            DelimiterDisplay::Show,
        );
        assert_eq!(spans_text(&spans), r"\`not code`");
        assert!(
            !spans
                .iter()
                .any(|s| s.style.fg == Some(Color::Rgb(200, 160, 255)))
        );
    }

    #[test]
    fn inline_md_hide_delimiters() {
        let spans = style_inline_markdown(
            "a **bold** b",
            InlineStyleMode::Muted,
            DelimiterDisplay::Hide,
        );
        assert_eq!(spans_text(&spans), "a bold b");
        assert!(
            spans
                .iter()
                .any(|s| s.content.as_ref() == "bold"
                    && s.style.add_modifier.contains(Modifier::BOLD))
        );
    }

    #[test]
    fn inline_md_hide_delimiters_multiple() {
        let spans = style_inline_markdown(
            "**b** and `c`",
            InlineStyleMode::Muted,
            DelimiterDisplay::Hide,
        );
        assert_eq!(spans_text(&spans), "b and c");
    }

    // --- ANSI parsing tests ---

    #[test]
    fn ansi_plain_text() {
        let (spans, _) = parse_ansi_spans("hello world", Style::default());
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content.as_ref(), "hello world");
        assert_eq!(spans[0].style, Style::default());
    }

    #[test]
    fn ansi_bold() {
        let (spans, _) = parse_ansi_spans("before \x1b[1mbold\x1b[0m after", Style::default());
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].content.as_ref(), "before ");
        assert_eq!(spans[1].content.as_ref(), "bold");
        assert!(spans[1].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(spans[2].content.as_ref(), " after");
        assert_eq!(spans[2].style, Style::default());
    }

    #[test]
    fn ansi_256_color() {
        // \x1b[38;5;10m = 256-color green foreground
        let (spans, _) = parse_ansi_spans("\x1b[38;5;10mgreen\x1b[39m normal", Style::default());
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].content.as_ref(), "green");
        assert_eq!(spans[0].style.fg, Some(Color::Indexed(10)));
        assert_eq!(spans[1].content.as_ref(), " normal");
        assert_eq!(spans[1].style.fg, None);
    }

    #[test]
    fn ansi_basic_colors() {
        let (spans, _) = parse_ansi_spans("\x1b[31mred\x1b[32mgreen\x1b[0mplain", Style::default());
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].style.fg, Some(Color::Red));
        assert_eq!(spans[1].style.fg, Some(Color::Green));
        assert_eq!(spans[2].style, Style::default());
    }

    #[test]
    fn ansi_empty_string() {
        let (spans, _) = parse_ansi_spans("", Style::default());
        assert!(spans.is_empty());
    }

    #[test]
    fn ansi_only_escapes() {
        let (spans, _) = parse_ansi_spans("\x1b[1m\x1b[0m", Style::default());
        assert!(spans.is_empty());
    }

    #[test]
    fn ansi_style_carries_across_chunks() {
        // Simulate a color opened in chunk 1 that should carry into chunk 2
        let (spans1, carry) = parse_ansi_spans("\x1b[31mhello", Style::default());
        assert_eq!(spans1.len(), 1);
        assert_eq!(spans1[0].style.fg, Some(Color::Red));
        assert_eq!(carry.fg, Some(Color::Red));

        let (spans2, _) = parse_ansi_spans("world\x1b[0m", carry);
        assert_eq!(spans2.len(), 1);
        assert_eq!(spans2[0].content.as_ref(), "world");
        assert_eq!(spans2[0].style.fg, Some(Color::Red));
    }

    // --- ANSI-aware wrapping tests ---

    #[test]
    fn wrap_ignores_ansi_escape_width() {
        // "hello" is 5 visible chars; the ANSI escapes should not count.
        // With width 10 this should NOT wrap.
        let line = "\x1b[1mhello\x1b[0m";
        assert_eq!(wrap_content(line, 10), vec![line]);
    }

    #[test]
    fn wrap_ansi_line_fits_exactly() {
        // 5 visible chars in width 5 — no wrap needed
        let line = "\x1b[31mhello\x1b[0m";
        assert_eq!(wrap_content(line, 5), vec![line]);
    }

    #[test]
    fn wrap_ansi_table_row_no_spurious_break() {
        // Simulate a table row with ANSI bold column headers.
        // Visual content is 20 chars; ANSI adds invisible bytes.
        let line = "│ \x1b[1mName\x1b[0m  \x1b[1mType\x1b[0m  \x1b[1mLang\x1b[0m │";
        // Width 30 should be enough — no wrapping
        let chunks = wrap_content(line, 30);
        assert_eq!(chunks.len(), 1, "should not wrap: {chunks:?}");
    }

    #[test]
    fn visual_lines_ansi_no_extra() {
        let line = "\x1b[1mhello world\x1b[0m";
        // 11 visible chars, width 20 -> 1 line
        assert_eq!(visual_line_count(line, 20), 1);
    }
}
