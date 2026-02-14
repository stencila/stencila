use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, WizardStep};

use super::common::{
    dim, render_popup, selected_secondary_style, selected_style, unselected_style, wrap_content,
};

/// Render the new-agent wizard as a centered modal dialog.
#[allow(clippy::too_many_lines)]
pub(super) fn new_agent(frame: &mut Frame, app: &App, area: Rect) {
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
            models_in_area(frame, app, picker_area);
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
fn models_in_area(frame: &mut Frame, app: &App, area: Rect) {
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
