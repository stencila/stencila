//! Gemini provider profile (spec 3.6, gemini-cli-aligned).
//!
//! Extends the shared core tools with `read_many_files` (batch reading)
//! and `list_dir` (directory listing with depth).
//!
//! Tool set: `read_file`, `read_many_files`, `write_file`, `edit_file`,
//! `shell` (10s timeout), `grep`, `glob`, `list_dir`.
//! Subagent tools added in Phase 9.

use crate::error::AgentResult;
use crate::profile::ProviderProfile;
use crate::registry::{RegisteredTool, ToolRegistry};
use crate::tools::{
    edit_file, glob, grep, list_dir, read_file, read_many_files, shell, write_file,
};

/// Default shell timeout for Gemini: 10 seconds.
const DEFAULT_SHELL_TIMEOUT_MS: u64 = 10_000;

/// Base instructions for the Gemini profile (spec 6.2).
///
/// Topic-level coverage mirroring gemini-cli: identity, tool usage,
/// GEMINI.md conventions, coding best practices.
const BASE_INSTRUCTIONS: &str = "\
You are a coding assistant. You help users with software engineering tasks \
including writing code, debugging, refactoring, and explaining code.

# Tool Usage

You have access to tools for reading files, writing files, editing files, \
running shell commands, listing directories, and searching code.

## Reading and Editing Files

- Use `read_file` to read a single file, or `read_many_files` to batch-read \
multiple files at once.
- Use `edit_file` to make targeted search-and-replace changes.
- Use `write_file` for creating new files.
- Use `list_dir` to explore directory structure.
- Always read files before editing them.

## Shell Commands

Use `shell` to run commands. The default timeout is 10 seconds. \
For long-running commands, set a longer `timeout_ms`.

## Searching Code

Use `grep` to search file contents by pattern and `glob` to find files by name.

# Project Configuration

If the project contains a `GEMINI.md` file, follow the instructions within it. \
These files provide project-specific guidance and conventions.

# Coding Best Practices

- Write clean, readable code that follows the project's existing conventions.
- Prefer simple, focused changes over large refactors.
- Handle errors appropriately.
- Do not introduce security vulnerabilities.";

/// Gemini provider profile (gemini-cli-aligned).
#[derive(Debug)]
pub struct GeminiProfile {
    model: String,
    registry: ToolRegistry,
}

impl GeminiProfile {
    /// Create a new Gemini profile with the given model identifier.
    ///
    /// Populates the tool registry with the Gemini-specific tool set:
    /// `read_file`, `read_many_files`, `write_file`, `edit_file`,
    /// `shell`, `grep`, `glob`, `list_dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if tool registration fails (e.g. invalid definition).
    pub fn new(model: impl Into<String>) -> AgentResult<Self> {
        let mut registry = ToolRegistry::new();

        // Register tools in the order listed in spec 3.6.
        let tools: Vec<RegisteredTool> = vec![
            RegisteredTool::new(read_file::definition(), read_file::executor()),
            RegisteredTool::new(read_many_files::definition(), read_many_files::executor()),
            RegisteredTool::new(write_file::definition(), write_file::executor()),
            RegisteredTool::new(edit_file::definition(), edit_file::executor()),
            RegisteredTool::new(
                shell::definition(),
                shell::executor_with_timeout(DEFAULT_SHELL_TIMEOUT_MS),
            ),
            RegisteredTool::new(grep::definition(), grep::executor()),
            RegisteredTool::new(glob::definition(), glob::executor()),
            RegisteredTool::new(list_dir::definition(), list_dir::executor()),
        ];
        for tool in tools {
            registry.register(tool)?;
        }

        Ok(Self {
            model: model.into(),
            registry,
        })
    }
}

impl ProviderProfile for GeminiProfile {
    fn id(&self) -> &str {
        "gemini"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn tool_registry_mut(&mut self) -> &mut ToolRegistry {
        &mut self.registry
    }

    fn tool_registry(&self) -> &ToolRegistry {
        &self.registry
    }

    fn base_instructions(&self) -> &str {
        BASE_INSTRUCTIONS
    }

    fn supports_reasoning(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_parallel_tool_calls(&self) -> bool {
        true
    }

    fn context_window_size(&self) -> u64 {
        1_000_000
    }
}
