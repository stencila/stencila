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
use crate::interview::{
    DraftAnswer, InterviewSource, InterviewStatus, is_answered_draft, preview_multi_select,
    preview_selection,
};

use super::common::{
    BRAILLE_SPINNER_FRAMES, DelimiterDisplay, InlineStyleMode, NUM_GUTTER, SIDEBAR_CHAR,
    SYM_CANCELLED, SYM_QUESTION_CLOSED, SYM_QUESTION_OPEN, SYM_SELECTED, SYM_UNSELECTED,
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

    for msg_idx in 0..app.messages.len() {
        let message = app.messages[msg_idx].clone();
        // Inline interviews (with a parent message) skip the blank separator
        // so they appear as a continuation of their parent.
        let is_inline_interview = matches!(
            &message,
            AppMessage::Interview {
                parent_msg_index: Some(_),
                ..
            }
        );

        // Add a blank line separator between messages (except before the first)
        if !lines.is_empty() && !is_inline_interview {
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
                    let color = app.color_registry.get(&name).unwrap_or(Color::DarkGray);
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
                    kind,
                    status,
                    &request,
                    response.as_deref(),
                    response_segments.as_deref(),
                    exit_code,
                    tick_count,
                    content_width,
                    agent_tag.as_ref(),
                    msg_idx,
                    &app.messages,
                    app.active_interview.as_ref(),
                    &app.interview_preview_input,
                    &mut app.color_registry,
                    &mut app.md_render_cache,
                );
            }
            AppMessage::WorkflowStatus {
                state,
                label,
                detail,
            } => {
                workflow_status_lines(
                    &mut lines,
                    state,
                    &label,
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
                    kind,
                    &label,
                    detail.as_deref(),
                    tick_count,
                    content_width,
                );
            }
            AppMessage::Interview {
                id,
                source,
                agent_name,
                status,
                interview,
                answers,
                parent_msg_index,
            } => {
                if matches!(source, InterviewSource::Agent) {
                    continue;
                }

                // For inline workflow interviews, inherit the parent's color
                // so interview sidebars match (e.g. grey for stages without
                // an agent like human gates).
                let parent_color = parent_msg_index.and_then(|pi| {
                    if let AppMessage::Exchange {
                        kind: ExchangeKind::Workflow,
                        agent_name: wf_agent,
                        ..
                    } = app.messages.get(pi)?
                    {
                        let color = wf_agent
                            .as_ref()
                            .and_then(|n| app.color_registry.get(n))
                            .unwrap_or(Color::DarkGray);
                        Some(color)
                    } else {
                        None
                    }
                });

                interview_lines(
                    &mut lines,
                    &id,
                    &source,
                    &agent_name,
                    status,
                    &interview,
                    &answers,
                    app.active_interview.as_ref(),
                    &app.interview_preview_input,
                    &mut app.color_registry,
                    content_width,
                    msg_idx,
                    &mut app.md_render_cache,
                    parent_msg_index.is_some(),
                    parent_color,
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
    app_messages: &[AppMessage],
    active_interview: Option<&crate::interview::InterviewState>,
    interview_preview_input: &str,
    color_registry: &mut crate::app::AgentColorRegistry,
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
            app_messages,
            active_interview,
            interview_preview_input,
            color_registry,
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
#[allow(clippy::too_many_lines)]
fn response_segments_lines(
    lines: &mut Vec<Line>,
    segments: &[ResponseSegment],
    base_color: Color,
    annotation_tick: Option<u32>,
    content_width: usize,
    msg_idx: usize,
    app_messages: &[AppMessage],
    active_interview: Option<&crate::interview::InterviewState>,
    interview_preview_input: &str,
    color_registry: &mut crate::app::AgentColorRegistry,
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
            ResponseSegment::Info(message)
            | ResponseSegment::Warning(message)
            | ResponseSegment::Error(message) => {
                let color = match segment {
                    ResponseSegment::Info(_) => Color::Blue,
                    ResponseSegment::Error(_) => Color::Red,
                    _ => Color::Yellow,
                };
                // Strip internal `LLM_RETRY:` tag used for in-place updates
                let display_message = message.strip_prefix("LLM_RETRY:").unwrap_or(message);
                if !prev_was_annotation {
                    lines.push(blank_line());
                }
                push_annotation_lines(
                    lines,
                    dim_sidebar_style,
                    '\u{25cf}',
                    Style::new().fg(color),
                    display_message,
                    dim(),
                    content_width,
                );
                prev_was_annotation = true;
            }
            ResponseSegment::Interview {
                interview_msg_index,
            } => {
                if let Some(AppMessage::Interview {
                    id,
                    source,
                    agent_name,
                    status,
                    interview,
                    answers,
                    ..
                }) = app_messages.get(*interview_msg_index)
                {
                    if !prev_was_annotation {
                        lines.push(blank_line());
                    }

                    interview_lines(
                        lines,
                        id,
                        source,
                        agent_name,
                        *status,
                        interview,
                        answers,
                        active_interview,
                        interview_preview_input,
                        color_registry,
                        content_width,
                        *interview_msg_index,
                        md_cache,
                        true,
                        Some(base_color),
                    );
                    prev_was_annotation = true;
                }
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

// ─── Interview rendering ────────────────────────────────────────────

/// Render a structured interview block inline in the message area.
///
/// When `inline` is `true`, the interview is rendered as a continuation of its
/// parent message (no header, no separator) — only questions and preamble are shown.
///
/// `parent_color` overrides the accent color when set, so inline interviews
/// inherit their parent message's sidebar color (e.g. grey for workflow stages
/// that have no agent).
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn interview_lines(
    lines: &mut Vec<Line>,
    id: &str,
    source: &InterviewSource,
    agent_name: &str,
    status: InterviewStatus,
    interview: &stencila_attractor::interviewer::Interview,
    answers: &[stencila_attractor::interviewer::Answer],
    active_state: Option<&crate::interview::InterviewState>,
    preview_input: &str,
    color_registry: &mut crate::app::AgentColorRegistry,
    content_width: usize,
    msg_idx: usize,
    md_cache: &mut MdRenderCache,
    inline: bool,
    parent_color: Option<Color>,
) {
    // Determine accent color: inherit from parent when available, otherwise
    // derive from the interview source.
    let accent_color = parent_color.unwrap_or_else(|| match source {
        InterviewSource::Agent => color_registry.color_for(agent_name),
        InterviewSource::Workflow => WORKFLOW_COLOR,
    });

    let sidebar_style = Style::new().fg(accent_color).add_modifier(Modifier::DIM);

    let num_padding = "   ";

    // Deduct the gutter + sidebar + space prefix from content width for wrapping.
    // Prefix is: num_padding (3) + SIDEBAR_CHAR (1) + space (1) = 5 chars.
    let inner_width = content_width.saturating_sub(5);

    // Determine which question is active (only if this interview is the active one)
    let active_state = active_state.filter(|state| state.interview_id == id);

    if inline {
        // Inline mode: no header, just a blank sidebar line to separate from
        // the parent message content, then questions directly.
        blank_sidebar_line(lines, sidebar_style);
    } else {
        // Standalone mode: full header with gutter indicator and agent name.
        let gutter_text = match status {
            InterviewStatus::Active | InterviewStatus::Completed => " ? ".to_string(),
            InterviewStatus::Cancelled => format!(" {SYM_CANCELLED} "),
        };
        let gutter_style = Style::new().fg(accent_color).add_modifier(Modifier::BOLD);

        let status_suffix = match status {
            InterviewStatus::Active => " asks".to_string(),
            InterviewStatus::Completed => " \u{00b7} completed".to_string(),
            InterviewStatus::Cancelled => {
                let answered = active_state.map_or(0, |state| {
                    state
                        .draft_answers
                        .iter()
                        .filter(|draft| is_answered_draft(draft))
                        .count()
                });
                format!(
                    " \u{00b7} cancelled ({answered}/{})",
                    interview.questions.len()
                )
            }
        };
        let header_spans = vec![
            Span::styled(gutter_text.to_string(), gutter_style),
            Span::styled(SIDEBAR_CHAR, sidebar_style),
            Span::raw(" "),
            Span::styled(
                agent_name.to_string(),
                Style::new().fg(accent_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(status_suffix, dim()),
        ];
        lines.push(Line::from(header_spans));

        // Add a blank line between the interview header and preamble/questions.
        blank_sidebar_line(lines, sidebar_style);
    }

    let active_question = active_state.map(|s| s.current_question);

    let questions = &interview.questions;

    let has_preamble = interview
        .preamble
        .as_ref()
        .is_some_and(|preamble| !preamble.is_empty());

    // Preamble (if present and non-empty) — deviation from plan: keep visible in all states.
    if let Some(preamble) = &interview.preamble
        && !preamble.is_empty()
    {
        // Render preamble through markdown cache
        let preamble_spans = md_cache.get_or_render(msg_idx, usize::MAX, preamble, inner_width);
        for line_spans in preamble_spans {
            let mut spans = vec![
                Span::raw(num_padding),
                Span::styled(SIDEBAR_CHAR, sidebar_style),
                Span::raw(" "),
            ];
            spans.extend(line_spans.iter().cloned());
            lines.push(Line::from(spans));
        }
    }

    // Blank line between preamble and questions.
    if has_preamble {
        blank_sidebar_line(lines, sidebar_style);
    }

    let max_future = if questions.len() > 3 { 1 } else { usize::MAX };
    let mut future_rendered = 0usize;
    for (q_idx, question) in questions.iter().enumerate() {
        let is_active = active_question == Some(q_idx);
        let is_answered = if status == InterviewStatus::Completed {
            true
        } else {
            active_question.is_some_and(|aq| q_idx < aq)
        };

        if is_answered {
            answered_question_line(
                lines,
                question,
                status,
                answers.get(q_idx),
                active_state.and_then(|s| s.draft_answers.get(q_idx)),
                sidebar_style,
                inner_width,
            );
        } else if is_active {
            if questions.len() > 1 {
                // Horizontal rule above active question
                rule_sidebar_line(lines, sidebar_style, inner_width);
            }
            active_question_lines(
                lines,
                question,
                active_state.and_then(|s| s.draft_answers.get(q_idx)),
                active_state.map(|_| q_idx + 1),
                questions.len(),
                active_state,
                preview_input,
                sidebar_style,
                inner_width,
                msg_idx,
                q_idx,
                md_cache,
            );
            if questions.len() > 1 {
                // Horizontal rule below active question
                rule_sidebar_line(lines, sidebar_style, inner_width);
            }
        } else if future_rendered < max_future {
            future_question_line(lines, question, sidebar_style);
            future_rendered += 1;
        } else if future_rendered == max_future {
            let remaining = questions.len().saturating_sub(q_idx);
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(SIDEBAR_CHAR, sidebar_style),
                Span::raw(" "),
                Span::styled(format!("… {remaining} more questions"), dim()),
            ]));
            break;
        }
    }
}

/// Push a blank sidebar continuation line.
fn blank_sidebar_line(lines: &mut Vec<Line>, sidebar_style: Style) {
    lines.push(Line::from(vec![
        Span::raw("   "),
        Span::styled(SIDEBAR_CHAR, sidebar_style),
    ]));
}

/// Push a dim horizontal rule within the sidebar continuation area.
fn rule_sidebar_line(lines: &mut Vec<Line>, sidebar_style: Style, inner_width: usize) {
    lines.push(Line::from(vec![
        Span::raw("   "),
        Span::styled(SIDEBAR_CHAR, sidebar_style),
        Span::raw(" "),
        Span::styled(
            "─".repeat(inner_width.max(1)),
            Style::new().add_modifier(Modifier::DIM),
        ),
    ]));
}

/// Render the expanded active-question block: header/text, options, and yes/no toggles.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn active_question_lines(
    lines: &mut Vec<Line>,
    question: &stencila_attractor::interviewer::Question,
    draft: Option<&DraftAnswer>,
    progress: Option<usize>,
    total_questions: usize,
    active_state: Option<&crate::interview::InterviewState>,
    preview_input: &str,
    sidebar_style: Style,
    inner_width: usize,
    _msg_idx: usize,
    _q_idx: usize,
    _md_cache: &mut MdRenderCache,
) {
    use stencila_attractor::interviewer::QuestionType;

    let num_padding = "   ";

    // Active question symbol
    let sym_span = Span::styled(
        format!("{SYM_QUESTION_OPEN} "),
        Style::new().fg(Color::White),
    );

    let header_text = question
        .header
        .as_deref()
        .unwrap_or(&question.text)
        .to_string();

    let q_lines = wrap_content(&question.text, inner_width);
    if question.header.is_some() {
        // Show header in bold with ○ symbol, then question text below
        if let Some(chunk) = wrap_content(&header_text, inner_width).into_iter().next() {
            let mut line_spans = vec![
                Span::raw(num_padding),
                Span::styled(SIDEBAR_CHAR, sidebar_style),
                Span::raw(" "),
            ];
            line_spans.push(sym_span.clone());
            line_spans.push(Span::styled(
                chunk,
                Style::new().add_modifier(Modifier::BOLD),
            ));
            if let Some(progress) = progress
                && total_questions > 1
            {
                line_spans.push(Span::styled(
                    format!("  {progress}/{total_questions}"),
                    dim(),
                ));
            }
            lines.push(Line::from(line_spans));
        }
        // Blank line after header
        blank_sidebar_line(lines, sidebar_style);
        for chunk in q_lines {
            let mut spans = vec![
                Span::raw(num_padding),
                Span::styled(SIDEBAR_CHAR, sidebar_style),
                Span::raw(" "),
            ];
            spans.extend(style_inline_markdown(
                &chunk,
                InlineStyleMode::Normal,
                DelimiterDisplay::Hide,
            ));
            lines.push(Line::from(spans));
        }
    } else {
        // No header — render question text directly with ○ symbol
        for (i, chunk) in q_lines.into_iter().enumerate() {
            let mut spans = vec![
                Span::raw(num_padding),
                Span::styled(SIDEBAR_CHAR, sidebar_style),
                Span::raw(" "),
            ];
            if i == 0 {
                spans.push(sym_span.clone());
            } else {
                spans.push(Span::raw("  "));
            }
            spans.extend(style_inline_markdown(
                &chunk,
                InlineStyleMode::Normal,
                DelimiterDisplay::Hide,
            ));
            if i == 0
                && let Some(progress) = progress
                && total_questions > 1
            {
                spans.push(Span::styled(
                    format!("  {progress}/{total_questions}"),
                    dim(),
                ));
            }
            lines.push(Line::from(spans));
        }
    }

    // Blank line before options
    let has_options = !question.options.is_empty()
        || matches!(
            question.question_type,
            QuestionType::YesNo | QuestionType::Confirmation
        );
    if has_options {
        blank_sidebar_line(lines, sidebar_style);
    }

    // Options (vertical layout)
    option_lines(
        lines,
        question,
        draft,
        active_state,
        preview_input,
        sidebar_style,
    );

    // Yes/No options
    if matches!(
        question.question_type,
        QuestionType::YesNo | QuestionType::Confirmation
    ) {
        yes_no_option_lines(
            lines,
            question,
            draft,
            active_state,
            preview_input,
            sidebar_style,
        );
    }
}

/// Render a collapsed answered-question line: `■ Header: Answer`.
///
/// The header text is word-wrapped to `content_width` so that long
/// questions don't overflow the sidebar. The answer is appended in
/// blue after the last header line.
fn answered_question_line(
    lines: &mut Vec<Line>,
    question: &stencila_attractor::interviewer::Question,
    status: InterviewStatus,
    answer: Option<&stencila_attractor::interviewer::Answer>,
    draft: Option<&DraftAnswer>,
    sidebar_style: Style,
    content_width: usize,
) {
    let num_padding = "   ";
    let answer_value = if status == InterviewStatus::Completed {
        answer.map(|a| &a.value)
    } else {
        None
    };
    let answer_text = answer_value.map_or_else(
        || {
            draft.map_or_else(String::new, |d| {
                friendly_answer_text(&d.to_answer(question).value, question)
            })
        },
        |v| friendly_answer_text(v, question),
    );
    let header = question.header.as_deref().unwrap_or(&question.text);
    // "■ " prefix is 2 chars
    let avail = content_width.saturating_sub(2);
    let header_with_colon = format!("{header}: ");
    let chunks = wrap_content(&header_with_colon, avail);
    let last = chunks.len().saturating_sub(1);
    for (i, chunk) in chunks.iter().enumerate() {
        let prefix = if i == 0 {
            Span::styled(format!("{SYM_QUESTION_CLOSED} "), dim())
        } else {
            Span::raw("  ")
        };
        let mut spans = vec![
            Span::raw(num_padding),
            Span::styled(SIDEBAR_CHAR, sidebar_style),
            Span::raw(" "),
            prefix,
            Span::styled(chunk.clone(), dim()),
        ];
        if i == last {
            spans.push(Span::styled(
                answer_text.clone(),
                Style::new().fg(Color::Blue),
            ));
        }
        lines.push(Line::from(spans));
    }
}

/// Format an answer value as user-friendly text, using question context to
/// resolve option keys to labels.
fn friendly_answer_text(
    value: &stencila_attractor::interviewer::AnswerValue,
    question: &stencila_attractor::interviewer::Question,
) -> String {
    use stencila_attractor::interviewer::AnswerValue;

    match value {
        AnswerValue::Yes => "Yes".to_string(),
        AnswerValue::No => "No".to_string(),
        AnswerValue::Skipped => "Skipped".to_string(),
        AnswerValue::Timeout => "Timed out".to_string(),
        AnswerValue::Text(text) => text.clone(),
        AnswerValue::Selected(key) => question
            .options
            .iter()
            .find(|o| &o.key == key)
            .map_or_else(|| key.clone(), |o| o.label.clone()),
        AnswerValue::MultiSelected(keys) => {
            let labels: Vec<&str> = keys
                .iter()
                .map(|k| {
                    question
                        .options
                        .iter()
                        .find(|o| &o.key == k)
                        .map_or(k.as_str(), |o| o.label.as_str())
                })
                .collect();
            labels.join(", ")
        }
    }
}

/// Render option lines for `MultiSelect` / `MultipleChoice` questions.
fn option_lines(
    lines: &mut Vec<Line>,
    question: &stencila_attractor::interviewer::Question,
    draft: Option<&DraftAnswer>,
    active_state: Option<&crate::interview::InterviewState>,
    preview_input: &str,
    sidebar_style: Style,
) {
    use stencila_attractor::interviewer::QuestionType;

    let current_input = if active_state.is_some() {
        preview_input
    } else {
        ""
    };
    let preview_single = preview_selection(current_input, question);
    let preview_multi = preview_multi_select(current_input, question);
    let focused_option =
        active_state.and_then(crate::interview::InterviewState::current_option_focus);

    let num_padding = "   ";
    for (o_idx, option) in question.options.iter().enumerate() {
        let is_selected = match question.question_type {
            QuestionType::MultiSelect => draft
                .and_then(|d| {
                    if let DraftAnswer::MultiSelected(set) = d {
                        Some(set.contains(&o_idx))
                    } else {
                        None
                    }
                })
                .unwrap_or(false),
            QuestionType::MultipleChoice => draft
                .and_then(|d| {
                    if let DraftAnswer::Selected(idx) = d {
                        Some(*idx == Some(o_idx))
                    } else {
                        None
                    }
                })
                .unwrap_or(false),
            _ => false,
        } || matches!(question.question_type, QuestionType::MultiSelect)
            && preview_multi.contains(&o_idx)
            || matches!(question.question_type, QuestionType::MultipleChoice)
                && preview_single.selected == Some(o_idx);

        let symbol = if is_selected {
            SYM_SELECTED
        } else {
            SYM_UNSELECTED
        };

        let is_focused = focused_option == Some(o_idx);

        let key_style = if is_selected && is_focused {
            Style::new().fg(Color::Blue).add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::new().fg(Color::Blue)
        } else if is_focused {
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::Cyan)
        };
        let label_style = if is_focused {
            Style::new().add_modifier(Modifier::BOLD)
        } else {
            Style::new()
        };

        let opt_spans = vec![
            Span::raw(num_padding),
            Span::styled(SIDEBAR_CHAR, sidebar_style),
            Span::raw(" "),
            Span::styled(format!("{symbol} "), key_style),
            Span::styled(format!("{} ", option.key), key_style),
            Span::styled(option.label.clone(), label_style),
        ];
        lines.push(Line::from(opt_spans));
        if let Some(desc) = &option.description {
            let mut desc_spans = vec![
                Span::raw(num_padding),
                Span::styled(SIDEBAR_CHAR, sidebar_style),
                Span::raw(" "),
                Span::raw("   "),
            ];
            desc_spans.extend(style_inline_markdown(
                desc,
                InlineStyleMode::Normal,
                DelimiterDisplay::Hide,
            ));
            lines.push(Line::from(desc_spans).style(dim()));
        }
    }
}

/// Render Yes / No toggle lines.
fn yes_no_option_lines(
    lines: &mut Vec<Line>,
    question: &stencila_attractor::interviewer::Question,
    draft: Option<&DraftAnswer>,
    active_state: Option<&crate::interview::InterviewState>,
    preview_input: &str,
    sidebar_style: Style,
) {
    let num_padding = "   ";
    let focused_option =
        active_state.and_then(crate::interview::InterviewState::current_option_focus);
    let selected = draft.and_then(|d| {
        if let DraftAnswer::YesNo(v) = d {
            *v
        } else {
            None
        }
    });
    let preview = preview_selection(
        if active_state.is_some() {
            preview_input
        } else {
            ""
        },
        question,
    )
    .yes_no;
    let yes_selected = selected == Some(true) || preview == Some(true);
    let no_selected = selected == Some(false) || preview == Some(false);
    let yes_focused = focused_option == Some(0);
    let no_focused = focused_option == Some(1);
    let (yes_key_style, yes_label_style) = if yes_selected && yes_focused {
        (
            Style::new().fg(Color::Blue).add_modifier(Modifier::BOLD),
            Style::new().add_modifier(Modifier::BOLD),
        )
    } else if yes_selected {
        (Style::new().fg(Color::Blue), Style::new())
    } else if yes_focused {
        (
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            Style::new().add_modifier(Modifier::BOLD),
        )
    } else {
        (Style::new().fg(Color::Cyan), Style::new())
    };
    let (no_key_style, no_label_style) = if no_selected && no_focused {
        (
            Style::new().fg(Color::Blue).add_modifier(Modifier::BOLD),
            Style::new().add_modifier(Modifier::BOLD),
        )
    } else if no_selected {
        (Style::new().fg(Color::Blue), Style::new())
    } else if no_focused {
        (
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            Style::new().add_modifier(Modifier::BOLD),
        )
    } else {
        (Style::new().fg(Color::Cyan), Style::new())
    };
    lines.push(Line::from(vec![
        Span::raw(num_padding),
        Span::styled(SIDEBAR_CHAR, sidebar_style),
        Span::raw(" "),
        Span::styled(
            format!(
                "{} ",
                if yes_selected {
                    SYM_SELECTED
                } else {
                    SYM_UNSELECTED
                }
            ),
            if yes_selected {
                yes_key_style
            } else {
                Style::new().fg(Color::Cyan)
            },
        ),
        Span::styled("y ", yes_key_style),
        Span::styled("Yes", yes_label_style),
    ]));
    lines.push(Line::from(vec![
        Span::raw(num_padding),
        Span::styled(SIDEBAR_CHAR, sidebar_style),
        Span::raw(" "),
        Span::styled(
            format!(
                "{} ",
                if no_selected {
                    SYM_SELECTED
                } else {
                    SYM_UNSELECTED
                }
            ),
            if no_selected {
                no_key_style
            } else {
                Style::new().fg(Color::Cyan)
            },
        ),
        Span::styled("n ", no_key_style),
        Span::styled("No", no_label_style),
    ]));
}

/// Render a dimmed future-question preview line with ○ symbol.
fn future_question_line(
    lines: &mut Vec<Line>,
    question: &stencila_attractor::interviewer::Question,
    sidebar_style: Style,
) {
    let num_padding = "   ";
    let preview = question
        .header
        .as_deref()
        .unwrap_or_else(|| question.text.lines().next().unwrap_or(&question.text));
    lines.push(Line::from(vec![
        Span::raw(num_padding),
        Span::styled(SIDEBAR_CHAR, sidebar_style),
        Span::raw(" "),
        Span::styled(format!("{SYM_QUESTION_OPEN} "), dim()),
        Span::styled(preview.to_string(), dim()),
    ]));
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
