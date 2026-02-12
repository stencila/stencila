use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::PathBuf,
};

/// Maximum number of history entries to keep.
const MAX_ENTRIES: usize = 1000;

/// Input history with navigation and disk persistence.
///
/// Entries are stored newest-last. The position cursor starts past the end
/// (meaning "no history selected"). Navigating up moves toward older entries,
/// down moves toward newer ones. The current input is saved as a draft when
/// the user first navigates away from it.
pub struct InputHistory {
    /// History entries, oldest first.
    entries: Vec<String>,
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

    /// Push a new entry. Deduplicates against the most recent entry.
    pub fn push(&mut self, entry: String) {
        if entry.trim().is_empty() {
            return;
        }
        // Deduplicate against last entry
        if self.entries.last().is_some_and(|last| last == &entry) {
            self.reset_position();
            return;
        }
        self.entries.push(entry);
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
            Some(&self.entries[self.position])
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
            Some(&self.entries[self.position])
        }
    }

    /// Reset position to "draft" (past the end of entries).
    pub fn reset_position(&mut self) {
        self.position = self.entries.len();
        self.draft.clear();
    }

    /// Load history from a JSONL file.
    ///
    /// Each line is a JSON-encoded string, allowing multiline entries.
    /// Silently ignores I/O or parse errors (history is best-effort).
    pub fn load_from_file(&mut self, path: &PathBuf) {
        let Ok(contents) = std::fs::read_to_string(path) else {
            return;
        };
        for line in contents.lines() {
            if let Ok(entry) = serde_json::from_str::<String>(line)
                && !entry.trim().is_empty()
            {
                self.entries.push(entry);
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
    /// Each entry is written as a JSON-encoded string on its own line.
    /// Silently ignores I/O errors (history is best-effort).
    pub fn save_to_file(&self, path: &PathBuf) {
        let mut output = String::new();
        for entry in &self.entries {
            if let Ok(json) = serde_json::to_string(entry) {
                output.push_str(&json);
                output.push('\n');
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

    /// Iterate over entries (oldest first).
    pub fn entries(&self) -> &[String] {
        &self.entries
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
}
