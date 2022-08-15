use std::{collections::HashMap, path::Path, sync::Arc};

use common::{
    eyre::Result,
    tokio::sync::{mpsc::UnboundedSender, RwLock},
    tracing,
};
use graph::Graph;
use graph_triples::{resources, Resource};
use node_address::{Address, AddressMap};
use node_patch::diff_address;
use node_pointer::resolve;
use path_utils::path_slash::PathBufExt;
use stencila_schema::{
    CodeChunk, CodeExecutableCodeDependencies, CodeExecutableCodeDependents,
    CodeExecutableExecuteRequired, CodeExpression, File, Include, Node, Parameter,
    ParameterExecuteRequired,
};

use crate::{utils::send_patches, CompileContext, Executable, PatchRequest, When};

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
/// Uses a `RwLock` for `root` so that write lock can be held for as short as
/// time as possible and for consistency with the `execute` function.
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
pub async fn compile(
    path: &Path,
    project: &Path,
    root: &Arc<RwLock<Node>>,
    patch_sender: &UnboundedSender<PatchRequest>,
) -> Result<(AddressMap, Graph)> {
    // Walk over the root node calling `compile` on children
    let mut address = Address::default();
    let mut context = CompileContext::new(path, project);
    root.write()
        .await
        .compile(&mut address, &mut context)
        .await?;
    let (address_map, resource_infos) = (context.address_map, context.resource_infos);

    // Send any patches generated during compile
    send_patches(patch_sender, context.patches, When::Never);

    // Construct a new `Graph` from the collected `ResourceInfo`s and get an updated
    // set of resource infos from it (with data on inter-dependencies etc)
    let graph = Graph::from_resource_infos(path, resource_infos)?;

    // Generate patches for properties that can only be derived from the graph (i.e. those based on inter-dependencies)

    // In this first pass, iterate over the resources in the document's graph and collect all the code related nodes and generate new `execute_required`
    // properties for them. This needs to be done first so that these properties can be set in `code_dependencies` and `code_dependents` arrays
    // of other nodes
    let resource_infos = graph.get_resource_infos();
    let root = root.read().await;
    let nodes: HashMap<String, _> = resource_infos
        .iter()
        .filter_map(|(resource, resource_info)| {
            // Get the node id from the resource
            let node_id = match resource {
                Resource::Code(resources::Code { id, .. })
                | Resource::Node(resources::Node { id, .. }) => id,
                _ => {
                    return None;
                }
            };

            // Get the node from the document
            let address = address_map.get(node_id).cloned();
            let node = match resolve(&*root, address.clone(), Some(node_id.clone()))
                .and_then(|pointer| pointer.to_node())
            {
                Ok(node) => node,
                Err(error) => {
                    tracing::warn!(
                        "Unable to resolve node with id `{}` and address `{}`: {}",
                        node_id,
                        address
                            .map(|address| address.to_string())
                            .unwrap_or_default(),
                        error
                    );
                    return None;
                }
            };

            if let Resource::Code(resources::Code { id: node_id, .. }) = resource {
                // Determine `execute_required` by comparing `compile_digest` to `execute_digest`
                let execute_required = if let Some(compile_digest) = &resource_info.compile_digest {
                    match &resource_info.execute_digest {
                        None => CodeExecutableExecuteRequired::NeverExecuted,
                        Some(execute_digest) => {
                            if compile_digest.semantic_digest != execute_digest.semantic_digest {
                                CodeExecutableExecuteRequired::SemanticsChanged
                            } else if compile_digest.dependencies_digest
                                != execute_digest.dependencies_digest
                            {
                                CodeExecutableExecuteRequired::DependenciesChanged
                            } else if compile_digest.dependencies_failed > 0 {
                                CodeExecutableExecuteRequired::DependenciesFailed
                            } else {
                                CodeExecutableExecuteRequired::No
                            }
                        }
                    }
                } else {
                    tracing::warn!(
                        "Compile digest was unexpectedly missing for code node `{}`",
                        node_id
                    );
                    return None;
                };

                Some((
                    node_id.clone(),
                    (address, node, resource_info, Some(execute_required)),
                ))
            } else if let Resource::Node(resources::Node { id: node_id, .. }) = resource {
                Some((node_id.clone(), (address, node, resource_info, None)))
            } else {
                None
            }
        })
        .collect();
    drop(root);

    // In this second pass, iterate over the nodes collected above, derive some more properties, and calculate patches
    let patches = nodes
        .values()
        .map(|(address, node, resource_info, new_execute_required)| {
            let dependencies = resource_info
                .dependencies
                .iter()
                .flatten()
                .filter_map(|dependency| match dependency {
                    Resource::Code(code) => {
                        let (_address, node, .., new_execute_required) = match nodes.get(&code.id) {
                            Some(entry) => entry,
                            None => return None,
                        };
                        match node {
                            Node::CodeChunk(node) => {
                                Some(CodeExecutableCodeDependencies::CodeChunk(CodeChunk {
                                    id: node.id.clone(),
                                    label: node.label.clone(),
                                    programming_language: node.programming_language.clone(),
                                    execute_auto: node.execute_auto.clone(),
                                    execute_required: new_execute_required.clone(),
                                    execute_status: node.execute_status.clone(),
                                    ..Default::default()
                                }))
                            }
                            Node::Parameter(node) => {
                                let execute_required = if node.execute_digest.is_none() {
                                    ParameterExecuteRequired::NeverExecuted
                                } else if node.execute_digest == node.compile_digest {
                                    ParameterExecuteRequired::No
                                } else {
                                    ParameterExecuteRequired::SemanticsChanged
                                };

                                Some(CodeExecutableCodeDependencies::Parameter(Parameter {
                                    id: node.id.clone(),
                                    name: node.name.clone(),
                                    execute_required: Some(execute_required),
                                    ..Default::default()
                                }))
                            }
                            _ => None,
                        }
                    }
                    Resource::File(file) => Some(CodeExecutableCodeDependencies::File(File {
                        path: graph
                            .relative_path(&file.path, true)
                            .to_slash_lossy()
                            .to_string(),
                        ..Default::default()
                    })),
                    _ => None,
                })
                .collect();

            let dependents = resource_info
                .dependents
                .iter()
                .flatten()
                .filter_map(|dependency| match dependency {
                    Resource::Code(code) => {
                        let (_address, node, .., new_execute_required) = match nodes.get(&code.id) {
                            Some(entry) => entry,
                            None => return None,
                        };
                        match node {
                            Node::CodeChunk(dependant) => {
                                Some(CodeExecutableCodeDependents::CodeChunk(CodeChunk {
                                    id: dependant.id.clone(),
                                    label: dependant.label.clone(),
                                    programming_language: dependant.programming_language.clone(),
                                    execute_auto: dependant.execute_auto.clone(),
                                    execute_required: new_execute_required.clone(),
                                    execute_status: dependant.execute_status.clone(),
                                    ..Default::default()
                                }))
                            }
                            Node::CodeExpression(dependant) => Some(
                                CodeExecutableCodeDependents::CodeExpression(CodeExpression {
                                    id: dependant.id.clone(),
                                    programming_language: dependant.programming_language.clone(),
                                    execute_required: new_execute_required.clone(),
                                    execute_status: dependant.execute_status.clone(),
                                    ..Default::default()
                                }),
                            ),
                            _ => None,
                        }
                    }
                    Resource::File(file) => Some(CodeExecutableCodeDependents::File(File {
                        path: file.path.to_slash_lossy().to_string(),
                        ..Default::default()
                    })),
                    _ => None,
                })
                .collect();

            let new_compile_digest = resource_info
                .compile_digest
                .as_ref()
                .map(|digest| Box::new(digest.to_cord()));

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
                    *compile_digest = new_compile_digest;
                    *execute_required = new_execute_required.to_owned();
                }
                Node::Parameter(Parameter { compile_digest, .. }) => {
                    *compile_digest = new_compile_digest;
                }
                Node::Include(Include { compile_digest, .. }) => {
                    *compile_digest = new_compile_digest;
                }
                _ => (),
            }

            diff_address(address.clone().unwrap(), node, &after)
        })
        .collect();

    // Send the interdependency patches
    send_patches(patch_sender, patches, When::Never);

    Ok((address_map, graph))
}
