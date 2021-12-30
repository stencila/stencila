use eyre::Result;
use graph::Graph;
use graph_triples::{Relations, ResourceEntry};
use node_address::Addresses;
use serde::Serialize;
use std::path::Path;
use stencila_schema::Node;

/// An execution plan for the nodes in a document (or project)
#[derive(Debug, Default, Serialize)]
pub struct Plan {
    /// The topological order of resources in the document's dependency graph
    /// 
    /// Topological order ensures that dependencies of a a node are executed
    /// before it is. If there are no inter-dependencies between nodes
    /// in a document then the order will be the order that the nodes
    /// appear in the document (i.e. top to bottom, left to right).
    /// 
    /// Includes resources other than document nodes (e.g. symbols and files)
    /// so that this order can be used to react to changes in those resources
    /// as well.
    order: Vec<ResourceEntry>,
}

impl Plan {
    /// Make an execution plan for executable nodes in document
    ///
    /// # Arguments
    ///
    /// - `document`: The root node for which the plan will be generated
    /// - `path`: The path of the document (needed to create a dependency graph)
    /// - `addresses`: The addresses of executable nodes in the document (used to
    ///    collect information on the node e.g. its `programmingLanguage`)
    /// - `relations`: The dependency relations between nodes (used to create a
    ///    dependency graph)
    #[allow(clippy::ptr_arg)]
    pub fn make(
        _document: &Node,
        path: &Path,
        _addresses: &Addresses,
        relations: &Relations,
    ) -> Result<Plan> {
        // Create a dependency graph and do a topological sort
        let graph = Graph::from_relations(path, relations);
        let order = graph.toposort()?;
        Ok(Plan { order })
    }
}
