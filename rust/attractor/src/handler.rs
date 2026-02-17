//! Handler trait and registry (§4.1–4.2).
//!
//! Handlers execute pipeline nodes. Each handler receives the node
//! definition, the current context, the full graph, and a path for
//! writing logs, and returns an [`Outcome`].
//!
//! The [`HandlerRegistry`] maps handler type strings (from
//! [`Node::handler_type()`]) to concrete handler implementations.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;

use crate::context::Context;
use crate::error::AttractorResult;
use crate::graph::{Graph, Node};
use crate::handlers;
use crate::types::Outcome;

/// A handler that executes a pipeline node.
///
/// Handlers are the core extension point. Built-in handlers cover
/// start, exit, and conditional nodes; external handlers implement
/// LLM calls, tool execution, parallel fan-out, etc.
#[async_trait]
pub trait Handler: Send + Sync {
    /// Execute the given node and return an outcome.
    ///
    /// # Arguments
    ///
    /// * `node` — the node being executed
    /// * `context` — the shared pipeline context
    /// * `graph` — the full pipeline graph (for reading attributes)
    /// * `logs_root` — directory for writing per-node logs
    async fn execute(
        &self,
        node: &Node,
        context: &Context,
        graph: &Graph,
        logs_root: &Path,
    ) -> AttractorResult<Outcome>;
}

/// A registry that maps handler type strings to handler implementations.
///
/// Resolution uses the node's `handler_type()` (which checks
/// `attrs["type"]` first, then maps the shape). If no specific handler
/// is registered, the optional default handler is used.
pub struct HandlerRegistry {
    handlers: HashMap<String, Arc<dyn Handler>>,
    default: Option<Arc<dyn Handler>>,
}

impl std::fmt::Debug for HandlerRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandlerRegistry")
            .field(
                "registered_types",
                &self.handlers.keys().collect::<Vec<_>>(),
            )
            .field("has_default", &self.default.is_some())
            .finish()
    }
}

impl HandlerRegistry {
    /// Create an empty registry with no handlers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            default: None,
        }
    }

    /// Create a registry pre-loaded with the built-in handlers:
    /// `start`, `exit`, `conditional`, `codergen` (simulation), `tool`,
    /// and `parallel.fan_in`.
    ///
    /// Handlers that require runtime dependencies are not included:
    /// - `parallel` — requires `Arc<HandlerRegistry>` + `Arc<dyn EventEmitter>`
    /// - `wait.human` — requires `Arc<dyn Interviewer>`
    ///
    /// Register these explicitly after construction.
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register("start", handlers::StartHandler);
        registry.register("exit", handlers::ExitHandler);
        registry.register("conditional", handlers::ConditionalHandler);
        registry.register("codergen", handlers::CodergenHandler::simulation());
        registry.register("tool", handlers::ToolHandler);
        registry.register("parallel.fan_in", handlers::FanInHandler);
        registry
    }

    /// Register a handler for the given type string.
    ///
    /// Replaces any previously registered handler for the same type.
    pub fn register(&mut self, type_string: impl Into<String>, handler: impl Handler + 'static) {
        self.handlers.insert(type_string.into(), Arc::new(handler));
    }

    /// Set the default fallback handler used when no type-specific
    /// handler matches.
    pub fn set_default(&mut self, handler: impl Handler + 'static) {
        self.default = Some(Arc::new(handler));
    }

    /// Resolve a handler for the given node per §4.2:
    ///
    /// 1. Explicit `type` attribute → look up in registry
    /// 2. Well-known start/exit ID (for unadorned box nodes) → look up
    /// 3. Shape-based resolution → look up in registry
    /// 4. Default handler
    #[must_use]
    pub fn resolve(&self, node: &Node) -> Option<Arc<dyn Handler>> {
        // Step 1: explicit type attribute
        if let Some(handler) = node.get_str_attr("type").and_then(|t| self.handlers.get(t)) {
            return Some(handler.clone());
        }

        // Step 2: well-known start/exit IDs (only for default "box" shape)
        if node.shape() == "box" {
            if crate::Graph::START_IDS.contains(&node.id.as_str()) {
                if let Some(handler) = self.handlers.get("start") {
                    return Some(handler.clone());
                }
            }
            if crate::Graph::EXIT_IDS.contains(&node.id.as_str()) {
                if let Some(handler) = self.handlers.get("exit") {
                    return Some(handler.clone());
                }
            }
        }

        // Step 3: shape-based resolution
        let shape_type = crate::graph::shape_to_handler_type(node.shape());
        if let Some(handler) = self.handlers.get(shape_type) {
            return Some(handler.clone());
        }

        // Step 4: default
        self.default.clone()
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
