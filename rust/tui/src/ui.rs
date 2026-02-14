use std::borrow::Cow;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::agent::{ResponseSegment, ToolCallStatus};
use crate::app::{
    AgentSession, App, AppMessage, AppMode, ExchangeKind, ExchangeStatus, WizardStep,
};

/// Dim style used for hint descriptions.
const fn dim() -> Style {
    Style::new().fg(Color::DarkGray)
}

/// Background color for the input area.
const INPUT_BG: Color = Color::Rgb(40, 40, 40);

// ─── Annotation presentation ────────────────────────────────────────
//
// Maps `ResponseSegment` variants to visual presentation (symbol + style).
// To change annotation *content*, see `format_tool_start()` etc. in agent.rs.
// To change annotation *appearance*, edit here.

/// Rotating half-circle spinner frames for in-progress tool calls.
const SPINNER_FRAMES: [char; 4] = ['\u{25d0}', '\u{25d3}', '\u{25d1}', '\u{25d2}'];

/// Pulsating frames for in-progress thinking: · ∗ ✱ (small → medium → large).
const THINKING_FRAMES: [char; 3] = ['\u{00b7}', '\u{2217}', '\u{2731}'];

/// Render the entire UI for one frame.
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Calculate input area height based on visual lines (accounting for wrapping).
    // Include ghost text suffix when computing height so it doesn't clip.
    // Inner width: gutter (2) + sidebar (1) + space (1) + content area, no borders
    let inner_width = area.width.saturating_sub(NUM_GUTTER + 2).max(1) as usize;
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

    // --- Render wizard modal (overlays everything else) ---
    if app.wizard.is_some() {
        render_new_agent_modal(frame, app, area);
        return;
    }

    // --- Render autocomplete popup (floats above input) ---
    // Cancel popup has highest priority, then agents, model, history, commands, files, responses.
    if app.cancel_state.is_visible() {
        render_cancel_autocomplete(frame, app, input_area);
    } else if app.agents_state.is_visible() {
        render_agents_autocomplete(frame, app, input_area);
    } else if app.models_state.is_visible() {
        render_models_autocomplete(frame, app, input_area);
    } else if app.history_state.is_visible() {
        render_history_autocomplete(frame, app, input_area);
    } else if app.commands_state.is_visible() {
        render_autocomplete(frame, app, input_area);
    } else if app.files_state.is_visible() {
        render_files_autocomplete(frame, app, input_area);
    } else if app.responses_state.is_visible() {
        render_responses_autocomplete(frame, app, input_area);
    }
}

/// The sidebar character (U+258C, left half block).
const SIDEBAR_CHAR: &str = "\u{258c}";

/// Width of the exchange number gutter (2-digit number + space).
const NUM_GUTTER: u16 = 3;

/// Append lines for a welcome message.
fn render_welcome_lines(lines: &mut Vec<Line>, upgrade_available: Option<&str>) {
    let version = stencila_version::STENCILA_VERSION;
    let green = Color::Rgb(102, 255, 102);
    let teal = Color::Rgb(15, 104, 96);
    let blue = Color::Rgb(37, 104, 239);
    let cwd = std::env::current_dir()
        .ok()
        .map(|p| {
            let path = p.display().to_string();
            // Replace home directory prefix with ~ for a shorter, familiar display
            if let Some(base) = directories::BaseDirs::new() {
                let home = base.home_dir().display().to_string();
                if let Some(rest) = path.strip_prefix(&home) {
                    return format!("~{rest}");
                }
            }
            path
        })
        .unwrap_or_default();

    let pad = "   ";
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::raw(pad),
        Span::styled("███████", Style::new().fg(green)),
        Span::raw("  "),
        Span::styled("Stencila ", Style::new().add_modifier(Modifier::BOLD)),
        Span::styled(format!("v{version}"), dim()),
    ]));
    lines.push(Line::from(vec![
        Span::raw(pad),
        Span::styled("██", Style::new().fg(green)),
        Span::raw("  "),
        Span::styled("█", Style::new().fg(teal)),
        Span::styled("██", Style::new().fg(blue)),
    ]));
    lines.push(Line::from(vec![
        Span::raw(pad),
        Span::styled("███████", Style::new().fg(green)),
        Span::raw("  "),
        Span::styled(cwd, dim()),
    ]));

    if let Some(version) = upgrade_available {
        let gold = Color::Rgb(255, 191, 0);
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::raw(pad),
            Span::styled(
                format!("Update available (v{version}). Use "),
                Style::new().fg(gold),
            ),
            Span::styled("/upgrade", Style::new().fg(blue)),
            Span::styled(" to install.", Style::new().fg(gold)),
        ]));
    }
}

