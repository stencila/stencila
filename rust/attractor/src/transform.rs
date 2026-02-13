//! Transform trait and registry (§9.1–9.3).
//!
//! Transforms preprocess the pipeline graph before execution begins.
//! Built-in transforms handle variable expansion; custom transforms
//! can be registered for domain-specific preprocessing.

use crate::error::AttractorResult;
use crate::graph::Graph;

/// A graph transform applied before pipeline execution.
///
/// Transforms modify the graph in-place (e.g., expanding variables,
/// applying stylesheet overrides) and run in a deterministic order:
/// built-in transforms first, then custom transforms.
pub trait Transform: Send + Sync {
    /// A short identifier for this transform (used in diagnostics).
    fn name(&self) -> &str;

    /// Apply the transform to the graph, modifying it in-place.
    ///
    /// # Errors
    ///
    /// Returns an error if the transform cannot be applied.
    fn apply(&self, graph: &mut Graph) -> AttractorResult<()>;
}

/// An ordered collection of transforms to apply before execution.
///
/// Transforms are partitioned into two groups:
/// 1. **Built-in** transforms (e.g., variable expansion)
/// 2. **Custom** transforms registered by the caller
///
/// [`apply_all`](Self::apply_all) runs all built-in transforms first,
/// then all custom transforms, in registration order within each group.
pub struct TransformRegistry {
    builtin: Vec<Box<dyn Transform>>,
    custom: Vec<Box<dyn Transform>>,
}

impl TransformRegistry {
    /// Create an empty registry with no transforms.
    #[must_use]
    pub fn new() -> Self {
        Self {
            builtin: Vec::new(),
            custom: Vec::new(),
        }
    }

    /// Create a registry pre-loaded with the default built-in transforms.
    ///
    /// Currently includes:
    /// - [`VariableExpansionTransform`](crate::transforms::VariableExpansionTransform)
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_builtin(crate::transforms::VariableExpansionTransform);
        registry
    }

    /// Register a built-in transform.
    pub fn register_builtin(&mut self, transform: impl Transform + 'static) {
        self.builtin.push(Box::new(transform));
    }

    /// Register a custom transform.
    pub fn register_custom(&mut self, transform: impl Transform + 'static) {
        self.custom.push(Box::new(transform));
    }

    /// Apply all transforms to the graph: built-ins first, then custom.
    ///
    /// Stops on the first error.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered by any transform.
    pub fn apply_all(&self, graph: &mut Graph) -> AttractorResult<()> {
        for transform in &self.builtin {
            transform.apply(graph)?;
        }
        for transform in &self.custom {
            transform.apply(graph)?;
        }
        Ok(())
    }
}

impl Default for TransformRegistry {
    fn default() -> Self {
        Self::new()
    }
}
