use std::path::{Path, PathBuf};

/// A file candidate shown in the autocomplete popup.
#[derive(Debug, Clone)]
pub struct FileCandidate {
    /// Filename shown in the popup (directories have trailing `/`).
    display: String,
    /// Relative path for insertion (directories have trailing `/`).
    path: String,
    /// Whether this candidate is a directory.
    is_dir: bool,
}

impl FileCandidate {
    /// The display name (filename) shown in the popup.
    pub fn display(&self) -> &str {
        &self.display
    }

    /// The relative path to insert into the input buffer.
    pub fn path(&self) -> &str {
        &self.path
    }
}

/// The mode of file autocomplete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileMode {
    /// Triggered by `@` — searches attachable files and directories by name.
    AtSearch,
    /// Triggered by `~/`, `./`, `../` — lists directory contents.
    PathCompletion,
}

/// Result returned when a file candidate is accepted.
pub struct FileAcceptResult {
    /// Byte range in the input buffer to replace.
    pub range: std::ops::Range<usize>,
    /// Text to insert in place of the range.
    pub text: String,
    /// Whether the popup should immediately refresh (directory drill-down).
    pub refresh: bool,
}

/// State for the file autocomplete popup.
pub struct FilesState {
    /// Whether the popup is currently visible.
    visible: bool,
    /// Which mode is active.
    mode: FileMode,
    /// Filtered candidates matching the current query.
    candidates: Vec<FileCandidate>,
    /// Currently selected index within `candidates`.
    selected: usize,
    /// Lazily populated cache of all attachable files and directories (for `@` search).
    cached_files: Option<Vec<FileCandidate>>,
    /// Byte range in the input buffer that the current token occupies.
    token_range: std::ops::Range<usize>,
    /// Directory prefix for path completion (e.g., `./src/`), prepended on accept.
    path_dir_prefix: String,
}

/// Maximum number of candidates to display.
const MAX_DISPLAY: usize = 20;

/// Maximum depth for walking the file tree.
const MAX_WALK_DEPTH: usize = 8;

/// Maximum number of entries to collect during a walk.
const MAX_WALK_ENTRIES: usize = 5000;

impl FilesState {
    /// Create a new hidden file autocomplete state.
    pub fn new() -> Self {
        Self {
            visible: false,
            mode: FileMode::AtSearch,
            candidates: Vec::new(),
            selected: 0,
            cached_files: None,
            token_range: 0..0,
            path_dir_prefix: String::new(),
        }
    }

    /// Whether the popup is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// The current list of matching candidates.
    pub fn candidates(&self) -> &[FileCandidate] {
        &self.candidates
    }

