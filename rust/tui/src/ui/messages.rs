use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
};

use crate::agent::{ResponseSegment, ToolCallStatus, truncate_for_display};
use crate::app::{
    App, AppMessage, ExchangeKind, ExchangeStatus, WorkflowProgressKind, WorkflowStatusState,
};

use super::common::{
    BRAILLE_SPINNER_FRAMES, DelimiterDisplay, InlineStyleMode, NUM_GUTTER, SIDEBAR_CHAR,
    THINKING_FRAMES, TOOL_CALL_FRAMES, dim, style_inline_markdown, wrap_content,
};
use super::markdown::MdRenderCache;

/// Render the scrollable message area.
#[allow(clippy::too_many_lines)]
pub(super) fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let content_width = area.width.saturating_sub(NUM_GUTTER + 2).max(1) as usize;
    let mut lines: Vec<Line> = Vec::new();
    let tick_count = app.tick_count;
    let mut exchange_num = 0usize;

    let md_cache = &mut app.md_render_cache;
    for (msg_idx, message) in app.messages.iter().enumerate() {
        // Add a blank line separator between messages (except before the first)
        if !lines.is_empty() {
            lines.push(Line::raw(""));
        }

        match message {
            AppMessage::Welcome => {
                welcome_lines(&mut lines, app.upgrade_available.as_deref());
            }
            AppMessage::SitePreviewReady { url } => {
                lines.push(Line::from(vec![
                    Span::styled(" ● ", Color::Blue),
                    Span::styled(SIDEBAR_CHAR, Color::Blue),
                    Span::raw(" Site preview ready at: "),
                ]));
                lines.push(Line::from(vec![
                    Span::raw("   "),
                    Span::styled(SIDEBAR_CHAR, Color::Blue),
                    Span::raw(" "),
                    Span::styled(url, Color::Blue),
                ]));
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
            AppMessage::Exchange {
                kind,
                status,
                request,
                response,
                response_segments,
                exit_code,
                agent_index,
                agent_name: workflow_agent_name,
            } => {
                exchange_num += 1;
                let agent_tag = if let Some(name) = workflow_agent_name {
                    let color = app.color_registry.get(name).unwrap_or(Color::DarkGray);
                    Some((name.clone(), color))
                } else {
                    agent_index.and_then(|idx| {
                        app.sessions.get(idx).map(|s| {
                            let color = app.color_registry.get(&s.name).unwrap_or(Color::Blue);
                            (s.name.clone(), color)
                        })
                    })
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
                    msg_idx,
                    md_cache,
                );
            }
            AppMessage::WorkflowStatus {
                state,
                label,
                detail,
            } => {
                workflow_status_lines(
                    &mut lines,
                    *state,
                    label,
                    detail.as_deref(),
                    tick_count,
                    content_width,
                );
            }
            AppMessage::WorkflowProgress {
                kind,
                label,
                detail,
            } => {
                workflow_progress_lines(
                    &mut lines,
                    *kind,
                    label,
                    detail.as_deref(),
                    tick_count,
                    content_width,
                );
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
    msg_idx: usize,
    md_cache: &mut MdRenderCache,
) {
    let kind_color = agent_tag.map_or_else(|| kind.color(), |(_, color)| *color);
    let base_color = if kind == ExchangeKind::Workflow {
        // Workflow exchanges start grey (agent unknown) and adopt the
        // agent's registered color once StageInput has been received.
        match status {
            ExchangeStatus::Running | ExchangeStatus::Succeeded => {
                agent_tag.map_or(Color::DarkGray, |(_, color)| *color)
            }
            ExchangeStatus::Failed => Color::Red,
            ExchangeStatus::Cancelled => Color::DarkGray,
        }
    } else {
        match status {
            ExchangeStatus::Running | ExchangeStatus::Succeeded => kind_color,
            ExchangeStatus::Failed => Color::Red,
            ExchangeStatus::Cancelled => Color::DarkGray,
        }
    };

    let sidebar_style = Style::new().fg(base_color);

    // Shell commands are prefixed with "$ "
    let prefix = if kind == ExchangeKind::Shell {
        "$ "
    } else {
        ""
    };

    let num_style = Style::new().fg(base_color);
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
            let mut line_spans = vec![
                num_col,
                Span::styled(SIDEBAR_CHAR, sidebar_style),
                Span::raw(" "),
            ];
            line_spans.extend(style_inline_markdown(
                &chunk,
                InlineStyleMode::Normal,
                DelimiterDisplay::Hide,
            ));
            lines.push(Line::from(line_spans).style(Style::new().bg(super::common::INPUT_BG)));
        }
    }

    // Agent name tag after the request.
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
        response_segments_lines(
            lines,
            segments,
            base_color,
            annotation_tick,
            content_width,
            msg_idx,
            md_cache,
        );
    } else if let Some(resp) = response {
        response_text(lines, resp, base_color, content_width);
    }

    // For running exchanges, show spinner in the gutter of the last line so it
    // stays close to where new content is appearing without adding an extra line.
    if status == ExchangeStatus::Running {
        let frame_idx = (tick_count as usize / 2) % BRAILLE_SPINNER_FRAMES.len();
        let spinner_color = agent_tag.map_or(base_color, |(_, color)| *color);
        let spinner_span = Span::styled(
            format!(" {} ", BRAILLE_SPINNER_FRAMES[frame_idx]),
            Style::new().fg(spinner_color),
        );
        if let Some(last_line) = lines.last_mut() {
            let mut spans: Vec<Span> = last_line.spans.clone();
            if !spans.is_empty() {
                spans[0] = spinner_span;
            }
            *last_line = Line::from(spans).style(last_line.style);
        } else {
            let dim_sidebar_style = Style::new().fg(base_color).add_modifier(Modifier::DIM);
            lines.push(Line::from(vec![
                spinner_span,
                Span::styled(SIDEBAR_CHAR, dim_sidebar_style),
            ]));
        }
    }

    // Cancelled indicator appended after any existing response (skip for workflow
    // exchanges since their cancellation is shown via a dedicated system message).
    if status == ExchangeStatus::Cancelled && kind != ExchangeKind::Workflow {
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
#[allow(clippy::too_many_arguments)]
fn response_segments_lines(
    lines: &mut Vec<Line>,
    segments: &[ResponseSegment],
    base_color: Color,
    annotation_tick: Option<u32>,
    content_width: usize,
    msg_idx: usize,
    md_cache: &mut MdRenderCache,
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

    for (seg_idx, segment) in segments.iter().enumerate() {
        match segment {
            ResponseSegment::Text(text) => {
                // Skip truly empty text segments (only horizontal whitespace)
                // so they don't introduce extra blank lines between consecutive
                // annotations. Segments containing newlines are preserved since
                // they may represent intentional paragraph breaks.
                if text.chars().all(|c| c == ' ' || c == '\t') {
                    continue;
                }
                if prev_was_annotation {
                    lines.push(blank_line());
                }
                let content_spans = md_cache.get_or_render(msg_idx, seg_idx, text, content_width);
                for span_line in content_spans {
                    let mut line_spans = vec![
                        Span::raw(num_padding),
                        Span::styled(SIDEBAR_CHAR, dim_sidebar_style),
                        Span::raw(" "),
                    ];
                    line_spans.extend(span_line.iter().cloned());
                    lines.push(Line::from(line_spans));
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
                    for (i, detail_line) in detail.lines().enumerate() {
                        let sym = if i == 0 { '\u{21b3}' } else { ' ' };
                        push_annotation_lines(
                            lines,
                            dim_sidebar_style,
                            sym,
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
                let symbol_color = if message.trim_start().starts_with("Error:") {
                    Color::Red
                } else {
                    Color::Yellow
                };
                push_annotation_lines(
                    lines,
                    dim_sidebar_style,
                    '\u{25cf}',
                    Style::new().fg(symbol_color),
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
            (sym, Style::new().fg(Color::Gray))
        }
        ToolCallStatus::Done => ('\u{25cf}', Style::new().fg(Color::LightGreen)),
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
#[allow(clippy::too_many_lines)]
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
        ('\u{2731}', Style::new().fg(Color::DarkGray))
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
    // When complete: show first N lines (head) with "[+N more lines]".
    // When streaming: show last N lines (tail) so user sees latest thinking.
    if !text.is_empty() {
        let max_visual = 5;
        let avail = content_width.saturating_sub(2); // prefix width

        // Pre-compute visual line counts per source line for tail mode
        let source_lines: Vec<&str> = text.lines().collect();
        let wrapped_counts: Vec<usize> = source_lines
            .iter()
            .map(|l| wrap_content(l, avail).len())
            .collect();
        let total_visual: usize = wrapped_counts.iter().sum();

        if complete {
            // Head mode: show first max_visual lines
            let mut visual_count = 0usize;
            let mut first_source_line = true;
            let mut remaining_visual = 0usize;
            let mut truncated = false;

            for (source_line, &wc) in source_lines.iter().zip(&wrapped_counts) {
                if truncated {
                    remaining_visual += wc;
                    continue;
                }
                if visual_count + wc > max_visual && visual_count > 0 {
                    truncated = true;
                    remaining_visual += wc;
                    continue;
                }
                let prefix = if first_source_line { '\u{21b3}' } else { ' ' };
                push_annotation_lines_inner(
                    lines,
                    dim_sidebar_style,
                    prefix,
                    Style::new().fg(Color::DarkGray),
                    source_line,
                    dim(),
                    content_width,
                    Some(InlineStyleMode::Muted),
                );
                visual_count += wc;
                first_source_line = false;
            }
            if remaining_visual > 0 {
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
        } else {
            // Tail mode: show last max_visual lines so user sees latest thinking
            let skipped_visual = total_visual.saturating_sub(max_visual);

            // Find first source line to include by skipping visual lines
            let mut skip_remaining = skipped_visual;
            let mut start_idx = 0;
            for (i, &wc) in wrapped_counts.iter().enumerate() {
                if skip_remaining == 0 {
                    start_idx = i;
                    break;
                }
                if wc <= skip_remaining {
                    skip_remaining -= wc;
                } else {
                    start_idx = i;
                    break;
                }
            }

            if skipped_visual > 0 {
                push_annotation_lines(
                    lines,
                    dim_sidebar_style,
                    '\u{21b3}',
                    Style::new().fg(Color::DarkGray),
                    &format!("[{skipped_visual} lines…]"),
                    dim(),
                    content_width,
                );
            }

            for (i, source_line) in source_lines[start_idx..].iter().enumerate() {
                let prefix = if skipped_visual == 0 && i == 0 {
                    '\u{21b3}'
                } else {
                    ' '
                };
                push_annotation_lines_inner(
                    lines,
                    dim_sidebar_style,
                    prefix,
                    Style::new().fg(Color::DarkGray),
                    source_line,
                    dim(),
                    content_width,
                    Some(InlineStyleMode::Muted),
                );
            }
        }
    }
}

/// Render plain text response lines (for shell commands and fallback).
fn response_text(lines: &mut Vec<Line>, resp: &str, base_color: Color, content_width: usize) {
    let num_padding = "   ";
    let dim_sidebar_style = Style::new().fg(base_color).add_modifier(Modifier::DIM);
    for text_line in resp.lines() {
        let mut carry_style = Style::default();
        for chunk in wrap_content(text_line, content_width) {
            let mut line_spans = vec![
                Span::raw(num_padding),
                Span::styled(SIDEBAR_CHAR, dim_sidebar_style),
                Span::raw(" "),
            ];
            let (ansi_spans, next_style) = super::common::parse_ansi_spans(&chunk, carry_style);
            carry_style = next_style;
            line_spans.extend(ansi_spans);
            lines.push(Line::from(line_spans));
        }
    }
}

/// Workflow-specific color palette.
const WORKFLOW_COLOR: Color = Color::Rgb(0, 180, 160);

/// Render an in-place-updatable workflow status line styled like an exchange.
#[allow(clippy::too_many_arguments)]
fn workflow_status_lines(
    lines: &mut Vec<Line>,
    state: WorkflowStatusState,
    label: &str,
    detail: Option<&str>,
    tick_count: u32,
    content_width: usize,
) {
    let gutter_text = match state {
        WorkflowStatusState::Running => {
            let frame_idx = (tick_count as usize / 2) % BRAILLE_SPINNER_FRAMES.len();
            format!(" {} ", BRAILLE_SPINNER_FRAMES[frame_idx])
        }
        WorkflowStatusState::Completed => " ⚙ ".to_string(),
        WorkflowStatusState::Failed => " ! ".to_string(),
        WorkflowStatusState::Cancelled => " ⊘ ".to_string(),
    };
    let gutter_style = Style::new().fg(WORKFLOW_COLOR);

    let sidebar_style = Style::new().fg(WORKFLOW_COLOR);
    let dim_sidebar_style = Style::new().fg(WORKFLOW_COLOR).add_modifier(Modifier::DIM);

    lines.push(Line::from(vec![
        Span::styled(gutter_text.clone(), gutter_style),
        Span::styled(SIDEBAR_CHAR, sidebar_style),
        Span::raw(" "),
        Span::styled(label.to_string(), Style::new().fg(Color::White)),
    ]));

    if let Some(detail_text) = detail {
        let detail_style = match state {
            WorkflowStatusState::Failed => Style::new().fg(Color::Red),
            _ => dim(),
        };
        for text_line in detail_text.lines() {
            for chunk in wrap_content(text_line, content_width) {
                lines.push(Line::from(vec![
                    Span::raw("   "),
                    Span::styled(SIDEBAR_CHAR, dim_sidebar_style),
                    Span::raw(" "),
                    Span::styled(chunk, detail_style),
                ]));
            }
        }
    }
}

/// Render workflow progress lines
fn workflow_progress_lines(
    lines: &mut Vec<Line>,
    kind: WorkflowProgressKind,
    label: &str,
    detail: Option<&str>,
    tick_count: u32,
    content_width: usize,
) {
    let gutter_text = match kind {
        WorkflowProgressKind::Running => {
            let frame_idx = (tick_count as usize / 2) % BRAILLE_SPINNER_FRAMES.len();
            format!(" {} ", BRAILLE_SPINNER_FRAMES[frame_idx])
        }
        WorkflowProgressKind::Started | WorkflowProgressKind::Completed => " ⚙ ".to_string(),
        WorkflowProgressKind::Failed => " ! ".to_string(),
        WorkflowProgressKind::Cancelled => " ⊘ ".to_string(),
        WorkflowProgressKind::Retrying => "   ".to_string(),
    };
    let gutter_style = Style::new().fg(WORKFLOW_COLOR);

    let sidebar_style = Style::new().fg(WORKFLOW_COLOR);
    let dim_sidebar_style = Style::new().fg(WORKFLOW_COLOR).add_modifier(Modifier::DIM);

    lines.push(Line::from(vec![
        Span::styled(gutter_text.clone(), gutter_style),
        Span::styled(SIDEBAR_CHAR, sidebar_style),
        Span::raw(" "),
        Span::styled(label.to_string(), Style::new().fg(Color::White)),
    ]));

    if let Some(detail_text) = detail {
        let detail_style = match kind {
            WorkflowProgressKind::Failed => Style::new().fg(Color::Red),
            _ => dim(),
        };
        for text_line in detail_text.lines() {
            for chunk in wrap_content(text_line, content_width) {
                lines.push(Line::from(vec![
                    Span::raw("   "),
                    Span::styled(SIDEBAR_CHAR, dim_sidebar_style),
                    Span::raw(" "),
                    Span::styled(chunk, detail_style),
                ]));
            }
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
    push_annotation_lines_inner(
        lines,
        sidebar_style,
        symbol,
        symbol_style,
        content,
        content_style,
        content_width,
        None,
    );
}

#[allow(clippy::too_many_arguments)]
fn push_annotation_lines_inner(
    lines: &mut Vec<Line>,
    sidebar_style: Style,
    symbol: char,
    symbol_style: Style,
    content: &str,
    content_style: Style,
    content_width: usize,
    inline_md: Option<InlineStyleMode>,
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
        let mut line_spans = vec![
            Span::raw(num_padding),
            Span::styled(SIDEBAR_CHAR, sidebar_style),
            Span::raw(" "),
            prefix,
        ];
        if let Some(mode) = inline_md {
            for mut span in style_inline_markdown(chunk, mode, DelimiterDisplay::Hide) {
                span.style = content_style.patch(span.style);
                line_spans.push(span);
            }
        } else {
            line_spans.push(Span::styled(chunk.clone(), content_style));
        }
        lines.push(Line::from(line_spans));
    }
}
