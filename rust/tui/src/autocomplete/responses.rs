use std::ops::Range;

/// A response autocomplete candidate with its exchange number and preview text.
#[derive(Debug, Clone)]
pub struct ResponseCandidate {
    /// The exchange number (1-based, as displayed in the UI).
    pub number: usize,
    /// Label describing the source (agent name or "shell").
    pub label: String,
    /// Preview of the response (first line of output).
    pub preview: String,
    /// Color for the number and label (agent color or shell color).
    pub color: ratatui::style::Color,
}

/// Result of accepting a response autocomplete candidate.
pub struct ResponseAcceptResult {
    /// Byte range in the input to replace.
    pub range: Range<usize>,
    /// Text to insert (e.g. `[Response #5: preview...]`).
    pub text: String,
}

/// State for the response autocomplete popup triggered by `$`.
pub struct ResponsesState {
    /// Whether the popup is currently visible.
    visible: bool,
    /// Filtered candidates matching the current query.
    candidates: Vec<ResponseCandidate>,
    /// Currently selected index within `candidates`.
    selected: usize,
    /// Byte range of the `#` token (including any digit query) in the input.
    token_range: Range<usize>,
}

impl ResponsesState {
    /// Create a new hidden responses autocomplete state.
    pub fn new() -> Self {
        Self {
            visible: false,
            candidates: Vec::new(),
            selected: 0,
            token_range: 0..0,
        }
    }

    /// Whether the popup is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// The current list of matching candidates.
    pub fn candidates(&self) -> &[ResponseCandidate] {
        &self.candidates
    }

    /// The currently selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Update the responses state based on current input and cursor position.
    ///
    /// `exchanges` is a pre-built list of [`ResponseCandidate`]s, ordered newest first.
    pub fn update(&mut self, input: &str, cursor: usize, exchanges: &[ResponseCandidate]) {
        if exchanges.is_empty() {
            self.visible = false;
            return;
        }

        let Some((range, query)) = find_dollar_token(input, cursor) else {
            self.visible = false;
            return;
        };

        self.token_range = range;

        // Filter candidates by number prefix
        self.candidates = exchanges
            .iter()
            .filter(|c| {
                if query.is_empty() {
                    true
                } else {
                    c.number.to_string().starts_with(query)
                }
            })
            .cloned()
            .collect();

        self.visible = !self.candidates.is_empty();

        // Clamp selection
        if self.selected >= self.candidates.len() {
            self.selected = 0;
        }
    }

    /// Move selection to the next candidate, wrapping around.
    pub fn select_next(&mut self) {
        if !self.candidates.is_empty() {
            self.selected = (self.selected + 1) % self.candidates.len();
        }
    }

    /// Move selection to the previous candidate, wrapping around.
    pub fn select_prev(&mut self) {
        if !self.candidates.is_empty() {
            self.selected = if self.selected == 0 {
                self.candidates.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    /// Accept the currently selected candidate.
    ///
    /// Returns the byte range to replace and the text to insert, or `None` if
    /// nothing to accept.
    pub fn accept(&mut self) -> Option<ResponseAcceptResult> {
        if !self.visible || self.candidates.is_empty() {
            return None;
        }

        let candidate = &self.candidates[self.selected];
        // Truncate preview for the inserted reference text
        let short: String = candidate
            .preview
            .trim_start()
            .chars()
            .map(|c| if c == '\n' { ' ' } else { c })
            .take(40)
            .collect();
        let ellipsis = if candidate.preview.trim_start().chars().count() > 40 {
            "\u{2026}"
        } else {
            ""
        };
        let text = format!("[Response #{}: {short}{ellipsis}]", candidate.number);
        let range = self.token_range.clone();

        self.dismiss();

        Some(ResponseAcceptResult { range, text })
    }

    /// Hide the popup and reset state.
    pub fn dismiss(&mut self) {
        self.visible = false;
        self.candidates.clear();
        self.selected = 0;
        self.token_range = 0..0;
    }
}

/// Find a `$` token at or before the cursor in the input.
///
/// Returns the byte range of the token (from `$` through any trailing digits)
/// and the digit query string after `$`. Returns `None` if no valid `$` token
/// is found at the cursor position.
///
/// A valid token is `$` optionally followed by digits only (no other characters).
/// The cursor must be within or immediately after the token.
fn find_dollar_token(input: &str, cursor: usize) -> Option<(Range<usize>, &str)> {
    // Search backwards from cursor for `$`
    let before = &input[..cursor];
    let dollar_pos = before.rfind('$')?;

    // Everything between `$` and cursor must be digits (or empty)
    let after_dollar = &input[dollar_pos + 1..cursor];
    if !after_dollar.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    // Token extends from `$` through any contiguous digits after cursor too
    let mut end = cursor;
    for c in input[cursor..].chars() {
        if c.is_ascii_digit() {
            end += c.len_utf8();
        } else {
            break;
        }
    }

    Some((dollar_pos..end, after_dollar))
}

#[cfg(test)]
mod tests {
    use super::*;

    use ratatui::style::Color;

    fn sample_exchanges() -> Vec<ResponseCandidate> {
        vec![
            ResponseCandidate {
                number: 3,
                label: "default".to_string(),
                preview: "latest response...".to_string(),
                color: Color::Blue,
            },
            ResponseCandidate {
                number: 2,
                label: "default".to_string(),
                preview: "middle response...".to_string(),
                color: Color::Blue,
            },
            ResponseCandidate {
                number: 1,
                label: "shell".to_string(),
                preview: "first response...".to_string(),
                color: Color::Yellow,
            },
        ]
    }

    #[test]
    fn initially_hidden() {
        let state = ResponsesState::new();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn dollar_triggers_all_candidates() {
        let mut state = ResponsesState::new();
        state.update("$", 1, &sample_exchanges());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 3);
    }

    #[test]
    fn dollar_with_digit_filters() {
        let mut state = ResponsesState::new();
        state.update("$1", 2, &sample_exchanges());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].number, 1);
    }

    #[test]
    fn dollar_no_match_hides() {
        let mut state = ResponsesState::new();
        state.update("$9", 2, &sample_exchanges());
        assert!(!state.is_visible());
    }

    #[test]
    fn dollar_mid_input() {
        let mut state = ResponsesState::new();
        state.update("look at $2", 10, &sample_exchanges());
        assert!(state.is_visible());
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.candidates()[0].number, 2);
    }