    /// The currently selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Whether the current mode is `@` search.
    pub fn is_at_search(&self) -> bool {
        self.mode == FileMode::AtSearch
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

    /// Hide the popup and reset state.
    ///
    /// The file cache is intentionally preserved across dismiss/reopen cycles
    /// to avoid repeated full tree walks on large repos.
    pub fn dismiss(&mut self) {
        self.visible = false;
        self.candidates.clear();
        self.selected = 0;
        self.token_range = 0..0;
        self.path_dir_prefix.clear();
    }

    /// Update the autocomplete state based on the current input and cursor position.
    ///
    /// Checks for `@` search tokens first (higher priority), then path tokens.
    /// Called after every keystroke.
    pub fn update(&mut self, input: &str, cursor: usize) {
        // @ search takes priority
        if let Some((token_start, query)) = find_at_token(input, cursor) {
            self.update_at_mode(input, token_start, query, cursor);
            return;
        }

        // If previously in @ mode, hide
        if self.mode == FileMode::AtSearch {
            self.visible = false;
            self.candidates.clear();
            self.selected = 0;
        }

        // Path completion
        if let Some((token_start, dir_part, file_prefix)) = find_path_token(input, cursor) {
            self.update_path_mode(input, token_start, dir_part, file_prefix, cursor);
            return;
        }

        // No token found — hide if in path mode
        if self.mode == FileMode::PathCompletion {
            self.visible = false;
            self.candidates.clear();
            self.selected = 0;
        }
    }

    /// Update state for `@` search mode.
    fn update_at_mode(&mut self, input: &str, token_start: usize, query: &str, cursor: usize) {
        self.mode = FileMode::AtSearch;
        self.token_range = token_start..token_end(input, cursor);
        self.path_dir_prefix.clear();

        // Lazily build the file cache
        if self.cached_files.is_none() {
            self.cached_files = Some(scan_attachable_files());
        }

        // Check if query contains a directory prefix (e.g., "src/main")
        let (dir_prefix, name_filter) = if let Some(last_slash) = query.rfind('/') {
            (&query[..=last_slash], &query[last_slash + 1..])
        } else {
            ("", query)
        };

        let dir_prefix_lower = dir_prefix.to_lowercase();
        let name_filter_lower = name_filter.to_lowercase();

        self.candidates = self
            .cached_files
            .as_ref()
            .map(|files| {
                files
                    .iter()
                    .filter(|f| {
                        if dir_prefix.is_empty() {
                            // No dir prefix: substring match on display name
                            name_filter_lower.is_empty()
                                || f.display.to_lowercase().contains(&name_filter_lower)
                        } else {
                            // Has dir prefix: show direct children of that directory
                            let path_lower = f.path.to_lowercase();
                            if !path_lower.starts_with(&dir_prefix_lower)
                                || path_lower == dir_prefix_lower
                            {
                                return false;
                            }
                            // Only direct children
                            let remaining = &f.path[dir_prefix.len()..];
                            let slash_count = remaining.matches('/').count();
                            let is_direct =
                                slash_count == 0 || (slash_count == 1 && remaining.ends_with('/'));
                            is_direct
                                && (name_filter_lower.is_empty()
                                    || f.display.to_lowercase().contains(&name_filter_lower))
                        }
                    })
                    .take(MAX_DISPLAY)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        self.apply_visibility();
    }

    /// Update state for path completion mode.
    fn update_path_mode(
        &mut self,
        input: &str,
        token_start: usize,
        dir_part: &str,
        file_prefix: &str,
        cursor: usize,
    ) {
        self.mode = FileMode::PathCompletion;
        self.token_range = token_start..token_end(input, cursor);
        self.path_dir_prefix = dir_part.to_string();

        let resolved_dir = resolve_dir(dir_part);
        let all = scan_directory(&resolved_dir);

        let prefix_lower = file_prefix.to_lowercase();
        self.candidates = all
            .into_iter()
            .filter(|f| {
                prefix_lower.is_empty() || f.display.to_lowercase().starts_with(&prefix_lower)
            })
            .take(MAX_DISPLAY)
            .collect();

        self.apply_visibility();
    }

    /// Show/hide the popup based on whether there are candidates.
    fn apply_visibility(&mut self) {
        if self.candidates.is_empty() {
            self.visible = false;
        } else {
            if self.selected >= self.candidates.len() {
                self.selected = 0;
            }
            self.visible = true;
        }
    }

    /// Accept the selected candidate via Tab.
    ///
    /// For directories: inserts the directory path and sets `refresh = true`
    /// so the caller can re-trigger `update()` for drill-down.
    /// For files: inserts the file path with a trailing space.
    pub fn accept_tab(&mut self, use_at_prefix: bool) -> Option<FileAcceptResult> {
        if !self.visible || self.candidates.is_empty() {
            return None;
        }

        let candidate = &self.candidates[self.selected];
        let is_dir = candidate.is_dir;
        let at = if use_at_prefix { "@" } else { "" };

        let text = match self.mode {
            FileMode::AtSearch => {
                if is_dir {
                    // Drill-down: @dir_path/ (no trailing space)
                    format!("{at}{}", candidate.path)
                } else {
                    format!("{at}{} ", candidate.path)
                }
            }
            FileMode::PathCompletion => {
                let full = format!("{}{}", self.path_dir_prefix, candidate.path);
                if is_dir {
                    // Drill-down: keep raw path so path-completion continues
                    full
                } else if use_at_prefix {
                    // Chat: add @ prefix so all file refs use the same marker
                    format!("@{full} ")
                } else {
                    // Shell: keep the raw path
                    format!("{full} ")
                }
            }
        };

        let range = self.token_range.clone();
        self.dismiss();

        Some(FileAcceptResult {
            range,
            text,
            refresh: is_dir,
        })
    }

    /// Accept the selected candidate via Enter.
    ///
    /// Always inserts the path with a trailing space and dismisses.
    pub fn accept_enter(&mut self, use_at_prefix: bool) -> Option<FileAcceptResult> {
        if !self.visible || self.candidates.is_empty() {
            return None;
        }

        let candidate = &self.candidates[self.selected];
        let at = if use_at_prefix { "@" } else { "" };

        let text = match self.mode {
            FileMode::AtSearch => format!("{at}{} ", candidate.path),
            FileMode::PathCompletion => {
                format!("{at}{}{} ", self.path_dir_prefix, candidate.path)
            }
        };

        let range = self.token_range.clone();
        self.dismiss();

        Some(FileAcceptResult {
            range,
            text,
            refresh: false,
        })
    }
}

/// Find the end of the current token starting from `cursor`.
///
/// Scans forward to the next whitespace character (or end of input).
/// This ensures the full token is replaced on accept, even when the cursor
/// is in the middle of an existing token.
fn token_end(input: &str, cursor: usize) -> usize {
    let after = &input[cursor..];
    cursor + after.find(char::is_whitespace).unwrap_or(after.len())
}

/// Find an `@` token in the input near the cursor.
///
/// Returns `(token_start, query)` where `token_start` is the byte offset of `@`
/// and `query` is the text between `@` and `cursor`.
///
/// The `@` must be at position 0 or preceded by whitespace, and the query must
/// not contain whitespace (to avoid matching email addresses).
fn find_at_token(input: &str, cursor: usize) -> Option<(usize, &str)> {
    let before = &input[..cursor];

    // Scan backwards for `@`
    let at_pos = before.rfind('@')?;

    // The `@` must be at the start or preceded by whitespace
    if at_pos > 0 {
        let preceding = before.as_bytes()[at_pos - 1];
        if !preceding.is_ascii_whitespace() {
            return None;
        }
    }

    let query = &before[at_pos + 1..];

    // Query must not contain whitespace
    if query.contains(char::is_whitespace) {
        return None;
    }

    Some((at_pos, query))
}

/// Find a path token in the input near the cursor.
///
/// Returns `(token_start, dir_part, file_prefix)` where:
/// - `token_start` is the byte offset of the token start
/// - `dir_part` is the directory portion (e.g. `./src/`)
/// - `file_prefix` is the partial filename after the last `/`
///
/// Only triggers for tokens starting with `~/`, `./`, or `../`.
fn find_path_token(input: &str, cursor: usize) -> Option<(usize, &str, &str)> {
    let before = &input[..cursor];

    // Find the start of the current whitespace-delimited token
    let token_start = before.rfind(char::is_whitespace).map_or(0, |pos| pos + 1);

    let token = &before[token_start..];

    // Must start with ~/, ./, or ../
    if !token.starts_with("~/") && !token.starts_with("./") && !token.starts_with("../") {
        return None;
    }

    // Split at the last `/` to get dir_part and file_prefix
    let last_slash = token.rfind('/')?;
    let dir_part = &token[..=last_slash];
    let file_prefix = &token[last_slash + 1..];

    Some((token_start, dir_part, file_prefix))
}

/// Resolve a directory part to an absolute path.
///
/// Expands `~` to the home directory.
fn resolve_dir(dir_part: &str) -> PathBuf {
    if let Some(rest) = dir_part.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }
    PathBuf::from(dir_part)
}

/// Home directory lookup (minimal reimplementation to avoid a full `dirs` dependency).
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

/// Count the depth of a relative path (number of directory components).
///
/// `"package.json"` → 0, `"src/main.rs"` → 1, `"src/models/"` → 1.
fn path_depth(path: &str) -> usize {
    path.trim_end_matches('/').matches('/').count()
}

/// Compare display names for sorting: underscore-prefixed names sort after
/// alphabetical ones, otherwise case-insensitive alphabetical.
fn cmp_display_name(a: &str, b: &str) -> std::cmp::Ordering {
    let a_underscore = a.starts_with('_');
    let b_underscore = b.starts_with('_');
    a_underscore
        .cmp(&b_underscore)
        .then_with(|| a.to_lowercase().cmp(&b.to_lowercase()))
}

/// Scan the current directory for attachable files and directories using `ignore::WalkBuilder`.
///
/// Respects `.gitignore`, skips hidden files, limits depth and count.
/// Includes both files (filtered by `is_attachable`) and directories.
fn scan_attachable_files() -> Vec<FileCandidate> {
    let Ok(cwd) = std::env::current_dir() else {
        return Vec::new();
    };

    let walker = ignore::WalkBuilder::new(&cwd)
        .max_depth(Some(MAX_WALK_DEPTH))
        .hidden(true) // skip hidden
        .git_ignore(true)
        .build();

    let mut entries: Vec<FileCandidate> = Vec::new();

    for entry in walker {
        if entries.len() >= MAX_WALK_ENTRIES {
            break;
        }
        let Ok(entry) = entry else { continue };
        let path = entry.path();

        // Skip the root directory itself
        if path == cwd {
            continue;
        }

        let is_dir = path.is_dir();

        // For files, check if attachable; directories are always included
        if !is_dir && !is_attachable(path) {
            continue;
        }

        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        let display = if is_dir {
            format!("{file_name}/")
        } else {
            file_name
        };

        let mut rel = path
            .strip_prefix(&cwd)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();

        if is_dir && !rel.ends_with('/') {
            rel.push('/');
        }

        entries.push(FileCandidate {
            display,
            path: rel,
            is_dir,
        });
    }

    // Sort: closest to cwd first, then files before directories,
    // then underscore-prefixed names last, then alphabetical.
    entries.sort_by(|a, b| {
        path_depth(&a.path)
            .cmp(&path_depth(&b.path))
            .then_with(|| a.is_dir.cmp(&b.is_dir))
            .then_with(|| cmp_display_name(&a.display, &b.display))
    });

    entries
}

/// Scan a directory's immediate contents.
///
/// Skips hidden entries. Directories get a trailing `/` in their display/path.
/// Sorted: directories first, then alphabetically.
fn scan_directory(dir: &Path) -> Vec<FileCandidate> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut files: Vec<FileCandidate> = Vec::new();

    for entry in entries {
        let Ok(entry) = entry else { continue };
        let name = entry.file_name().to_string_lossy().into_owned();

        // Skip hidden entries
        if name.starts_with('.') {
            continue;
        }

        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

        let display = if is_dir { format!("{name}/") } else { name };

        files.push(FileCandidate {
            path: display.clone(),
            display,
            is_dir,
        });
    }

    // Sort: directories first (navigation), then alphabetically
    // with underscore-prefixed names after alphabetical ones.
    files.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| cmp_display_name(&a.display, &b.display))
    });

    files
}

