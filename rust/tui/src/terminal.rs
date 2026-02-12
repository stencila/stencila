use std::io::{self, stdout};

use crossterm::{
    event::{DisableBracketedPaste, EnableBracketedPaste},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use eyre::Result;
use ratatui::{Terminal, backend::CrosstermBackend};

/// RAII guard that restores the terminal on drop.
///
/// Ensures raw mode, alternate screen, and bracketed paste are cleaned up
/// even when the event loop returns early via `?`.
pub struct TerminalGuard {
    pub terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(stdout(), DisableBracketedPaste);
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
}

/// Initialize the terminal for TUI rendering.
///
/// Returns a [`TerminalGuard`] that automatically restores the terminal
/// when dropped (including on early error returns and panics).
///
/// # Errors
///
/// Returns an error if raw mode or alternate screen cannot be enabled.
pub fn init() -> Result<TerminalGuard> {
    install_panic_hook();

    enable_raw_mode()?;

    // If alternate screen or bracketed paste fail, restore raw mode before propagating.
    if let Err(e) = execute!(stdout(), EnterAlternateScreen, EnableBracketedPaste) {
        let _ = disable_raw_mode();
        return Err(e.into());
    }

    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(TerminalGuard { terminal })
}

/// Install a panic hook that restores the terminal before printing the panic.
fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = execute!(stdout(), DisableBracketedPaste);
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
}
