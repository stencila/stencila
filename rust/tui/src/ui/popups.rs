use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::app::App;
use crate::autocomplete::agents::AgentCandidateKind;
use crate::autocomplete::resume::ResumableKind;

use super::common::{
    dim, popup_area, render_popup, selected_secondary_style, selected_style, unselected_style,
};

/// Render the command autocomplete popup floating above the input area.
pub(super) fn commands(frame: &mut Frame, app: &App, input_area: Rect) {
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
pub(super) fn history(frame: &mut Frame, app: &App, input_area: Rect) {
    let candidates = app.history_state.candidates();
    let Some(area) = popup_area(input_area, candidates.len()) else {
        return;
    };

    let selected = app.history_state.selected();
    let lines: Vec<Line> = candidates
        .iter()
        .enumerate()
        .map(|(i, candidate)| {
            let base_style = if i == selected {
                Style::new().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                dim()
            };
            let match_style = Style::new().fg(Color::Blue);

            let mut spans = vec![Span::styled(" • ", base_style)];
            let chars: Vec<char> = candidate.preview.chars().collect();
            let mut last = 0;

            for (start, end) in &candidate.match_indices {
                if *start > last {
                    spans.push(Span::styled(
                        chars[last..*start].iter().collect::<String>(),
                        base_style,
                    ));
                }

                let mut style = match_style;
                if i == selected {
                    style = style.add_modifier(Modifier::BOLD);
                }

                spans.push(Span::styled(
                    chars[*start..*end].iter().collect::<String>(),
                    style,
                ));
                last = *end;
            }

            if last < chars.len() {
                spans.push(Span::styled(
                    chars[last..].iter().collect::<String>(),
                    base_style,
                ));
            }

            Line::from(spans)
        })
        .collect();

    render_popup(frame, area, lines, Some(" History "));
}

/// Render the file autocomplete popup floating above the input area.
pub(super) fn files(frame: &mut Frame, app: &App, input_area: Rect) {
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
pub(super) fn responses(frame: &mut Frame, app: &App, input_area: Rect) {
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
pub(super) fn cancel(frame: &mut Frame, app: &App, input_area: Rect) {
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

/// Render the agent picker popup floating above the input area.
pub(super) fn agents(frame: &mut Frame, app: &App, input_area: Rect) {
    let candidates = app.agents_state.candidates();
    let Some(area) = popup_area(input_area, candidates.len()) else {
        return;
    };

    let max_name_width = candidates.iter().map(|c| c.name.len()).max().unwrap_or(0);
    let selected = app.agents_state.selected();

    let lines: Vec<Line> = candidates
        .iter()
        .enumerate()
        .map(|(i, candidate)| match &candidate.kind {
            AgentCandidateKind::Session {
                is_active,
                definition,
                ..
            } => {
                let bullet = if *is_active { "> " } else { "\u{25cf} " };
                let name_col = format!(" {bullet}{:<max_name_width$}  ", candidate.name);
                let color = app
                    .color_registry
                    .get(&candidate.name)
                    .unwrap_or(ratatui::style::Color::Blue);

                let detail = definition.as_ref().and_then(|info| {
                    if !info.description.is_empty() {
                        Some(info.description.clone())
                    } else if !info.source.is_empty() {
                        Some(info.source.clone())
                    } else {
                        None
                    }
                });

                let name_style = if i == selected {
                    Style::new().fg(color).add_modifier(Modifier::BOLD)
                } else {
                    Style::new().fg(color)
                };

                if let Some(detail) = detail {
                    let detail_style = if i == selected {
                        selected_secondary_style()
                    } else {
                        dim()
                    };
                    Line::from(vec![
                        Span::styled(name_col, name_style),
                        Span::styled(detail, detail_style),
                    ])
                } else {
                    Line::from(Span::styled(name_col, name_style))
                }
            }
            AgentCandidateKind::Definition(info) => {
                let name_col = format!(" \u{25cb} {:<max_name_width$}  ", candidate.name);
                let detail = if info.description.is_empty() {
                    info.source.clone()
                } else {
                    info.description.clone()
                };

                if i == selected {
                    Line::from(vec![
                        Span::styled(name_col, selected_style().add_modifier(Modifier::BOLD)),
                        Span::styled(detail, selected_secondary_style()),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(name_col, unselected_style()),
                        Span::styled(detail, dim()),
                    ])
                }
            }
        })
        .collect();

    render_popup(frame, area, lines, Some(" Agents "));
}

/// Render the workflow picker popup floating above the input area.
pub(super) fn workflows(frame: &mut Frame, app: &App, input_area: Rect) {
    let candidates = app.workflows_state.candidates();
    let Some(area) = popup_area(input_area, candidates.len()) else {
        return;
    };

    let max_name_width = candidates.iter().map(|c| c.name.len()).max().unwrap_or(0);
    let selected = app.workflows_state.selected();

    let lines: Vec<Line> = candidates
        .iter()
        .enumerate()
        .map(|(i, candidate)| {
            let name_col = format!("   {:<max_name_width$}  ", candidate.name);
            let detail = if candidate.info.description.is_empty() {
                candidate.info.goal.as_deref().unwrap_or("").to_string()
            } else {
                candidate.info.description.clone()
            };

            if i == selected {
                Line::from(vec![
                    Span::styled(name_col, selected_style()),
                    Span::styled(detail, selected_secondary_style()),
                ])
            } else {
                Line::from(vec![
                    Span::styled(name_col, unselected_style()),
                    Span::styled(detail, dim()),
                ])
            }
        })
        .collect();

    render_popup(frame, area, lines, Some(" Workflows "));
}

/// Render the resume picker popup floating above the input area.
pub(super) fn resume(frame: &mut Frame, app: &App, input_area: Rect) {
    let candidates = app.resume_state.candidates();
    let row_count = candidates.len().max(1);
    let Some(area) = popup_area(input_area, row_count) else {
        return;
    };

    if !app.resume_state.has_matches() {
        let lines = vec![Line::from(Span::styled(" No matches ", dim()))];
        render_popup(frame, area, lines, Some(" Resume "));
        return;
    }

    let max_time_width = candidates
        .iter()
        .map(|c| c.time_ago.len())
        .max()
        .unwrap_or(0);
    let max_status_width = candidates
        .iter()
        .map(|c| match c.status.as_str() {
            "fail" | "failed" => "failed".len(),
            other => other.len(),
        })
        .max()
        .unwrap_or(0);
    let max_name_width = candidates.iter().map(|c| c.name.len()).max().unwrap_or(0);
    let selected = app.resume_state.selected();

    let lines: Vec<Line> = candidates
        .iter()
        .enumerate()
        .map(|(i, candidate)| {
            let kind_label = match candidate.kind {
                ResumableKind::WorkflowRun => "workflow",
                ResumableKind::AgentSession => "agent   ",
            };
            let kind_col = format!(" {kind_label}  ");
            let time_col = format!(
                "{:>max_time_width$}  ",
                candidate.time_ago
            );
            let status_label = match candidate.status.as_str() {
                "fail" | "failed" => "failed",
                other => other,
            };
            let status_color = match candidate.status.as_str() {
                "failed" | "fail" => Color::Red,
                "cancelled" => Color::Magenta,
                "running" => Color::Yellow,
                _ => Color::DarkGray,
            };
            let status_col = format!("{status_label:<max_status_width$}  ");
            let name_col = format!("{:<max_name_width$}  ", candidate.name);
            let desc = &candidate.description;

            if i == selected {
                Line::from(vec![
                    Span::styled(kind_col, selected_secondary_style()),
                    Span::styled(name_col, selected_style()),
                    Span::styled(status_col, Style::new().fg(status_color)),
                    Span::styled(time_col, selected_secondary_style()),
                    Span::styled(desc.to_string(), selected_secondary_style()),
                ])
            } else {
                Line::from(vec![
                    Span::styled(kind_col, dim()),
                    Span::styled(name_col, unselected_style()),
                    Span::styled(status_col, Style::new().fg(status_color)),
                    Span::styled(time_col, dim()),
                    Span::styled(desc.to_string(), dim()),
                ])
            }
        })
        .collect();

    render_popup(frame, area, lines, Some(" Resume "));
}

/// Render the agent mention autocomplete popup floating above the input area.
pub(super) fn mentions(frame: &mut Frame, app: &App, input_area: Rect) {
    let candidates = app.mentions_state.candidates();
    let Some(area) = popup_area(input_area, candidates.len()) else {
        return;
    };

    let max_name_width = candidates.iter().map(|c| c.name.len()).max().unwrap_or(0);
    let selected = app.mentions_state.selected();

    let lines: Vec<Line> = candidates
        .iter()
        .enumerate()
        .map(|(i, candidate)| {
            let name_col = format!(" {:<max_name_width$}  ", candidate.name);

            let detail = candidate.definition.as_ref().and_then(|info| {
                if !info.description.is_empty() {
                    Some(info.description.clone())
                } else if !info.source.is_empty() {
                    Some(info.source.clone())
                } else {
                    None
                }
            });

            let name_style = if i == selected {
                Style::new()
                    .fg(candidate.color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(candidate.color)
            };

            if let Some(detail) = detail {
                let detail_style = if i == selected {
                    selected_secondary_style()
                } else {
                    dim()
                };
                Line::from(vec![
                    Span::styled(name_col, name_style),
                    Span::styled(detail, detail_style),
                ])
            } else {
                Line::from(Span::styled(name_col, name_style))
            }
        })
        .collect();

    render_popup(frame, area, lines, Some(" Prompt agent "));
}