/// Whether a file path refers to an attachable file type.
///
/// Checks by extension (allowlist) and by known extensionless filenames.
fn is_attachable(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };

    // Exclude known lock files
    if is_lock_file(file_name) {
        return false;
    }

    // Known extensionless files
    if matches!(
        file_name,
        "Dockerfile"
            | "Makefile"
            | "Justfile"
            | "Vagrantfile"
            | "Gemfile"
            | "Rakefile"
            | "Procfile"
            | "Brewfile"
            | "Taskfile"
            | "Earthfile"
            | "CMakeLists.txt"
    ) {
        return true;
    }

    let ext = match path.extension().and_then(|e| e.to_str()) {
        Some(e) => e.to_lowercase(),
        None => return false,
    };

    matches!(
        ext.as_str(),
        // Code
        "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "go" | "java" | "c" | "cpp" | "h" | "hpp"
        | "cs" | "rb" | "php" | "swift" | "kt" | "kts" | "sh" | "bash" | "zsh" | "fish"
        | "lua" | "r" | "jl" | "pl" | "pm" | "ex" | "exs" | "erl" | "hrl" | "hs" | "ml"
        | "mli" | "fs" | "fsi" | "fsx" | "scala" | "sc" | "clj" | "cljs" | "cljc" | "edn"
        | "el" | "vim" | "sql" | "graphql" | "gql" | "proto" | "thrift" | "zig" | "nim"
        | "v" | "sv" | "vhdl" | "vhd" | "dart" | "groovy" | "gradle" | "cmake"
        // Text / config
        | "md" | "txt" | "toml" | "yaml" | "yml" | "json" | "jsonc" | "json5" | "xml"
        | "csv" | "tsv" | "html" | "htm" | "css" | "scss" | "sass" | "less" | "ini" | "cfg"
        | "conf" | "env" | "envrc" | "properties" | "plist" | "hcl" | "tf" | "tfvars"
        | "nix" | "dhall" | "ron" | "kdl"
        // Images
        | "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "bmp" | "ico"
        // Documents
        | "pdf"
        // Stencila
        | "smd" | "qmd" | "myst" | "llmd" | "ipynb"
        // Build / project files
        | "dockerfile" | "containerfile" | "makefile" | "justfile"
        // Template / markup
        | "tex" | "rst" | "adoc" | "org" | "wiki"
    )
}

