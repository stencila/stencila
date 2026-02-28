#![warn(clippy::pedantic)]

mod agent;
mod app;
mod autocomplete;
pub mod cli_commands;
mod commands;
mod config;
mod event;
mod history;
mod input;
mod logging;
mod shell;
mod site_preview;
mod terminal;
mod ui;
mod workflow;

use clap::Args;
use eyre::Result;
use tokio::task::JoinHandle;
use tracing_subscriber::filter::LevelFilter;

use crate::{app::App, event::EventReader};

/// Run interactively
#[derive(Debug, Default, Clone, Args)]
pub struct Tui {
    /// Do not serve a preview of the workspace site
    #[arg(long, help_heading = "TUI Options")]
    pub no_preview: bool,
}

impl Tui {
    /// Run the interactive TUI application.
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal cannot be initialized or an I/O error occurs.
    pub async fn run(
        self,
        log_level: LevelFilter,
        log_filter: &str,
        upgrade_handle: Option<JoinHandle<Option<String>>>,
        cli_command: Option<clap::Command>,
    ) -> Result<()> {
        let log_receiver = logging::setup(log_level, log_filter);

        let cli_tree = cli_command.map(|cmd| {
            let allowlist = commands::SlashCommand::cli_allowlist();
            cli_commands::arc_tree(cli_commands::build_command_tree(&cmd, &allowlist))
        });

        let mut guard = terminal::init()?;
        let mut events = EventReader::new();
        let mut app = App::new(log_receiver, upgrade_handle, cli_tree).await;

        if !self.no_preview {
            match site_preview::spawn_preview() {
                Ok(handle) => app.site_preview = handle,
                Err(error) => {
                    app.messages.push(app::AppMessage::System {
                        content: format!("Failed to start site preview: {error}"),
                    });
                }
            }
        }

        // Load history from disk (best-effort)
        let history_path = history::history_file_path();
        if let Some(path) = &history_path {
            app.input_history.load_from_file(path);
        }

        loop {
            guard.terminal.draw(|frame| ui::render(frame, &mut app))?;
            match events.next().await {
                Some(event::AppEvent::Terminal(ref evt)) => {
                    if app.handle_event(evt).await {
                        break;
                    }
                }
                Some(event::AppEvent::Tick) => {
                    app.poll_running_commands();
                    app.poll_running_agent_exchanges();
                    app.poll_workflow_events();
                    app.poll_log_events();
                    app.poll_upgrade_check();
                    app.poll_site_preview();
                }
                None => break,
            }
        }

        // Save history to disk (best-effort)
        if let Some(path) = &history_path {
            app.input_history.save_to_file(path);
        }

        // Guard's Drop restores the terminal automatically, but dropping
        // explicitly here makes the intent clear.
        drop(guard);
        Ok(())
    }
}
