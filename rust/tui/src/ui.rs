use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::{App, ChatMessage};

/// Dim style used for hint descriptions.
const fn dim() -> Style {
    Style::new().fg(Color::DarkGray)
}

/// Render the entire UI for one frame.
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Calculate input area height based on visual lines (accounting for wrapping).
    // Use a conservative estimate of inner width (total - 2 for borders).
    let inner_width = area.width.saturating_sub(2).max(1) as usize;
    #[allow(clippy::cast_possible_truncation)]
    let visual_lines = visual_line_count(app.input.text(), inner_width) as u16;
    let max_input_height = (area.height / 3).max(3);
    // +2 for the border
    let input_height = (visual_lines + 2).clamp(3, max_input_height);

    // Layout: messages | input | hints
    let layout = Layout::vertical([
        Constraint::Min(1),               // message area
        Constraint::Length(input_height), // input area
        Constraint::Length(1),            // hint line below input
    ])
    .split(area);

    let messages_area = layout[0];
    let input_area = layout[1];
    let hints_area = layout[2];

    // --- Render messages ---
    render_messages(frame, app, messages_area);

    // --- Render input ---
    render_input(frame, app, input_area);

    // --- Render hints below input ---
    render_hints(frame, hints_area);

    // --- Render autocomplete popup (floats above input) ---
    if app.commands_state.is_visible() {
        render_autocomplete(frame, app, input_area);
    }
}

/// Render the scrollable message area.
fn render_messages(frame: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    for message in &app.messages {
        // Add a blank line separator between messages (except before the first)
        if !lines.is_empty() {
            lines.push(Line::raw(""));
        }

        match message {
            ChatMessage::User { content } => {
                lines.push(Line::from(vec![Span::styled(
                    "You: ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )]));
                for text_line in content.lines() {
                    lines.push(Line::raw(format!("  {text_line}")));
                }
            }
            ChatMessage::System { content } => {
                for text_line in content.lines() {
                    lines.push(Line::styled(text_line.to_string(), dim()));
                }
            }
        }
    }

    // Count visual lines (accounting for soft wrapping within the area width)
    let wrap_width = area.width.max(1) as usize;
    #[allow(clippy::cast_possible_truncation)]
    let total_lines = lines
        .iter()
        .map(|line| {
            let char_len: usize = line.spans.iter().map(|s| s.content.chars().count()).sum();
            if char_len == 0 {
                1
            } else {
                char_len.div_ceil(wrap_width)
            }
        })
        .sum::<usize>() as u16;
    let visible_height = area.height;

    // Update app state for scroll bounds
    app.total_message_lines = total_lines;
    app.visible_message_height = visible_height;

    // Calculate scroll: lines from bottom
    let scroll = if total_lines > visible_height {
        total_lines
            .saturating_sub(visible_height)
            .saturating_sub(app.scroll_offset)
    } else {
        0
    };

    let paragraph = Paragraph::new(Text::from(lines))
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Render the hint line below the input area.
fn render_hints(frame: &mut Frame, area: Rect) {
    let hints = Line::from(vec![
        Span::raw("alt+\u{21b5} "),
        Span::styled("newline", dim()),
        Span::raw("  ctrl+c "),
        Span::styled("quit", dim()),
    ]);

    let paragraph = Paragraph::new(hints).alignment(Alignment::Right);
    frame.render_widget(paragraph, area);
}

/// Render the input area with cursor.
fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let input_text = app.input.text();

    let paragraph = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(" > "),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);

    // Show "↵ send" hint on the right — hide when input text gets close (~8 chars)
    let inner_width = area.width.saturating_sub(2);
    let hint_display_width: u16 = 6; // "↵ send" = 6 display columns
    let input_char_len = app.input.text().chars().count();
    if inner_width > hint_display_width + 8
        && input_char_len < (inner_width - hint_display_width - 2) as usize
    {
        let hint_area = Rect {
            x: area.x + area.width - hint_display_width - 2,
            y: area.y + 1,
            width: hint_display_width,
            height: 1,
        };
        let hint = Line::from(vec![Span::raw("\u{21b5}"), Span::styled(" send", dim())]);
        frame.render_widget(Paragraph::new(hint), hint_area);
    }

    // Inner width available for text (excluding left and right borders)
    let inner_width = area.width.saturating_sub(2).max(1) as usize;

    // Position the cursor within the input area, accounting for wrapping
    let (cursor_col, cursor_row) =
        cursor_position_wrapped(app.input.text(), app.input.cursor(), inner_width);

    // +1 for the border
    #[allow(clippy::cast_possible_truncation)]
    let x = area.x + 1 + cursor_col as u16;
    #[allow(clippy::cast_possible_truncation)]
    let y = area.y + 1 + cursor_row as u16;

    // Only show cursor if it fits within the input area
    if x < area.x + area.width - 1 && y < area.y + area.height - 1 {
        frame.set_cursor_position(Position::new(x, y));
    }
}

/// Render the autocomplete popup floating above the input area.
fn render_autocomplete(frame: &mut Frame, app: &App, input_area: Rect) {
    let candidates = app.commands_state.candidates();
    if candidates.is_empty() {
        return;
    }

    // Use full input width for the popup so descriptions aren't truncated
    let popup_width = input_area.width;
    #[allow(clippy::cast_possible_truncation)]
    let popup_height = (candidates.len() as u16 + 2).min(input_area.y); // +2 for borders

    if popup_height < 3 || popup_width < 10 {
        return; // Not enough space
    }

    let max_name_width = candidates.iter().map(|c| c.name().len()).max().unwrap_or(0);

    // Position: above the input area, aligned to left
    let popup_area = Rect {
        x: input_area.x,
        y: input_area.y.saturating_sub(popup_height),
        width: popup_width,
        height: popup_height,
    };

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Build popup lines with dim descriptions
    let selected = app.commands_state.selected();
    let lines: Vec<Line> = candidates
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let name = cmd.name();
            let desc = cmd.description();
            let padded_name = format!(" {name:<max_name_width$}  ");

            if i == selected {
                Line::from(vec![
                    Span::styled(
                        padded_name,
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        desc.to_string(),
                        Style::default().fg(Color::DarkGray).bg(Color::Cyan),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(padded_name, Style::default().fg(Color::White)),
                    Span::styled(desc.to_string(), dim()),
                ])
            }
        })
        .collect();

    let popup = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).border_style(dim()));

    frame.render_widget(popup, popup_area);
}

/// Count the number of visual lines the text occupies, accounting for wrapping.
fn visual_line_count(text: &str, wrap_width: usize) -> usize {
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
fn cursor_position_wrapped(text: &str, byte_offset: usize, wrap_width: usize) -> (usize, usize) {
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
