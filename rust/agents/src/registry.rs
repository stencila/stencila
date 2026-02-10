//! Tool registry: name-to-executor mapping with validation (spec 3.8).
//!
//! The [`ToolRegistry`] maps tool names to [`RegisteredTool`] entries, each
//! pairing a [`ToolDefinition`] with an async executor function. Argument
//! validation uses JSON Schema (same graceful-skip pattern as models3).

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use indexmap::IndexMap;
use serde_json::Value;
use stencila_models3::types::tool::ToolDefinition;

use crate::error::{AgentError, AgentResult};
use crate::execution::ExecutionEnvironment;

// ---------------------------------------------------------------------------
// ToolOutput
// ---------------------------------------------------------------------------

/// Maximum image file size (bytes) for multimodal tool output.
/// Larger images fall back to the text placeholder to avoid
/// inflating memory and request payloads across the session.
pub const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024;

/// Output from a tool executor.
///
/// Most tools return plain text. Tools that produce multimodal content
/// (e.g. `read_file` for images) return the richer `ImageWithText` variant,
/// which carries both a text summary (for events, truncation, fallback) and
/// the raw image bytes.
#[derive(Debug, Clone)]
pub enum ToolOutput {
    /// Plain text output (most tools).
    Text(String),
    /// Image output with a text fallback.
    ImageWithText {
        /// Human-readable summary (used for events, truncation, and providers
        /// that do not support images in tool results).
        text: String,
        /// Raw image bytes.
        data: Vec<u8>,
        /// MIME type (e.g. `"image/png"`).
        media_type: String,
    },
}

impl ToolOutput {
    /// Text representation (always available).
    #[must_use]
    pub fn as_text(&self) -> &str {
        match self {
            Self::Text(s) => s,
            Self::ImageWithText { text, .. } => text,
        }
    }
}

// ---------------------------------------------------------------------------
// ToolExecutorFn
// ---------------------------------------------------------------------------

/// Async tool executor: takes JSON arguments and an execution environment,
/// returns the tool's output.
///
/// The closure must be `Send + Sync` so registries can be shared across tasks.
/// The returned future borrows the environment reference for its lifetime.
pub type ToolExecutorFn = Box<
    dyn Fn(
            Value,
            &dyn ExecutionEnvironment,
        ) -> Pin<Box<dyn Future<Output = AgentResult<ToolOutput>> + Send + '_>>
        + Send
        + Sync,
>;

// ---------------------------------------------------------------------------
// RegisteredTool
// ---------------------------------------------------------------------------

/// A tool registered in the registry (spec 3.8).
///
/// Pairs a [`ToolDefinition`] (name, description, parameter schema) with an
/// async executor function that performs the tool's action.
pub struct RegisteredTool {
    definition: ToolDefinition,
    executor: ToolExecutorFn,
}

impl RegisteredTool {
    /// Create a new registered tool from a definition and executor.
    pub fn new(definition: ToolDefinition, executor: ToolExecutorFn) -> Self {
        Self {
            definition,
            executor,
        }
    }

    /// The tool's definition (name, description, parameter schema).
    #[must_use]
    pub fn definition(&self) -> &ToolDefinition {
        &self.definition
    }

    /// Execute the tool with the given arguments and execution environment.
    pub async fn execute(
        &self,
        args: Value,
        env: &dyn ExecutionEnvironment,
    ) -> AgentResult<ToolOutput> {
        (self.executor)(args, env).await
    }
}

/// Manual `Debug` — the executor closure cannot be printed.
impl fmt::Debug for RegisteredTool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegisteredTool")
            .field("name", &self.definition.name)
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// ToolRegistry
// ---------------------------------------------------------------------------

/// Maps tool names to registered executors (spec 3.8).
///
/// Backed by [`IndexMap`] for deterministic insertion-order iteration in
/// [`definitions()`](Self::definitions) and [`names()`](Self::names).
#[derive(Default)]
pub struct ToolRegistry {
    tools: IndexMap<String, RegisteredTool>,
}

impl fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToolRegistry")
            .field("tools", &self.tools.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl ToolRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tool, replacing any existing tool with the same name.
    ///
    /// Validates the tool definition (name, description, parameter schema)
    /// before inserting. If a tool with the same name already exists, it is
    /// replaced in-place (preserving its insertion position). New tools are
    /// appended at the end.
    ///
    /// # Errors
    ///
    /// Returns `AgentError::Sdk` if the definition fails validation (e.g.
    /// invalid name, empty description, or non-object parameter schema root).
    pub fn register(&mut self, tool: RegisteredTool) -> AgentResult<()> {
        tool.definition.validate()?;
        let name = tool.definition.name.clone();
        // `insert` on IndexMap replaces the value if the key exists,
        // preserving the key's position in the order.
        self.tools.insert(name, tool);
        Ok(())
    }

    /// Remove a tool by name. Returns `true` if the tool existed.
    pub fn unregister(&mut self, name: &str) -> bool {
        self.tools.shift_remove(name).is_some()
    }

    /// Look up a tool by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&RegisteredTool> {
        self.tools.get(name)
    }

    /// All tool definitions in insertion order (cloned).
    #[must_use]
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition.clone()).collect()
    }

    /// All tool names in insertion order.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.tools.keys().map(String::as_str).collect()
    }

    /// Number of registered tools.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Validate arguments against the named tool's parameter schema.
    ///
    /// Returns:
    /// - `Err(UnknownTool)` if the tool is not registered.
    /// - `Err(ValidationError)` if the arguments fail schema validation.
    /// - `Ok(())` if validation passes or the schema cannot be compiled
    ///   (graceful skip — matches models3 behavior).
    pub fn validate_arguments(&self, name: &str, args: &Value) -> AgentResult<()> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| AgentError::UnknownTool {
                name: name.to_string(),
            })?;

        let schema = &tool.definition.parameters;

        // Graceful skip: if the schema can't compile, don't block execution.
        let Ok(validator) = jsonschema::validator_for(schema) else {
            return Ok(());
        };

        let errors: Vec<String> = validator.iter_errors(args).map(|e| e.to_string()).collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AgentError::ValidationError {
                reason: errors.join("; "),
            })
        }
    }
}
