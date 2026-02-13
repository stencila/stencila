use std::borrow::Cow;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::{App, AppMessage, AppMode, ExchangeKind, ExchangeStatus};

/// Dim style used for hint descriptions.
const fn dim() -> Style {
    Style::new().fg(Color::DarkGray)
}

/// Background color for the input area.
const INPUT_BG: Color = Color::Rgb(40, 40, 40);

/// Render the entire UI for one frame.
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Calculate input area height based on visual lines (accounting for wrapping).
    // Include ghost text suffix when computing height so it doesn't clip.
    // Inner width: sidebar (1) + space (1) + content area, no borders
    let inner_width = area.width.saturating_sub(2).max(1) as usize;
    let text_for_height: Cow<str> = match &app.ghost_suggestion {
        Some(ghost) => Cow::Owned(format!("{}{ghost}", app.input.text())),
        None => Cow::Borrowed(app.input.text()),
    };
    #[allow(clippy::cast_possible_truncation)]
    let visual_lines = visual_line_count(&text_for_height, inner_width) as u16;
    let max_input_height = (area.height / 3).max(3);
    let input_height = visual_lines.clamp(1, max_input_height);

    // Layout: messages | spacer | input | hints
    let layout = Layout::vertical([
        Constraint::Min(1),               // message area
        Constraint::Length(1),            // blank line above input
        Constraint::Length(input_height), // input area
        Constraint::Length(1),            // hint line below input
    ])
    .split(area);

    let messages_area = layout[0];
    let input_area = layout[2];
    let hints_area = layout[3];

    // --- Render messages ---
    render_messages(frame, app, messages_area);

    // --- Render input ---
    render_input(frame, app, input_area);

    // --- Render hints below input ---
    render_hints(frame, app, hints_area);

    // --- Render autocomplete popup (floats above input) ---
    // History popup has highest priority, then commands, then files.
    if app.history_state.is_visible() {
        render_history_autocomplete(frame, app, input_area);
    } else if app.commands_state.is_visible() {
        render_autocomplete(frame, app, input_area);
    } else if app.files_state.is_visible() {
        render_files_autocomplete(frame, app, input_area);
    }
}

/// The sidebar character (U+258C, left half block).
const SIDEBAR_CHAR: &str = "\u{258c}";

/// Append lines for a welcome message.
fn render_welcome_lines(lines: &mut Vec<Line>) {
    let version = stencila_version::STENCILA_VERSION;
    let green = Color::Rgb(102, 255, 102);
    let teal = Color::Rgb(15, 104, 96);
    let blue = Color::Rgb(37, 104, 239);
    let cwd = std::env::current_dir()
        .ok()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled("███████", Style::new().fg(green)),
        Span::raw("  "),
        Span::styled("Stencila ", Style::new().add_modifier(Modifier::BOLD)),
        Span::styled(format!("v{version}"), dim()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("██", Style::new().fg(green)),
        Span::raw("  "),
        Span::styled("█", Style::new().fg(teal)),
        Span::styled("██", Style::new().fg(blue)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("███████", Style::new().fg(green)),
        Span::raw("  "),
        Span::styled(cwd, dim()),
    ]));
}

