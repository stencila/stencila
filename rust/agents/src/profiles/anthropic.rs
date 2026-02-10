//! Anthropic provider profile (spec 3.5, Claude Code-aligned).
//!
//! Uses `edit_file` with `old_string`/`new_string` as the native editing
//! format. Anthropic models are specifically trained on this exact-match
//! search-and-replace pattern. Do NOT use `apply_patch` with Anthropic models.
//!
//! Tool set: `read_file`, `write_file`, `edit_file`, `shell` (120s timeout),
//! `grep`, `glob`. Subagent tools added in Phase 9.

use std::collections::HashMap;

use serde_json::Value;

use crate::error::AgentResult;
use crate::profile::ProviderProfile;
use crate::registry::{RegisteredTool, ToolRegistry};
use crate::tools::{edit_file, glob, grep, read_file, shell, write_file};

/// Default shell timeout for Anthropic: 120 seconds (per Claude Code convention).
const DEFAULT_SHELL_TIMEOUT_MS: u64 = 120_000;

/// Base instructions for the Anthropic profile (spec 6.2).
///
/// Topic-level coverage mirroring Claude Code: identity, tool selection
/// guidance (read before edit, edit over write), the edit_file format
/// (old_string must be unique), file operation preferences.
const BASE_INSTRUCTIONS: &str = "\
You are a coding assistant. You help users with software engineering tasks \
including writing code, debugging, refactoring, and explaining code.

# Tool Usage

You have access to tools for reading files, writing files, editing files, \
running shell commands, and searching code.

## Reading and Editing Files

- Always read a file before editing it to understand its current content.
- Prefer editing existing files over creating new ones.
- Use `edit_file` to make targeted changes. The `old_string` parameter must \
exactly match a unique section of the file. If the string appears multiple times, \
include enough surrounding context to make it unique.
- Use `write_file` only for creating new files, not for modifying existing ones.
- When making multiple changes to the same file, apply them one at a time with \
separate `edit_file` calls to avoid conflicts.

## Shell Commands

Use `shell` to run commands. The default timeout is 120 seconds. \
For long-running commands, set a longer `timeout_ms`.

## Searching Code

Use `grep` to search file contents by pattern and `glob` to find files by name. \
Search the codebase to understand existing patterns before making changes.

# Coding Best Practices

- Write clean, readable code that follows the project's existing conventions.
- Prefer simple, focused changes over large refactors.
- Handle errors appropriately.
- Do not introduce security vulnerabilities.
- Do not add comments, docstrings, or type annotations to code you did not change.";

/// Anthropic provider profile (Claude Code-aligned).
#[derive(Debug)]
pub struct AnthropicProfile {
    model: String,
    registry: ToolRegistry,
}

impl AnthropicProfile {
    /// Create a new Anthropic profile with the given model identifier.
    ///
    /// `max_command_timeout_ms` clamps per-call `timeout_ms` on the shell tool
    /// to prevent unbounded execution. Pass [`SessionConfig::max_command_timeout_ms`]
    /// (default 600 000 = 10 minutes).
    ///
    /// Populates the tool registry with the Anthropic-specific tool set:
    /// `read_file`, `write_file`, `edit_file`, `shell` (120s default),
    /// `grep`, `glob`.
    ///
    /// # Errors
    ///
    /// Returns an error if tool registration fails (e.g. invalid definition).
    pub fn new(model: impl Into<String>, max_command_timeout_ms: u64) -> AgentResult<Self> {
        let mut registry = ToolRegistry::new();

        // Register tools in the order listed in spec 3.5.
        let tools: Vec<RegisteredTool> = vec![
            RegisteredTool::new(read_file::definition(), read_file::executor()),
            RegisteredTool::new(write_file::definition(), write_file::executor()),
            RegisteredTool::new(edit_file::definition(), edit_file::executor()),
            RegisteredTool::new(
                shell::definition(),
                shell::executor_with_timeout(DEFAULT_SHELL_TIMEOUT_MS, max_command_timeout_ms),
            ),
            RegisteredTool::new(grep::definition(), grep::executor()),
            RegisteredTool::new(glob::definition(), glob::executor()),
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

impl ProviderProfile for AnthropicProfile {
    fn id(&self) -> &str {
        "anthropic"
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
        Some(HashMap::from([(
            "anthropic".into(),
            serde_json::json!({
                // Enable extended thinking (needed for reasoning models) and
                // prompt caching (reduces cost for long system prompts / tools).
                "beta_headers": [
                    "prompt-caching-2024-07-31"
                ],
                // Auto-inject cache_control markers on system prompts, tool
                // definitions, and conversation prefix for prompt caching.
                "auto_cache": true
            }),
        )]))
    }
}
