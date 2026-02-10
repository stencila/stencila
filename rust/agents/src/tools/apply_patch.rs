//! `apply_patch` tool: v4a format parser and applicator (spec Appendix A).
//!
//! The v4a patch format supports creating, deleting, updating, and renaming
//! files in a single patch. OpenAI models are specifically trained on this
//! format (spec line 586).

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::error::{AgentError, AgentResult};
use crate::execution::ExecutionEnvironment;
use crate::registry::ToolExecutorFn;

use super::required_str;

// ---------------------------------------------------------------------------
// Parser types
// ---------------------------------------------------------------------------

/// A single line within a hunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HunkLine {
    /// Context line — expected in the file, kept unchanged.
    Context(String),
    /// Delete line — expected in the file, removed.
    Delete(String),
    /// Add line — inserted into the file.
    Add(String),
}

/// A hunk describes a contiguous region of changes within a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    /// The `@@ ... @@` context hint (text between the `@@` markers).
    pub context_hint: String,
    /// The lines in this hunk.
    pub lines: Vec<HunkLine>,
}

/// A single operation within a patch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchOperation {
    /// Create a new file with the given content lines.
    AddFile { path: String, lines: Vec<String> },
    /// Delete an existing file.
    DeleteFile { path: String },
    /// Update (and optionally rename) an existing file.
    UpdateFile {
        path: String,
        move_to: Option<String>,
        hunks: Vec<Hunk>,
    },
}

/// A parsed v4a patch containing one or more operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    /// The operations in this patch, applied sequentially.
    pub operations: Vec<PatchOperation>,
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Internal parser state.
enum ParseState {
    ExpectBegin,
    ExpectOperation,
    AddingFile {
        path: String,
        lines: Vec<String>,
    },
    UpdateFileHeader {
        path: String,
        move_to: Option<String>,
        hunks: Vec<Hunk>,
    },
    InHunk {
        path: String,
        move_to: Option<String>,
        hunks: Vec<Hunk>,
        context_hint: String,
        hunk_lines: Vec<HunkLine>,
    },
}

/// Whether `line` starts a new operation block or ends the patch.
fn is_operation_boundary(line: &str) -> bool {
    line == "*** End Patch"
        || line.starts_with("*** Add File: ")
        || line.starts_with("*** Delete File: ")
        || line.starts_with("*** Update File: ")
}

/// Parse a hunk header and return the context hint.
///
/// Supports both `@@ hint` and `@@ hint @@` styles.
fn parse_hunk_hint(line: &str) -> Option<&str> {
    let hint = line.strip_prefix("@@ ")?;
    Some(hint.strip_suffix(" @@").unwrap_or(hint))
}

