use std::{
    collections::hash_map::DefaultHasher,
    fmt::Write,
    hash::{Hash, Hasher},
    path::PathBuf,
};

use crate::app::AppMode;

/// Maximum number of history entries to keep.
const MAX_ENTRIES: usize = 1000;

/// A single history entry tagged with the mode it was entered in.
struct HistoryEntry {
    text: String,
    mode: AppMode,
}

/// Input history with navigation and disk persistence.
///
/// Entries are stored newest-last. The position cursor starts past the end
/// (meaning "no history selected"). Navigating up moves toward older entries,
/// down moves toward newer ones. The current input is saved as a draft when
/// the user first navigates away from it.
pub struct InputHistory {
    /// History entries, oldest first.
    entries: Vec<HistoryEntry>,
    /// Current navigation position. `entries.len()` means "draft" (no history selected).
    position: usize,
    /// Saved draft of what the user was typing before navigating history.
    draft: String,
}

#[allow(dead_code)]
impl InputHistory {
    /// Create an empty history.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            position: 0,
            draft: String::new(),
        }
    }

    /// Push a new entry (untagged, defaults to Chat mode).
    pub fn push(&mut self, entry: String) {
        self.push_tagged(entry, AppMode::Chat);
    }

    /// Push a new entry tagged with its mode. Deduplicates against the most recent entry.
    pub fn push_tagged(&mut self, entry: String, mode: AppMode) {
        if entry.trim().is_empty() {
            return;
        }
        // Deduplicate against last entry (same text AND same mode)
        if self
            .entries
            .last()
            .is_some_and(|last| last.text == entry && last.mode == mode)
        {
            self.reset_position();
            return;
        }
        self.entries.push(HistoryEntry { text: entry, mode });
        // Enforce cap
        if self.entries.len() > MAX_ENTRIES {
            self.entries.drain(..self.entries.len() - MAX_ENTRIES);
        }
        self.reset_position();
    }

    /// Navigate to the previous (older) entry.
    ///
    /// `current_input` is the current text in the input buffer.
    /// Returns the text to display, or `None` if already at the oldest entry.
    pub fn navigate_up(&mut self, current_input: &str) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }
        // Save draft when navigating away from the input area
        if self.position == self.entries.len() {
            self.draft = current_input.to_string();
        }
        if self.position > 0 {
            self.position -= 1;
            Some(&self.entries[self.position].text)
        } else {
            None
        }
    }

    /// Navigate to the next (newer) entry, or back to the draft.
    ///
    /// Returns the text to display, or `None` if already at the draft.
    pub fn navigate_down(&mut self) -> Option<&str> {
        if self.position >= self.entries.len() {
            return None;
        }
        self.position += 1;
        if self.position == self.entries.len() {
            Some(&self.draft)
        } else {
            Some(&self.entries[self.position].text)
        }
    }

    /// Navigate to the previous (older) entry matching the given mode.
    ///
    /// Skips entries that don't match the current mode.
    pub fn navigate_up_filtered(&mut self, current_input: &str, mode: AppMode) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }
        // Save draft when navigating away from the input area
        if self.position == self.entries.len() {
            self.draft = current_input.to_string();
        }
        // Search backwards for an entry matching the mode
        let mut pos = self.position;
        while pos > 0 {
            pos -= 1;
            if self.entries[pos].mode == mode && self.entries[pos].text.starts_with(&self.draft) {
                self.position = pos;
                return Some(&self.entries[pos].text);
            }
        }
        None
    }

    /// Navigate to the next (newer) entry matching the given mode, or back to draft.
    pub fn navigate_down_filtered(&mut self, mode: AppMode) -> Option<&str> {
        if self.position >= self.entries.len() {
            return None;
        }
        // Search forwards for an entry matching the mode
        let mut pos = self.position + 1;
        while pos < self.entries.len() {
            if self.entries[pos].mode == mode && self.entries[pos].text.starts_with(&self.draft) {
                self.position = pos;
                return Some(&self.entries[pos].text);
            }
            pos += 1;
        }
        // No more matching entries — return to draft
        self.position = self.entries.len();
        Some(&self.draft)
    }

    /// Reset position to "draft" (past the end of entries).
    pub fn reset_position(&mut self) {
        self.position = self.entries.len();
        self.draft.clear();
    }

    /// Load history from a JSONL file.
    ///
    /// Supports two formats per line for backwards compatibility:
    /// - New: `{"text":"...","mode":"shell"}` — JSON object with mode tag
    /// - Legacy: `"..."` — bare JSON string, loaded as Chat mode
    ///
    /// Silently ignores I/O or parse errors (history is best-effort).
    pub fn load_from_file(&mut self, path: &PathBuf) {
        let Ok(contents) = std::fs::read_to_string(path) else {
            return;
        };
        for line in contents.lines() {
            // Try new format first: {"text":"...","mode":"shell"}
            if let Ok(record) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(obj) = record.as_object()
                    && let Some(text) = obj.get("text").and_then(|v| v.as_str())
                    && !text.trim().is_empty()
                {
                    let mode = match obj.get("mode").and_then(|v| v.as_str()) {
                        Some("shell") => AppMode::Shell,
                        _ => AppMode::Chat,
                    };
                    self.entries.push(HistoryEntry {
                        text: text.to_string(),
                        mode,
                    });
                    continue;
                }
                // Legacy format: bare JSON string
                if let Some(text) = record.as_str()
                    && !text.trim().is_empty()
                {
                    self.entries.push(HistoryEntry {
                        text: text.to_string(),
                        mode: AppMode::Chat,
                    });
                }
            }
        }
        // Enforce cap after loading
        if self.entries.len() > MAX_ENTRIES {
            self.entries.drain(..self.entries.len() - MAX_ENTRIES);
        }
        self.reset_position();
    }

    /// Save history to a JSONL file.
    ///
    /// Each entry is written as a JSON object with text and mode fields.
    /// Silently ignores I/O errors (history is best-effort).
    pub fn save_to_file(&self, path: &PathBuf) {
        let mut output = String::new();
        for entry in &self.entries {
            let mode_str = match entry.mode {
                AppMode::Chat => "chat",
                AppMode::Shell => "shell",
            };
            if let (Ok(text_json), Ok(mode_json)) = (
                serde_json::to_string(&entry.text),
                serde_json::to_string(mode_str),
            ) {
                let _ = writeln!(output, "{{\"text\":{text_json},\"mode\":{mode_json}}}");
            }
        }
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, output);
    }

    /// The number of entries in history.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the history is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the text of all entries (oldest first).
    pub fn entries(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.text.as_str()).collect()
    }

    /// Whether the navigation position is at the draft (no history entry selected).
    pub fn is_at_draft(&self) -> bool {
        self.position >= self.entries.len()
    }

    /// Find the most recent history entry matching the given prefix and mode.
    ///
    /// Scans entries in reverse (newest first), filtered by mode. Returns the
    /// full text of the first entry that starts with `prefix` but isn't an exact
    /// match. Returns `None` for an empty prefix or when nothing matches.
    pub fn prefix_match(&self, prefix: &str, mode: AppMode) -> Option<&str> {
        self.prefix_match_nth(prefix, mode, 0)
    }

    /// Find the Nth prefix-matching history entry (0 = most recent).
    ///
    /// Like `prefix_match` but skips the first `n` matches to allow cycling
    /// through older entries with the same prefix.
    pub fn prefix_match_nth(&self, prefix: &str, mode: AppMode, n: usize) -> Option<&str> {
        if prefix.is_empty() {
            return None;
        }
        self.entries
            .iter()
            .rev()
            .filter(|e| e.mode == mode && e.text.starts_with(prefix) && e.text != prefix)
            .nth(n)
            .map(|e| e.text.as_str())
    }

    /// Get entries filtered by mode, newest first, as `(preview, full_text)` tuples.
    ///
    /// The preview is the first line of the entry, truncated with " ..." if multiline.
    /// Returns at most `limit` entries.
    pub fn entries_for_mode(&self, mode: AppMode, limit: usize) -> Vec<(String, String)> {
        self.entries
            .iter()
            .rev()
            .filter(|e| e.mode == mode)
            .take(limit)
            .map(|e| {
                let first_line = e.text.lines().next().unwrap_or("");
                let preview = if e.text.contains('\n') {
                    format!("{first_line} ...")
                } else {
                    first_line.to_string()
                };
                (preview, e.text.clone())
            })
            .collect()
    }
}

