//! Provider profile trait (spec 3.2).
//!
//! A [`ProviderProfile`] assembles tools into a registry and provides
//! capability metadata for a specific LLM provider. Each provider (OpenAI,
//! Anthropic, Gemini) gets its native tool set and system prompt so that
//! the model operates with the exact interface it was trained on.

use std::collections::HashMap;
use std::fmt;

use serde_json::Value;
use stencila_models3::types::tool::ToolDefinition;

use crate::registry::ToolRegistry;

/// A provider profile that configures tools and prompts for a specific
/// LLM provider (spec 3.2).
///
/// Implementations populate a [`ToolRegistry`] with the provider's native
/// tool set and expose capability flags that inform session behavior.
///
/// Subagent tools are not registered by the profile itself — they are
/// added via [`register_subagent_tools()`](ProviderProfile::register_subagent_tools)
/// by the session layer (Phase 9).
pub trait ProviderProfile: fmt::Debug + Send + Sync {
    /// Provider identifier: `"openai"`, `"anthropic"`, or `"gemini"`.
    fn id(&self) -> &str;

    /// Model identifier (e.g. `"gpt-5.2-codex"`, `"claude-opus-4-6"`).
    fn model(&self) -> &str;

    /// Mutable reference to the tool registry for custom tool registration.
    fn tool_registry_mut(&mut self) -> &mut ToolRegistry;

    /// Shared reference to the tool registry.
    fn tool_registry(&self) -> &ToolRegistry;

    /// Provider-specific base instructions (spec 6.2, layer 1).
    ///
    /// Returns the base system prompt text covering identity, tool usage
    /// guidance, and coding best practices for this provider's model family.
    fn base_instructions(&self) -> &str;

    /// Build the system prompt for this profile (spec 6.1).
    ///
    /// Assembles layers 1-4 of the system prompt:
    /// 1. Provider-specific base instructions (from [`base_instructions()`])
    /// 2. Environment context (pre-built by [`prompts::build_environment_context()`])
    /// 3. Tool descriptions — not serialized as a separate layer; tool definitions
    ///    are passed via the API's `tools` parameter, and base instructions already
    ///    mention tools by name for topic-level coverage. This is pragmatic: most
    ///    LLM APIs handle tool schemas separately from the system prompt.
    /// 4. Project-specific instructions (pre-built by [`project_docs::discover_project_docs()`])
    ///
    /// Layer 5 (user instructions override) is appended by the session layer
    /// via [`SessionConfig::user_instructions`].
    ///
    /// [`prompts::build_environment_context()`]: crate::prompts::build_environment_context
    /// [`project_docs::discover_project_docs()`]: crate::project_docs::discover_project_docs
    /// [`SessionConfig::user_instructions`]: crate::types::SessionConfig
    fn build_system_prompt(&self, environment_context: &str, project_docs: &str) -> String {
        let mut prompt = self.base_instructions().to_string();

        if !environment_context.is_empty() {
            prompt.push_str("\n\n");
            prompt.push_str(environment_context);
        }

        if !project_docs.is_empty() {
            prompt.push_str("\n\n");
            prompt.push_str(project_docs);
        }

        prompt
    }

    /// All tool definitions from the registry (cloned, insertion order).
    fn tools(&self) -> Vec<ToolDefinition> {
        self.tool_registry().definitions()
    }

    /// Provider-specific options to include in LLM requests.
    ///
    /// Returns `None` by default. Profiles override this to pass
    /// provider-specific headers, safety settings, etc.
    fn provider_options(&self) -> Option<HashMap<String, Value>> {
        None
    }

    // -- Capability flags --

    /// Whether the model supports extended reasoning / chain-of-thought.
    fn supports_reasoning(&self) -> bool;

    /// Whether the model supports streaming responses.
    fn supports_streaming(&self) -> bool;

    /// Whether the model supports parallel tool calls in a single response.
    fn supports_parallel_tool_calls(&self) -> bool;

    /// The model's context window size in tokens.
    fn context_window_size(&self) -> u64;

    /// Register the four subagent tools (`spawn_agent`, `send_input`,
    /// `wait`, `close_agent`) into this profile's tool registry.
    ///
    /// Called by [`Session::new()`](crate::session::Session::new) when the
    /// session depth allows spawning. The default implementation delegates
    /// to [`subagents::register_subagent_tools()`](crate::subagents::register_subagent_tools)
    /// which works for any profile that exposes [`tool_registry_mut()`].
    fn register_subagent_tools(&mut self) -> crate::error::AgentResult<()> {
        crate::subagents::register_subagent_tools(self.tool_registry_mut())
    }
}
