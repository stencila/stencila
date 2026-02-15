use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
};

use crate::agent::{ResponseSegment, ToolCallStatus, truncate_for_display};
use crate::app::{AgentSession, App, AppMessage, ExchangeKind, ExchangeStatus};

use super::common::{
    NUM_GUTTER, SIDEBAR_CHAR, THINKING_FRAMES, TOOL_CALL_FRAMES, dim, wrap_content,
};

/// Render the scrollable message area.
pub(super) fn render(frame: &mut Frame, app: &mut App, area: Rect) {
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
                welcome_lines(&mut lines, app.upgrade_available.as_deref());
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
                exchange_lines(
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

    // Calculate scroll (top-line position).
    // When pinned, always show the bottom. When unpinned, use the stored
    // absolute top-line position (clamped to valid range).
    let max_top = total_lines.saturating_sub(visible_height);
    let scroll = if app.scroll_pinned {
        max_top
    } else {
        app.scroll_offset.min(max_top)
    };

    let paragraph = Paragraph::new(Text::from(lines))
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Append lines for a welcome message.
fn welcome_lines(lines: &mut Vec<Line>, upgrade_available: Option<&str>) {
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

/// Append lines for an exchange (request/response with sidebar).
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn exchange_lines(
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
                .style(Style::new().bg(super::common::INPUT_BG)),
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
        response_segments_lines(lines, segments, base_color, annotation_tick, content_width);
    } else if let Some(resp) = response {
        response_text(lines, resp, base_color, content_width);
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

/// Render structured response segments with interleaved tool annotations.
fn response_segments_lines(
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
                // Skip empty text segments so they don't introduce extra
                // blank lines between consecutive annotations.
                if text.trim().is_empty() {
                    continue;
                }
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
                // Truncate label to fit on a single line (avail = content_width - "● " prefix)
                let avail = content_width.saturating_sub(2);
                let display_label = truncate_for_display(label, avail);
                push_annotation_lines(
                    lines,
                    dim_sidebar_style,
                    symbol,
                    symbol_style,
                    &display_label,
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
                thinking_segment(
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
                Some(t) => TOOL_CALL_FRAMES[(t as usize / 2) % TOOL_CALL_FRAMES.len()],
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

fn thinking_symbol(tick: Option<u32>) -> (char, Style) {
    let sym = match tick {
        Some(t) => THINKING_FRAMES[(t as usize / 2) % THINKING_FRAMES.len()],
        None => THINKING_FRAMES[0],
    };
    (sym, Style::new().fg(Color::DarkGray))
}

/// Render a thinking/reasoning segment: header + indented content with
/// visual-line-based truncation.
#[allow(clippy::too_many_arguments)]
fn thinking_segment(
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

/// Render plain text response lines (for shell commands and fallback).
fn response_text(lines: &mut Vec<Line>, resp: &str, base_color: Color, content_width: usize) {
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
