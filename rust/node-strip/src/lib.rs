//! Implements a `Strip` trait for removing properties from nodes

mod index_map;
mod text;
mod vec;

/// The target properties for the strip e.g. identifiers, code execution related etc
#[derive(Clone, Copy)]
pub enum Targets {
    /// Strip the `id` property of the node (ie. set to `None`)
    Id,
}

pub trait Strip: Sized {
    /// Strip one or more properties from a node
    /// 
    /// # Arguments
    /// 
    /// - `targets`: The target properties to be stripped
    #[allow(unused_variables)]
    fn strip(&mut self, targets: Targets) -> &mut Self {
        self
    }
}
