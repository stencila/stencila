use crate::{utils::send_patches, CompileContext, Executable, PatchRequest};
use eyre::Result;
use graph::Graph;
use graph_triples::{resources, Resource};
use node_address::{Address, AddressMap};
use node_patch::diff_address;
use node_pointer::resolve;
use std::{collections::HashMap, path::Path, sync::Arc};
use stencila_schema::{
    CodeChunk, CodeExecutableCodeDependencies, CodeExecutableCodeDependents,
    CodeExecutableExecuteRequired, CodeExpression, Cord, Node, Parameter,
};
use tokio::sync::{mpsc::UnboundedSender, RwLock};

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
    root.write().await.compile(&mut address, &mut context)?;
    let (address_map, resource_infos) = (context.address_map, context.resource_infos);

    // Construct a new `Graph` from the collected `ResourceInfo`s and get an updated
    // set of resource infos from it (with data on inter-dependencies etc)
    let graph = Graph::from_resource_infos(path, resource_infos)?;

    // Send patches with new dependency information to root
    compile_patches_and_send(&*root.read().await, &address_map, &graph, patch_sender);

    Ok((address_map, graph))
}

/// Compile a node, usually the `root` node of a document, wihout walking it
/// by using an existing address map and graph
///
/// Uses a `RwLock` for the first three parameters for consistency with the `compile` function
/// but only takes read locks of these.
pub async fn compile_no_walk(
    root: &Arc<RwLock<Node>>,
    address_map: &Arc<RwLock<AddressMap>>,
    graph: &Arc<RwLock<Graph>>,
    patch_sender: &UnboundedSender<PatchRequest>,
) -> Result<()> {
    compile_patches_and_send(
        &*root.read().await,
        &*address_map.read().await,
        &*graph.read().await,
        patch_sender,
    );
    Ok(())
}

/// Update nodes in root with information from dependency graph by sending patches
fn compile_patches_and_send(
    root: &Node,
    address_map: &AddressMap,
    graph: &Graph,
    patch_sender: &UnboundedSender<PatchRequest>,
) {
    // Collect all the code nodes in the graph and new values for some of their properties
    let resource_infos = graph.get_resource_infos();
    let nodes: HashMap<String, _> = resource_infos
        .iter()
        .filter_map(|(resource, resource_info)| {
            if let Resource::Code(resources::Code { id: node_id, .. }) = resource {
                let address = address_map.get(node_id).cloned();
                let node = if let Ok(node) = resolve(root, address.clone(), Some(node_id.clone()))
                    .and_then(|pointer| pointer.to_node())
                {
                    node
                } else {
                    tracing::warn!("Unable to resolve node `{}`", node_id);
                    return None;
                };

                // Collect ids of dependencies and dependents for use later

                let dependencies: Vec<String> = resource_info
                    .dependencies
                    .iter()
                    .flatten()
                    .filter_map(|resource| resource.node_id().map(|id| id.to_string()))
                    .collect();

                let dependents: Vec<String> = resource_info
                    .dependents
                    .iter()
                    .flatten()
                    .filter_map(|resource| resource.node_id().map(|id| id.to_string()))
                    .collect();

                let (compile_digest, execute_required) = if let Some(compile_digest) =
                    &resource_info.compile_digest
                {
                    // Determine `execute_required` by comparing `compile_digest` to `execute_digest`
                    let execute_required = match &resource_info.execute_digest {
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
                    };
                    // Make `compile_digest` of correct type
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

    // Derive some more properties from the first set, apply the new properties to the node, calculate patches
    let patches = nodes
        .values()
        .map(
            |(
                address,
                node,
                dependencies,
                dependents,
                new_compile_digest,
                new_execute_required,
            )| {
                let dependencies = dependencies
                    .iter()
                    .filter_map(|dependency_id| nodes.get(dependency_id))
                    .filter_map(
                        |(_address, dependency, .., new_execute_required)| match dependency {
                            Node::CodeChunk(dependency) => {
                                Some(CodeExecutableCodeDependencies::CodeChunk(CodeChunk {
                                    id: dependency.id.clone(),
                                    label: dependency.label.clone(),
                                    programming_language: dependency.programming_language.clone(),
                                    execute_auto: dependency.execute_auto.clone(),
                                    execute_required: Some(new_execute_required.clone()),
                                    execute_status: dependency.execute_status.clone(),
                                    ..Default::default()
                                }))
                            }
                            Node::Parameter(dependency) => {
                                Some(CodeExecutableCodeDependencies::Parameter(Parameter {
                                    id: dependency.id.clone(),
                                    name: dependency.name.clone(),
                                    ..Default::default()
                                }))
                            }
                            _ => None,
                        },
                    )
                    .collect();

                let dependents = dependents
                    .iter()
                    .filter_map(|dependent_id| nodes.get(dependent_id))
                    .filter_map(
                        |(_address, dependent, .., new_execute_required)| match dependent {
                            Node::CodeChunk(dependant) => {
                                Some(CodeExecutableCodeDependents::CodeChunk(CodeChunk {
                                    id: dependant.id.clone(),
                                    label: dependant.label.clone(),
                                    programming_language: dependant.programming_language.clone(),
                                    execute_auto: dependant.execute_auto.clone(),
                                    execute_required: Some(new_execute_required.clone()),
                                    execute_status: dependant.execute_status.clone(),
                                    ..Default::default()
                                }))
                            }
                            Node::CodeExpression(dependant) => Some(
                                CodeExecutableCodeDependents::CodeExpression(CodeExpression {
                                    id: dependant.id.clone(),
                                    programming_language: dependant.programming_language.clone(),
                                    execute_required: Some(new_execute_required.clone()),
                                    execute_status: dependant.execute_status.clone(),
                                    ..Default::default()
                                }),
                            ),
                            _ => None,
                        },
                    )
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

    send_patches(patch_sender, patches, false)
}
