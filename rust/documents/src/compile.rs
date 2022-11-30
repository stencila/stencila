use std::{
    path::Path,
    sync::Arc,
    time::{Duration},
};

use common::{
    eyre::{bail, Result},
    itertools::Itertools,
    tokio::{
        self,
        sync::{RwLock},
    },
    tracing,
};

use graph::Graph;
use graph_triples::{TagMap};
use kernels::KernelSpace;
use node_address::Address;





use crate::{
    document::{
        Document, DocumentCompileRequestReceiver, DocumentEventListeners, DocumentEventSender,
        DocumentExecuteRequestSender, DocumentPatchRequestSender, DocumentResponseSender,
        DocumentRoot, DocumentWriteRequestSender,
    },
    executable::{CompileContext, Executable},
    messages::{
        await_response, forward_execute_requests, forward_write_requests, send_patches,
        send_responses, CompileRequest, RequestId, Then, When,
    },
    send_request,
};

impl Document {
    /// Compile the document
    ///
    /// This method is the same as `compile_request` but will wait for the compilation to finish
    /// before returning. This is useful in some circumstances, such as ensuring the document
    /// is compiled before it is encoded as HTML, on initial opening.
    #[tracing::instrument(skip(self))]
    pub async fn compile(&mut self, then: Option<Then>) -> Result<()> {
        let request_id = self.compile_request(then).await?;
        await_response(
            &mut self.response_receiver,
            &self.id,
            "compile",
            request_id,
            5,
        )
        .await
    }

    /// Request that the the document be compiled
    ///
    /// Sends a [`CompileRequest`] to the document's `compile_task`.
    #[tracing::instrument(skip(self))]
    pub async fn compile_request(&self, then: Option<Then>) -> Result<RequestId> {
        let then = then.unwrap_or_else(|| Then::write(When::Later));
        let request = CompileRequest::now(then);
        send_request!(self.compile_request_sender, &self.id, "compile", request)
    }

    /// A background task to compile the root node of the document on request
    ///
    /// # Arguments
    ///
    /// - `id`: The id of the document
    ///
    /// - `path`: The path of the document to be compiled
    ///
    /// - `project`: The project of the document to be compiled
    ///
    /// - `root`: The root [`Node`] to apply the compilation patch to
    ///
    /// - `tags`: The document's global [`TagMap`] to be updated
    ///
    /// - `graph`:  The [`Graph`] to be updated
    ///
    /// - `kernel_space`: The [`KernelSpace`] within which to execute the plan
    ///
    /// - `patch_sender`: A [`PatchRequest`] channel to send patches describing the changes to
    ///                   compiled nodes
    ///
    /// - `execute_sender`: An [`ExecuteRequest`] channel to send any requests to execute the
    ///                     document after it has been compiled
    ///
    /// - `write_sender`: The channel to send any [`WriteRequest`]s after a patch is applied
    ///
    /// - `request_receiver`: The channel to receive [`CompileRequest`]s on
    ///
    /// - `response_sender`: The channel to send a [`Response`] on when each request if fulfilled
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn compile_task(
        id: &str,
        path: &Path,
        project: &Path,
        root: &DocumentRoot,
        tags: &Arc<RwLock<TagMap>>,
        graph: &Arc<RwLock<Graph>>,
        kernel_space: &Arc<RwLock<KernelSpace>>,
        event_sender: &DocumentEventSender,
        event_listeners: &DocumentEventListeners,
        patch_sender: &DocumentPatchRequestSender,
        execute_sender: &DocumentExecuteRequestSender,
        write_sender: &DocumentWriteRequestSender,
        request_receiver: &mut DocumentCompileRequestReceiver,
        response_sender: &DocumentResponseSender,
    ) {
        let duration = Duration::from_millis(Document::COMPILE_DEBOUNCE_MILLIS);
        let mut request_ids = Vec::new();
        let mut then = Then::nothing();
        loop {
            match tokio::time::timeout(duration, request_receiver.recv()).await {
                // Request received: record and continue to wait for timeout unless `when` is now
                Ok(Some(mut request)) => {
                    if !matches!(request.when, When::Never) {
                        request_ids.append(&mut request.ids);
                        then.no_later_than(request.then);
                        if !matches!(request.when, When::Now) {
                            continue;
                        }
                    }
                }
                // Sender dropped: end of task
                Ok(None) => break,
                // Timeout so do the following with the last unhandled request, if any
                Err(..) => {}
            };

            if request_ids.is_empty() {
                continue;
            }

            tracing::trace!(
                "Compiling document `{}` for requests `{}`",
                id,
                request_ids.iter().join(",")
            );
            match compile(
                path,
                project,
                root,
                tags,
                kernel_space,
                event_sender,
                event_listeners,
                patch_sender,
            )
            .await
            {
                Ok(new_graph) => {
                    *graph.write().await = new_graph;
                }
                Err(error) => tracing::error!("While compiling document `{}`: {}", id, error),
            }

            // Forward requests or respond
            if !matches!(then.execute, When::Never) {
                forward_execute_requests(execute_sender, request_ids, then.execute, then).await;
            } else if !matches!(then.write, When::Never) {
                forward_write_requests(write_sender, request_ids, then.write).await
            } else {
                send_responses(response_sender, request_ids)
            }

            request_ids = Vec::new();
            then = Then::nothing();
        }
    }
}

/// Compile the `root` node of a document
///
/// # Arguments
///
/// - `path`: The path of the document to be compiled
///
/// - `project`: The project of the document to be compiled
///
/// - `root`: The root node to be compiled
///
/// - `tag_map`: The document's [`TagMap`]
///
/// - `kernel_space`: The document's [`KernelSpace`]
///
/// - `patch_sender`: A [`Patch`] channel sender to send patches for changes to the compiled nodes
#[allow(clippy::too_many_arguments)]
pub async fn compile(
    path: &Path,
    project: &Path,
    root: &DocumentRoot,
    tag_map: &Arc<RwLock<TagMap>>,
    kernel_space: &Arc<RwLock<KernelSpace>>,
    event_sender: &DocumentEventSender,
    event_listeners: &DocumentEventListeners,
    patch_sender: &DocumentPatchRequestSender,
) -> Result<Graph> {
    let root = root.read().await;
    let kernel_space = kernel_space.read().await;

    // Compile the root node
    let mut address = Address::default();
    let mut context = CompileContext {
        path: path.into(),
        project: project.into(),
        kernel_space: &kernel_space,
        resource_infos: Vec::default(),
        global_tags: TagMap::default(),
        event_listeners: Vec::default(),
        patches: Vec::default(),
    };
    root.compile(&mut address, &mut context).await?;

    // Send patches collected during compilation reflecting changes to nodes
    send_patches(patch_sender, context.patches, Then::nothing()).await;

    // Register all event listeners
    Document::listen_many(event_sender, event_listeners, context.event_listeners).await?;

    // Update the document's tag map with any global tags collected during compilation
    *tag_map.write().await = context.global_tags;

    // Construct a new `Graph` from the collected `ResourceInfo`s and get an updated
    // set of resource infos from it (with data on inter-dependencies etc)
    let resource_infos = context.resource_infos;
    let graph = Graph::from_resource_infos(path, resource_infos)?;

    /*

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
    */

    Ok(graph)
}
