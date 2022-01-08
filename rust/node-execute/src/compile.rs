use crate::{CompileContext, Executable};
use eyre::Result;
use graph_triples::ResourceInfo;
use node_address::{Address, AddressMap};
use std::path::Path;
use stencila_schema::Node;

/// Compile a node
///
/// Compiling a node involves walking over its node tree and compiling each
/// child node so that it is ready to be executed. This includes
/// (but is not limited to):
///
/// - for those node types needing to be accesses directly (e.g. executable nodes) ensuring
///   they have an `id` and recording their address
///
/// - for executable nodes (e.g. `CodeChunk`) performing semantic analysis of the code
///
/// - determining dependencies within and between documents and other resources
///
#[tracing::instrument(skip(node))]
pub fn compile(
    node: &mut Node,
    path: &Path,
    project: &Path,
) -> Result<(AddressMap, Vec<ResourceInfo>)> {
    let mut address = Address::default();
    let mut context = CompileContext::new(path, project);
    node.compile(&mut address, &mut context)?;
    Ok((context.addresses, context.resources))
}