/// Whether a filename is a known lock file that should be excluded.
fn is_lock_file(name: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("lock"))
        || matches!(
            name,
            "package-lock.json"
                | "yarn.lock"
                | "pnpm-lock.yaml"
                | "bun.lockb"
                | "composer.lock"
                | "Gemfile.lock"
                | "poetry.lock"
                | "Pipfile.lock"
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Token extraction: @ search ---

    #[test]
    fn at_token_at_start() {
        let result = find_at_token("@foo", 4);
        assert_eq!(result, Some((0, "foo")));
    }

    #[test]
    fn at_token_in_middle() {
        let result = find_at_token("hello @bar", 10);
        assert_eq!(result, Some((6, "bar")));
    }

    #[test]
    fn at_token_email_not_triggered() {
        let result = find_at_token("user@example.com", 16);
        assert_eq!(result, None);
    }

    #[test]
    fn at_token_empty_query() {
        let result = find_at_token("@", 1);
        assert_eq!(result, Some((0, "")));
    }

    #[test]
    fn at_token_with_space_in_query() {
        let result = find_at_token("@foo bar", 8);
        assert_eq!(result, None);
    }

    #[test]
    fn at_token_after_space() {
        let result = find_at_token("look at @src", 12);
        assert_eq!(result, Some((8, "src")));
    }

    #[test]
    fn at_token_with_dir_prefix() {
        let result = find_at_token("@src/main", 9);
        assert_eq!(result, Some((0, "src/main")));
    }

    // --- Token end ---

    #[test]
    fn token_end_at_end_of_input() {
        assert_eq!(token_end("@foo", 4), 4);
    }

    #[test]
    fn token_end_mid_token() {
        assert_eq!(token_end("@foo", 2), 4);
    }

    #[test]
    fn token_end_before_space() {
        assert_eq!(token_end("@foo bar", 2), 4);
    }

    #[test]
    fn token_end_at_cursor_on_space() {
        assert_eq!(token_end("@foo bar", 4), 4);
    }

    // --- Token extraction: path completion ---

    #[test]
    fn path_token_dot_slash() {
        let result = find_path_token("./src/", 6);
        assert_eq!(result, Some((0, "./src/", "")));
    }

    #[test]
    fn path_token_tilde_slash() {
        let result = find_path_token("~/Documents/", 12);
        assert_eq!(result, Some((0, "~/Documents/", "")));
    }

    #[test]
    fn path_token_dot_dot_slash() {
        let result = find_path_token("../", 3);
        assert_eq!(result, Some((0, "../", "")));
    }

    #[test]
    fn path_token_with_prefix() {
        let result = find_path_token("./src/ma", 8);
        assert_eq!(result, Some((0, "./src/", "ma")));
    }

    #[test]
    fn path_token_after_text() {
        let result = find_path_token("look at ./src/", 14);
        assert_eq!(result, Some((8, "./src/", "")));
    }

    #[test]
    fn path_token_no_match() {
        let result = find_path_token("hello world", 11);
        assert_eq!(result, None);
    }

    #[test]
    fn path_token_bare_dot() {
        let result = find_path_token(".", 1);
        assert_eq!(result, None);
    }

    // --- is_attachable ---

    #[test]
    fn attachable_code_files() {
        assert!(is_attachable(Path::new("main.rs")));
        assert!(is_attachable(Path::new("app.tsx")));
        assert!(is_attachable(Path::new("script.py")));
        assert!(is_attachable(Path::new("lib.go")));
    }

    #[test]
    fn attachable_text_files() {
        assert!(is_attachable(Path::new("README.md")));
        assert!(is_attachable(Path::new("config.toml")));
        assert!(is_attachable(Path::new("data.json")));
        assert!(is_attachable(Path::new("style.css")));
    }

    #[test]
    fn attachable_images() {
        assert!(is_attachable(Path::new("logo.png")));
        assert!(is_attachable(Path::new("photo.jpg")));
        assert!(is_attachable(Path::new("icon.svg")));
    }

    #[test]
    fn attachable_stencila() {
        assert!(is_attachable(Path::new("doc.smd")));
        assert!(is_attachable(Path::new("notebook.ipynb")));
    }

    #[test]
    fn attachable_extensionless() {
        assert!(is_attachable(Path::new("Dockerfile")));
        assert!(is_attachable(Path::new("Makefile")));
        assert!(is_attachable(Path::new("Justfile")));
    }

    #[test]
    fn not_attachable_lock_files() {
        assert!(!is_attachable(Path::new("Cargo.lock")));
        assert!(!is_attachable(Path::new("package-lock.json")));
        assert!(!is_attachable(Path::new("yarn.lock")));
        assert!(!is_attachable(Path::new("poetry.lock")));
    }

    #[test]
    fn not_attachable_binary() {
        assert!(!is_attachable(Path::new("program.exe")));
        assert!(!is_attachable(Path::new("archive.tar.gz")));
        assert!(!is_attachable(Path::new("image.psd")));
    }

    #[test]
    fn not_attachable_no_extension() {
        assert!(!is_attachable(Path::new("randomfile")));
    }

    // --- is_lock_file ---

    #[test]
    fn lock_file_detection() {
        assert!(is_lock_file("Cargo.lock"));
        assert!(is_lock_file("package-lock.json"));
        assert!(is_lock_file("yarn.lock"));
        assert!(is_lock_file("pnpm-lock.yaml"));
        assert!(!is_lock_file("config.json"));
        assert!(!is_lock_file("main.rs"));
    }

    // --- accept_tab ---

    #[test]
    fn tab_accept_file_at_mode() {
        let mut state = FilesState::new();
        state.visible = true;
        state.mode = FileMode::AtSearch;
        state.token_range = 0..4;
        state.candidates = vec![FileCandidate {
            display: "main.rs".to_string(),
            path: "src/main.rs".to_string(),
            is_dir: false,
        }];

        let result = state.accept_tab(true).expect("should accept");
        assert_eq!(result.text, "@src/main.rs ");
        assert_eq!(result.range, 0..4);
        assert!(!result.refresh);
        assert!(!state.is_visible());
    }

    #[test]
    fn tab_accept_dir_at_mode_drills_down() {
        let mut state = FilesState::new();
        state.visible = true;
        state.mode = FileMode::AtSearch;
        state.token_range = 0..1;
        state.candidates = vec![FileCandidate {
            display: "src/".to_string(),
            path: "src/".to_string(),
            is_dir: true,
        }];

        let result = state.accept_tab(true).expect("should accept");
        assert_eq!(result.text, "@src/");
        assert!(result.refresh);
    }

    #[test]
    fn tab_accept_file_path_mode() {
        let mut state = FilesState::new();
        state.visible = true;
        state.mode = FileMode::PathCompletion;
        state.token_range = 0..6;
        state.path_dir_prefix = "./src/".to_string();
        state.candidates = vec![FileCandidate {
            display: "main.rs".to_string(),
            path: "main.rs".to_string(),
            is_dir: false,
        }];

        let result = state.accept_tab(true).expect("should accept");
        assert_eq!(result.text, "@./src/main.rs ");
        assert!(!result.refresh);
    }

    #[test]
    fn tab_accept_dir_path_mode_drills_down() {
        let mut state = FilesState::new();
        state.visible = true;
        state.mode = FileMode::PathCompletion;
        state.token_range = 0..2;
        state.path_dir_prefix = "./".to_string();
        state.candidates = vec![FileCandidate {
            display: "src/".to_string(),
            path: "src/".to_string(),
            is_dir: true,
        }];

        let result = state.accept_tab(true).expect("should accept");
        assert_eq!(result.text, "./src/");
        assert!(result.refresh);
    }

    // --- accept_tab shell mode (no @ prefix) ---

    #[test]
    fn tab_accept_file_at_mode_shell() {
        let mut state = FilesState::new();
        state.visible = true;
        state.mode = FileMode::AtSearch;
        state.token_range = 0..4;
        state.candidates = vec![FileCandidate {
            display: "main.rs".to_string(),
            path: "src/main.rs".to_string(),
            is_dir: false,
        }];

        let result = state.accept_tab(false).expect("should accept");
        assert_eq!(result.text, "src/main.rs ");
    }

    #[test]
    fn tab_accept_file_path_mode_shell() {
        let mut state = FilesState::new();
        state.visible = true;
        state.mode = FileMode::PathCompletion;
        state.token_range = 0..6;
        state.path_dir_prefix = "./src/".to_string();
        state.candidates = vec![FileCandidate {
            display: "main.rs".to_string(),
            path: "main.rs".to_string(),
            is_dir: false,
        }];

        let result = state.accept_tab(false).expect("should accept");
        assert_eq!(result.text, "./src/main.rs ");
    }

    // --- accept_enter ---

    #[test]
    fn enter_accept_file_at_mode() {
        let mut state = FilesState::new();
        state.visible = true;
        state.mode = FileMode::AtSearch;
        state.token_range = 0..4;
        state.candidates = vec![FileCandidate {
            display: "main.rs".to_string(),
            path: "src/main.rs".to_string(),
            is_dir: false,
        }];

        let result = state.accept_enter(true).expect("should accept");
        assert_eq!(result.text, "@src/main.rs ");
        assert!(!result.refresh);
    }

    #[test]
    fn enter_accept_dir_at_mode_no_refresh() {
        let mut state = FilesState::new();
        state.visible = true;
        state.mode = FileMode::AtSearch;
        state.token_range = 0..1;
        state.candidates = vec![FileCandidate {
            display: "src/".to_string(),
            path: "src/".to_string(),
            is_dir: true,
        }];

        let result = state.accept_enter(true).expect("should accept");
        assert_eq!(result.text, "@src/ ");
        assert!(!result.refresh);
    }

    #[test]
    fn enter_accept_file_path_mode() {
        let mut state = FilesState::new();
        state.visible = true;
        state.mode = FileMode::PathCompletion;
        state.token_range = 0..6;
        state.path_dir_prefix = "./src/".to_string();
        state.candidates = vec![FileCandidate {
            display: "main.rs".to_string(),
            path: "main.rs".to_string(),
            is_dir: false,
        }];

        let result = state.accept_enter(true).expect("should accept");
        assert_eq!(result.text, "@./src/main.rs ");
        assert!(!result.refresh);
    }

    // --- accept_enter shell mode (no @ prefix) ---

    #[test]
    fn enter_accept_file_at_mode_shell() {
        let mut state = FilesState::new();
        state.visible = true;
        state.mode = FileMode::AtSearch;
        state.token_range = 0..4;
        state.candidates = vec![FileCandidate {
            display: "main.rs".to_string(),
            path: "src/main.rs".to_string(),
            is_dir: false,
        }];

        let result = state.accept_enter(false).expect("should accept");
        assert_eq!(result.text, "src/main.rs ");
    }

    #[test]
    fn enter_accept_file_path_mode_shell() {
        let mut state = FilesState::new();
        state.visible = true;
        state.mode = FileMode::PathCompletion;
        state.token_range = 0..6;
        state.path_dir_prefix = "./src/".to_string();
        state.candidates = vec![FileCandidate {
            display: "main.rs".to_string(),
            path: "main.rs".to_string(),
            is_dir: false,
        }];

        let result = state.accept_enter(false).expect("should accept");
        assert_eq!(result.text, "./src/main.rs ");
    }

    #[test]
    fn accept_when_hidden_returns_none() {
        let mut state = FilesState::new();
        assert!(state.accept_tab(true).is_none());
        assert!(state.accept_enter(true).is_none());
    }

    // --- Navigation ---

    #[test]
    fn select_next_wraps() {
        let mut state = FilesState::new();
        state.candidates = vec![
            FileCandidate {
                display: "a".to_string(),
                path: "a".to_string(),
                is_dir: false,
            },
            FileCandidate {
                display: "b".to_string(),
                path: "b".to_string(),
                is_dir: false,
            },
        ];
        state.selected = 0;

        state.select_next();
        assert_eq!(state.selected(), 1);
        state.select_next();
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn select_prev_wraps() {
        let mut state = FilesState::new();
        state.candidates = vec![
            FileCandidate {
                display: "a".to_string(),
                path: "a".to_string(),
                is_dir: false,
            },
            FileCandidate {
                display: "b".to_string(),
                path: "b".to_string(),
                is_dir: false,
            },
        ];
        state.selected = 0;

        state.select_prev();
        assert_eq!(state.selected(), 1);
    }

    // --- Cache ---

    #[test]
    fn cache_preserved_on_dismiss() {
        let mut state = FilesState::new();
        state.cached_files = Some(vec![FileCandidate {
            display: "test.rs".to_string(),
            path: "test.rs".to_string(),
            is_dir: false,
        }]);

        state.dismiss();
        assert!(state.cached_files.is_some());
        assert!(state.path_dir_prefix.is_empty());
    }

    #[test]
    fn initially_hidden() {
        let state = FilesState::new();
        assert!(!state.is_visible());
        assert!(state.candidates().is_empty());
        assert_eq!(state.selected(), 0);
    }

    // --- @ mode dir-prefix filtering ---

    #[test]
    fn at_mode_dir_prefix_filters_direct_children() {
        let mut state = FilesState::new();
        state.cached_files = Some(vec![
            FileCandidate {
                display: "src/".to_string(),
                path: "src/".to_string(),
                is_dir: true,
            },
            FileCandidate {
                display: "main.rs".to_string(),
                path: "src/main.rs".to_string(),
                is_dir: false,
            },
            FileCandidate {
                display: "lib.rs".to_string(),
                path: "src/lib.rs".to_string(),
                is_dir: false,
            },
            FileCandidate {
                display: "models/".to_string(),
                path: "src/models/".to_string(),
                is_dir: true,
            },
            FileCandidate {
                display: "mod.rs".to_string(),
                path: "src/models/mod.rs".to_string(),
                is_dir: false,
            },
            FileCandidate {
                display: "README.md".to_string(),
                path: "README.md".to_string(),
                is_dir: false,
            },
        ]);

        // Query "src/" should show direct children of src/ only
        state.update_at_mode("@src/", 0, "src/", 5);

        assert!(state.is_visible());
        // Should include: main.rs, lib.rs, models/ (direct children)
        // Should exclude: src/ (the dir itself), mod.rs (nested), README.md (outside)
        assert_eq!(state.candidates.len(), 3);
        let names: Vec<&str> = state
            .candidates
            .iter()
            .map(super::FileCandidate::display)
            .collect();
        assert!(names.contains(&"main.rs"));
        assert!(names.contains(&"lib.rs"));
        assert!(names.contains(&"models/"));
    }

    #[test]
    fn at_mode_dir_prefix_with_name_filter() {
        let mut state = FilesState::new();
        state.cached_files = Some(vec![
            FileCandidate {
                display: "main.rs".to_string(),
                path: "src/main.rs".to_string(),
                is_dir: false,
            },
            FileCandidate {
                display: "lib.rs".to_string(),
                path: "src/lib.rs".to_string(),
                is_dir: false,
            },
        ]);

        // Query "src/main" should show only files matching "main"
        state.update_at_mode("@src/main", 0, "src/main", 9);

        assert!(state.is_visible());
        assert_eq!(state.candidates.len(), 1);
        assert_eq!(state.candidates[0].display(), "main.rs");
    }
}
