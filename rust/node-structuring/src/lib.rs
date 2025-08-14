use schema::WalkNode;

use crate::{collector::Collector, replacer::Replacer};

mod collector;
mod replacer;

#[cfg(test)]
mod tests;

/// Add structure to a document
pub fn structuring<T: WalkNode>(node: &mut T) {
    let mut collector = Collector::default();
    node.walk_mut(&mut collector);

    let mut replacer = Replacer::new(collector);
    node.walk_mut(&mut replacer);
}
