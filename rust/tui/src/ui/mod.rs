mod common;
mod input;
pub(crate) mod markdown;
mod messages;
mod popups;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;

use self::common::dim;

/// Render the entire UI for one frame.
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let input_height = input::input_height(app, area);
    let scrolled_up = !app.scroll_pinned;

    // Layout: messages | [scroll indicator] | spacer | input | hints
    // When scrolled up, add a dedicated line for the scroll indicator
    let scroll_rows = u16::from(scrolled_up);
    let layout = Layout::vertical([
        Constraint::Min(1),               // message area
        Constraint::Length(scroll_rows),  // scroll indicator
        Constraint::Length(1),            // blank line above input
        Constraint::Length(input_height), // input area
        Constraint::Length(1),            // hint line below input
    ])
    .split(area);

    let messages_area = layout[0];
    let scroll_indicator_area = layout[1];
    let input_area = layout[3];
    let hints_area = layout[4];

    // --- Render messages ---
    messages::render(frame, app, messages_area);

    // --- Render scroll indicator on its own line when scrolled up ---
    if scrolled_up {
        let lines_below = app
            .total_message_lines
            .saturating_sub(app.visible_message_height)
            .saturating_sub(app.scroll_offset);
        if lines_below > 0 {
            let indicator = Line::from(vec![
                Span::styled(format!("   +{lines_below} lines   "), dim()),
                Span::raw("esc"),
                Span::styled(" end", dim()),
            ]);
            frame.render_widget(
                Paragraph::new(indicator).alignment(Alignment::Left),
                scroll_indicator_area,
            );
        }
    }

    // --- Render input ---
    input::render(frame, app, input_area);

    // --- Render hints below input ---
    input::hints(frame, app, hints_area);

    // --- Render autocomplete popup (floats above input) ---
    // Cancel popup has highest priority, then agents, model, history, commands, files, responses.
    if app.cancel_state.is_visible() {
        popups::cancel(frame, app, input_area);
    } else if app.agents_state.is_visible() {
        popups::agents(frame, app, input_area);
    } else if app.workflows_state.is_visible() {
        popups::workflows(frame, app, input_area);
    } else if app.mentions_state.is_visible() {
        popups::mentions(frame, app, input_area);
    } else if app.history_state.is_visible() {
        popups::history(frame, app, input_area);
    } else if app.commands_state.is_visible() {
        popups::commands(frame, app, input_area);
    } else if app.files_state.is_visible() {
        popups::files(frame, app, input_area);
    } else if app.responses_state.is_visible() {
        popups::responses(frame, app, input_area);
    }
}
