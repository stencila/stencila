//! Project document discovery (spec 6.5).
//!
//! Walks from the git root (or working directory) to the current working
//! directory, collecting instruction files that match the active provider
//! profile. Results are concatenated with a 32KB byte budget.

use crate::error::AgentResult;
use crate::execution::{ExecutionEnvironment, FileContent};

/// Maximum total bytes for project documents (spec 6.5).
const MAX_PROJECT_DOCS_BYTES: usize = 32 * 1024;

/// Truncation marker appended when the byte budget is exceeded (spec 6.5).
const TRUNCATION_MARKER: &str = "[Project instructions truncated at 32KB]";

/// Return the recognized instruction file names for a provider (spec 6.5).
///
/// `AGENTS.md` is always included regardless of provider.
fn instruction_files(provider_id: &str) -> Vec<&'static str> {
    match provider_id {
        "openai" => vec!["AGENTS.md", ".codex/instructions.md"],
        "anthropic" => vec!["AGENTS.md", "CLAUDE.md"],
        "gemini" => vec!["AGENTS.md", "GEMINI.md"],
        _ => vec!["AGENTS.md"],
    }
}

/// Discover and load project instruction documents (spec 6.5).
///
/// Walks from `root` (typically the git root, or working directory if not
/// a git repo) through every directory on the path to `working_dir`.
/// At each directory, checks for recognized instruction files in order.
///
/// **Loading rules (spec 6.5):**
/// - Root-level files are loaded first
/// - Subdirectory files are appended (deeper = higher precedence)
/// - Total byte budget: 32KB — truncated with a marker if exceeded
/// - Only files matching the active provider are loaded
/// - `AGENTS.md` is always loaded regardless of provider
///
/// Returns the concatenated document content with section separators.
pub async fn discover_project_docs(
    env: &dyn ExecutionEnvironment,
    provider_id: &str,
    root: &str,
    working_dir: &str,
) -> AgentResult<String> {
    let file_names = instruction_files(provider_id);
    let directories = directories_from_root_to_working_dir(root, working_dir);

    let mut result = String::new();
    let mut total_bytes: usize = 0;

    for dir in &directories {
        for &file_name in &file_names {
            let file_path = if dir.ends_with('/') {
                format!("{dir}{file_name}")
            } else {
                format!("{dir}/{file_name}")
            };

            if !env.file_exists(&file_path).await {
                continue;
            }

            let content = match env.read_file(&file_path, None, Some(usize::MAX)).await {
                Ok(FileContent::Text(text)) => {
                    // Strip line numbers from read_file output
                    crate::tools::strip_line_numbers(&text)
                }
                Ok(FileContent::Image { .. }) => continue,
                Err(_) => continue,
            };

            // Account for the "\n\n" separator between documents
            let separator_len = if result.is_empty() { 0 } else { 2 };
            let content_bytes = content.len() + separator_len;

            if total_bytes + content_bytes > MAX_PROJECT_DOCS_BYTES {
                // Append what fits, then truncation marker
                let remaining = MAX_PROJECT_DOCS_BYTES.saturating_sub(total_bytes + separator_len);
                if remaining > 0 {
                    if !result.is_empty() {
                        result.push_str("\n\n");
                    }
                    // Find a safe UTF-8 boundary
                    let safe_end = safe_truncation_point(&content, remaining);
                    result.push_str(&content[..safe_end]);
                }
                result.push('\n');
                result.push_str(TRUNCATION_MARKER);
                return Ok(result);
            }

            if !result.is_empty() {
                result.push_str("\n\n");
            }
            result.push_str(&content);
            total_bytes += content_bytes;
        }
    }

    Ok(result)
}

/// Compute the list of directories from `root` to `working_dir`.
///
/// Returns a list starting with `root` and ending with `working_dir`,
/// including all intermediate directories. If `working_dir` is not under
/// `root`, returns just `[working_dir]`.
fn directories_from_root_to_working_dir(root: &str, working_dir: &str) -> Vec<String> {
    let root_normalized = root.trim_end_matches('/');
    let wd_normalized = working_dir.trim_end_matches('/');

    // If working_dir is not under root, just use working_dir.
    // Check for path boundary: root must be followed by '/' or be exactly equal,
    // otherwise /repo would incorrectly match /repo2.
    let is_under_root = wd_normalized == root_normalized
        || (wd_normalized.starts_with(root_normalized)
            && wd_normalized.as_bytes().get(root_normalized.len()) == Some(&b'/'));
    if !is_under_root {
        return vec![wd_normalized.to_string()];
    }

    let mut dirs = vec![root_normalized.to_string()];

    // Get the relative suffix after root
    let suffix = &wd_normalized[root_normalized.len()..];
    let suffix = suffix.trim_start_matches('/');

    if !suffix.is_empty() {
        let mut current = root_normalized.to_string();
        for component in suffix.split('/') {
            current = format!("{current}/{component}");
            dirs.push(current.clone());
        }
    }

    dirs
}

/// Find a safe UTF-8 truncation point at or before `max_bytes`.
fn safe_truncation_point(s: &str, max_bytes: usize) -> usize {
    if max_bytes >= s.len() {
        return s.len();
    }
    // Walk backwards from max_bytes to find a char boundary
    let mut pos = max_bytes;
    while pos > 0 && !s.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_files_openai() {
        let files = instruction_files("openai");
        assert_eq!(files, vec!["AGENTS.md", ".codex/instructions.md"]);
    }

    #[test]
    fn instruction_files_anthropic() {
        let files = instruction_files("anthropic");
        assert_eq!(files, vec!["AGENTS.md", "CLAUDE.md"]);
    }

    #[test]
    fn instruction_files_gemini() {
        let files = instruction_files("gemini");
        assert_eq!(files, vec!["AGENTS.md", "GEMINI.md"]);
    }

    #[test]
    fn instruction_files_unknown_only_agents() {
        let files = instruction_files("unknown");
        assert_eq!(files, vec!["AGENTS.md"]);
    }

    #[test]
    fn directories_same_root_and_wd() {
        let dirs = directories_from_root_to_working_dir("/project", "/project");
        assert_eq!(dirs, vec!["/project"]);
    }

    #[test]
    fn directories_nested_wd() {
        let dirs = directories_from_root_to_working_dir("/project", "/project/src/lib");
        assert_eq!(dirs, vec!["/project", "/project/src", "/project/src/lib"]);
    }

    #[test]
    fn directories_wd_not_under_root() {
        let dirs = directories_from_root_to_working_dir("/other", "/project");
        assert_eq!(dirs, vec!["/project"]);
    }

    #[test]
    fn directories_sibling_path_not_misclassified() {
        // /repo should NOT match /repo2 — they are siblings, not parent/child
        let dirs = directories_from_root_to_working_dir("/repo", "/repo2/sub");
        assert_eq!(dirs, vec!["/repo2/sub"]);
    }

    #[test]
    fn safe_truncation_ascii() {
        assert_eq!(safe_truncation_point("hello world", 5), 5);
    }

    #[test]
    fn safe_truncation_multibyte() {
        // "café" is 5 bytes: c(1) a(1) f(1) é(2)
        let s = "café";
        // Trying to cut at byte 4 would split the é
        let point = safe_truncation_point(s, 4);
        assert!(s.is_char_boundary(point));
        assert_eq!(point, 3); // before the é
    }
}
