//! Default provider profile for providers without a dedicated profile.
//!
//! Used as a fallback when the provider is not one of the first-class
//! profiles (Anthropic, OpenAI, Gemini). Uses a generic tool set with
//! `edit_file` for modifications and pulls capability metadata from the
//! model catalog when available.
//!
//! Tool set: `read_file`, `write_file`, `edit_file`, `shell` (10s timeout),
//! `grep`, `glob`, `web_fetch`.

use crate::error::AgentResult;
use crate::profile::ProviderProfile;
use crate::registry::{RegisteredTool, ToolRegistry};
use crate::tools::{edit_file, glob, grep, read_file, shell, web_fetch, write_file};

/// Default shell timeout: 10 seconds.
const DEFAULT_SHELL_TIMEOUT_MS: u64 = 10_000;

const BASE_INSTRUCTIONS: &str = "\
You are a coding assistant. You help users with software engineering tasks \
including writing code, debugging, refactoring, reviewing, and explaining code.

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

Use `shell` to run commands. The default timeout is 10 seconds. \
For long-running commands, set a longer `timeout_ms`.

## Searching Code

Use `grep` to search file contents by pattern and `glob` to find files by name. \
Search the codebase to understand existing patterns before making changes.

## Fetching Web Content

Use `web_fetch` to download web pages and save them locally for reading. \
The content is saved to `.stencila/cache/web/` and can be explored with \
`read_file`, `grep`, or `glob`. HTML pages are automatically converted to Markdown.

# Coding Best Practices

- Write clean, readable code that follows the project's existing conventions.
- Prefer simple, focused changes over large refactors.
- Handle errors appropriately.
- Do not introduce security vulnerabilities.";

/// Default provider profile for any provider without a dedicated profile.
#[derive(Debug)]
pub struct DefaultProfile {
    provider: String,
    model: String,
    context_window: u64,
    reasoning: bool,
    registry: ToolRegistry,
}

impl DefaultProfile {
    /// Create a new default profile for the given provider and model.
    ///
    /// Looks up the model in the catalog to determine context window size
    /// and reasoning support. Falls back to conservative defaults (128K
    /// context, no reasoning) when the model is not in the catalog.
    pub fn new(
        provider: impl Into<String>,
        model: impl Into<String>,
        max_command_timeout_ms: u64,
    ) -> AgentResult<Self> {
        let provider = provider.into();
        let model = model.into();

        // Try to pull capability info from the model catalog.
        let catalog_info = stencila_models3::catalog::get_model_info(&model)
            .ok()
            .flatten();

        let context_window = catalog_info.as_ref().map_or(128_000, |i| i.context_window);
        let reasoning = catalog_info
            .as_ref()
            .map_or(false, |i| i.supports_reasoning);

        let mut registry = ToolRegistry::new();

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
            RegisteredTool::new(web_fetch::definition(), web_fetch::executor()),
        ];
        for tool in tools {
            registry.register(tool)?;
        }

        Ok(Self {
            provider,
            model,
            context_window,
            reasoning,
            registry,
        })
    }
}

impl ProviderProfile for DefaultProfile {
    fn id(&self) -> &str {
        &self.provider
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
        self.reasoning
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_parallel_tool_calls(&self) -> bool {
        true
    }

    fn context_window_size(&self) -> u64 {
        self.context_window
    }
}
