//! Provides a `Strip` trait for removing properties from nodes

/// The target properties for the strip e.g. identifiers, code execution related etc
#[derive(Clone, Default)]
pub struct Targets {
    /// Whether to strip the `id` property of the node
    pub id: bool,

    /// Whether to strip code properties of executable nodes
    pub code: bool,

    /// Whether to strip derived properties of executable nodes
    pub execution: bool,

    /// Whether to strip output properties of executable nodes
    pub outputs: bool,
}

impl Targets {
    /// Strip the `id` property only
    pub fn id() -> Self {
        Self {
            id: true,
            ..Default::default()
        }
    }
}

pub trait Strip: Sized {
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

mod r#box;
mod hash_map;
mod index_map;
mod option;
mod primitives;
mod vec;
