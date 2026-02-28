//! OpenAI provider profile (spec 3.4, codex-rs-aligned).
//!
//! Uses `apply_patch` instead of `edit_file` for modifications. OpenAI models
//! are specifically trained on this format and produce better edits when
//! using it.
//!
//! Tool set: `read_file`, `apply_patch`, `write_file`, `shell` (10s timeout),
//! `grep`, `glob`, `web_fetch`.

use std::collections::HashMap;

use serde_json::Value;

use crate::error::AgentResult;
use crate::profile::ProviderProfile;
use crate::registry::{RegisteredTool, ToolRegistry};
use crate::tools::{apply_patch, glob, grep, read_file, shell, web_fetch, write_file};

/// Default shell timeout for OpenAI: 10 seconds.
const DEFAULT_SHELL_TIMEOUT_MS: u64 = 10_000;

/// Base instructions for the OpenAI profile (spec 6.2).
///
/// Topic-level coverage mirroring codex-rs: identity, tool usage
/// (especially apply_patch conventions), coding best practices, error
/// handling guidance.
const BASE_INSTRUCTIONS: &str = "\
You are a coding assistant. You help users with software engineering tasks \
including writing code, debugging, refactoring, and explaining code.

# Tool Usage

You have access to tools for reading files, writing files, applying patches, \
running shell commands, and searching code.

## Editing Files

Use `apply_patch` to modify existing files. The patch format supports creating, \
deleting, and updating files in a single operation. Each patch uses the v4a format \
with context lines to locate the correct position for changes.

- Prefer `apply_patch` over `write_file` for modifications to existing files.
- Use `write_file` for creating new files.
- Use `read_file` to understand existing code before making changes.

## Shell Commands

Use `shell` to run commands. The default timeout is 10 seconds. \
For long-running commands, set a longer `timeout_ms`.

## Searching Code

Use `grep` to search file contents by pattern and `glob` to find files by name.

## Fetching Web Content

Use `web_fetch` to download web pages and save them locally for reading. \
The content is saved to `.stencila/cache/web/` and can be explored with \
`read_file`, `grep`, or `glob`. HTML pages are automatically converted to Markdown.

# Coding Best Practices

- Write clean, readable code that follows the project's existing conventions.
- Prefer simple, focused changes over large refactors.
- Handle errors appropriately.
- Do not introduce security vulnerabilities.";

/// OpenAI provider profile (codex-rs-aligned).
#[derive(Debug)]
pub struct OpenAiProfile {
    model: String,
    registry: ToolRegistry,
}

impl OpenAiProfile {
    /// Create a new OpenAI profile with the given model identifier.
    ///
    /// `max_command_timeout_ms` clamps per-call `timeout_ms` on the shell tool
    /// to prevent unbounded execution. Pass [`SessionConfig::max_command_timeout_ms`]
    /// (default 600 000 = 10 minutes).
    ///
    /// Populates the tool registry with the OpenAI-specific tool set:
    /// `read_file`, `apply_patch`, `write_file`, `shell`, `grep`, `glob`,
    /// `web_fetch`.
    ///
    /// # Errors
    ///
    /// Returns an error if tool registration fails (e.g. invalid definition).
    pub fn new(model: impl Into<String>, max_command_timeout_ms: u64) -> AgentResult<Self> {
        let mut registry = ToolRegistry::new();

        // Register tools in the order listed in spec 3.4.
        let tools: Vec<RegisteredTool> = vec![
            RegisteredTool::new(read_file::definition(), read_file::executor()),
            RegisteredTool::new(apply_patch::definition(), apply_patch::executor()),
            RegisteredTool::new(write_file::definition(), write_file::executor()),
            RegisteredTool::new(
                shell::definition(),
                shell::executor_with_timeout(DEFAULT_SHELL_TIMEOUT_MS, max_command_timeout_ms),
            ),
            RegisteredTool::new(grep::definition(), grep::executor()),
            RegisteredTool::new(glob::definition(), glob::executor()),
            RegisteredTool::new(web_fetch::definition(), web_fetch::executor()),
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

impl ProviderProfile for OpenAiProfile {
    fn id(&self) -> &str {
        "openai"
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
        200_000
    }

    fn provider_options(&self) -> Option<HashMap<String, Value>> {
        // OpenAI provider options are not needed: the spec's §3.4 requirement
        // to set `reasoning.effort` is handled by the unified Request field
        // (request.reasoning_effort → OpenAI's `reasoning.effort` body param).
        None
    }
}