/// Append lines for an exchange (request/response with sidebar).
fn render_exchange_lines(
    lines: &mut Vec<Line>,
    kind: ExchangeKind,
    status: ExchangeStatus,
    request: &str,
    response: Option<&str>,
    exit_code: Option<i32>,
    pulsate_bright: bool,
) {
    let base_color = match status {
        ExchangeStatus::Running | ExchangeStatus::Succeeded => kind.color(),
        ExchangeStatus::Failed => Color::Red,
    };

    // Running exchanges pulsate between normal and dim of the same color
    let sidebar_style = match status {
        ExchangeStatus::Running => {
            if pulsate_bright {
                Style::new().fg(base_color)
            } else {
                Style::new().fg(base_color).add_modifier(Modifier::DIM)
            }
        }
        _ => Style::new().fg(base_color),
    };

    // Shell commands are prefixed with "$ "
    let prefix = if kind == ExchangeKind::Shell {
        "$ "
    } else {
        ""
    };

    // Request lines with sidebar
    for text_line in request.lines() {
        lines.push(Line::from(vec![
            Span::styled(SIDEBAR_CHAR, sidebar_style),
            Span::raw(" "),
            Span::raw(format!("{prefix}{text_line}")),
        ]));
    }

    // Response lines with dim sidebar
    if let Some(resp) = response {
        let dim_sidebar_style = Style::new().fg(base_color).add_modifier(Modifier::DIM);
        for text_line in resp.lines() {
            lines.push(Line::from(vec![
                Span::styled(SIDEBAR_CHAR, dim_sidebar_style),
                Span::raw(" "),
                Span::raw(text_line.to_string()),
            ]));
        }
    }

    // Exit code (non-zero) for shell commands
    if kind == ExchangeKind::Shell
        && let Some(code) = exit_code
        && code != 0
    {
        lines.push(Line::from(vec![
            Span::styled(
                SIDEBAR_CHAR,
                Style::new().fg(Color::Red).add_modifier(Modifier::DIM),
            ),
            Span::raw(" "),
            Span::styled(
                format!("exit code: {code}"),
                Style::new().fg(Color::Red).add_modifier(Modifier::DIM),
            ),
        ]));
    }
}

