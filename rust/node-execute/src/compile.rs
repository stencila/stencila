use crate::{CompileContext, Executable};
use eyre::Result;
use graph::Graph;
use graph_triples::{resources, Resource};
use node_address::{Address, AddressMap};
use node_patch::{diff_address, Patch};
use node_pointer::resolve;
use std::{collections::HashMap, path::Path};
use stencila_schema::{
    CodeChunk, CodeExecutableCodeDependencies, CodeExecutableCodeDependents,
    CodeExecutableExecuteRequired, CodeExpression, Cord, Node, Parameter,
};
use tokio::sync::mpsc::UnboundedSender;

/// Compile a node, usually the `root` node of a document
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
    let (address_map, resource_infos) = (context.address_map, context.resource_infos);

    // Construct a new `Graph` from the collected `ResourceInfo`s and get an updated
    // set of resource infos from it (with data on inter-dependencies etc)
    let graph = Graph::from_resource_infos(path, resource_infos)?;

    // Send patches with new dependency information to root
    compile_patches(root, &address_map, &graph, patch_sender);

    Ok((address_map, graph))
}

/// Update nodes in root with information from dependency graph by sending patches
pub fn compile_patches(
    root: &mut Node,
    address_map: &AddressMap,
    graph: &Graph,
    patch_sender: &UnboundedSender<Patch>,
) {
    // Collect the nodes and the values of their updated properties
    let resource_infos = graph.get_resource_infos();
    let nodes: HashMap<String, _> = resource_infos
        .iter()
        .filter_map(|(resource, resource_info)| {
            if let Resource::Code(resources::Code { id: node_id, .. }) = resource {
                let address = address_map.get(node_id).cloned();
                let node = if let Ok(node) =
                    resolve(&mut *root, address.clone(), Some(node_id.clone()))
                        .and_then(|pointer| pointer.to_node())
                {
                    node
                } else {
                    tracing::warn!("Unable to resolve node `{}`", node_id);
                    return None;
                };

                let dependencies: Vec<String> = resource_info
                    .dependencies
                    .iter()
                    .flatten()
                    .filter_map(|resource| resource.node_id())
                    .collect();

                let dependents: Vec<String> = resource_info
                    .dependents
                    .iter()
                    .flatten()
                    .filter_map(|resource| resource.node_id())
                    .collect();

                let (compile_digest, execute_required) = if let Some(compile_digest) =
                    &resource_info.compile_digest
                {
                    let execute_required = match &resource_info.execute_digest {
                        None => CodeExecutableExecuteRequired::NeverExecuted,
                        Some(execute_digest) => {
                            if compile_digest.semantic_digest != execute_digest.semantic_digest {
                                CodeExecutableExecuteRequired::SemanticsChanged
                            } else if compile_digest.dependencies_digest
                                != execute_digest.dependencies_digest
                            {
                                CodeExecutableExecuteRequired::DependenciesChanged
                            } else {
                                CodeExecutableExecuteRequired::No
                            }
                        }
                    };
                    let compile_digest = Box::new(Cord(compile_digest.to_string()));
                    (compile_digest, execute_required)
                } else {
                    tracing::warn!(
                        "Compile digest was unexpectedly missing for node `{}`",
                        node_id
                    );
                    return None;
                };

                Some((
                    node_id.clone(),
                    (
                        address,
                        node,
                        dependencies,
                        dependents,
                        compile_digest,
                        execute_required,
                    ),
                ))
            } else {
                None
            }
        })
        .collect();

    // Derive some more properties from the first set, apply the new properties to the node, calculate patches and send them
    // over `patch_sender` for application.
    let patches = nodes
        .iter()
        .map(
            |(
                _node_id,
                (address, node, dependencies, dependents, new_compile_digest, new_execute_required),
            )| {
                let dependencies = dependencies
                    .iter()
                    .filter_map(|node_id| nodes.get(node_id))
                    .filter_map(|(_address, node, .., execute_required)| match node {
                        Node::CodeChunk(node) => {
                            Some(CodeExecutableCodeDependencies::CodeChunk(CodeChunk {
                                id: node.id.clone(),
                                programming_language: node.programming_language.clone(),
                                execute_required: Some(execute_required.clone()),
                                execute_status: node.execute_status.clone(),
                                ..Default::default()
                            }))
                        }
                        Node::Parameter(node) => {
                            Some(CodeExecutableCodeDependencies::Parameter(Parameter {
                                id: node.id.clone(),
                                name: node.name.clone(),
                                ..Default::default()
                            }))
                        }
                        _ => None,
                    })
                    .collect();

                let dependents = dependents
                    .iter()
                    .filter_map(|node_id| nodes.get(node_id))
                    .filter_map(|(_address, .., execute_required)| match node {
                        Node::CodeChunk(node) => {
                            Some(CodeExecutableCodeDependents::CodeChunk(CodeChunk {
                                id: node.id.clone(),
                                programming_language: node.programming_language.clone(),
                                execute_required: Some(execute_required.clone()),
                                execute_status: node.execute_status.clone(),
                                ..Default::default()
                            }))
                        }
                        Node::CodeExpression(node) => Some(
                            CodeExecutableCodeDependents::CodeExpression(CodeExpression {
                                id: node.id.clone(),
                                programming_language: node.programming_language.clone(),
                                execute_required: Some(execute_required.clone()),
                                execute_status: node.execute_status.clone(),
                                ..Default::default()
                            }),
                        ),
                        _ => None,
                    })
                    .collect();

                let mut after = node.clone();
                match &mut after {
                    Node::CodeChunk(CodeChunk {
                        code_dependencies,
                        code_dependents,
                        compile_digest,
                        execute_required,
                        ..
                    })
                    | Node::CodeExpression(CodeExpression {
                        code_dependencies,
                        code_dependents,
                        compile_digest,
                        execute_required,
                        ..
                    }) => {
                        *code_dependencies = Some(dependencies);
                        *code_dependents = Some(dependents);
                        *compile_digest = Some(new_compile_digest.clone());
                        *execute_required = Some(new_execute_required.clone());
                    }
                    _ => (),
                }

                diff_address(address.clone().unwrap(), node, &after)
            },
        )
        .collect();

    // Rather than sending many patches, combine them into one
    let patch = Patch::from_patches(patches);

    // Send patch to root, if it's not empty
    if !patch.is_empty() {
        if let Err(error) = patch_sender.send(patch) {
            tracing::debug!("When sending patch during compilation: {}", error);
        }
    }
}
