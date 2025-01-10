//! Handling of formatting related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_formatting

use std::sync::Arc;

use async_lsp::{
    lsp_types::{Position, Range, TextEdit},
    ErrorCode, ResponseError,
};

use codecs::{EncodeOptions, Format, LossesResponse};
use common::{
    itertools::Itertools,
    similar::{self, capture_diff_slices, Algorithm},
    tokio::sync::RwLock,
    tracing,
};
use document::Document;

/// Handle to format a document
#[tracing::instrument(skip(doc))]
pub(crate) async fn request(
    doc: Arc<RwLock<Document>>,
    format: Format,
    source: Arc<RwLock<String>>,
) -> Result<Option<Vec<TextEdit>>, ResponseError> {
    format_doc(doc, format, source).await
}

// Create a text edit to format the document
pub(crate) async fn format_doc(
    doc: Arc<RwLock<Document>>,
    format: Format,
    source: Arc<RwLock<String>>,
) -> Result<Option<Vec<TextEdit>>, ResponseError> {
    // Generate formatted version of document
    let formatted = match doc
        .read()
        .await
        .dump(Some(EncodeOptions {
            format: Some(format.clone()),
            // Reduce log level for reporting encoding losses
            losses: LossesResponse::Trace,
            ..Default::default()
        }))
        .await
    {
        Ok(content) => content,
        Err(error) => {
            let message = format!("When encoding document to {format}: {error}");
            tracing::error!("{message}");
            return Err(ResponseError::new(ErrorCode::INTERNAL_ERROR, message));
        }
    };

    let source = source.read().await;

    // Do not return any edits if formatted equals current source
    if formatted == *source {
        return Ok(None);
    }

    // Compute a set of edits
    let edits = compute_text_edits(&source, &formatted);

    Ok(Some(edits))
}