/// Render the scrollable message area.
fn render_messages(frame: &mut Frame, app: &mut App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let pulsate_bright = app.tick_count / 2 % 2 == 0;

    for message in &app.messages {
        // Add a blank line separator between messages (except before the first)
        if !lines.is_empty() {
            lines.push(Line::raw(""));
        }

        match message {
            AppMessage::Welcome => render_welcome_lines(&mut lines),
            AppMessage::Exchange {
                kind,
                status,
                request,
                response,
                exit_code,
            } => render_exchange_lines(
                &mut lines,
                *kind,
                *status,
                request,
                response.as_deref(),
                *exit_code,
                pulsate_bright,
            ),
            AppMessage::System { content } => {
                for text_line in content.lines() {
                    lines.push(Line::from(vec![
                        Span::styled(SIDEBAR_CHAR, dim()),
                        Span::raw(" "),
                        Span::styled(text_line.to_string(), dim()),
                    ]));
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

/// Render the hint line below the input area: mode label on left, keyboard hints on right.
#[rustfmt::skip]
fn render_hints(frame: &mut Frame, app: &App, area: Rect) {
    let is_running = app.has_running_commands();
    let has_ghost = app.ghost_suggestion.is_some();

    // Mode label on the left
    let kind = ExchangeKind::from(app.mode);
    let label = Line::from(Span::styled(
        kind.label(),
        Style::new().fg(kind.color()).add_modifier(Modifier::DIM),
    ));
    #[allow(clippy::cast_possible_truncation)]
    let label_width = kind.label().len() as u16 + 1; // +1 for padding

    // Keyboard hints on the right
    let hints = if is_running {
        Line::from(vec![Span::raw("ctrl+c "), Span::styled("cancel", dim())])
    } else if has_ghost {
        Line::from(vec![
            Span::raw("tab "), Span::styled("word", dim()),
            Span::raw("  \u{2192} "), Span::styled("accept", dim()),
            Span::raw("  alt+\u{21b5} "), Span::styled("newline", dim()),
        ])
    } else {
        match app.mode {
            AppMode::Chat => Line::from(vec![
                Span::raw("alt+\u{21b5} "), Span::styled("newline", dim()),
                Span::raw("  ctrl+s "), Span::styled("shell", dim()),
                Span::raw("  ctrl+c "), Span::styled("quit", dim()),
            ]),
            AppMode::Shell => Line::from(vec![
                Span::raw("alt+\u{21b5} "), Span::styled("newline", dim()),
                Span::raw("  ctrl+d "), Span::styled("chat", dim()),
                Span::raw("  ctrl+c "), Span::styled("clear", dim()),
            ]),
        }
    };

    let layout = Layout::horizontal([
        Constraint::Length(label_width),
        Constraint::Min(1),
    ]).split(area);

    frame.render_widget(Paragraph::new(label), layout[0]);
    frame.render_widget(Paragraph::new(hints).alignment(Alignment::Right), layout[1]);
}

/// Render the input area with cursor: dark grey background, colored sidebar, no border.
fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let input_text = app.input.text();
    let kind = ExchangeKind::from(app.mode);
    let is_running = app.has_running_commands();

    // Dark grey background block (no border)
    let bg_block = Block::default().style(Style::new().bg(INPUT_BG));
    frame.render_widget(bg_block, area);

    // Sidebar in first column — one glyph per row for full-height coverage
    if area.height > 0 {
        let sidebar_lines: Vec<Line> = (0..area.height)
            .map(|_| Line::from(Span::styled(SIDEBAR_CHAR, Style::new().fg(kind.color()))))
            .collect();
        let sidebar = Paragraph::new(Text::from(sidebar_lines)).style(Style::new().bg(INPUT_BG));
        let sidebar_area = Rect {
            x: area.x,
            y: area.y,
            width: 1,
            height: area.height,
        };
        frame.render_widget(sidebar, sidebar_area);
    }

    // Content area starts at column 2 (sidebar + space)
    let content_area = Rect {
        x: area.x + 2,
        y: area.y,
        width: area.width.saturating_sub(2),
        height: area.height,
    };

    let dim_style = Style::default().add_modifier(Modifier::DIM);
    let content = if let Some(ghost) = &app.ghost_suggestion {
        let mut spans = vec![
            Span::raw(input_text.to_string()),
            Span::styled(ghost.as_str(), dim_style),
        ];
        if app.ghost_is_truncated {
            spans.push(Span::styled("\u{2026}", dim_style));
        }
        Line::from(spans)
    } else {
        Line::from(input_text.to_string())
    };

    let paragraph = Paragraph::new(content)
        .style(Style::new().bg(INPUT_BG))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, content_area);

    // Show inline send/run hint — hide when running or when input text gets close
    // to the edge. The hint renders as an overlay on top of any ghost text.
    if !is_running {
        let hint_text = match app.mode {
            AppMode::Chat => " send",
            AppMode::Shell => " run",
        };
        let inner_width = content_area.width;
        #[allow(clippy::cast_possible_truncation)]
        let hint_display_width = (1 + hint_text.len()) as u16;
        let input_char_len = app.input.text().chars().count();
        if inner_width > hint_display_width + 8
            && input_char_len < (inner_width - hint_display_width - 2) as usize
        {
            let hint_area = Rect {
                x: content_area.x + content_area.width - hint_display_width,
                y: content_area.y,
                width: hint_display_width,
                height: 1,
            };
            let hint = Line::from(vec![Span::raw("\u{21b5}"), Span::styled(hint_text, dim())]);
            frame.render_widget(
                Paragraph::new(hint).style(Style::new().bg(INPUT_BG)),
                hint_area,
            );
        }
    }

    // Inner width available for text (content area width)
    let inner_width = content_area.width.max(1) as usize;

    // Position the cursor within the input area, accounting for wrapping
    let (cursor_col, cursor_row) =
        cursor_position_wrapped(app.input.text(), app.input.cursor(), inner_width);

    // +2 for sidebar + space
    #[allow(clippy::cast_possible_truncation)]
    let x = area.x + 2 + cursor_col as u16;
    #[allow(clippy::cast_possible_truncation)]
    let y = area.y + cursor_row as u16;

    // Only show cursor if it fits within the input area
    if x < area.x + area.width && y < area.y + area.height {
        frame.set_cursor_position(Position::new(x, y));
    }
}

/// Compute the popup area above the input area for the given number of items.
///
/// Returns `None` if there isn't enough space to render a meaningful popup.
fn popup_area(input_area: Rect, item_count: usize) -> Option<Rect> {
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

/// Style for unselected row primary text.
const fn unselected_style() -> Style {
    Style::new().fg(Color::White)
}

/// Style for the selected row's primary text in autocomplete popups.
const fn selected_style() -> Style {
    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
}

/// Style for the selected row's secondary text (e.g. descriptions, paths).
const fn selected_secondary_style() -> Style {
    Style::new().fg(Color::White)
}

/// Render an autocomplete popup with the given lines and optional title.
fn render_popup(frame: &mut Frame, area: Rect, lines: Vec<Line>, title: Option<&str>) {
    frame.render_widget(Clear, area);

    let mut block = Block::default().borders(Borders::ALL).border_style(dim());
    if let Some(t) = title {
        block = block.title(t);
    }

    let popup = Paragraph::new(Text::from(lines)).block(block);
    frame.render_widget(popup, area);
}

/// Render the autocomplete popup floating above the input area.
fn render_autocomplete(frame: &mut Frame, app: &App, input_area: Rect) {
    let candidates = app.commands_state.candidates();
    let Some(area) = popup_area(input_area, candidates.len()) else {
        return;
    };

    let max_name_width = candidates.iter().map(|c| c.name().len()).max().unwrap_or(0);
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
                    Span::styled(padded_name, selected_style()),
                    Span::styled(desc.to_string(), selected_secondary_style()),
                ])
            } else {
                Line::from(vec![
                    Span::styled(padded_name, unselected_style()),
                    Span::styled(desc.to_string(), dim()),
                ])
            }
        })
        .collect();

    render_popup(frame, area, lines, None);
}

