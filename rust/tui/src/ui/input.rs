use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Position, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
};

use crate::app::{AgentSession, App, AppMode, ExchangeKind};

use super::common::{
    BRAILLE_SPINNER_FRAMES, INPUT_BG, NUM_GUTTER, SIDEBAR_CHAR, cursor_position_wrapped, dim,
    visual_line_count, wrap_content,
};

const MAX_GHOST_LINES: usize = 3;

fn last_visual_line_len(text: &str, wrap_width: usize) -> usize {
    let logical_lines: Vec<&str> = if text.is_empty() {
        vec![""]
    } else {
        text.split('\n').collect()
    };

    logical_lines.last().map_or(0, |last| {
        wrap_content(last, wrap_width)
            .last()
            .map_or(0, |s| s.chars().count())
    })
}

fn ghost_chunks(ghost: &str, remaining_on_last: usize, wrap_width: usize) -> (Vec<String>, bool) {
    let ghost_chars: Vec<char> = ghost.chars().collect();
    let mut chunks: Vec<String> = Vec::new();
    let mut start = 0;

    if remaining_on_last > 0 && !ghost_chars.is_empty() {
        let end = remaining_on_last.min(ghost_chars.len());
        let chunk_end = ghost_chars[start..end]
            .iter()
            .position(|&c| c == '\n')
            .map_or(end, |p| start + p);
        chunks.push(ghost_chars[start..chunk_end].iter().collect());
        start = chunk_end;
        if start < ghost_chars.len() && ghost_chars[start] == '\n' {
            start += 1;
        }
    }

    let mut ghost_line_count: usize = 0;
    while start < ghost_chars.len() && ghost_line_count < MAX_GHOST_LINES {
        let end = (start + wrap_width).min(ghost_chars.len());
        let chunk_end = ghost_chars[start..end]
            .iter()
            .position(|&c| c == '\n')
            .map_or(end, |p| start + p);
        chunks.push(ghost_chars[start..chunk_end].iter().collect());
        start = chunk_end;
        if start < ghost_chars.len() && ghost_chars[start] == '\n' {
            start += 1;
        }
        ghost_line_count += 1;
    }

    (chunks, start < ghost_chars.len())
}

