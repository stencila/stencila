use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::{history::CommandHistory, input::InputState};

/// A message displayed in the chat area.
#[derive(Debug, Clone)]
pub enum ChatMessage {
    /// A message from the user.
    User { content: String },
    /// A system/informational message.
    System { content: String },
}

/// Top-level application state.
///
/// All mutable state lives here. The render function takes `&App` (immutable)
/// while event handlers take `&mut App`.
pub struct App {
    /// Whether the app should exit.
    pub should_quit: bool,
    /// Chat messages displayed in the message area.
    pub messages: Vec<ChatMessage>,
    /// Current input buffer.
    pub input: InputState,
    /// Command history with navigation.
    pub history: CommandHistory,
    /// Scroll offset for the message area (lines from the bottom).
    pub scroll_offset: u16,
    /// Total lines rendered in the last frame's message area (set by `ui::render`).
    pub total_message_lines: u16,
    /// Visible height of the message area in the last frame (set by `ui::render`).
    pub visible_message_height: u16,
}

impl App {
    /// Create a new App with a welcome banner.
    pub fn new() -> Self {
        let version = stencila_version::STENCILA_VERSION;
        let welcome = format!("Stencila {version} — Ctrl+C to quit");

        Self {
            should_quit: false,
            messages: vec![ChatMessage::System { content: welcome }],
            input: InputState::default(),
            history: CommandHistory::new(),
            scroll_offset: 0,
            total_message_lines: 0,
            visible_message_height: 0,
        }
    }

