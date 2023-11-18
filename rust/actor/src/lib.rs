// Re-exports for the convenience of crates implementing `Actor`
pub use common;
pub use graph;
pub use schema;

use common::{async_trait::async_trait, eyre::Result};
use graph::Graph;
use schema::Node;

/// An actor on a document
#[async_trait]
pub trait Actor {
    /// Perform actions on a document
    ///
    /// To avoid circular dependencies, this function takes the `root` node
    /// and `graph` of the document, rather than the entire document.
    async fn perform(&mut self, root: &mut Node, graph: &mut Graph) -> Result<()>;
}