/// Render the input area with cursor: dark grey background, colored sidebar, no border.
#[allow(clippy::too_many_lines)]
pub(super) fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let input_text = app.input.text();
    let kind = ExchangeKind::from(app.mode);
    let is_running = app.has_running();

    // Use agent color when in multi-agent chat mode, otherwise kind color
    let bar_color = if app.mode == AppMode::Chat && app.sessions.len() > 1 {
        AgentSession::color(app.active_session)
    } else {
        kind.color()
    };

    // Mode indicator in the gutter (no dark bg)
    let gutter = NUM_GUTTER;
    if area.height > 0 {
        let indicator = if app.mode == AppMode::Chat && app.active_session_is_running() {
            let frame_idx = (app.tick_count as usize / 2) % BRAILLE_SPINNER_FRAMES.len();
            format!(" {} ", BRAILLE_SPINNER_FRAMES[frame_idx])
        } else {
            match app.mode {
                AppMode::Chat => " > ".to_string(),
                AppMode::Shell => " $ ".to_string(),
            }
        };
        let indicator_line = Line::from(Span::styled(&indicator, Style::new().fg(bar_color)));
        let gutter_area = Rect {
            x: area.x,
            y: area.y,
            width: gutter,
            height: 1,
        };
        frame.render_widget(Paragraph::new(indicator_line), gutter_area);
    }

    // Dark grey background after gutter
    let bg_area = Rect {
        x: area.x + gutter,
        y: area.y,
        width: area.width.saturating_sub(gutter),
        height: area.height,
    };
    let bg_block = Block::default().style(Style::new().bg(INPUT_BG));
    frame.render_widget(bg_block, bg_area);

    // Sidebar after gutter — one glyph per row for full-height coverage
    if area.height > 0 {
        let sidebar_lines: Vec<Line> = (0..area.height)
            .map(|_| Line::from(Span::styled(SIDEBAR_CHAR, Style::new().fg(bar_color))))
            .collect();
        let sidebar = Paragraph::new(Text::from(sidebar_lines)).style(Style::new().bg(INPUT_BG));
        let sidebar_area = Rect {
            x: area.x + gutter,
            y: area.y,
            width: 1,
            height: area.height,
        };
        frame.render_widget(sidebar, sidebar_area);
    }

    // Content area starts after gutter (3) + sidebar (1) + space (1)
    let content_offset = gutter + 2;
    let content_area = Rect {
        x: area.x + content_offset,
        y: area.y,
        width: area.width.saturating_sub(content_offset),
        height: area.height,
    };

    let dim_style = Style::default().add_modifier(Modifier::DIM);

    // Pre-wrap text into visual lines using character-level wrapping so
    // rendering matches cursor_position_wrapped / visual_line_count exactly.
    let wrap_width = content_area.width.max(1) as usize;
    let mut visual_lines: Vec<Line> = Vec::new();
    let logical_lines: Vec<&str> = if input_text.is_empty() {
        vec![""]
    } else {
        input_text.split('\n').collect()
    };
    for logical_line in &logical_lines {
        if logical_line.is_empty() {
            visual_lines.push(Line::from(String::new()));
        } else {
            for chunk in wrap_content(logical_line, wrap_width) {
                visual_lines.push(Line::from(chunk));
            }
        }
    }

    // Append ghost text, wrapping it across visual lines and capping at 3 ghost lines.
    if let Some(ghost) = &app.ghost_suggestion {
        // How many chars remain on the last visual line before wrapping
        let last_line_len = visual_lines.last().map_or(0, |l| {
            l.spans
                .iter()
                .map(|s| s.content.chars().count())
                .sum::<usize>()
        });
        let remaining_on_last = wrap_width.saturating_sub(last_line_len);

        let (ghost_chunks, ghost_is_truncated) = ghost_chunks(ghost, remaining_on_last, wrap_width);

        // Append the first ghost chunk to the last visual line
        let mut chunks_iter = ghost_chunks.into_iter();
        if let Some(first_chunk) = chunks_iter.next()
            && let Some(last) = visual_lines.last_mut()
        {
            let existing: String = last.spans.iter().map(|s| s.content.as_ref()).collect();
            *last = Line::from(vec![
                Span::raw(existing),
                Span::styled(first_chunk, dim_style),
            ]);
        }

        // Add remaining ghost chunks as new visual lines
        for chunk in chunks_iter {
            visual_lines.push(Line::from(Span::styled(chunk, dim_style)));
        }

        // Append ellipsis indicator on the last ghost line if truncated
        if ghost_is_truncated && let Some(last) = visual_lines.last_mut() {
            last.spans.push(Span::styled(" \u{2026}", dim_style));
        }
    }

    // Inner width available for text (content area width)
    let inner_width = content_area.width.max(1) as usize;

    // Position the cursor within the input area, accounting for wrapping
    let (cursor_col, cursor_row) =
        cursor_position_wrapped(app.input.text(), app.input.cursor(), inner_width);

    // Adjust persistent scroll offset only when cursor leaves the visible window
    #[allow(clippy::cast_possible_truncation)]
    let cursor_row_u16 = cursor_row as u16;
    let visible_height = content_area.height;
    if cursor_row_u16 < app.input_scroll {
        app.input_scroll = cursor_row_u16;
    } else if cursor_row_u16 >= app.input_scroll + visible_height {
        app.input_scroll = cursor_row_u16 - visible_height + 1;
    }
    let scroll_y = app.input_scroll;

    // Char count per visual line — needed for the send/run hint overlap check
    // after visual_lines is moved into the Paragraph.
    let last_visible_row = (scroll_y + visible_height).saturating_sub(1) as usize;
    let last_visible_line_chars: usize = visual_lines.get(last_visible_row).map_or(0, |l| {
        l.spans.iter().map(|s| s.content.chars().count()).sum()
    });

    let content = Text::from(visual_lines);
    let paragraph = Paragraph::new(content)
        .style(Style::new().bg(INPUT_BG))
        .scroll((scroll_y, 0));
    frame.render_widget(paragraph, content_area);

    // Show inline send/run hint anchored to the bottom-right of the input area.
    // Hide when running or when the last visible line's text would overlap.
    if !is_running {
        let hint_text = match app.mode {
            AppMode::Chat => " send",
            AppMode::Shell => " run",
        };
        let hint_inner_width = content_area.width;
        #[allow(clippy::cast_possible_truncation)]
        let hint_display_width = (1 + hint_text.len()) as u16;

        #[allow(clippy::cast_possible_truncation)]
        let has_room = hint_inner_width > hint_display_width + 8
            && last_visible_line_chars < (hint_inner_width - hint_display_width - 2) as usize;

        if has_room {
            let hint_area = Rect {
                x: content_area.x + content_area.width - hint_display_width,
                y: content_area.y + content_area.height.saturating_sub(1),
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

    // +gutter + sidebar + space
    #[allow(clippy::cast_possible_truncation)]
    let x = area.x + content_offset + cursor_col as u16;
    #[allow(clippy::cast_possible_truncation)]
    let y = area.y + cursor_row_u16.saturating_sub(scroll_y);

    if x < area.x + area.width && y < area.y + area.height {
        frame.set_cursor_position(Position::new(x, y));
    }
}

/// Render the hint line below the input area: mode label on left, keyboard hints on right.
#[rustfmt::skip]
pub(super) fn hints(frame: &mut Frame, app: &App, area: Rect) {
    let is_running = app.has_running();
    let has_ghost = app.ghost_suggestion.is_some();

    // Mode label on the left, indented to align with sidebar bars
    let label_spans = if app.mode == AppMode::Chat {
        // Always show active agent name in agent's color
        let name = &app.active().name;
        let color = AgentSession::color(app.active_session);
        let pct = app.active().context_usage_percent;
        let mut spans = vec![Span::styled(format!("   {name}"), Style::new().fg(color))];
        if pct > 0 {
            spans.push(Span::styled(format!(" {pct}%"), Style::new().fg(color).add_modifier(Modifier::DIM)));
        }
        spans
    } else {
        let kind = ExchangeKind::from(app.mode);
        vec![Span::styled(format!("   {}", kind.label()), Style::new().fg(kind.color()))]
    };

    #[allow(clippy::cast_possible_truncation)]
    let label_width: u16 = label_spans.iter().map(|s| s.content.len() as u16).sum::<u16>() + 1; // +1 for padding
    let label = Line::from(label_spans);

    // Keyboard hints on the right
    let hints = if is_running {
        Line::from(vec![Span::raw("esc "), Span::styled("cancel", dim())])
    } else if has_ghost {
        Line::from(vec![
            Span::raw("tab "), Span::styled("word", dim()),
            Span::raw("  \u{2192} "), Span::styled("accept", dim()),
            Span::raw("  alt+\u{21b5} "), Span::styled("newline", dim()),
        ])
    } else {
        match app.mode {
            AppMode::Chat => {
                let mut spans = vec![
                    Span::raw("alt+\u{21b5} "), Span::styled("newline", dim()),
                    Span::raw("  ctrl+s "), Span::styled("shell", dim()),
                ];
                if app.sessions.len() > 1 {
                    spans.push(Span::raw("  ctrl+a "));
                    spans.push(Span::styled("agent", dim()));
                }
                spans.push(Span::raw("  ctrl+c "));
                spans.push(Span::styled("quit", dim()));
                Line::from(spans)
            }
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

/// Compute input area height based on visual lines (accounting for wrapping).
pub(super) fn input_height(app: &App, area: Rect) -> u16 {
    // Inner width: gutter (2) + sidebar (1) + space (1) + content area, no borders
    let inner_width = area.width.saturating_sub(NUM_GUTTER + 2).max(1) as usize;

    // Base height from input text alone
    #[allow(clippy::cast_possible_truncation)]
    let input_lines = visual_line_count(app.input.text(), inner_width) as u16;

    // Add ghost text contribution, capped at MAX_GHOST_LINES extra visual lines
    let visual_lines = if let Some(ghost) = &app.ghost_suggestion {
        // How many chars remain on the last input visual line
        let last_input_line_len = last_visual_line_len(app.input.text(), inner_width);
        let remaining = inner_width.saturating_sub(last_input_line_len);
        let (ghost_chunks, _) = ghost_chunks(ghost, remaining, inner_width);
        let extra_lines = ghost_chunks.len().saturating_sub(1);

        #[allow(clippy::cast_possible_truncation)]
        let total = input_lines + extra_lines as u16;
        total
    } else {
        input_lines
    };

    // Ensure enough height for the cursor row (cursor may be on the line
    // *after* the last visual content line when it sits at a wrap boundary).
    let (_, cursor_row) =
        cursor_position_wrapped(app.input.text(), app.input.cursor(), inner_width);
    #[allow(clippy::cast_possible_truncation)]
    let cursor_lines = cursor_row as u16 + 1;

    let max_input_height = (area.height / 3).max(3);
    visual_lines.max(cursor_lines).clamp(1, max_input_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ghost_chunks_caps_continuation_lines() {
        let (chunks, truncated) = ghost_chunks("abcdefghi", 2, 2);
        assert_eq!(chunks, vec!["ab", "cd", "ef", "gh"]);
        assert!(truncated);
    }

    #[test]
    fn ghost_chunks_respects_newline() {
        let (chunks, truncated) = ghost_chunks("ab\ncd", 4, 4);
        assert_eq!(chunks, vec!["ab", "cd"]);
        assert!(!truncated);
    }
}
