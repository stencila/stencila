use ratatui::style::Color;

use crate::autocomplete::{
    agents::AgentDefinitionInfo, cancel::CancelCandidate, mentions::MentionCandidate,
    responses::ResponseCandidate,
};

use super::{App, AppMessage, ExchangeKind, ExchangeStatus, discover_agents, truncate_preview};

impl App {
    /// Build response candidates from existing exchanges.
    ///
    /// Returns `(exchange_number, truncated_preview)` tuples for exchanges that
    /// have a response, ordered newest first.
    pub fn response_candidates(&self) -> Vec<ResponseCandidate> {
        let mut exchange_num = 0usize;
        let mut candidates = Vec::new();

        for message in &self.messages {
            if let AppMessage::Exchange {
                kind,
                response: Some(resp),
                agent_index,
                agent_name,
                ..
            } = message
            {
                exchange_num += 1;
                if resp.is_empty() {
                    continue;
                }
                // Label + color: agent name/color or "shell"/yellow
                let (label, color) = if *kind == ExchangeKind::Shell {
                    ("shell".to_string(), ExchangeKind::Shell.color())
                } else {
                    let name = agent_name
                        .as_deref()
                        .or_else(|| {
                            agent_index
                                .and_then(|idx| self.sessions.get(idx).map(|s| s.name.as_str()))
                        })
                        .unwrap_or("chat");
                    let c = self
                        .color_registry
                        .get(name)
                        .unwrap_or(ExchangeKind::Agent.color());
                    (name.to_string(), c)
                };
                // First line of response as preview (no truncation — renderer handles it)
                let preview = resp.lines().next().unwrap_or("").to_string();
                candidates.push(ResponseCandidate {
                    number: exchange_num,
                    label,
                    preview,
                    color,
                });
            } else if matches!(message, AppMessage::Exchange { .. }) {
                exchange_num += 1;
            }
        }

        candidates.reverse();
        candidates
    }

    /// Build mention candidates from existing sessions and discovered agents.
    pub fn mention_candidates(&self) -> Vec<MentionCandidate> {
        let mut candidates: Vec<MentionCandidate> = self
            .sessions
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != self.active_session)
            .map(|(_, s)| MentionCandidate {
                name: s.name.clone(),
                color: self.color_registry.get(&s.name).unwrap_or(Color::DarkGray),
                definition: s.definition.clone(),
            })
            .collect();

        // Discovered agent definitions not yet in sessions
        let session_names: Vec<&str> = self.sessions.iter().map(|s| s.name.as_str()).collect();
        let definitions = discover_agents();
        for def in definitions {
            if session_names.contains(&def.name.as_str()) {
                continue;
            }
            candidates.push(MentionCandidate {
                name: def.name.clone(),
                color: self
                    .color_registry
                    .get(&def.name)
                    .unwrap_or(Color::DarkGray),
                definition: Some(AgentDefinitionInfo {
                    name: def.name.clone(),
                    description: def.description.clone(),
                    model: def.model.clone(),
                    provider: def.provider.clone(),
                    source: def.source().map(|s| s.to_string()).unwrap_or_default(),
                }),
            });
        }

