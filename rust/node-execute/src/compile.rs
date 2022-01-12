use crate::{CompileContext, Executable};
use eyre::Result;
use graph::Graph;
use graph_triples::{resources, Resource};
use node_address::{Address, AddressMap};
use node_patch::{diff, Patch};
use node_pointer::resolve;
use std::path::Path;
use stencila_schema::{CodeChunk, CodeExecutableExecuteRequired, CodeExpression, Cord, Node};
use tokio::sync::mpsc::UnboundedSender;

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
/// # Arguments
///
/// - `path`: The path of the document to be compiled
///
/// - `project`: The project of the document to be compiled
///
/// - `root`: The root node to be compiled
///
/// - `patch_sender`: A [`Patch`] channel sender to send patches describing the changes to
///                   executed nodes
///
#[tracing::instrument(skip(root))]
pub fn compile(
    path: &Path,
    project: &Path,
    root: &mut Node,
    patch_sender: &UnboundedSender<Patch>,
) -> Result<(AddressMap, Graph)> {
    // Walk over the root node calling `compile` on children
    let mut address = Address::default();
    let mut context = CompileContext::new(path, project);
    root.compile(&mut address, &mut context)?;
    let (address_map, resource_infos) = (context.addresses, context.resources);

    // Construct a `Graph` from the collected `ResourceInfo`s
    let mut graph = Graph::from_resource_infos(path, resource_infos)?;

    // Calculate the patches required to root to reflect the compilation done by the graph
    // and send them on for handling
    for resource_info in graph.get_resource_infos() {
        let resource = &resource_info.resource;
        if let Resource::Code(resources::Code { id, .. }) = resource {
            let address = address_map.get(id).cloned();
            let id = Some(id.clone());
            let pointer = resolve(&mut *root, address.clone(), id.clone())?;

            let before = pointer.to_node()?;
            let mut after = before.clone();
            match &mut after {
                Node::CodeChunk(CodeChunk {
                    compile_digest,
                    execute_digest,
                    execute_required,
                    ..
                })
                | Node::CodeExpression(CodeExpression {
                    compile_digest,
                    execute_digest,
                    execute_required,
                    ..
                }) => {
                    if let Some(new_compile_digest) = resource_info.compile_digest.as_ref() {
                        *compile_digest = Some(Box::new(Cord(new_compile_digest.to_string())));
                        *execute_required = Some(match execute_digest {
                            None => CodeExecutableExecuteRequired::NeverExecuted,
                            Some(execute_digest) => {
                                let parts = execute_digest.0.split('.').collect::<Vec<&str>>();
                                if new_compile_digest.semantic_digest
                                    != *parts.get(1).unwrap_or(&"")
                                {
                                    CodeExecutableExecuteRequired::SemanticsChanged
                                } else if new_compile_digest.dependencies_digest
                                    != *parts.get(2).unwrap_or(&"")
                                {
                                    CodeExecutableExecuteRequired::DependenciesChanged
                                } else {
                                    CodeExecutableExecuteRequired::No
                                }
                            }
                        })
                    } else {
                        tracing::warn!("The compile digest for a node was unexpectedly None");
                    }
                }
                _ => {}
            }

            let mut patch = diff(&before, &after);
            patch.address = address;
            patch.target = id;

            if !patch.is_empty() {
                if let Err(error) = patch_sender.send(patch) {
                    tracing::debug!("When sending patch during compilation: {}", error);
                }
            }
        }
    }

    Ok((address_map, graph))
}
