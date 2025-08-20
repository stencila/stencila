use common::smart_default::SmartDefault;
use schema::WalkNode;

use crate::{collector::Collector, replacer::Replacer, sectioner::Sectioner};

mod collector;
mod replacer;
mod sectioner;

#[cfg(test)]
mod tests;

/// Options for document structuring
#[derive(Debug, Clone, SmartDefault)]
pub struct StructuringOptions {
    /// Whether to create nested sections from headings
    #[default = true]
    pub sectioning: bool,
}

/// Add structure to a document with default options
pub fn structuring<T: WalkNode>(node: &mut T) {
    structuring_with_options(node, StructuringOptions::default())
}

/// Add structure to a document with custom options
pub fn structuring_with_options<T: WalkNode>(node: &mut T, options: StructuringOptions) {
    let mut collector = Collector::default();
    node.walk_mut(&mut collector);
    collector.determine_citation_style();

    let mut replacer = Replacer::new(collector);
    node.walk_mut(&mut replacer);

    if options.sectioning {
        let mut sectioner = Sectioner::default();
        node.walk_mut(&mut sectioner);
    }
}
