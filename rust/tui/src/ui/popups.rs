use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::app::{AgentSession, App};
use crate::autocomplete::agents::AgentCandidateKind;

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

/// Render the model picker popup floating above the input area.
pub(super) fn models(frame: &mut Frame, app: &App, input_area: Rect) {
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
            AgentCandidateKind::New => {
                let styles = if i == selected {
                    (selected_style(), selected_secondary_style())
                } else {
                    (Style::new(), dim())
                };

                Line::from(vec![
                    Span::styled(format!(" + {:<max_name_width$}  ", "new"), styles.0),
                    Span::styled("Create a new agent", styles.1),
                ])
            }
            AgentCandidateKind::Session {
                index,
                is_active,
                definition,
            } => {
                let bullet = if *is_active { "\u{25cf} " } else { "  " };
                let name_col = format!(" {bullet}{:<max_name_width$}  ", candidate.name);
                let color = AgentSession::color(*index);

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
                let name_col = format!("   {:<max_name_width$}  ", candidate.name);
                let detail = if info.description.is_empty() {
                    info.source.clone()
                } else {
                    info.description.clone()
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
            }
        })
        .collect();

    render_popup(frame, area, lines, Some(" Agents "));
}
