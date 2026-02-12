#![warn(clippy::pedantic)]

mod app;
mod event;
mod input;
mod terminal;
mod ui;

use clap::Parser;
use eyre::Result;

use crate::{app::App, event::EventReader};

/// Run interactively
#[derive(Debug, Default, Parser)]
pub struct Tui {}

impl Tui {
    /// Run the interactive TUI application.
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal cannot be initialized or an I/O error occurs.
    pub async fn run(self) -> Result<()> {
        let mut guard = terminal::init()?;
        let mut events = EventReader::new();
        let mut app = App::new();

        loop {
            guard.terminal.draw(|frame| ui::render(frame, &mut app))?;
            match events.next().await {
                Some(event::AppEvent::Terminal(ref evt)) => {
                    if app.handle_event(evt) {
                        break;
                    }
                }
                Some(event::AppEvent::Tick) => {}
                None => break,
            }
        }

        // Guard's Drop restores the terminal automatically, but dropping
        // explicitly here makes the intent clear.
        drop(guard);
        Ok(())
    }
}