/// Get the path for a per-directory history file.
///
/// History is stored under `<config>/tui-history/<dir_name>-<hash>.jsonl`
/// so each working directory gets its own history. The hash avoids
/// collisions between directories with the same leaf name.
///
/// Returns `None` if the config or working directory cannot be determined.
pub fn history_file_path() -> Option<PathBuf> {
    let config = stencila_dirs::get_app_dir(stencila_dirs::DirType::Config, true).ok()?;
    let cwd = std::env::current_dir().ok()?;

    let dir_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let mut hasher = DefaultHasher::new();
    cwd.hash(&mut hasher);
    let hash = hasher.finish();

    Some(config.join(format!("tui-history/{dir_name}-{hash:016x}.jsonl")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_navigate() {
        let mut history = InputHistory::new();
        history.push("first".to_string());
        history.push("second".to_string());
        history.push("third".to_string());

        assert_eq!(history.len(), 3);

        // Navigate up from draft
        assert_eq!(history.navigate_up("current"), Some("third"));
        assert_eq!(history.navigate_up("current"), Some("second"));
        assert_eq!(history.navigate_up("current"), Some("first"));
        // At oldest, returns None
        assert_eq!(history.navigate_up("current"), None);

        // Navigate back down
        assert_eq!(history.navigate_down(), Some("second"));
        assert_eq!(history.navigate_down(), Some("third"));
        // Back to draft
        assert_eq!(history.navigate_down(), Some("current"));
        // Already at draft
        assert_eq!(history.navigate_down(), None);
    }

    #[test]
    fn draft_preserved() {
        let mut history = InputHistory::new();
        history.push("old".to_string());

        // Navigate up saves the draft
        let _ = history.navigate_up("my draft");
        // Navigate back down restores it
        assert_eq!(history.navigate_down(), Some("my draft"));
    }

    #[test]
    fn empty_history_navigation() {
        let mut history = InputHistory::new();
        assert_eq!(history.navigate_up("anything"), None);
        assert_eq!(history.navigate_down(), None);
    }

    #[test]
    fn deduplicates_last() {
        let mut history = InputHistory::new();
        history.push("hello".to_string());
        history.push("hello".to_string());
        assert_eq!(history.len(), 1);

        // Different entry is kept
        history.push("world".to_string());
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn dedup_respects_mode() {
        let mut history = InputHistory::new();
        history.push_tagged("ls".to_string(), AppMode::Chat);
        // Same text but different mode — should NOT be deduplicated
        history.push_tagged("ls".to_string(), AppMode::Shell);
        assert_eq!(history.len(), 2);

        // Same text and same mode — should be deduplicated
        history.push_tagged("ls".to_string(), AppMode::Shell);
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn ignores_blank() {
        let mut history = InputHistory::new();
        history.push(String::new());
        history.push("   ".to_string());
        history.push("\n".to_string());
        assert!(history.is_empty());
    }

    #[test]
    fn caps_at_max() {
        let mut history = InputHistory::new();
        for i in 0..1100 {
            history.push(format!("entry {i}"));
        }
        assert_eq!(history.len(), MAX_ENTRIES);
        // Oldest entries were dropped
        assert_eq!(history.entries()[0], "entry 100");
    }

    #[test]
    fn reset_position_after_push() {
        let mut history = InputHistory::new();
        history.push("first".to_string());
        let _ = history.navigate_up("");
        // Push resets position
        history.push("second".to_string());
        // Next navigate_up should give "second" (the newest)
        assert_eq!(history.navigate_up(""), Some("second"));
    }

    #[test]
    fn persistence_round_trip() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("test_history.jsonl");

        let mut history = InputHistory::new();
        history.push("single line".to_string());
        history.push("multi\nline\nentry".to_string());
        history.push("with \"quotes\"".to_string());
        history.save_to_file(&path);

        let mut loaded = InputHistory::new();
        loaded.load_from_file(&path);
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded.entries()[0], "single line");
        assert_eq!(loaded.entries()[1], "multi\nline\nentry");
        assert_eq!(loaded.entries()[2], "with \"quotes\"");
    }

    #[test]
    fn persistence_preserves_mode() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("test_mode_history.jsonl");

        let mut history = InputHistory::new();
        history.push_tagged("chat msg".to_string(), AppMode::Chat);
        history.push_tagged("ls -la".to_string(), AppMode::Shell);
        history.push_tagged("echo hi".to_string(), AppMode::Shell);
        history.save_to_file(&path);

        let mut loaded = InputHistory::new();
        loaded.load_from_file(&path);
        assert_eq!(loaded.len(), 3);

        // Shell-filtered navigation should find only shell entries
        assert_eq!(
            loaded.navigate_up_filtered("", AppMode::Shell),
            Some("echo hi")
        );
        assert_eq!(
            loaded.navigate_up_filtered("", AppMode::Shell),
            Some("ls -la")
        );
        assert_eq!(loaded.navigate_up_filtered("", AppMode::Shell), None);
    }

    #[test]
    fn persistence_loads_legacy_format() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("test_legacy.jsonl");

        // Write legacy format: bare JSON strings
        std::fs::write(&path, "\"old entry\"\n\"another one\"\n").expect("write legacy file");

        let mut loaded = InputHistory::new();
        loaded.load_from_file(&path);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.entries()[0], "old entry");
        assert_eq!(loaded.entries()[1], "another one");

        // Legacy entries should be Chat mode
        assert_eq!(
            loaded.navigate_up_filtered("", AppMode::Chat),
            Some("another one")
        );
        assert_eq!(loaded.navigate_up_filtered("", AppMode::Shell), None);
    }

    #[test]
    fn mode_filtered_navigation() {
        let mut history = InputHistory::new();
        history.push_tagged("chat1".to_string(), AppMode::Chat);
        history.push_tagged("shell1".to_string(), AppMode::Shell);
        history.push_tagged("chat2".to_string(), AppMode::Chat);
        history.push_tagged("shell2".to_string(), AppMode::Shell);

        // Navigate up in shell mode — should skip chat entries
        assert_eq!(
            history.navigate_up_filtered("", AppMode::Shell),
            Some("shell2")
        );
        assert_eq!(
            history.navigate_up_filtered("", AppMode::Shell),
            Some("shell1")
        );
        // No more shell entries
        assert_eq!(history.navigate_up_filtered("", AppMode::Shell), None);

        // Navigate down in shell mode
        assert_eq!(
            history.navigate_down_filtered(AppMode::Shell),
            Some("shell2")
        );
        // No more shell entries after shell2 — returns draft
        assert_eq!(history.navigate_down_filtered(AppMode::Shell), Some(""));
    }

    #[test]
    fn mode_filtered_preserves_draft() {
        let mut history = InputHistory::new();
        history.push_tagged("shell1".to_string(), AppMode::Shell);

        // Empty draft matches everything, so shell1 is found
        assert_eq!(
            history.navigate_up_filtered("", AppMode::Shell),
            Some("shell1")
        );
        assert_eq!(history.navigate_down_filtered(AppMode::Shell), Some(""));
    }

    #[test]
    fn mode_filtered_empty_for_mode() {
        let mut history = InputHistory::new();
        history.push_tagged("chat1".to_string(), AppMode::Chat);

        // No shell entries
        assert_eq!(history.navigate_up_filtered("", AppMode::Shell), None);
    }

    #[test]
    fn entries_for_mode_filters_and_limits() {
        let mut history = InputHistory::new();
        history.push_tagged("chat1".to_string(), AppMode::Chat);
        history.push_tagged("shell1".to_string(), AppMode::Shell);
        history.push_tagged("chat2".to_string(), AppMode::Chat);
        history.push_tagged("shell2".to_string(), AppMode::Shell);

        let chat_entries = history.entries_for_mode(AppMode::Chat, 10);
        assert_eq!(chat_entries.len(), 2);
        // Newest first
        assert_eq!(chat_entries[0].1, "chat2");
        assert_eq!(chat_entries[1].1, "chat1");

        let shell_entries = history.entries_for_mode(AppMode::Shell, 1);
        assert_eq!(shell_entries.len(), 1);
        assert_eq!(shell_entries[0].1, "shell2");
    }

    #[test]
    fn entries_for_mode_multiline_preview() {
        let mut history = InputHistory::new();
        history.push_tagged("single line".to_string(), AppMode::Chat);
        history.push_tagged("first line\nsecond line".to_string(), AppMode::Chat);

        let entries = history.entries_for_mode(AppMode::Chat, 10);
        // Newest first: multiline entry first
        assert_eq!(entries[0].0, "first line ...");
        assert_eq!(entries[0].1, "first line\nsecond line");
        // Single line: preview equals full text
        assert_eq!(entries[1].0, "single line");
        assert_eq!(entries[1].1, "single line");
    }

    #[test]
    fn entries_for_mode_empty() {
        let history = InputHistory::new();
        let entries = history.entries_for_mode(AppMode::Chat, 10);
        assert!(entries.is_empty());
    }

    #[test]
    fn prefix_match_returns_newest() {
        let mut history = InputHistory::new();
        history.push_tagged("cargo build".to_string(), AppMode::Shell);
        history.push_tagged("cargo test".to_string(), AppMode::Shell);
        history.push_tagged("cargo test --release".to_string(), AppMode::Shell);

        // Should match newest entry starting with "cargo t"
        assert_eq!(
            history.prefix_match("cargo t", AppMode::Shell),
            Some("cargo test --release")
        );
    }

    #[test]
    fn prefix_match_filters_by_mode() {
        let mut history = InputHistory::new();
        history.push_tagged("cargo build".to_string(), AppMode::Shell);
        history.push_tagged("cargo build is great".to_string(), AppMode::Chat);

        // Shell mode should find the shell entry
        assert_eq!(
            history.prefix_match("cargo", AppMode::Shell),
            Some("cargo build")
        );
        // Chat mode should find the chat entry
        assert_eq!(
            history.prefix_match("cargo", AppMode::Chat),
            Some("cargo build is great")
        );
    }

    #[test]
    fn prefix_match_none_for_empty_prefix() {
        let mut history = InputHistory::new();
        history.push_tagged("hello".to_string(), AppMode::Chat);
        assert_eq!(history.prefix_match("", AppMode::Chat), None);
    }

    #[test]
    fn prefix_match_none_for_exact_match_only() {
        let mut history = InputHistory::new();
        history.push_tagged("hello".to_string(), AppMode::Chat);
        // Exact match is not returned
        assert_eq!(history.prefix_match("hello", AppMode::Chat), None);
    }

    #[test]
    fn prefix_match_none_when_nothing_matches() {
        let mut history = InputHistory::new();
        history.push_tagged("hello".to_string(), AppMode::Chat);
        assert_eq!(history.prefix_match("xyz", AppMode::Chat), None);
    }

    #[test]
    fn prefix_match_nth_cycles_through_matches() {
        let mut history = InputHistory::new();
        history.push_tagged("cargo build".to_string(), AppMode::Shell);
        history.push_tagged("rustc main.rs".to_string(), AppMode::Shell);
        history.push_tagged("cargo test".to_string(), AppMode::Shell);
        history.push_tagged("cargo bench".to_string(), AppMode::Shell);

        // n=0: most recent match
        assert_eq!(
            history.prefix_match_nth("cargo", AppMode::Shell, 0),
            Some("cargo bench")
        );
        // n=1: second most recent
        assert_eq!(
            history.prefix_match_nth("cargo", AppMode::Shell, 1),
            Some("cargo test")
        );
        // n=2: third most recent
        assert_eq!(
            history.prefix_match_nth("cargo", AppMode::Shell, 2),
            Some("cargo build")
        );
        // n=3: no more matches
        assert_eq!(history.prefix_match_nth("cargo", AppMode::Shell, 3), None);
    }

    #[test]
    fn is_at_draft_reflects_navigation_state() {
        let mut history = InputHistory::new();
        history.push_tagged("entry".to_string(), AppMode::Chat);

        assert!(history.is_at_draft());
        let _ = history.navigate_up_filtered("", AppMode::Chat);
        assert!(!history.is_at_draft());
        let _ = history.navigate_down_filtered(AppMode::Chat);
        assert!(history.is_at_draft());
    }

    #[test]
    fn prefix_filtered_navigation() {
        let mut history = InputHistory::new();
        history.push_tagged("rustc main.rs".to_string(), AppMode::Shell);
        history.push_tagged("cargo build".to_string(), AppMode::Shell);
        history.push_tagged("rustc --version".to_string(), AppMode::Shell);
        history.push_tagged("cargo test".to_string(), AppMode::Shell);

        // Type "cargo" then navigate up — should only see cargo entries
        assert_eq!(
            history.navigate_up_filtered("cargo", AppMode::Shell),
            Some("cargo test")
        );
        assert_eq!(
            history.navigate_up_filtered("cargo", AppMode::Shell),
            Some("cargo build")
        );
        // No more cargo entries
        assert_eq!(history.navigate_up_filtered("cargo", AppMode::Shell), None);

        // Navigate back down through cargo entries
        assert_eq!(
            history.navigate_down_filtered(AppMode::Shell),
            Some("cargo test")
        );
        // Past last cargo entry — back to draft
        assert_eq!(
            history.navigate_down_filtered(AppMode::Shell),
            Some("cargo")
        );
    }

    #[test]
    fn empty_prefix_shows_all() {
        let mut history = InputHistory::new();
        history.push_tagged("alpha".to_string(), AppMode::Shell);
        history.push_tagged("beta".to_string(), AppMode::Shell);
        history.push_tagged("gamma".to_string(), AppMode::Shell);

        // Empty input — all entries match (preserves current behavior)
        assert_eq!(
            history.navigate_up_filtered("", AppMode::Shell),
            Some("gamma")
        );
        assert_eq!(
            history.navigate_up_filtered("", AppMode::Shell),
            Some("beta")
        );
        assert_eq!(
            history.navigate_up_filtered("", AppMode::Shell),
            Some("alpha")
        );
        assert_eq!(history.navigate_up_filtered("", AppMode::Shell), None);
    }

    #[test]
    fn prefix_filtered_draft_preserved() {
        let mut history = InputHistory::new();
        history.push_tagged("cargo build".to_string(), AppMode::Shell);
        history.push_tagged("cargo test".to_string(), AppMode::Shell);

        // Navigate up with prefix "cargo t" — saves it as draft
        assert_eq!(
            history.navigate_up_filtered("cargo t", AppMode::Shell),
            Some("cargo test")
        );
        // "cargo build" doesn't start with "cargo t", so skip to draft
        assert_eq!(
            history.navigate_up_filtered("cargo t", AppMode::Shell),
            None
        );

        // Navigate down returns to draft
        assert_eq!(
            history.navigate_down_filtered(AppMode::Shell),
            Some("cargo t")
        );
    }
}
