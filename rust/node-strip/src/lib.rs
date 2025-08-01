//! Provides a `NodeStrip` trait for stripping properties from nodes

use std::{collections::HashMap, fmt::Display};

use common::{
    clap::{self, ValueEnum},
    indexmap::IndexMap,
    serde::{Deserialize, Serialize},
};

pub use node_strip_derive::StripNode;

/// Strip properties from a node and its descendants
pub fn strip<T>(node: &mut T, targets: StripTargets)
where
    T: StripNode,
{
    node.strip(&targets);
}

/// Strip all non-content properties from a node and its descendants
///
/// Does not strip [`StripScope::Content`] or [`StripScope::Code`].
pub fn strip_non_content<T>(node: &mut T)
where
    T: StripNode,
{
    strip(
        node,
        StripTargets::scopes(vec![
            StripScope::Authors,
            StripScope::Provenance,
            StripScope::Metadata,
            StripScope::Archive,
            StripScope::Temporary,
            StripScope::Compilation,
            StripScope::Execution,
            StripScope::Output,
        ]),
    );
}

/// Predefined scopes for properties to be stripped across node types
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
pub enum StripScope {
    /// Strip authorship properties of nodes
    Authors,

    /// Strip provenance properties of nodes
    Provenance,

    /// Strip metadata properties of nodes
    Metadata,

    /// Strip content properties of nodes
    Content,

    /// Strip archive properties of nodes
    Archive,

    /// Strip temporary properties of nodes
    Temporary,

    /// Strip code properties of executable nodes
    ///
    /// Includes any properties that define the execution of a node e.g.
    ///
    /// - `code` and `programmingLanguage` of any `CodeExecutable` node
    /// - `source` of a `Include` or `Call` node
    Code,

    /// Strip compilation related properties of executable nodes
    ///
    /// Includes any properties that record the compilation state of a node e.g.
    ///
    /// - `compilationDigest` of any `Executable` node
    /// - `directory` of a `PromptBlock` node
    Compilation,

    /// Strip execution related properties of executable nodes
    ///
    /// Includes any properties that record the execution state of a node e.g.
    ///
    /// - `executionCount` of any `Executable` node
    Execution,

    /// Strip output properties of executable nodes
    ///
    /// Includes any properties that are the result of executing a node e.g.
    ///
    /// - `outputs` of a `CodeChunk` node
    /// - `output` of a `CodeExpression` node
    /// - `content` of a `Include` or `Call` node
    Output,

    /// Strip timestamp properties
    ///
    /// Includes `AuthorRole.lastModified` and the `executionEnded` property
    /// of executable nodes. Useful for making test snapshots deterministic.
    Timestamps,
}

/// The target properties for the strip
#[derive(Clone, Default)]
pub struct StripTargets {
    /// Scopes defining which properties of nodes should be stripped
    pub scopes: Vec<StripScope>,

    /// Types of nodes to strip
    ///
    /// A list of node types to remove e.g. "ExecutionError"
    pub types: Vec<String>,

    /// Properties of nodes to strip
    ///
    /// A list of type/property names to remove e.g. "CodeChunk.errors".
    /// Use `scopes` over `properties` if possible.
    pub properties: Vec<String>,
}

impl StripTargets {
    /// Create a new set of strip targets
    pub fn new(scopes: Vec<StripScope>, types: Vec<String>, properties: Vec<String>) -> Self {
        Self {
            scopes,
            types,
            properties,
        }
    }

    /// Strip a single scope
    pub fn scope(scope: StripScope) -> Self {
        Self {
            scopes: vec![scope],
            ..Default::default()
        }
    }

    /// Strip several scopes
    pub fn scopes(scopes: Vec<StripScope>) -> Self {
        Self {
            scopes,
            ..Default::default()
        }
    }
}

pub trait StripNode: Sized {
    /// Strip a node
    #[allow(unused_variables)]
    fn strip(&mut self, targets: &StripTargets) -> &mut Self {
        self
    }
}

impl StripNode for bool {}
impl StripNode for i64 {}
impl StripNode for u64 {}
impl StripNode for f64 {}
impl StripNode for String {}

impl<T> StripNode for Box<T>
where
    T: StripNode,
{
    fn strip(&mut self, targets: &StripTargets) -> &mut Self {
        self.as_mut().strip(targets);

        self
    }
}

impl<T> StripNode for Option<T>
where
    T: StripNode,
{
    fn strip(&mut self, targets: &StripTargets) -> &mut Self {
        if let Some(value) = self {
            value.strip(targets);
        }

        self
    }
}

impl<T> StripNode for Vec<T>
where
    T: StripNode + Display,
{
    fn strip(&mut self, targets: &StripTargets) -> &mut Self {
        if !targets.types.is_empty() {
            self.retain(|child| !targets.types.contains(&child.to_string()));
        }

        for node in self.iter_mut() {
            node.strip(targets);
        }

        self
    }
}

impl<T> StripNode for HashMap<String, T>
where
    T: StripNode,
{
    fn strip(&mut self, targets: &StripTargets) -> &mut Self {
        for node in self.values_mut() {
            node.strip(targets);
        }

        self
    }
}

impl<T> StripNode for IndexMap<String, T>
where
    T: StripNode + Display,
{
    fn strip(&mut self, targets: &StripTargets) -> &mut Self {
        if !targets.types.is_empty() {
            self.retain(|_, child| !targets.types.contains(&child.to_string()));
        }

        for node in self.values_mut() {
            node.strip(targets);
        }

        self
    }
}
