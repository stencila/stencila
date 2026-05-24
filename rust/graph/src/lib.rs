//! Build Stencila Schema graphs for workspaces and documents.
//!
//! The crate emits resource-flow [`stencila_schema::Graph`] values while
//! keeping graph construction separate from any later projection into content
//! credential assertions, reactivity plans, or code dependency graphs.
//! Concrete activities can be attached to graph edges when they explain how a
//! resource-flow relationship came about.
//!
//! Document graphs are deliberately compact. They promote the document root and
//! coarse boundary nodes such as figures, tables, files, and executable code.
//! Prose-level syntax such as headings, links, citations, media references, and
//! include markers contributes relationships to retained document containers
//! without becoming graph nodes themselves.
//!
//! Workspace graphs follow git ignore rules, retain hidden authored files, skip
//! common cache and build directories, and can decode supported document files
//! into nested document graphs.
//!
//! # Graph-local ids
//!
//! Graph endpoints are local identifiers, not dereferenceable URLs. The current
//! grammar uses readable prefixes such as `dir:<path>`, `file:<path>`,
//! `datatable:<path>`, `symlink:<path>`, `node:<scope>#<node-id>`,
//! `code:<scope>`, `package:<ecosystem>/<name>`, and `resource:<uri>`. Dynamic
//! components are percent-encoded so delimiters like `:`, `#`, and `%` remain
//! unambiguous while normal workspace path separators stay visible.
//!
//! # Static analysis limits
//!
//! Code and environment facts are best-effort static observations. The crate does
//! not execute package managers, resolve package indexes, run source code, or
//! expand dynamic file paths. Some lineage edges are intentionally marked with
//! lower confidence when they come from coarse unit-level read/write pairing.
//!
//! # Example
//!
//! ```no_run
//! use eyre::Result;
//! use stencila_graph::{WorkspaceOptions, graph_from_path};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let graph = graph_from_path(".", Some(WorkspaceOptions::default())).await?;
//!     println!("{} nodes", graph.nodes.len());
//!     Ok(())
//! }
//! ```

mod builder;
pub mod code;
mod document;
pub mod dot;
mod environment;
mod evidence;
mod ids;
mod package;
pub mod project;
mod reference;
mod workspace;

pub use builder::GraphBuilder;
pub(crate) use document::{DocumentReferenceKind, add_document_with_reference_resolver};
pub use document::{add_document, graph_from_node};
pub use project::{
    GraphConnectedMode, GraphContainmentMode, GraphEdgeFamily, GraphProjectionDetail,
    GraphProjectionOptions, GraphProjectionPreset, GraphView, GraphViewEdge, GraphViewNode,
    GraphViewNodeKind, edge_family, filter_graph_view_connected_to, project_graph,
};
pub use stencila_schema::{Graph, GraphEdge, GraphEdgeKind, GraphNode};
pub use workspace::{WorkspaceOptions, graph_from_path};
