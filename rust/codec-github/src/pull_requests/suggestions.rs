use stencila_codec::stencila_schema::SuggestionType;

use super::export::{PullRequestComment, PullRequestCommentResolution, PullRequestExport};

/// Classification of the concrete edit operation a suggestion implies.
///
/// Derived from [`SuggestionType`] when available, otherwise inferred
/// from the relationship between `selected_text` and `replacement_text`.
/// Used to determine how the GitHub suggestion block is constructed.
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionEditKind {
    /// New content inserted at the anchor point; no existing text replaced.
    Insert,
    /// Existing text removed; the replacement is empty.
    Delete,
    /// Existing text replaced with different content.
    Replace,
}

/// A GitHub-ready suggestion block derived from a resolved pull request comment.
///
/// Contains the whole-line expansion needed for GitHub's suggestion
/// comment format. GitHub suggestion blocks replace entire lines
/// (`start_line` through `end_line` inclusive) with `replacement_lines`.
///
/// The `body` field holds the formatted Markdown body including the
/// fenced suggestion block, ready to be submitted as a pull request comment.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GitHubSuggestion {
    /// The kind of edit this suggestion represents.
    pub edit_kind: SuggestionEditKind,
    /// 1-based start line of the suggestion (whole-line expanded).
    pub start_line: u32,
    /// 1-based end line of the suggestion (whole-line expanded).
    pub end_line: u32,
    /// The full replacement text for the covered lines (without trailing newline).
    pub replacement_lines: String,
    /// The formatted suggestion block body ready for GitHub submission.
    pub body: String,
}

/// Derive a GitHub suggestion block for an exported review item when safe.
///
/// Suggestion blocks are a GitHub-specific rendering derived from the export's
/// normalized source text and a precisely anchored suggestion item. Items that
/// degraded to coarse parent ranges or encoded syntax spans are excluded.
pub fn github_suggestion_for_item(
    export: &PullRequestExport,
    item: &PullRequestComment,
) -> Option<GitHubSuggestion> {
    if !matches!(item.kind, super::export::PullRequestCommentKind::Suggestion)
        || !matches!(item.resolution, PullRequestCommentResolution::Anchored)
    {
        return None;
    }

    let has_coarse_diagnostic = export.diagnostics.iter().any(|diagnostic| {
        diagnostic.item_node_id.as_deref() == item.node_id.as_deref()
            && matches!(
                diagnostic.code.as_str(),
                "coarse-parent-range" | "suggestion-syntax-range"
            )
    });

    if has_coarse_diagnostic {
        return None;
    }

    build_github_suggestion(&export.content.text, item)
}

/// Classify the concrete edit operation for a suggestion item.
///
/// Uses [`SuggestionType`] when available, otherwise infers from the
/// relationship between `selected_text` and `replacement_text`.
fn classify_suggestion_edit(item: &PullRequestComment) -> SuggestionEditKind {
    match item.suggestion_type {
        Some(SuggestionType::Insert) => SuggestionEditKind::Insert,
        Some(SuggestionType::Delete) => SuggestionEditKind::Delete,
        Some(SuggestionType::Replace) => SuggestionEditKind::Replace,
        _ => match (&item.selected_text, &item.replacement_text) {
            (None | Some(_), Some(replacement)) if replacement.is_empty() => {
                SuggestionEditKind::Delete
            }
            (None, Some(_)) => SuggestionEditKind::Insert,
            _ => SuggestionEditKind::Replace,
        },
    }
}

/// Expand byte offsets to the boundaries of the lines they fall on.
///
/// Returns `(line_start_byte, line_end_byte)` where `line_start_byte` is
/// the byte offset of the first character on the start line and
/// `line_end_byte` is the byte offset just past the last non-newline
/// character on the end line (i.e. before the trailing `\n`, if any).
fn expand_to_line_boundaries(source_text: &str, start: usize, end: usize) -> (usize, usize) {
    let line_start = source_text[..start]
        .rfind('\n')
        .map(|pos| pos + 1)
        .unwrap_or(0);

    let line_end = source_text[end..]
        .find('\n')
        .map(|pos| end + pos)
        .unwrap_or(source_text.len());

    (line_start, line_end)
}

/// Build a [`GitHubSuggestion`] from a resolved suggestion [`PullRequestComment`].
///
/// Returns `None` if the item lacks the byte offsets or line/column
/// information needed to construct a whole-line suggestion block (i.e.
/// items with `Unanchored` or `FallbackLine` resolution).
///
/// The suggestion block is constructed by:
/// 1. Classifying the edit as insert/delete/replace.
/// 2. Expanding the resolved byte range to cover whole source lines.
/// 3. Replacing only the selected portion within those lines according
///    to the edit kind.
/// 4. Formatting the result as a GitHub suggestion Markdown block.
fn build_github_suggestion(
    source_text: &str,
    item: &PullRequestComment,
) -> Option<GitHubSuggestion> {
    let start_offset = item.range.start_offset?;
    let end_offset = item.range.end_offset?;
    let start_line = item.range.start_line?;
    let end_line = item.range.end_line?;
    let replacement_text = item.replacement_text.as_deref()?;

    let edit_kind = classify_suggestion_edit(item);

    if edit_kind == SuggestionEditKind::Insert
        && item.selected_text.as_deref() == Some("")
        && item.preceding_text.is_none()
    {
        let body = format!("```suggestion\n{replacement_text}\n```");

        return Some(GitHubSuggestion {
            edit_kind,
            start_line,
            end_line,
            replacement_lines: replacement_text.to_string(),
            body,
        });
    }

    let (line_start, line_end) = expand_to_line_boundaries(source_text, start_offset, end_offset);

    let full_lines = source_text.get(line_start..line_end)?;
    let prefix = full_lines.get(..start_offset - line_start)?;
    let suffix = full_lines.get(end_offset - line_start..)?;

    let replacement_lines = match edit_kind {
        SuggestionEditKind::Insert => format!("{prefix}{replacement_text}{suffix}"),
        SuggestionEditKind::Delete => format!("{prefix}{suffix}"),
        SuggestionEditKind::Replace => format!("{prefix}{replacement_text}{suffix}"),
    };

    let body = format!("```suggestion\n{replacement_lines}\n```");

    Some(GitHubSuggestion {
        edit_kind,
        start_line,
        end_line,
        replacement_lines,
        body,
    })
}