    #[test]
    fn no_dollar_stays_hidden() {
        let mut state = ResponsesState::new();
        state.update("hello", 5, &sample_exchanges());
        assert!(!state.is_visible());
    }

    #[test]
    fn dollar_with_non_digit_stays_hidden() {
        let mut state = ResponsesState::new();
        state.update("$abc", 4, &sample_exchanges());
        assert!(!state.is_visible());
    }

    #[test]
    fn empty_exchanges_stays_hidden() {
        let mut state = ResponsesState::new();
        state.update("$", 1, &[]);
        assert!(!state.is_visible());
    }

    #[test]
    fn select_next_wraps() {
        let mut state = ResponsesState::new();
        state.update("$", 1, &sample_exchanges());
        assert_eq!(state.selected(), 0);

        state.select_next();
        assert_eq!(state.selected(), 1);
        state.select_next();
        assert_eq!(state.selected(), 2);
        state.select_next();
        assert_eq!(state.selected(), 0); // wrapped
    }

    #[test]
    fn select_prev_wraps() {
        let mut state = ResponsesState::new();
        state.update("$", 1, &sample_exchanges());
        assert_eq!(state.selected(), 0);

        state.select_prev();
        assert_eq!(state.selected(), 2); // wrapped to last
    }

    #[test]
    fn accept_returns_result() {
        let mut state = ResponsesState::new();
        state.update("$2", 2, &sample_exchanges());
        assert!(state.is_visible());

        let result = state.accept();
        assert!(result.is_some());
        let result = result.expect("accept should return Some");
        assert_eq!(result.range, 0..2);
        assert_eq!(result.text, "[Response #2: middle response...]");
        assert!(!state.is_visible());
    }

    #[test]
    fn candidate_has_label() {
        let mut state = ResponsesState::new();
        state.update("$", 1, &sample_exchanges());
        assert_eq!(state.candidates()[0].label, "default");
        assert_eq!(state.candidates()[2].label, "shell");
    }

    #[test]
    fn accept_when_hidden_returns_none() {
        let mut state = ResponsesState::new();
        assert!(state.accept().is_none());
    }

    #[test]
    fn dismiss_resets() {
        let mut state = ResponsesState::new();
        state.update("$", 1, &sample_exchanges());
        assert!(state.is_visible());

        state.dismiss();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn selection_clamped_on_filter() {
        let mut state = ResponsesState::new();
        state.update("$", 1, &sample_exchanges());
        state.select_next();
        state.select_next();
        assert_eq!(state.selected(), 2);

        // Re-filter to fewer candidates
        state.update("$1", 2, &sample_exchanges());
        assert_eq!(state.candidates().len(), 1);
        assert_eq!(state.selected(), 0);
    }

    // --- find_dollar_token tests ---

    #[test]
    fn find_dollar_bare() {
        let result = find_dollar_token("$", 1);
        assert_eq!(result, Some((0..1, "")));
    }

    #[test]
    fn find_dollar_with_digits() {
        let result = find_dollar_token("$12", 3);
        assert_eq!(result, Some((0..3, "12")));
    }

    #[test]
    fn find_dollar_mid_input() {
        // Cursor right after `$` (position 5)
        let result = find_dollar_token("see $3 here", 5);
        assert_eq!(result, Some((4..6, "")));
    }

    #[test]
    fn find_dollar_mid_input_with_digit() {
        let result = find_dollar_token("see $3 here", 6);
        assert_eq!(result, Some((4..6, "3")));
    }

    #[test]
    fn find_dollar_none_without_dollar() {
        let result = find_dollar_token("hello", 5);
        assert!(result.is_none());
    }

    #[test]
    fn find_dollar_none_with_non_digits() {
        let result = find_dollar_token("$abc", 4);
        assert!(result.is_none());
    }

    #[test]
    fn find_dollar_cursor_right_after_dollar() {
        let result = find_dollar_token("text $", 6);
        assert_eq!(result, Some((5..6, "")));
    }

    #[test]
    fn find_dollar_cursor_between_digits() {
        // Input: "$12", cursor at position 2 (between 1 and 2)
        let result = find_dollar_token("$12", 2);
        // Token should extend through all digits
        assert_eq!(result, Some((0..3, "1")));
    }
}