/// Render the history autocomplete popup floating above the input area.
fn render_history_autocomplete(frame: &mut Frame, app: &App, input_area: Rect) {
    let candidates = app.history_state.candidates();
    let Some(area) = popup_area(input_area, candidates.len()) else {
        return;
    };

    let selected = app.history_state.selected();
    let lines: Vec<Line> = candidates
        .iter()
        .enumerate()
        .map(|(i, candidate)| {
            let display = format!(" {}", candidate.preview);
            if i == selected {
                Line::from(Span::styled(display, selected_style()))
            } else {
                Line::from(Span::styled(display, unselected_style()))
            }
        })
        .collect();

    render_popup(frame, area, lines, Some(" History "));
}

/// Render the file autocomplete popup floating above the input area.
fn render_files_autocomplete(frame: &mut Frame, app: &App, input_area: Rect) {
    let candidates = app.files_state.candidates();
    let Some(area) = popup_area(input_area, candidates.len()) else {
        return;
    };

    let selected = app.files_state.selected();
    let is_at = app.files_state.is_at_search();

    // For @ mode, compute max display name width for column alignment
    let max_name_width = if is_at {
        candidates
            .iter()
            .map(|c| c.display().len())
            .max()
            .unwrap_or(0)
    } else {
        0
    };

    let lines: Vec<Line> = candidates
        .iter()
        .enumerate()
        .map(|(i, candidate)| {
            let display = candidate.display();
            let path = candidate.path();

            let (name_part, path_part) = if is_at {
                // @ mode: padded name column + aligned path column
                let name = format!(" {display:<max_name_width$}  ");
                let show_path =
                    path != display && !path.is_empty() && path != format!("{display}/");
                let path_str = if show_path {
                    path.to_string()
                } else {
                    String::new()
                };
                (name, path_str)
            } else {
                // Path mode: just the display name
                (format!(" {display}"), String::new())
            };

            if i == selected {
                let mut spans = vec![Span::styled(name_part, selected_style())];
                if !path_part.is_empty() {
                    spans.push(Span::styled(path_part, selected_secondary_style()));
                }
                Line::from(spans)
            } else {
                let mut spans = vec![Span::styled(name_part, unselected_style())];
                if !path_part.is_empty() {
                    spans.push(Span::styled(path_part, dim()));
                }
                Line::from(spans)
            }
        })
        .collect();

    let title = if is_at {
        " Search files "
    } else {
        " Select file "
    };

    render_popup(frame, area, lines, Some(title));
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