/// Render a prefixed annotation line: `"● label"` with symbol styling and dim content.
fn push_annotation_lines(
    lines: &mut Vec<Line>,
    sidebar_style: Style,
    symbol: char,
    symbol_style: Style,
    content: &str,
    content_style: Style,
    content_width: usize,
) {
    let num_padding = "   ";
    let avail = content_width.saturating_sub(2); // "● " prefix
    let chunks = wrap_content(content, avail);
    for (i, chunk) in chunks.iter().enumerate() {
        let prefix = if i == 0 {
            Span::styled(format!("{symbol} "), symbol_style)
        } else {
            Span::raw("  ")
        };
        lines.push(Line::from(vec![
            Span::raw(num_padding),
            Span::styled(SIDEBAR_CHAR, sidebar_style),
            Span::raw(" "),
            prefix,
            Span::styled(chunk.clone(), content_style),
        ]));
    }
}

/// Render structured response segments with interleaved tool annotations.
fn render_response_segments(
    lines: &mut Vec<Line>,
    segments: &[ResponseSegment],
    base_color: Color,
    annotation_tick: Option<u32>,
    content_width: usize,
) {
    let num_padding = "   ";
    let dim_sidebar_style = Style::new().fg(base_color).add_modifier(Modifier::DIM);
    let blank_line = || {
        Line::from(vec![
            Span::raw(num_padding),
            Span::styled(SIDEBAR_CHAR, dim_sidebar_style),
        ])
    };
    let mut prev_was_annotation = false;

    for segment in segments {
        match segment {
            ResponseSegment::Text(text) => {
                if prev_was_annotation {
                    lines.push(blank_line());
                }
                for text_line in text.lines() {
                    for chunk in wrap_content(text_line, content_width) {
                        lines.push(Line::from(vec![
                            Span::raw(num_padding),
                            Span::styled(SIDEBAR_CHAR, dim_sidebar_style),
                            Span::raw(" "),
                            Span::raw(chunk),
                        ]));
                    }
                }
                prev_was_annotation = false;
            }
            ResponseSegment::ToolCall { label, status, .. } => {
                if !prev_was_annotation {
                    lines.push(blank_line());
                }
                let (symbol, symbol_style) = tool_call_symbol(status, annotation_tick);
                push_annotation_lines(
                    lines,
                    dim_sidebar_style,
                    symbol,
                    symbol_style,
                    label,
                    dim(),
                    content_width,
                );
                if let ToolCallStatus::Error { detail } = status {
                    for detail_line in detail.lines() {
                        push_annotation_lines(
                            lines,
                            dim_sidebar_style,
                            '\u{21b3}',
                            Style::new().fg(Color::Red),
                            detail_line,
                            Style::new().fg(Color::Red),
                            content_width,
                        );
                    }
                }
                prev_was_annotation = true;
            }
            ResponseSegment::Thinking { text, complete } => {
                if !prev_was_annotation {
                    lines.push(blank_line());
                }
                render_thinking_segment(
                    lines,
                    dim_sidebar_style,
                    text,
                    *complete,
                    annotation_tick,
                    content_width,
                );
                prev_was_annotation = true;
            }
            ResponseSegment::Warning(message) => {
                if !prev_was_annotation {
                    lines.push(blank_line());
                }
                push_annotation_lines(
                    lines,
                    dim_sidebar_style,
                    '\u{25cf}',
                    Style::new().fg(Color::Yellow),
                    message,
                    dim(),
                    content_width,
                );
                prev_was_annotation = true;
            }
        }
    }
}

