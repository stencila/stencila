use std::{collections::HashMap, path::Path, sync::Arc};

use common::{
    eyre::Result,
    tokio::sync::{mpsc::UnboundedSender, RwLock},
    tracing,
};
use graph::Graph;
use graph_triples::{resources, Resource, TagMap};
use kernels::KernelSpace;

use node_patch::diff_id;
use node_pointer::find;
use path_utils::path_slash::PathBufExt;
use stencila_schema::{
    Button, Call, CodeChunk, CodeExpression, Division, ExecutionDependencies, ExecutionDependents,
    ExecutionRequired, File, Include, Node, Parameter, Span,
};

use crate::{
    executable::{CompileContext, Executable},
    messages::{PatchRequest, When},
    utils::send_patch,
};

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
/// - `kernel_space`: The [`KernelSpace`] within which to execute the plan
///
/// - `patch_sender`: A [`Patch`] channel sender to send patches describing the changes to
///                   executed nodes
pub async fn compile(
    path: &Path,
    project: &Path,
    root: &Arc<RwLock<Node>>,
    tag_map: &Arc<RwLock<TagMap>>,
    kernel_space: &Arc<RwLock<KernelSpace>>,
    patch_sender: &UnboundedSender<PatchRequest>,
) -> Result<Graph> {
    let mut root = root.write().await;
    let kernel_space = kernel_space.read().await;

    // Call compile on the root
    let mut context = CompileContext {
        path: path.into(),
        project: project.into(),
        kernel_space: &*kernel_space,
        resource_infos: Vec::default(),
        global_tags: TagMap::default(),
        patches: Vec::default(),
    };
    // TODO: send patches
    root.compile(&mut context).await?;

    // Update the document's global tag map with those from those collected by the compile context
    *tag_map.write().await = context.global_tags;

    // Construct a new `Graph` from the collected `ResourceInfo`s and get an updated
    // set of resource infos from it (with data on inter-dependencies etc)
    let resource_infos = context.resource_infos;
    let graph = Graph::from_resource_infos(path, resource_infos)?;

    // Generate patches for properties that can only be derived from the graph (i.e. those based on inter-dependencies)

    // In this first pass, iterate over the resources in the document's graph and collect all the code related nodes and generate new `execution_required`
    // properties for them. This needs to be done first so that these properties can be set in `execution_dependencies` and `execution_dependents` arrays
    // of other nodes
    let resource_infos = graph.get_resource_infos();
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
            let node = match find(&*root, &node_id.clone()).and_then(|pointer| pointer.to_node()) {
                Ok(node) => node,
                Err(error) => {
                    tracing::warn!("Unable to resolve node with id `{}`: {}", node_id, error);
                    return None;
                }
            };

            if let Resource::Code(resources::Code { id: node_id, .. }) = resource {
                // Determine `execution_required` by comparing `compile_digest` to `execute_digest`
                let execution_required = if let Some(compile_digest) = &resource_info.compile_digest
                {
                    match &resource_info.execute_digest {
                        None => ExecutionRequired::NeverExecuted,
                        Some(execute_digest) => {
                            if compile_digest.semantic_digest != execute_digest.semantic_digest {
                                ExecutionRequired::SemanticsChanged
                            } else if compile_digest.dependencies_digest
                                != execute_digest.dependencies_digest
                            {
                                ExecutionRequired::DependenciesChanged
                            } else if compile_digest.dependencies_failed > 0 {
                                ExecutionRequired::DependenciesFailed
                            } else {
                                ExecutionRequired::No
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
                    (node, resource_info, Some(execution_required)),
                ))
            } else if let Resource::Node(resources::Node { id: node_id, .. }) = resource {
                Some((node_id.clone(), (node, resource_info, None)))
            } else {
                None
            }
        })
        .collect();

    drop(root);

    // In this second pass, iterate over the nodes collected above, derive some more properties,
    // and calculate and send patches
    for (id, (node, resource_info, new_execution_required)) in &nodes {
        let dependencies = resource_info
            .dependencies
            .iter()
            .flatten()
            .filter_map(|dependency| match dependency {
                Resource::Code(code) => {
                    let (node, .., new_execution_required) = match nodes.get(&code.id) {
                        Some(entry) => entry,
                        None => return None,
                    };
                    match node {
                        Node::CodeChunk(node) => {
                            Some(ExecutionDependencies::CodeChunk(CodeChunk {
                                id: node.id.clone(),
                                label: node.label.clone(),
                                programming_language: node.programming_language.clone(),
                                execution_auto: node.execution_auto.clone(),
                                execution_required: new_execution_required.clone(),
                                execution_status: node.execution_status.clone(),
                                ..Default::default()
                            }))
                        }
                        Node::Parameter(node) => {
                            let execution_required = if node.execute_digest.is_none() {
                                ExecutionRequired::NeverExecuted
                            } else if node.execute_digest == node.compile_digest {
                                ExecutionRequired::No
                            } else {
                                ExecutionRequired::SemanticsChanged
                            };

                            Some(ExecutionDependencies::Parameter(Parameter {
                                id: node.id.clone(),
                                name: node.name.clone(),
                                execution_required: Some(execution_required),
                                ..Default::default()
                            }))
                        }
                        Node::Button(node) => {
                            let execution_required = if node.execute_digest.is_none() {
                                ExecutionRequired::NeverExecuted
                            } else if node.execute_digest == node.compile_digest {
                                ExecutionRequired::No
                            } else {
                                ExecutionRequired::SemanticsChanged
                            };

                            Some(ExecutionDependencies::Button(Button {
                                id: node.id.clone(),
                                name: node.name.clone(),
                                execution_required: Some(execution_required),
                                ..Default::default()
                            }))
                        }
                        _ => None,
                    }
                }
                Resource::File(file) => Some(ExecutionDependencies::File(File {
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
                    let (node, .., new_execution_required) = match nodes.get(&code.id) {
                        Some(entry) => entry,
                        None => return None,
                    };
                    match node {
                        Node::CodeChunk(dependant) => {
                            Some(ExecutionDependents::CodeChunk(CodeChunk {
                                id: dependant.id.clone(),
                                label: dependant.label.clone(),
                                programming_language: dependant.programming_language.clone(),
                                execution_auto: dependant.execution_auto.clone(),
                                execution_required: new_execution_required.clone(),
                                execution_status: dependant.execution_status.clone(),
                                ..Default::default()
                            }))
                        }
                        Node::CodeExpression(dependant) => {
                            Some(ExecutionDependents::CodeExpression(CodeExpression {
                                id: dependant.id.clone(),
                                programming_language: dependant.programming_language.clone(),
                                execution_required: new_execution_required.clone(),
                                execution_status: dependant.execution_status.clone(),
                                ..Default::default()
                            }))
                        }
                        Node::Division(dependant) => {
                            Some(ExecutionDependents::Division(Division {
                                id: dependant.id.clone(),
                                programming_language: dependant.programming_language.clone(),
                                execution_required: new_execution_required.clone(),
                                execution_status: dependant.execution_status.clone(),
                                ..Default::default()
                            }))
                        }
                        Node::Span(dependant) => Some(ExecutionDependents::Span(Span {
                            id: dependant.id.clone(),
                            programming_language: dependant.programming_language.clone(),
                            execution_required: new_execution_required.clone(),
                            execution_status: dependant.execution_status.clone(),
                            ..Default::default()
                        })),
                        _ => None,
                    }
                }
                Resource::File(file) => Some(ExecutionDependents::File(File {
                    path: file.path.to_slash_lossy().to_string(),
                    ..Default::default()
                })),
                _ => None,
            })
            .collect();

        let new_compile_digest = resource_info.compile_digest.clone();

        let mut after = node.clone();
        match &mut after {
            Node::CodeChunk(CodeChunk {
                execution_dependencies,
                execution_dependents,
                compile_digest,
                execution_required,
                ..
            })
            | Node::CodeExpression(CodeExpression {
                execution_dependencies,
                execution_dependents,
                compile_digest,
                execution_required,
                ..
            }) => {
                *execution_dependencies = Some(dependencies);
                *execution_dependents = Some(dependents);
                *compile_digest = new_compile_digest;
                *execution_required = new_execution_required.to_owned();
            }
            Node::Parameter(Parameter { compile_digest, .. })
            | Node::Button(Button { compile_digest, .. })
            | Node::Include(Include { compile_digest, .. }) => {
                *compile_digest = new_compile_digest;
            }
            Node::Call(Call {
                execution_dependencies,
                compile_digest,
                execution_required,
                ..
            })
            | Node::Division(Division {
                execution_dependencies,
                compile_digest,
                execution_required,
                ..
            })
            | Node::Span(Span {
                execution_dependencies,
                compile_digest,
                execution_required,
                ..
            }) => {
                *execution_dependencies = Some(dependencies);
                *compile_digest = new_compile_digest;
                *execution_required = new_execution_required.to_owned();
            }
            _ => (),
        }

        let patch = diff_id(id, node, &after);
        send_patch(patch_sender, patch, When::Never);
    }

    Ok(graph)
}
