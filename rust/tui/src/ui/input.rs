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

/// Render the input area with cursor: dark grey background, colored sidebar, no border.
#[allow(clippy::too_many_lines)]
pub(super) fn render(frame: &mut Frame, app: &App, area: Rect) {
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

    // Append ghost text to the last visual line
    if let Some(ghost) = &app.ghost_suggestion {
        if let Some(last) = visual_lines.last_mut() {
            let existing: String = last.spans.iter().map(|s| s.content.as_ref()).collect();
            let mut spans = vec![Span::raw(existing)];
            spans.push(Span::styled(ghost.as_str(), dim_style));
            if app.ghost_is_truncated {
                spans.push(Span::styled("\u{2026}", dim_style));
            }
            *last = Line::from(spans);
        }
    }

    let content = Text::from(visual_lines);

    let paragraph = Paragraph::new(content).style(Style::new().bg(INPUT_BG));
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

    // +gutter + sidebar + space
    #[allow(clippy::cast_possible_truncation)]
    let x = area.x + content_offset + cursor_col as u16;
    #[allow(clippy::cast_possible_truncation)]
    let y = area.y + cursor_row as u16;

    // Only show cursor if it fits within the input area
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
    let (label_text, label_color) = if app.mode == AppMode::Chat {
        // Always show active agent name in agent's color
        let name = &app.active().name;
        let color = AgentSession::color(app.active_session);
        (format!("   {name}"), color)
    } else {
        let kind = ExchangeKind::from(app.mode);
        (format!("   {}", kind.label()), kind.color())
    };

    #[allow(clippy::cast_possible_truncation)]
    let label_width = label_text.len() as u16 + 1; // +1 for padding
    let label = Line::from(Span::styled(
        label_text,
        Style::new().fg(label_color),
    ));

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
    use std::borrow::Cow;

    // Inner width: gutter (2) + sidebar (1) + space (1) + content area, no borders
    let inner_width = area.width.saturating_sub(NUM_GUTTER + 2).max(1) as usize;
    let text_for_height: Cow<str> = match &app.ghost_suggestion {
        Some(ghost) => Cow::Owned(format!("{}{ghost}", app.input.text())),
        None => Cow::Borrowed(app.input.text()),
    };
    #[allow(clippy::cast_possible_truncation)]
    let visual_lines = visual_line_count(&text_for_height, inner_width) as u16;

    // Ensure enough height for the cursor row (cursor may be on the line
    // *after* the last visual content line when it sits at a wrap boundary).
    let (_, cursor_row) =
        cursor_position_wrapped(app.input.text(), app.input.cursor(), inner_width);
    #[allow(clippy::cast_possible_truncation)]
    let cursor_lines = cursor_row as u16 + 1;

    let max_input_height = (area.height / 3).max(3);
    visual_lines.max(cursor_lines).clamp(1, max_input_height)
}