    /// Handle a terminal event. Returns `true` if the app should exit.
    pub fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Paste(text) => self.handle_paste(text),
            _ => {}
        }
        self.should_quit
    }

    /// Dispatch a key event.
    fn handle_key(&mut self, key: &KeyEvent) {
        match (key.modifiers, key.code) {
            // Quit
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.should_quit = true;
            }

            // Clear messages
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                self.messages.clear();
                self.scroll_offset = 0;
            }

            // Delete to line start / end
            (KeyModifiers::CONTROL, KeyCode::Char('u')) => self.input.delete_to_line_start(),
            (KeyModifiers::CONTROL, KeyCode::Char('k')) => self.input.delete_to_line_end(),

            // Insert newline: Shift+Enter or Alt+Enter
            (m, KeyCode::Enter)
                if m.contains(KeyModifiers::SHIFT) || m.contains(KeyModifiers::ALT) =>
            {
                self.input.insert_newline();
            }

            // Submit input
            (KeyModifiers::NONE, KeyCode::Enter) => {
                self.submit_input();
            }

            // Word-level cursor movement
            (KeyModifiers::CONTROL, KeyCode::Left) => self.input.move_word_left(),
            (KeyModifiers::CONTROL, KeyCode::Right) => self.input.move_word_right(),

            // Cursor movement
            (KeyModifiers::NONE, KeyCode::Left) => self.input.move_left(),
            (KeyModifiers::NONE, KeyCode::Right) => self.input.move_right(),
            (KeyModifiers::NONE, KeyCode::Home) => self.input.move_home(),
            (KeyModifiers::NONE, KeyCode::End) => self.input.move_end(),

            // History navigation (only when input is single-line)
            (KeyModifiers::NONE, KeyCode::Up) if self.input.is_single_line() => {
                self.navigate_history_up();
            }
            (KeyModifiers::NONE, KeyCode::Down) if self.input.is_single_line() => {
                self.navigate_history_down();
            }

            // Word-level deletion
            (KeyModifiers::CONTROL, KeyCode::Backspace) => self.input.delete_word_back(),
            (KeyModifiers::CONTROL, KeyCode::Delete) => self.input.delete_word_forward(),

            // Deletion
            (KeyModifiers::NONE, KeyCode::Backspace) => self.input.delete_char_before(),
            (KeyModifiers::NONE, KeyCode::Delete) => self.input.delete_char_at(),

            // Scroll
            (KeyModifiers::NONE, KeyCode::PageUp) => self.scroll_up(10),
            (KeyModifiers::NONE, KeyCode::PageDown) => self.scroll_down(10),

            // Character input
            (modifier, KeyCode::Char(c))
                if modifier.is_empty() || modifier == KeyModifiers::SHIFT =>
            {
                self.input.insert_char(c);
                self.scroll_offset = 0;
            }

            _ => {}
        }
    }

    /// Handle pasted text — insert as-is without triggering submit.
    fn handle_paste(&mut self, text: &str) {
        self.input.insert_str(text);
        self.scroll_offset = 0;
    }

    /// Submit the current input as a user message.
    fn submit_input(&mut self) {
        let text = self.input.take();
        if text.trim().is_empty() {
            return;
        }

        self.history.push(text.clone());
        self.messages.push(ChatMessage::User { content: text });

        // Reset scroll to bottom
        self.scroll_offset = 0;
    }

    /// Navigate to the previous (older) history entry.
    fn navigate_history_up(&mut self) {
        let current = self.input.text().to_string();
        if let Some(entry) = self.history.navigate_up(&current) {
            self.input.set_text(entry);
        }
    }

    /// Navigate to the next (newer) history entry, or back to draft.
    fn navigate_history_down(&mut self) {
        if let Some(entry) = self.history.navigate_down() {
            self.input.set_text(entry);
        }
    }

    /// Scroll up by the given number of lines.
    fn scroll_up(&mut self, lines: u16) {
        let max_scroll = self
            .total_message_lines
            .saturating_sub(self.visible_message_height);
        self.scroll_offset = self.scroll_offset.saturating_add(lines).min(max_scroll);
    }

    /// Scroll down by the given number of lines.
    fn scroll_down(&mut self, lines: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEventKind, KeyEventState};

    use super::*;

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    #[test]
    fn welcome_message() {
        let app = App::new();
        assert_eq!(app.messages.len(), 1);
        assert!(
            matches!(&app.messages[0], ChatMessage::System { content } if content.contains("Stencila"))
        );
    }

    #[test]
    fn ctrl_c_quits() {
        let mut app = App::new();
        let quit = app.handle_event(&key_event(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(quit);
        assert!(app.should_quit);
    }

    #[test]
    fn typing_and_submit() {
        let mut app = App::new();

        // Type "hello"
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        assert_eq!(app.input.text(), "hello");

        // Submit
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert!(app.input.is_empty());
        assert_eq!(app.messages.len(), 2);
        assert!(matches!(&app.messages[1], ChatMessage::User { content } if content == "hello"));
    }

    #[test]
    fn empty_submit_ignored() {
        let mut app = App::new();
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        // Only the welcome message
        assert_eq!(app.messages.len(), 1);
    }

    #[test]
    fn ctrl_l_clears() {
        let mut app = App::new();

        // Type and submit a message
        app.handle_event(&key_event(KeyCode::Char('x'), KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.messages.len(), 2);

        // Clear
        app.handle_event(&key_event(KeyCode::Char('l'), KeyModifiers::CONTROL));
        assert!(app.messages.is_empty());
    }

    #[test]
    fn shift_enter_inserts_newline() {
        let mut app = App::new();
        app.handle_event(&key_event(KeyCode::Char('a'), KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::SHIFT));
        app.handle_event(&key_event(KeyCode::Char('b'), KeyModifiers::NONE));
        assert_eq!(app.input.text(), "a\nb");
    }

    #[test]
    fn alt_enter_inserts_newline() {
        let mut app = App::new();
        app.handle_event(&key_event(KeyCode::Char('x'), KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::ALT));
        app.handle_event(&key_event(KeyCode::Char('y'), KeyModifiers::NONE));
        assert_eq!(app.input.text(), "x\ny");
    }

    #[test]
    fn paste_inserts_without_submit() {
        let mut app = App::new();
        app.handle_event(&Event::Paste("hello\nworld".to_string()));
        assert_eq!(app.input.text(), "hello\nworld");
        // Should not have submitted — only the welcome message
        assert_eq!(app.messages.len(), 1);
    }

    #[test]
    fn history_up_down() {
        let mut app = App::new();

        // Submit a few messages
        for msg in ["first", "second", "third"] {
            for c in msg.chars() {
                app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
            }
            app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));
        }

        // Navigate up through history
        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "third");
        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "second");
        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "first");

        // Navigate back down
        app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "second");
    }

    #[test]
    fn history_preserves_draft() {
        let mut app = App::new();

        // Submit something to have history
        for c in "old".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE));

        // Type a draft
        for c in "draft".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }

        // Navigate up, then back down to draft
        app.handle_event(&key_event(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "old");
        app.handle_event(&key_event(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.input.text(), "draft");
    }

    #[test]
    fn ctrl_u_deletes_to_line_start() {
        let mut app = App::new();
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Char('u'), KeyModifiers::CONTROL));
        assert_eq!(app.input.text(), "");
    }

    #[test]
    fn ctrl_k_deletes_to_line_end() {
        let mut app = App::new();
        for c in "hello".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_event(&key_event(KeyCode::Home, KeyModifiers::NONE));
        app.handle_event(&key_event(KeyCode::Char('k'), KeyModifiers::CONTROL));
        assert_eq!(app.input.text(), "");
    }

    #[test]
    fn scroll_bounds() {
        let mut app = App::new();
        // Simulate a frame that rendered 20 total lines with 10 visible
        app.total_message_lines = 20;
        app.visible_message_height = 10;

        app.scroll_up(5);
        assert_eq!(app.scroll_offset, 5);

        app.scroll_up(10);
        assert_eq!(app.scroll_offset, 10); // clamped to max (20 - 10)

        app.scroll_down(3);
        assert_eq!(app.scroll_offset, 7);

        app.scroll_down(100);
        assert_eq!(app.scroll_offset, 0);
    }
}