/// Map a tool call status to its display symbol and style.
fn tool_call_symbol(status: &ToolCallStatus, tick: Option<u32>) -> (char, Style) {
    match status {
        ToolCallStatus::Running => {
            let sym = match tick {
                Some(t) => SPINNER_FRAMES[(t as usize / 2) % SPINNER_FRAMES.len()],
                None => '\u{25cf}',
            };
            (sym, Style::new().fg(Color::DarkGray))
        }
        ToolCallStatus::Done => (
            '\u{25cf}',
            Style::new().fg(Color::Green).add_modifier(Modifier::DIM),
        ),
        ToolCallStatus::Error { .. } => ('\u{25cf}', Style::new().fg(Color::Red)),
    }
}

/// Map in-progress thinking to its pulsating display symbol and style.
/// Render a thinking/reasoning segment: header + indented content with
/// visual-line-based truncation.
#[allow(clippy::too_many_arguments)]
fn render_thinking_segment(
    lines: &mut Vec<Line>,
    dim_sidebar_style: Style,
    text: &str,
    complete: bool,
    annotation_tick: Option<u32>,
    content_width: usize,
) {
    // Header: pulsating symbol + "Thinking"
    let (symbol, symbol_style) = if complete {
        ('\u{2217}', Style::new().fg(Color::DarkGray))
    } else {
        thinking_symbol(annotation_tick)
    };
    push_annotation_lines(
        lines,
        dim_sidebar_style,
        symbol,
        symbol_style,
        "Thinking",
        dim(),
        content_width,
    );

    // Content: ↳ on first line, space indent on rest.
    // Truncate by visual (wrapped) lines, not source newlines.
    if !text.is_empty() {
        let max_visual = if complete { 5 } else { usize::MAX };
        let avail = content_width.saturating_sub(2); // prefix width
        let mut visual_count = 0usize;
        let mut first_source_line = true;
        let mut remaining_visual = 0usize;
        let mut truncated = false;

        for source_line in text.lines() {
            let wrapped_count = wrap_content(source_line, avail).len();
            if truncated {
                remaining_visual += wrapped_count;
                continue;
            }
            if visual_count + wrapped_count > max_visual && visual_count > 0 {
                truncated = true;
                remaining_visual += wrapped_count;
                continue;
            }
            let prefix = if first_source_line { '\u{21b3}' } else { ' ' };
            push_annotation_lines(
                lines,
                dim_sidebar_style,
                prefix,
                Style::new().fg(Color::DarkGray),
                source_line,
                dim(),
                content_width,
            );
            visual_count += wrapped_count;
            first_source_line = false;
        }
        if complete && remaining_visual > 0 {
            push_annotation_lines(
                lines,
                dim_sidebar_style,
                ' ',
                dim(),
                &format!("[+{remaining_visual} more lines]"),
                dim(),
                content_width,
            );
        }
    }
}

fn thinking_symbol(tick: Option<u32>) -> (char, Style) {
    let sym = match tick {
        Some(t) => THINKING_FRAMES[(t as usize / 2) % THINKING_FRAMES.len()],
        None => THINKING_FRAMES[0],
    };
    (sym, Style::new().fg(Color::DarkGray))
}

/// Render plain text response lines (for shell commands and fallback).
fn render_response_text(
    lines: &mut Vec<Line>,
    resp: &str,
    base_color: Color,
    content_width: usize,
) {
    let num_padding = "   ";
    let dim_sidebar_style = Style::new().fg(base_color).add_modifier(Modifier::DIM);
    for text_line in resp.lines() {
        for chunk in wrap_content(text_line, content_width) {
            lines.push(Line::from(vec![
                Span::raw(num_padding),
                Span::styled(SIDEBAR_CHAR, dim_sidebar_style),
                Span::raw(" "),
                Span::raw(chunk),
            ]));
        }
    }
}

