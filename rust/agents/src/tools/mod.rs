//! Core tool implementations (spec 3.3, 3.6, Appendix A).
//!
//! Each submodule exposes `definition() -> ToolDefinition` and
//! `executor() -> ToolExecutorFn`. Registration functions group tools
//! into the shared core set, Gemini-specific extensions, and
//! OpenAI-specific extensions.

pub mod apply_patch;
pub mod edit_file;
pub mod glob;
pub mod grep;
pub mod list_dir;
pub mod read_file;
pub mod read_many_files;
pub mod shell;
pub mod write_file;

use serde_json::Value;

use crate::error::{AgentError, AgentResult};
use crate::execution::{ExecutionEnvironment, FileContent};
use crate::registry::{RegisteredTool, ToolRegistry};

/// Register the 6 shared core tools (spec 3.3).
///
/// These tools are common to all provider profiles:
/// `read_file`, `write_file`, `edit_file`, `shell`, `grep`, `glob`.
pub fn register_core_tools(registry: &mut ToolRegistry) -> AgentResult<()> {
    let tools: Vec<RegisteredTool> = vec![
        RegisteredTool::new(read_file::definition(), read_file::executor()),
        RegisteredTool::new(write_file::definition(), write_file::executor()),
        RegisteredTool::new(edit_file::definition(), edit_file::executor()),
        RegisteredTool::new(shell::definition(), shell::executor()),
        RegisteredTool::new(grep::definition(), grep::executor()),
        RegisteredTool::new(glob::definition(), glob::executor()),
    ];
    for tool in tools {
        registry.register(tool)?;
    }
    Ok(())
}

/// Register the 2 Gemini-specific tools (spec 3.6).
///
/// These tools are added on top of the core set for Gemini profiles:
/// `read_many_files`, `list_dir`.
pub fn register_gemini_tools(registry: &mut ToolRegistry) -> AgentResult<()> {
    let tools: Vec<RegisteredTool> = vec![
        RegisteredTool::new(read_many_files::definition(), read_many_files::executor()),
        RegisteredTool::new(list_dir::definition(), list_dir::executor()),
    ];
    for tool in tools {
        registry.register(tool)?;
    }
    Ok(())
}

/// Register the 1 OpenAI-specific tool (spec Appendix A).
///
/// This tool is added on top of the core set for OpenAI profiles:
/// `apply_patch`.
pub fn register_openai_tools(registry: &mut ToolRegistry) -> AgentResult<()> {
    registry.register(RegisteredTool::new(
        apply_patch::definition(),
        apply_patch::executor(),
    ))?;
    Ok(())
}

/// Strip line-number prefixes from `FileContent::Text` output.
///
/// The line format is `"{:>6} | {content}"` — this function finds the
/// first ` | ` on each line and takes everything after it. Lines that
/// don't contain ` | ` are passed through unchanged.
///
/// Preserves a trailing newline if present in the input, so that
/// `edit_file` round-trips file content without silently altering it.
pub fn strip_line_numbers(text: &str) -> String {
    let mut result: String = text
        .lines()
        .map(|line| line.find(" | ").map_or(line, |pos| &line[pos + 3..]))
        .collect::<Vec<_>>()
        .join("\n");

    if text.ends_with('\n') {
        result.push('\n');
    }

    result
}

/// Read the full text content of a file, stripping line-number prefixes.
///
/// Uses `usize::MAX` as the line limit to ensure the entire file is read,
/// not just the default 2000 lines that `read_file` returns for the
/// user-facing tool.
///
/// Shared by `edit_file` and `apply_patch` — both need raw file content
/// for search-and-replace or hunk application.
///
/// # Errors
///
/// Returns `FileNotFound` if the path does not exist, or `ValidationError`
/// if the file is a binary/image file.
pub async fn read_raw_content(
    env: &dyn ExecutionEnvironment,
    file_path: &str,
) -> AgentResult<String> {
    let content = env.read_file(file_path, None, Some(usize::MAX)).await?;
    match content {
        FileContent::Text(text) => Ok(strip_line_numbers(&text)),
        FileContent::Image { .. } => Err(AgentError::ValidationError {
            reason: format!("cannot edit binary/image file: {file_path}"),
        }),
    }
}

/// Extract a required string parameter from JSON arguments.
///
/// # Errors
///
/// Returns `ValidationError` if the key is missing or not a string.
pub fn required_str<'a>(args: &'a Value, name: &str) -> AgentResult<&'a str> {
    args.get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| AgentError::ValidationError {
            reason: format!("missing required string parameter: {name}"),
        })
}
