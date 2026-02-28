use crate::autocomplete::agents::AgentDefinitionInfo;

use super::{AgentMention, AgentSession, App, AppMessage, discover_agents};

impl App {
    pub(super) fn create_session_from_definition(&mut self, info: &AgentDefinitionInfo) {
        let mut session = AgentSession::new(&info.name);
        session.definition = Some(info.clone());

        self.color_registry.color_for(&info.name);
        self.sessions.push(session);
        self.active_session = self.sessions.len() - 1;
        self.input.clear();
        self.input_scroll = 0;

        let model_info = match (&info.provider, &info.model) {
            (Some(p), Some(m)) => format!(" using {m} ({p})"),
            _ => String::new(),
        };
        self.messages.push(AppMessage::System {
            content: format!("Agent '{}' activated{model_info}.", info.name),
        });
    }

    /// Switch to the agent session at the given index.
    pub fn switch_to_session(&mut self, index: usize) {
        if index < self.sessions.len() && index != self.active_session {
            self.active_session = index;
        }
    }

    /// Parse a `#agent-name [prompt]` pattern from the input text.
    ///
    /// Returns `None` if the text doesn't start with `#name` or the name doesn't
    /// match any known session or discovered agent.
    pub(super) fn parse_agent_mention(&self, text: &str) -> Option<AgentMention> {
        let trimmed = text.trim_start();
        if !trimmed.starts_with('#') {
            return None;
        }

        // Extract the name: everything after `#` up to first whitespace
        let after_hash = &trimmed[1..];
        let (name, rest) = after_hash
            .split_once(char::is_whitespace)
            .unwrap_or((after_hash, ""));

        if name.is_empty() {
            return None;
        }

        // Validate the name matches a session or discovered agent
        let name_lower = name.to_ascii_lowercase();
        let is_session = self
            .sessions
            .iter()
            .any(|s| s.name.to_ascii_lowercase() == name_lower);

        let is_definition = if is_session {
            false
        } else {
            discover_agents()
                .iter()
                .any(|d| d.name.to_ascii_lowercase() == name_lower)
        };

        if !is_session && !is_definition {
            return None;
        }

        let prompt = rest.trim();
        if prompt.is_empty() {
            // Just `#agent-name` — switch without sending
            Some(AgentMention {
                agent_name: name.to_string(),
                prompt: None,
                switch_back: false,
            })
        } else if prompt.ends_with('&') {
            // Prompt ends with `&` — send but don't switch back
            let prompt = prompt.trim_end_matches('&').trim_end().to_string();
            Some(AgentMention {
                agent_name: name.to_string(),
                prompt: if prompt.is_empty() {
                    None
                } else {
                    Some(prompt)
                },
                switch_back: false,
            })
        } else {
            // Normal prompt — send and switch back
            Some(AgentMention {
                agent_name: name.to_string(),
                prompt: Some(prompt.to_string()),
                switch_back: true,
            })
        }
    }

    /// Execute a parsed agent mention: switch, optionally send, optionally switch back.
    pub(super) fn execute_agent_mention(&mut self, mention: AgentMention) {
        let original_session = self.active_session;

        // Find or create the target session
        let name_lower = mention.agent_name.to_ascii_lowercase();
        let target_idx = self
            .sessions
            .iter()
            .position(|s| s.name.to_ascii_lowercase() == name_lower);

        let target_idx = match target_idx {
            Some(idx) => {
                if idx != self.active_session {
                    self.switch_to_session(idx);
                }
                idx
            }
            None => {
                // Create from definition
                if let Some(def) = discover_agents()
                    .into_iter()
                    .find(|d| d.name.to_ascii_lowercase() == name_lower)
                {
                    let info = AgentDefinitionInfo {
                        name: def.name.clone(),
                        description: def.description.clone(),
                        model: def.model.clone(),
                        provider: def.provider.clone(),
                        source: def.source().map(|s| s.to_string()).unwrap_or_default(),
                    };
                    self.create_session_from_definition(&info);
                    self.active_session
                } else {
                    return;
                }
            }
        };

        // Send the prompt if present
        if let Some(prompt) = mention.prompt {
            self.submit_agent_message(prompt);
        }

        // Switch back if needed
        if mention.switch_back && target_idx != original_session {
            self.active_session = original_session;
        }
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    use super::super::{AgentSession, App, AppMessage};

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    #[tokio::test]
    async fn default_session_exists() {
        let app = App::new_for_test().await;
        assert_eq!(app.sessions.len(), 1);
        let expected = stencila_agents::convenience::resolve_default_agent_name("default").await;
        assert_eq!(app.sessions[0].name, expected);
        assert_eq!(app.active_session, 0);
    }

    #[tokio::test]
    async fn switch_to_session() {
        let mut app = App::new_for_test().await;
        app.sessions.push(AgentSession::new("test-agent"));
        let initial = app.messages.len();

        app.switch_to_session(1);
        assert_eq!(app.active_session, 1);
        assert_eq!(app.messages.len(), initial);
    }

    #[tokio::test]
    async fn switch_to_same_session_noop() {
        let mut app = App::new_for_test().await;
        let initial = app.messages.len();
        app.switch_to_session(0);
        assert_eq!(app.messages.len(), initial); // no message added
    }

    #[tokio::test]
    async fn ctrl_a_cycles_agents() {
        let mut app = App::new_for_test().await;
        app.sessions.push(AgentSession::new("agent-a"));
        app.sessions.push(AgentSession::new("agent-b"));

        // Ctrl+A cycles forward
        app.handle_event(&key_event(KeyCode::Char('a'), KeyModifiers::CONTROL))
            .await;
        assert_eq!(app.active_session, 1);

        app.handle_event(&key_event(KeyCode::Char('a'), KeyModifiers::CONTROL))
            .await;
        assert_eq!(app.active_session, 2);

        app.handle_event(&key_event(KeyCode::Char('a'), KeyModifiers::CONTROL))
            .await;
        assert_eq!(app.active_session, 0); // wraps around
    }

    #[tokio::test]
    async fn ctrl_a_noop_single_session() {
        let mut app = App::new_for_test().await;
        app.handle_event(&key_event(KeyCode::Char('a'), KeyModifiers::CONTROL))
            .await;
        assert_eq!(app.active_session, 0);
    }

    #[tokio::test]
    async fn exchange_has_agent_index() {
        let mut app = App::new_for_test().await;
        // Submit a chat message (agent will be unavailable in test, which is fine)
        for c in "test".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;

        // The last exchange should have agent_index = Some(0)
        let exchange = app
            .messages
            .iter()
            .find(|m| matches!(m, AppMessage::Exchange { .. }));
        assert!(exchange.is_some());
        if let Some(AppMessage::Exchange { agent_index, .. }) = exchange {
            assert_eq!(*agent_index, Some(0));
        }
    }
}