/// Append lines for an exchange (request/response with sidebar).
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn render_exchange_lines(
    lines: &mut Vec<Line>,
    exchange_num: usize,
    kind: ExchangeKind,
    status: ExchangeStatus,
    request: &str,
    response: Option<&str>,
    response_segments: Option<&[ResponseSegment]>,
    exit_code: Option<i32>,
    tick_count: u32,
    content_width: usize,
    agent_tag: Option<&(String, Color)>,
) {
    let kind_color = agent_tag.map_or_else(|| kind.color(), |(_, color)| *color);
    let base_color = match status {
        ExchangeStatus::Running | ExchangeStatus::Succeeded => kind_color,
        ExchangeStatus::Failed => Color::Red,
        ExchangeStatus::Cancelled => Color::DarkGray,
    };

    // Running exchanges pulsate between normal and dim of the same color
    let pulsate_bright = tick_count / 2 % 2 == 0;
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

    let num_style = if status == ExchangeStatus::Running {
        Style::new().fg(base_color).add_modifier(Modifier::DIM)
    } else {
        Style::new().fg(base_color)
    };
    let num_padding = "   ";

    // Request lines with sidebar and exchange number (wraps at 99)
    let display_num = ((exchange_num - 1) % 99) + 1;
    let mut first_line = true;
    for text_line in request.lines() {
        let prefixed = format!("{prefix}{text_line}");
        for chunk in wrap_content(&prefixed, content_width) {
            let num_col = if first_line {
                first_line = false;
                Span::styled(format!("{display_num:>2} "), num_style)
            } else {
                Span::raw(num_padding)
            };
            lines.push(
                Line::from(vec![
                    num_col,
                    Span::styled(SIDEBAR_CHAR, sidebar_style),
                    Span::raw(" "),
                    Span::raw(chunk),
                ])
                .style(Style::new().bg(INPUT_BG)),
            );
        }
    }

    // Agent name tag after the request in multi-agent mode
    if let Some((name, color)) = agent_tag {
        let dim_sidebar_style = Style::new().fg(base_color).add_modifier(Modifier::DIM);
        lines.push(Line::from(vec![
            Span::raw(num_padding),
            Span::styled(SIDEBAR_CHAR, dim_sidebar_style),
            Span::raw(" "),
            Span::styled(
                name.clone(),
                Style::new().fg(*color).add_modifier(Modifier::DIM),
            ),
        ]));
    }

    // Response lines with dim sidebar
    if let Some(segments) = response_segments {
        let annotation_tick = match status {
            ExchangeStatus::Running => Some(tick_count),
            _ => None,
        };
        render_response_segments(lines, segments, base_color, annotation_tick, content_width);
    } else if let Some(resp) = response {
        render_response_text(lines, resp, base_color, content_width);
    }

    // Cancelled indicator appended after any existing response
    if status == ExchangeStatus::Cancelled {
        let dim_style = Style::new().fg(Color::DarkGray).add_modifier(Modifier::DIM);
        lines.push(Line::from(vec![
            Span::raw(num_padding),
            Span::styled(SIDEBAR_CHAR, dim_style),
        ]));
        lines.push(Line::from(vec![
            Span::raw(num_padding),
            Span::styled(SIDEBAR_CHAR, dim_style),
            Span::raw(" "),
            Span::styled("\u{2298} Cancelled", dim_style),
        ]));
    }

    // Exit code (non-zero) for shell commands
    if kind == ExchangeKind::Shell
        && let Some(code) = exit_code
        && code != 0
    {
        lines.push(Line::from(vec![
            Span::raw(num_padding),
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
    let content_width = area.width.saturating_sub(NUM_GUTTER + 2).max(1) as usize;
    let mut lines: Vec<Line> = Vec::new();
    let tick_count = app.tick_count;
    let mut exchange_num = 0usize;

    for message in &app.messages {
        // Add a blank line separator between messages (except before the first)
        if !lines.is_empty() {
            lines.push(Line::raw(""));
        }

        match message {
            AppMessage::Welcome => {
                render_welcome_lines(&mut lines, app.upgrade_available.as_deref());
            }
            AppMessage::Exchange {
                kind,
                status,
                request,
                response,
                response_segments,
                exit_code,
                agent_index,
            } => {
                exchange_num += 1;
                // Determine agent tag (name + color) for multi-agent display
                let agent_tag = if app.sessions.len() > 1 {
                    agent_index.and_then(|idx| {
                        app.sessions
                            .get(idx)
                            .map(|s| (s.name.clone(), AgentSession::color(idx)))
                    })
                } else {
                    None
                };
                render_exchange_lines(
                    &mut lines,
                    exchange_num,
                    *kind,
                    *status,
                    request,
                    response.as_deref(),
                    response_segments.as_deref(),
                    *exit_code,
                    tick_count,
                    content_width,
                    agent_tag.as_ref(),
                );
            }
            AppMessage::System { content } => {
                for text_line in content.lines() {
                    for chunk in wrap_content(text_line, content_width) {
                        lines.push(Line::from(vec![
                            Span::raw("   "),
                            Span::styled(SIDEBAR_CHAR, dim()),
                            Span::raw(" "),
                            Span::styled(chunk, dim()),
                        ]));
                    }
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

/// Render the input area with cursor: dark grey background, colored sidebar, no border.
#[allow(clippy::too_many_lines)]
fn render_input(frame: &mut Frame, app: &App, area: Rect) {
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
        let indicator = match app.mode {
            AppMode::Chat => " > ",
            AppMode::Shell => " $ ",
        };
        let indicator_line = Line::from(Span::styled(indicator, Style::new().fg(bar_color)));
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

    // Build a Text with explicit newlines so multiline input renders correctly.
    // Ghost text is appended to the last line only.
    let input_lines: Vec<&str> = if input_text.is_empty() {
        vec![""]
    } else {
        input_text.split('\n').collect()
    };
    let content = if let Some(ghost) = &app.ghost_suggestion {
        let mut text_lines: Vec<Line> = Vec::with_capacity(input_lines.len());
        let last_idx = input_lines.len() - 1;
        for (i, line) in input_lines.iter().enumerate() {
            if i == last_idx {
                let mut spans = vec![Span::raw((*line).to_string())];
                spans.push(Span::styled(ghost.as_str(), dim_style));
                if app.ghost_is_truncated {
                    spans.push(Span::styled("\u{2026}", dim_style));
                }
                text_lines.push(Line::from(spans));
            } else {
                text_lines.push(Line::from((*line).to_string()));
            }
        }
        Text::from(text_lines)
    } else {
        Text::from(
            input_lines
                .iter()
                .map(|l| Line::from((*l).to_string()))
                .collect::<Vec<_>>(),
        )
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

/// Render the responses autocomplete popup floating above the input area.
fn render_responses_autocomplete(frame: &mut Frame, app: &App, input_area: Rect) {
    let candidates = app.responses_state.candidates();
    let Some(area) = popup_area(input_area, candidates.len()) else {
        return;
    };

    // Available width inside the popup (minus borders)
    let inner_width = area.width.saturating_sub(2) as usize;
    let selected = app.responses_state.selected();

    let lines: Vec<Line> = candidates
        .iter()
        .enumerate()
        .map(|(i, candidate)| {
            let num_label = format!(" {}  {}  ", candidate.number, candidate.label);
            let num_label_len = num_label.len();
            // Truncate preview to fit remaining width
            let avail = inner_width.saturating_sub(num_label_len);
            let preview = if candidate.preview.len() > avail {
                let truncated: String = candidate
                    .preview
                    .chars()
                    .take(avail.saturating_sub(1))
                    .collect();
                format!("{truncated}\u{2026}")
            } else {
                candidate.preview.clone()
            };

            let color_style = if i == selected {
                Style::new()
                    .fg(candidate.color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(candidate.color)
            };
            let preview_style = if i == selected {
                selected_secondary_style()
            } else {
                dim()
            };

            Line::from(vec![
                Span::styled(num_label, color_style),
                Span::styled(preview, preview_style),
            ])
        })
        .collect();

    render_popup(frame, area, lines, Some(" Responses "));
}

/// Render the cancel picker popup floating above the input area.
fn render_cancel_autocomplete(frame: &mut Frame, app: &App, input_area: Rect) {
    let candidates = app.cancel_state.candidates();
    let Some(area) = popup_area(input_area, candidates.len()) else {
        return;
    };

    let selected = app.cancel_state.selected();
    let lines: Vec<Line> = candidates
        .iter()
        .enumerate()
        .map(|(i, candidate)| {
            let display = format!(
                " #{:<3} {}",
                candidate.exchange_num, candidate.request_preview
            );
            if i == selected {
                Line::from(Span::styled(display, selected_style()))
            } else {
                Line::from(Span::styled(display, unselected_style()))
            }
        })
        .collect();

    render_popup(frame, area, lines, Some(" Cancel "));
}

/// Render the model picker popup floating above the input area.
fn render_models_autocomplete(frame: &mut Frame, app: &App, input_area: Rect) {
    let candidates = app.models_state.candidates();
    let Some(area) = popup_area(input_area, candidates.len()) else {
        return;
    };

    let max_id_width = candidates
        .iter()
        .map(|c| c.model_id.len())
        .max()
        .unwrap_or(0);
    let selected = app.models_state.selected();

    let lines: Vec<Line> = candidates
        .iter()
        .enumerate()
        .map(|(i, candidate)| {
            let id_col = format!(" {:<max_id_width$}  ", candidate.model_id);
            let provider_col = candidate.provider.clone();

            if i == selected {
                Line::from(vec![
                    Span::styled(id_col, selected_style()),
                    Span::styled(provider_col, selected_secondary_style()),
                ])
            } else {
                Line::from(vec![
                    Span::styled(id_col, unselected_style()),
                    Span::styled(provider_col, dim()),
                ])
            }
        })
        .collect();

    render_popup(frame, area, lines, Some(" Model "));
}

/// Render the agent picker popup floating above the input area.
fn render_agents_autocomplete(frame: &mut Frame, app: &App, input_area: Rect) {
    let candidates = app.agents_state.candidates();
    let Some(area) = popup_area(input_area, candidates.len()) else {
        return;
    };

    let max_name_width = candidates.iter().map(|c| c.name.len()).max().unwrap_or(0);
    let selected = app.agents_state.selected();

    let lines: Vec<Line> = candidates
        .iter()
        .enumerate()
        .map(|(i, candidate)| {
            if candidate.is_new {
                // "New agent" entry rendered in dim/grey style
                let label = format!(" {:<max_name_width$}", candidate.name);
                if i == selected {
                    Line::from(Span::styled(
                        label,
                        Style::new()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else {
                    Line::from(Span::styled(label, dim()))
                }
            } else {
                let bullet = if candidate.is_active {
                    "\u{25cf} "
                } else {
                    "  "
                };
                let name_col = format!("{bullet}{:<max_name_width$}", candidate.name);
                let color = AgentSession::color(candidate.index);

                if i == selected {
                    Line::from(Span::styled(
                        format!(" {name_col}"),
                        Style::new().fg(color).add_modifier(Modifier::BOLD),
                    ))
                } else {
                    Line::from(Span::styled(format!(" {name_col}"), Style::new().fg(color)))
                }
            }
        })
        .collect();

    render_popup(frame, area, lines, Some(" Agents "));
}

/// Render the new-agent wizard as a centered modal dialog.
#[allow(clippy::too_many_lines)]
fn render_new_agent_modal(frame: &mut Frame, app: &App, area: Rect) {
    let Some(wizard) = &app.wizard else { return };

    // Modal size: roughly centered, 60 wide, 12 tall (or smaller if terminal is small)
    let modal_width = area.width.clamp(20, 60);
    let modal_height = area.height.clamp(6, 14);
    let modal_area = Rect {
        x: area.x + (area.width.saturating_sub(modal_width)) / 2,
        y: area.y + (area.height.saturating_sub(modal_height)) / 2,
        width: modal_width,
        height: modal_height,
    };
    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::Magenta))
        .title(" New Agent ");
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    // Layout inside the modal: lines of content
    let mut lines: Vec<Line> = Vec::new();
    let inner_width = inner.width.saturating_sub(2) as usize; // padding

    // Step indicators: Name > Prompt > Model
    let step_line = {
        let (name_style, prompt_style, model_style) = match wizard.step {
            WizardStep::Name => (
                Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                dim(),
                dim(),
            ),
            WizardStep::SystemPrompt => (
                Style::new().fg(Color::Green),
                Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                dim(),
            ),
            WizardStep::Model => (
                Style::new().fg(Color::Green),
                Style::new().fg(Color::Green),
                Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            ),
        };
        Line::from(vec![
            Span::styled(" Name", name_style),
            Span::styled(" \u{203a} ", dim()),
            Span::styled("Prompt", prompt_style),
            Span::styled(" \u{203a} ", dim()),
            Span::styled("Model", model_style),
        ])
    };
    lines.push(step_line);
    lines.push(Line::raw(""));

    // Current step label and input value
    match wizard.step {
        WizardStep::Name => {
            lines.push(Line::from(vec![
                Span::styled(" Name: ", Style::new().fg(Color::Magenta)),
                Span::raw(app.input.text().to_string()),
            ]));
            lines.push(Line::raw(""));
            lines.push(Line::styled(" A short name for this agent", dim()));
        }
        WizardStep::SystemPrompt => {
            lines.push(Line::from(Span::styled(
                " System prompt:",
                Style::new().fg(Color::Magenta),
            )));
            // Show the input text (potentially multiline)
            let input_text = app.input.text();
            if input_text.is_empty() {
                lines.push(Line::styled(" (optional, press Enter to skip)", dim()));
            } else {
                for text_line in input_text.split('\n') {
                    let chunks = wrap_content(text_line, inner_width.saturating_sub(1));
                    for chunk in chunks {
                        lines.push(Line::from(Span::raw(format!(" {chunk}"))));
                    }
                }
            }
        }
        WizardStep::Model => {
            lines.push(Line::from(Span::styled(
                " Model:",
                Style::new().fg(Color::Magenta),
            )));
            if app.models_state.is_visible() {
                lines.push(Line::from(vec![
                    Span::styled(" Filter: ", dim()),
                    Span::raw(app.input.text().to_string()),
                ]));
            } else {
                lines.push(Line::styled(" Press Enter to use default model", dim()));
            }
        }
    }

    // Pad to fill modal, then add hint at the bottom
    let content_lines = lines.len();
    #[allow(clippy::cast_possible_truncation)]
    let available = inner.height as usize;
    let hint_lines = 1;
    let padding = available.saturating_sub(content_lines + hint_lines);
    for _ in 0..padding {
        lines.push(Line::raw(""));
    }

    // Hint line at the bottom
    let hint = match wizard.step {
        WizardStep::Name => Line::from(vec![
            Span::raw(" \u{21b5} "),
            Span::styled("next", dim()),
            Span::raw("  esc "),
            Span::styled("cancel", dim()),
        ]),
        WizardStep::SystemPrompt => Line::from(vec![
            Span::raw(" \u{21b5} "),
            Span::styled("next", dim()),
            Span::raw("  alt+\u{21b5} "),
            Span::styled("newline", dim()),
            Span::raw("  esc "),
            Span::styled("cancel", dim()),
        ]),
        WizardStep::Model => Line::from(vec![
            Span::raw(" \u{21b5} "),
            Span::styled("select", dim()),
            Span::raw("  esc "),
            Span::styled("cancel", dim()),
        ]),
    };
    lines.push(hint);

    let paragraph = Paragraph::new(Text::from(lines));
    frame.render_widget(paragraph, inner);

    // Render model picker inside the modal if visible during Model step
    if wizard.step == WizardStep::Model && app.models_state.is_visible() {
        // Place the model picker popup within the modal area
        let candidates = app.models_state.candidates();
        #[allow(clippy::cast_possible_truncation)]
        let picker_height = (candidates.len() as u16 + 2).min(inner.height.saturating_sub(4));
        if picker_height >= 3 {
            let picker_area = Rect {
                x: inner.x,
                y: inner.y + inner.height.saturating_sub(picker_height + 1),
                width: inner.width,
                height: picker_height,
            };
            render_models_autocomplete_in_area(frame, app, picker_area);
        }
    }

    // Position the cursor inside the modal for Name / SystemPrompt / Model filter
    if wizard.step != WizardStep::Model || app.models_state.is_visible() {
        let cursor_offset = match wizard.step {
            WizardStep::Name => " Name: ".len(),
            WizardStep::SystemPrompt => 1, // " " prefix
            WizardStep::Model => " Filter: ".len(),
        };

        // For SystemPrompt, cursor may be on a later line
        if wizard.step == WizardStep::SystemPrompt {
            let input_text = app.input.text();
            let line_count = input_text.split('\n').count();
            let last_line = input_text.split('\n').next_back().unwrap_or("");
            #[allow(clippy::cast_possible_truncation)]
            let cursor_x = inner.x + cursor_offset as u16 + last_line.len() as u16;
            // +2 for step indicator line + blank line, then lines of prompt
            #[allow(clippy::cast_possible_truncation)]
            let cursor_y = inner.y + 2 + (line_count.saturating_sub(1)) as u16;
            if cursor_x < modal_area.x + modal_area.width
                && cursor_y < modal_area.y + modal_area.height
            {
                frame.set_cursor_position(Position::new(cursor_x, cursor_y));
            }
        } else {
            #[allow(clippy::cast_possible_truncation)]
            let cursor_x = inner.x + cursor_offset as u16 + app.input.text().len() as u16;
            let cursor_y = inner.y + 2; // after step indicator + blank line
            if cursor_x < modal_area.x + modal_area.width
                && cursor_y < modal_area.y + modal_area.height
            {
                frame.set_cursor_position(Position::new(cursor_x, cursor_y));
            }
        }
    }
}

/// Render the model picker popup within a given area (used inside wizard modal).
fn render_models_autocomplete_in_area(frame: &mut Frame, app: &App, area: Rect) {
    let candidates = app.models_state.candidates();
    if candidates.is_empty() {
        return;
    }

    let max_id_width = candidates
        .iter()
        .map(|c| c.model_id.len())
        .max()
        .unwrap_or(0);
    let selected = app.models_state.selected();

    let lines: Vec<Line> = candidates
        .iter()
        .enumerate()
        .map(|(i, candidate)| {
            let id_col = format!(" {:<max_id_width$}  ", candidate.model_id);
            let provider_col = candidate.provider.clone();

            if i == selected {
                Line::from(vec![
                    Span::styled(id_col, selected_style()),
                    Span::styled(provider_col, selected_secondary_style()),
                ])
            } else {
                Line::from(vec![
                    Span::styled(id_col, unselected_style()),
                    Span::styled(provider_col, dim()),
                ])
            }
        })
        .collect();

    render_popup(frame, area, lines, Some(" Model "));
}

/// Split text into chunks that fit within `width` characters, for manual
/// line wrapping that preserves the gutter/sidebar prefix on each visual line.
fn wrap_content(text: &str, width: usize) -> Vec<String> {
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
