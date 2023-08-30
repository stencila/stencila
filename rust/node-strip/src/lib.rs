//! Provides a `NodeStrip` trait for stripping properties from nodes

use std::collections::HashMap;

use common::indexmap::IndexMap;

pub use node_strip_derive::StripNode;

/// The target properties for the strip
#[derive(Clone, Default)]
pub struct Targets {
    /// Whether to strip the `id` property of the node
    pub id: bool,

    /// Whether to strip code properties of executable nodes
    ///
    /// Includes any properties that define the execution of a node e.g.
    ///
    /// - `code` and `programmingLanguage` of any `CodeExecutable` node
    /// - `source` of a `Include` or `Call` node
    pub code: bool,

    /// Whether to strip execution related properties of executable nodes
    ///
    /// Includes any properties that record the execution state of a node e.g.
    ///
    /// - `executionCount` of any `Executable` node
    pub execution: bool,

    /// Whether to strip output properties of executable nodes
    ///
    /// Includes any properties that are the result of executing a node e.g.
    ///
    /// - `outputs` of a `CodeChunk` node
    /// - `output` of a `CodeExpression` node
    /// - `content` of a `Include` or `Call` node
    pub output: bool,
}

impl Targets {
    /// Strip the `id` property only
    pub fn id() -> Self {
        Self {
            id: true,
            ..Default::default()
        }
    }

    /// Strip all targets
    pub fn all() -> Self {
        Self {
            id: true,
            code: true,
            execution: true,
            output: true,
        }
    }
}

pub trait StripNode: Sized {
    /// Strip one or more properties from a node
    ///
    /// # Arguments
    ///
    /// - `targets`: The target properties to be stripped
    #[allow(unused_variables)]
    fn strip(&mut self, targets: &Targets) -> &mut Self {
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
    fn strip(&mut self, targets: &Targets) -> &mut Self {
        self.as_mut().strip(targets);

        self
    }
}

impl<T> StripNode for Option<T>
where
    T: StripNode,
{
    fn strip(&mut self, targets: &Targets) -> &mut Self {
        if let Some(value) = self {
            value.strip(targets);
        }

        self
    }
}

impl<T> StripNode for Vec<T>
where
    T: StripNode,
{
    fn strip(&mut self, targets: &Targets) -> &mut Self {
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
    fn strip(&mut self, targets: &Targets) -> &mut Self {
        for node in self.values_mut() {
            node.strip(targets);
        }

        self
    }
}

impl<T> StripNode for IndexMap<String, T>
where
    T: StripNode,
{
    fn strip(&mut self, targets: &Targets) -> &mut Self {
        for node in self.values_mut() {
            node.strip(targets);
        }

        self
    }
}