        candidates
    }

    /// Build cancel candidates from running exchanges.
    ///
    /// Returns a list of `CancelCandidate` for exchanges with `Running` status,
    /// ordered by their position in `messages`.
    pub fn running_exchange_candidates(&self) -> Vec<CancelCandidate> {
        let mut exchange_num = 0usize;
        let mut candidates = Vec::new();

        for (msg_index, message) in self.messages.iter().enumerate() {
            if let AppMessage::Exchange {
                status: ExchangeStatus::Running,
                request,
                ..
            } = message
            {
                exchange_num += 1;
                let preview = truncate_preview(request, 40);
                candidates.push(CancelCandidate {
                    exchange_num,
                    msg_index,
                    request_preview: preview,
                });
            } else if matches!(message, AppMessage::Exchange { .. }) {
                exchange_num += 1;
            }
        }

        candidates
    }

    /// Expand `[Response #N: ...]` references in text with full response content.
    ///
    /// Unknown references are left as-is.
    pub fn expand_response_refs(&self, text: &str) -> String {
        // Build a map of exchange_number → response text
        let mut exchange_num = 0usize;
        let mut response_map = Vec::new();
        for message in &self.messages {
            if let AppMessage::Exchange { response, .. } = message {
                exchange_num += 1;
                if let Some(resp) = response {
                    response_map.push((exchange_num, resp.as_str()));
                }
            }
        }

        let mut result = String::with_capacity(text.len());
        let mut remaining = text;

        while let Some(start) = remaining.find("[Response #") {
            // Copy everything before the match
            result.push_str(&remaining[..start]);

            let after_prefix = &remaining[start + "[Response #".len()..];

            // Parse the number (digits until ':' or ']')
            let num_end = after_prefix
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(after_prefix.len());
            let num_str = &after_prefix[..num_end];

            // Find the closing ']'
            if let Some(close) = after_prefix.find(']') {
                if let Ok(num) = num_str.parse::<usize>() {
                    // Look up the response
                    if let Some((_, resp)) = response_map.iter().find(|(n, _)| *n == num) {
                        result.push_str(resp);
                        remaining = &after_prefix[close + 1..];
                        continue;
                    }
                }
                // Unknown ref or parse failure — keep original text
                result.push_str(&remaining[start..=(start + "[Response #".len() + close)]);
                remaining = &after_prefix[close + 1..];
            } else {
                // No closing bracket — keep the rest as-is
                result.push_str(&remaining[start..]);
                remaining = "";
            }
        }

        result.push_str(remaining);
        result
    }

    /// Expand `[Paste #N: ...]` tokens in text with the stored paste contents.
    ///
    /// Unknown paste references are left as-is.
    pub(super) fn expand_paste_refs(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut remaining = text;

        while let Some(start) = remaining.find("[Paste #") {
            result.push_str(&remaining[..start]);

            let after_prefix = &remaining[start + "[Paste #".len()..];

            let num_end = after_prefix
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(after_prefix.len());
            let num_str = &after_prefix[..num_end];

            if let Some(close) = after_prefix.find(']') {
                if let Ok(num) = num_str.parse::<usize>()
                    && let Some(content) = self.pastes.get(&num)
                {
                    result.push_str(content);
                    remaining = &after_prefix[close + 1..];
                    continue;
                }
                result.push_str(&remaining[start..=(start + "[Paste #".len() + close)]);
                remaining = &after_prefix[close + 1..];
            } else {
                result.push_str(&remaining[start..]);
                remaining = "";
            }
        }

        result.push_str(remaining);
        result
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    use super::super::{App, AppMessage, ExchangeKind, ExchangeStatus, truncate_preview};
    use crossterm::event::Event;

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    /// Helper: create an app with some exchanges that have responses.
    async fn app_with_exchanges() -> App {
        let mut app = App::new_for_test().await;
        // Exchange 1: has response
        app.messages.push(AppMessage::Exchange {
            kind: ExchangeKind::Shell,
            status: ExchangeStatus::Succeeded,
            request: "echo hello".to_string(),
            response: Some("hello".to_string()),
            response_segments: None,
            exit_code: Some(0),
            agent_index: None,
            agent_name: None,
        });
        // Exchange 2: no response yet
        app.messages.push(AppMessage::Exchange {
            kind: ExchangeKind::Agent,
            status: ExchangeStatus::Running,
            request: "what is rust".to_string(),
            response: None,
            response_segments: None,
            exit_code: None,
            agent_index: Some(0),
            agent_name: None,
        });
        // Exchange 3: has response
        app.messages.push(AppMessage::Exchange {
            kind: ExchangeKind::Shell,
            status: ExchangeStatus::Succeeded,
            request: "ls -la".to_string(),
            response: Some("total 42\ndrwxr-xr-x 2 user user 4096".to_string()),
            response_segments: None,
            exit_code: Some(0),
            agent_index: None,
            agent_name: None,
        });
        app
    }

    #[tokio::test]
    async fn response_candidates_returns_correct_list() {
        let app = app_with_exchanges().await;
        let candidates = app.response_candidates();
        // Exchange 1 and 3 have responses; newest first
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].number, 3); // newest first
        assert_eq!(candidates[1].number, 1);
    }

    #[tokio::test]
    async fn dollar_triggers_response_autocomplete() {
        let mut app = app_with_exchanges().await;
        app.handle_event(&key_event(KeyCode::Char('$'), KeyModifiers::SHIFT))
            .await;
        assert!(app.responses_state.is_visible());
        assert_eq!(app.responses_state.candidates().len(), 2);
    }

    #[tokio::test]
    async fn dollar_with_digit_filters_responses() {
        let mut app = app_with_exchanges().await;
        app.handle_event(&key_event(KeyCode::Char('$'), KeyModifiers::SHIFT))
            .await;
        app.handle_event(&key_event(KeyCode::Char('1'), KeyModifiers::NONE))
            .await;
        assert!(app.responses_state.is_visible());
        assert_eq!(app.responses_state.candidates().len(), 1);
        assert_eq!(app.responses_state.candidates()[0].number, 1);
    }

    #[tokio::test]
    async fn response_esc_dismisses() {
        let mut app = app_with_exchanges().await;
        app.handle_event(&key_event(KeyCode::Char('$'), KeyModifiers::SHIFT))
            .await;
        assert!(app.responses_state.is_visible());

        app.handle_event(&key_event(KeyCode::Esc, KeyModifiers::NONE))
            .await;
        assert!(!app.responses_state.is_visible());
    }

    #[tokio::test]
    async fn response_tab_accepts() {
        let mut app = app_with_exchanges().await;
        app.handle_event(&key_event(KeyCode::Char('$'), KeyModifiers::SHIFT))
            .await;
        assert!(app.responses_state.is_visible());

        app.handle_event(&key_event(KeyCode::Tab, KeyModifiers::NONE))
            .await;
        assert!(!app.responses_state.is_visible());
        // Input should contain [Response #N: ...]
        assert!(app.input.text().contains("[Response #"));
    }

    #[tokio::test]
    async fn expand_response_refs_replaces_known() {
        let app = app_with_exchanges().await;
        let expanded = app.expand_response_refs("see [Response #1: hello...]");
        assert_eq!(expanded, "see hello");
    }

    #[tokio::test]
    async fn expand_response_refs_leaves_unknown() {
        let app = app_with_exchanges().await;
        let expanded = app.expand_response_refs("see [Response #99: unknown...]");
        assert_eq!(expanded, "see [Response #99: unknown...]");
    }

    #[tokio::test]
    async fn expand_response_refs_no_refs() {
        let app = app_with_exchanges().await;
        let expanded = app.expand_response_refs("plain text");
        assert_eq!(expanded, "plain text");
    }

    #[tokio::test]
    async fn expand_response_refs_multiple() {
        let app = app_with_exchanges().await;
        let expanded =
            app.expand_response_refs("[Response #1: hello...] and [Response #3: total 42...]");
        assert_eq!(expanded, "hello and total 42\ndrwxr-xr-x 2 user user 4096");
    }

    #[tokio::test]
    async fn truncate_preview_short() {
        assert_eq!(truncate_preview("hello", 50), "hello...");
    }

    #[tokio::test]
    async fn truncate_preview_long() {
        let long = "a".repeat(100);
        let preview = truncate_preview(&long, 50);
        assert_eq!(preview.len(), 53); // 50 chars + "..."
        assert!(preview.ends_with("..."));
    }

    #[tokio::test]
    async fn truncate_preview_multiline() {
        assert_eq!(truncate_preview("line one\nline two", 50), "line one...");
    }
}