/// Parse a v4a format patch string into a [`Patch`].
///
/// # Errors
///
/// Returns `ValidationError` for any parse failure (missing begin/end
/// markers, unexpected lines, etc.).
pub fn parse_patch(input: &str) -> AgentResult<Patch> {
    let mut state = ParseState::ExpectBegin;
    let mut operations = Vec::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let mut reprocess_line = false;

        state = match state {
            ParseState::ExpectBegin => {
                if line.trim() == "*** Begin Patch" {
                    ParseState::ExpectOperation
                } else {
                    return Err(AgentError::ValidationError {
                        reason: format!("expected '*** Begin Patch', got: {line}"),
                    });
                }
            }

            ParseState::ExpectOperation => {
                if line == "*** End Patch" {
                    if i + 1 != lines.len() {
                        return Err(AgentError::ValidationError {
                            reason: "unexpected content after '*** End Patch'".to_string(),
                        });
                    }
                    return Ok(Patch { operations });
                }

                if let Some(path) = line.strip_prefix("*** Add File: ") {
                    ParseState::AddingFile {
                        path: path.to_string(),
                        lines: Vec::new(),
                    }
                } else if let Some(path) = line.strip_prefix("*** Delete File: ") {
                    operations.push(PatchOperation::DeleteFile {
                        path: path.to_string(),
                    });
                    ParseState::ExpectOperation
                } else if let Some(path) = line.strip_prefix("*** Update File: ") {
                    ParseState::UpdateFileHeader {
                        path: path.to_string(),
                        move_to: None,
                        hunks: Vec::new(),
                    }
                } else {
                    return Err(AgentError::ValidationError {
                        reason: format!("expected operation or '*** End Patch', got: {line}"),
                    });
                }
            }

            ParseState::AddingFile { path, mut lines } => {
                if let Some(rest) = line.strip_prefix('+') {
                    lines.push(rest.to_string());
                    ParseState::AddingFile { path, lines }
                } else if is_operation_boundary(line) {
                    operations.push(PatchOperation::AddFile { path, lines });
                    reprocess_line = true;
                    ParseState::ExpectOperation
                } else {
                    return Err(AgentError::ValidationError {
                        reason: format!(
                            "expected '+' prefixed line in Add File block, got: {line}"
                        ),
                    });
                }
            }

            ParseState::UpdateFileHeader {
                path,
                mut move_to,
                hunks,
            } => {
                if let Some(dest) = line.strip_prefix("*** Move to: ") {
                    move_to = Some(dest.to_string());
                    ParseState::UpdateFileHeader {
                        path,
                        move_to,
                        hunks,
                    }
                } else if let Some(hint) = parse_hunk_hint(line) {
                    ParseState::InHunk {
                        path,
                        move_to,
                        hunks,
                        context_hint: hint.to_string(),
                        hunk_lines: Vec::new(),
                    }
                } else if is_operation_boundary(line) {
                    if hunks.is_empty() {
                        return Err(AgentError::ValidationError {
                            reason: format!("update file '{path}' has no hunks"),
                        });
                    }
                    operations.push(PatchOperation::UpdateFile {
                        path,
                        move_to,
                        hunks,
                    });
                    reprocess_line = true;
                    ParseState::ExpectOperation
                } else {
                    return Err(AgentError::ValidationError {
                        reason: format!(
                            "expected '@@ ...' hunk header or '*** Move to: ...', got: {line}"
                        ),
                    });
                }
            }

            ParseState::InHunk {
                path,
                move_to,
                mut hunks,
                context_hint,
                mut hunk_lines,
            } => {
                if let Some(rest) = line.strip_prefix(' ') {
                    hunk_lines.push(HunkLine::Context(rest.to_string()));
                    ParseState::InHunk {
                        path,
                        move_to,
                        hunks,
                        context_hint,
                        hunk_lines,
                    }
                } else if let Some(rest) = line.strip_prefix('-') {
                    hunk_lines.push(HunkLine::Delete(rest.to_string()));
                    ParseState::InHunk {
                        path,
                        move_to,
                        hunks,
                        context_hint,
                        hunk_lines,
                    }
                } else if let Some(rest) = line.strip_prefix('+') {
                    hunk_lines.push(HunkLine::Add(rest.to_string()));
                    ParseState::InHunk {
                        path,
                        move_to,
                        hunks,
                        context_hint,
                        hunk_lines,
                    }
                } else if line == "*** End of File" {
                    if hunk_lines.is_empty() {
                        return Err(AgentError::ValidationError {
                            reason: format!("hunk has no lines (context_hint: '{context_hint}')"),
                        });
                    }
                    hunks.push(Hunk {
                        context_hint,
                        lines: hunk_lines,
                    });
                    ParseState::UpdateFileHeader {
                        path,
                        move_to,
                        hunks,
                    }
                } else if let Some(hint) = parse_hunk_hint(line) {
                    if hunk_lines.is_empty() {
                        return Err(AgentError::ValidationError {
                            reason: format!("hunk has no lines (context_hint: '{context_hint}')"),
                        });
                    }
                    hunks.push(Hunk {
                        context_hint,
                        lines: hunk_lines,
                    });
                    ParseState::InHunk {
                        path,
                        move_to,
                        hunks,
                        context_hint: hint.to_string(),
                        hunk_lines: Vec::new(),
                    }
                } else if is_operation_boundary(line) {
                    if hunk_lines.is_empty() {
                        return Err(AgentError::ValidationError {
                            reason: format!("hunk has no lines (context_hint: '{context_hint}')"),
                        });
                    }
                    hunks.push(Hunk {
                        context_hint,
                        lines: hunk_lines,
                    });
                    operations.push(PatchOperation::UpdateFile {
                        path,
                        move_to,
                        hunks,
                    });
                    reprocess_line = true;
                    ParseState::ExpectOperation
                } else if line.is_empty() {
                    // Empty line treated as context with empty content.
                    hunk_lines.push(HunkLine::Context(String::new()));
                    ParseState::InHunk {
                        path,
                        move_to,
                        hunks,
                        context_hint,
                        hunk_lines,
                    }
                } else {
                    return Err(AgentError::ValidationError {
                        reason: format!(
                            "unexpected line in hunk (expected ' ', '-', '+', or control line): {line}"
                        ),
                    });
                }
            }
        };

        if !reprocess_line {
            i += 1;
        }
    }

    // Finalize any unclosed state
    match state {
        ParseState::ExpectBegin => Err(AgentError::ValidationError {
            reason: "empty patch: missing '*** Begin Patch'".to_string(),
        }),
        ParseState::ExpectOperation | ParseState::AddingFile { .. } | ParseState::InHunk { .. } => {
            Err(AgentError::ValidationError {
                reason: "patch missing '*** End Patch'".to_string(),
            })
        }
        ParseState::UpdateFileHeader { path, hunks, .. } => {
            if hunks.is_empty() {
                Err(AgentError::ValidationError {
                    reason: format!("update file '{path}' has no hunks"),
                })
            } else {
                Err(AgentError::ValidationError {
                    reason: "patch missing '*** End Patch'".to_string(),
                })
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Applicator
// ---------------------------------------------------------------------------

/// Apply parsed patch operations sequentially against the execution environment.
///
/// Returns a human-readable summary line per operation.
pub async fn apply_patch_ops(
    patch: &Patch,
    env: &dyn ExecutionEnvironment,
) -> AgentResult<Vec<String>> {
    let mut summaries = Vec::new();

    for op in &patch.operations {
        match op {
            PatchOperation::AddFile { path, lines } => {
                let mut content = lines.join("\n");
                if !content.is_empty() {
                    content.push('\n');
                }
                env.write_file(path, &content).await?;
                summaries.push(format!("Created {path} ({} lines)", lines.len()));
            }

            PatchOperation::DeleteFile { path } => {
                env.delete_file(path).await?;
                summaries.push(format!("Deleted {path}"));
            }

            PatchOperation::UpdateFile {
                path,
                move_to,
                hunks,
            } => {
                // Read the full file content
                let raw = super::read_raw_content(env, path).await?;

                let mut file_lines: Vec<String> = raw.lines().map(String::from).collect();

                // Apply hunks top-to-bottom on the evolving working copy
                for hunk in hunks {
                    file_lines = apply_hunk(&file_lines, hunk)?;
                }

                let new_content = if file_lines.is_empty() {
                    String::new()
                } else {
                    let mut s = file_lines.join("\n");
                    s.push('\n');
                    s
                };

                let dest = move_to.as_deref().unwrap_or(path);
                env.write_file(dest, &new_content).await?;

                if let Some(new_path) = move_to {
                    // Only delete the old path if it differs from the destination
                    if new_path != path {
                        env.delete_file(path).await?;
                    }
                    summaries.push(format!(
                        "Updated and moved {path} → {new_path} ({} hunks)",
                        hunks.len()
                    ));
                } else {
                    summaries.push(format!("Updated {path} ({} hunks)", hunks.len()));
                }
            }
        }
    }

    Ok(summaries)
}

/// Apply a single hunk to the current file lines.
fn apply_hunk(file_lines: &[String], hunk: &Hunk) -> AgentResult<Vec<String>> {
    // Extract "expected" lines: Context + Delete lines in order
    let expected: Vec<&str> = hunk
        .lines
        .iter()
        .filter_map(|hl| match hl {
            HunkLine::Context(s) | HunkLine::Delete(s) => Some(s.as_str()),
            HunkLine::Add(_) => None,
        })
        .collect();

    if expected.is_empty() {
        // Pure addition — insert at the beginning
        let mut result = Vec::new();
        for hl in &hunk.lines {
            if let HunkLine::Add(s) = hl {
                result.push(s.clone());
            }
        }
        result.extend_from_slice(file_lines);
        return Ok(result);
    }

    // TODO: Use context_hint for proximity-based disambiguation when multiple
    // matches exist (spec App A line 1368). Currently first match wins.

    // Try exact match first
    let match_pos = find_sequence_exact(file_lines, &expected);

    // If no exact match, try fuzzy (whitespace normalization)
    // TODO: Add Unicode punctuation equivalence (spec App A line 1370).
    let match_pos = match match_pos {
        Some(pos) => pos,
        None => {
            find_sequence_fuzzy(file_lines, &expected).ok_or_else(|| AgentError::EditConflict {
                reason: format!(
                    "could not locate hunk in file (context_hint: '{}')",
                    hunk.context_hint
                ),
            })?
        }
    };

    // Apply the hunk at match_pos
    let mut result = Vec::new();

    // Lines before the match
    result.extend_from_slice(&file_lines[..match_pos]);

    // Process hunk lines
    let mut file_idx = match_pos;
    for hl in &hunk.lines {
        match hl {
            HunkLine::Context(_) => {
                // Keep the original line (preserves original whitespace)
                if file_idx < file_lines.len() {
                    result.push(file_lines[file_idx].clone());
                    file_idx += 1;
                }
            }
            HunkLine::Delete(_) => {
                // Skip (remove) this line
                file_idx += 1;
            }
            HunkLine::Add(s) => {
                result.push(s.clone());
            }
        }
    }

    // Lines after the matched region
    result.extend_from_slice(&file_lines[file_idx..]);

    Ok(result)
}

/// Find exact match of a sequence in file lines. Returns start index.
fn find_sequence_exact(file_lines: &[String], expected: &[&str]) -> Option<usize> {
    if expected.is_empty() {
        return Some(0);
    }
    if expected.len() > file_lines.len() {
        return None;
    }
    (0..=file_lines.len() - expected.len()).find(|&start| {
        file_lines[start..start + expected.len()]
            .iter()
            .zip(expected.iter())
            .all(|(fl, ex)| fl == ex)
    })
}

/// Find fuzzy match using whitespace normalization. Returns start index.
fn find_sequence_fuzzy(file_lines: &[String], expected: &[&str]) -> Option<usize> {
    if expected.is_empty() {
        return Some(0);
    }
    if expected.len() > file_lines.len() {
        return None;
    }
    (0..=file_lines.len() - expected.len()).find(|&start| {
        file_lines[start..start + expected.len()]
            .iter()
            .zip(expected.iter())
            .all(|(fl, ex)| normalize_whitespace(fl) == normalize_whitespace(ex))
    })
}

/// Collapse whitespace runs to a single space and trim.
fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------------------
// Tool definition and executor
// ---------------------------------------------------------------------------

/// Tool definition matching `tests/fixtures/tool_schemas/apply_patch.json`.
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "apply_patch".into(),
        description: "Apply code changes using the patch format. Supports creating, \
            deleting, and modifying files in a single operation."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "patch": {
                    "type": "string",
                    "description": "The patch content in v4a format."
                }
            },
            "required": ["patch"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

/// Executor that parses and applies a v4a patch.
pub fn executor() -> ToolExecutorFn {
    Box::new(|args: Value, env: &dyn ExecutionEnvironment| {
        Box::pin(async move {
            let patch_str = required_str(&args, "patch")?;
            let patch = parse_patch(patch_str)?;
            let summaries = apply_patch_ops(&patch, env).await?;
            Ok(summaries.join("\n"))
        })
    })
}