/// Compute a set of [`TextEdit`]s
///
/// This uses a line based approach. It would be possible to go to character level but
/// that would be more complicated because it would require then calculating line
/// and UTF16 character indices for each diff.
pub fn compute_text_edits(source: &str, formatted: &str) -> Vec<TextEdit> {
    // Split while preserving line ends
    let source_lines: Vec<_> = source.split_inclusive('\n').map(String::from).collect();
    let formatted_lines: Vec<_> = formatted.split_inclusive('\n').map(String::from).collect();

    // Diff using LCS
    let ops = capture_diff_slices(Algorithm::Lcs, &source_lines, &formatted_lines);

    // Translate diff ops into text edits
    let edits = ops.into_iter().filter_map(|op| match op {
        similar::DiffOp::Insert {
            old_index,
            new_index,
            new_len,
        } => Some(TextEdit::new(
            Range::new(
                Position::new(old_index as u32, 0),
                Position::new(old_index as u32, 0),
            ),
            formatted_lines
                .iter()
                .skip(new_index)
                .take(new_len)
                .join(""),
        )),

        similar::DiffOp::Replace {
            old_index,
            old_len,
            new_index,
            new_len,
        } => Some(TextEdit::new(
            Range::new(
                Position::new(old_index as u32, 0),
                Position::new((old_index + old_len) as u32, 0),
            ),
            formatted_lines
                .iter()
                .skip(new_index)
                .take(new_len)
                .join(""),
        )),

        similar::DiffOp::Delete {
            old_index, old_len, ..
        } => (old_len > 0).then_some(TextEdit::new(
            Range::new(
                Position::new(old_index as u32, 0),
                Position::new((old_index + old_len) as u32, 0),
            ),
            "".into(),
        )),

        similar::DiffOp::Equal { .. } => None,
    });

    // Sort edits in reverse order by start line
    edits
        .sorted_by_key(|edit| std::cmp::Reverse(edit.range.start.line))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let source = "";
        let formatted = "";

        let edits = compute_text_edits(source, formatted);
        assert_eq!(edits.len(), 0);
    }

    #[test]
    fn same() {
        let source = "line1\nline2\n";
        let formatted = "line1\nline2\n";

        let edits = compute_text_edits(source, formatted);
        assert_eq!(edits.len(), 0);
    }

    #[test]
    fn empty_source() {
        let source = "";
        let formatted = "line1\nline2\n";

        let edits = compute_text_edits(source, formatted);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].range.start.line, 0);
        assert_eq!(edits[0].range.end.line, 0);
        assert_eq!(edits[0].new_text, "line1\nline2\n");
    }

    #[test]
    fn insert_lines() {
        let source = "line1\nline4\n";
        let formatted = "line1\nline2\nline3\nline4\n";

        let edits = compute_text_edits(source, formatted);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].range.start.line, 1);
        assert_eq!(edits[0].range.end.line, 1);
        assert_eq!(edits[0].new_text, "line2\nline3\n");
    }

    #[test]
    fn remove_lines() {
        let source = "line1\nline2\nline3\nline4\n";
        let formatted = "line1\nline4\n";

        let edits = compute_text_edits(source, formatted);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].range.start.line, 1);
        assert_eq!(edits[0].range.end.line, 3);
        assert_eq!(edits[0].new_text, "");
    }

    #[test]
    fn replace_lines() {
        let source = "line1\nline2\nline3\n";
        let formatted = "line1\nchanged2\nline3\n";

        let edits = compute_text_edits(source, formatted);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].range.start.line, 1);
        assert_eq!(edits[0].range.end.line, 2);
        assert_eq!(edits[0].new_text, "changed2\n");
    }

    #[test]
    fn inserts_blank_lines() {
        let source = "line1\nline2\n";
        let formatted = "line1\n\nline2\n";

        let edits = compute_text_edits(source, formatted);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].range.start.line, 1);
        assert_eq!(edits[0].range.end.line, 1);
        assert_eq!(edits[0].new_text, "\n");
    }

    #[test]
    fn adds_trailing_blank_lines() {
        let source = "line1\nline2";
        let formatted = "line1\nline2\n\n";

        let edits = compute_text_edits(source, formatted);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].range.start.line, 1);
        assert_eq!(edits[0].range.end.line, 2);
        assert_eq!(edits[0].new_text, "line2\n\n");
    }

    #[test]
    fn removes_trailing_blank_lines() {
        let source = "line1\nline2\n\n";
        let formatted = "line1\nline2";

        let edits = compute_text_edits(source, formatted);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].range.start.line, 1);
        assert_eq!(edits[0].range.end.line, 3);
        assert_eq!(edits[0].new_text, "line2");
    }

    #[test]
    fn consecutive_changes() {
        let source = "line1\nline2\nline3\nline4";
        let formatted = "line1\nmodified2\nmodified3\nline4";

        let edits = compute_text_edits(source, formatted);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].range.start.line, 1);
        assert_eq!(edits[0].range.end.line, 3);
        assert_eq!(edits[0].new_text, "modified2\nmodified3\n");
    }

    #[test]
    fn non_consecutive_changes() {
        let source = "line1\nline2\nline3\nline4";
        let formatted = "modified1\nline2\nline3\nmodified4\n";

        let edits = compute_text_edits(source, formatted);
        assert_eq!(edits.len(), 2);
        assert_eq!(edits[1].range.start.line, 0);
        assert_eq!(edits[1].range.end.line, 1);
        assert_eq!(edits[1].new_text, "modified1\n");
        assert_eq!(edits[0].range.start.line, 3);
        assert_eq!(edits[0].range.end.line, 4);
        assert_eq!(edits[0].new_text, "modified4\n");
    }

    #[test]
    fn mixed_changes() {
        let source = "keep1\nremove2\nkeep3\nremove4\nkeep5\nchange6\nkeep7\n";
        let formatted = "keep1\nkeep3\nadd1\nkeep5\nchanged6\nkeep7\n";

        let edits = compute_text_edits(source, formatted);
        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].range.start.line, 3);
        assert_eq!(edits[0].range.end.line, 6);
        assert_eq!(edits[0].new_text, "add1\nkeep5\nchanged6\n");
        assert_eq!(edits[1].range.start.line, 1);
        assert_eq!(edits[1].range.end.line, 2);
        assert_eq!(edits[1].new_text, "");
    }
}
