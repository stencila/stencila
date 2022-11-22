use std::{path::Path, sync::Arc, time::Instant};

use common::tokio::{
    self,
    sync::{broadcast, mpsc, RwLock},
};
use events::Event;
use formats::FormatSpec;
use graph::Graph;
use graph_triples::TagMap;
use kernels::KernelSpace;

use crate::{
    document::{
        Document, DocumentCancelRequestSender, DocumentCompileRequestSender,
        DocumentEventListeners, DocumentExecuteRequestSender, DocumentPatchRequestSender,
        DocumentResponseReceiver, DocumentRoot, DocumentVersion, DocumentWriteRequestSender,
    },
    messages::{
        CancelRequest, CompileRequest, DocumentRequestSenders, ExecuteRequest, PatchRequest,
        Response, WriteRequest,
    },
};

impl Document {
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub(crate) fn initialize(
        id: &str,
        path: &Path,
        project: &Path,
        format: &FormatSpec,
        version: &DocumentVersion,
        root: &DocumentRoot,
        tags: &Arc<RwLock<TagMap>>,
        graph: &Arc<RwLock<Graph>>,
        kernels: &Arc<RwLock<KernelSpace>>,
        event_listeners: &DocumentEventListeners,
        last_write: &Arc<RwLock<Instant>>,
    ) -> (
        DocumentPatchRequestSender,
        DocumentCompileRequestSender,
        DocumentExecuteRequestSender,
        DocumentCancelRequestSender,
        DocumentWriteRequestSender,
        DocumentResponseReceiver,
    ) {
        let (patch_request_sender, mut patch_request_receiver) = mpsc::channel::<PatchRequest>(100);

        let (compile_request_sender, mut compile_request_receiver) =
            mpsc::channel::<CompileRequest>(100);

        let (execute_request_sender, mut execute_request_receiver) =
            mpsc::channel::<ExecuteRequest>(100);

        let (cancel_request_sender, mut cancel_request_receiver) =
            mpsc::channel::<CancelRequest>(100);

        let (write_request_sender, mut write_request_receiver) = mpsc::channel::<WriteRequest>(100);

        let (response_sender, response_receiver) = broadcast::channel::<Response>(1);

        let (event_sender, mut event_receiver) = mpsc::unbounded_channel::<Event>();

        let id_clone = id.to_string();
        let version_clone = version.clone();
        let root_clone = root.clone();
        let compile_sender_clone = compile_request_sender.clone();
        let write_sender_clone = write_request_sender.clone();
        let response_sender_clone = response_sender.clone();
        tokio::spawn(async move {
            Self::patch_task(
                &id_clone,
                &version_clone,
                &root_clone,
                &compile_sender_clone,
                &write_sender_clone,
                &mut patch_request_receiver,
                &response_sender_clone,
            )
            .await
        });

        let id_clone = id.to_string();
        let path_clone = path.to_path_buf();
        let project_clone = project.to_path_buf();
        let root_clone = root.clone();
        let tags_clone = tags.clone();
        let graph_clone = graph.clone();
        let kernels_clone = kernels.clone();
        let event_listeners_clone = event_listeners.clone();
        let patch_sender_clone = patch_request_sender.clone();
        let execute_sender_clone = execute_request_sender.clone();
        let write_sender_clone = write_request_sender.clone();
        let response_sender_clone = response_sender.clone();
        tokio::spawn(async move {
            Self::compile_task(
                &id_clone,
                &path_clone,
                &project_clone,
                &root_clone,
                &tags_clone,
                &graph_clone,
                &kernels_clone,
                &event_sender,
                &event_listeners_clone,
                &patch_sender_clone,
                &execute_sender_clone,
                &write_sender_clone,
                &mut compile_request_receiver,
                &response_sender_clone,
            )
            .await
        });

        let id_clone = id.to_string();
        let path_clone = path.to_path_buf();
        let project_clone = project.to_path_buf();
        let root_clone = root.clone();
        let tags_clone = tags.clone();
        let graph_clone = graph.clone();
        let kernels_clone = kernels.clone();
        let patch_sender_clone = patch_request_sender.clone();
        let write_sender_clone = write_request_sender.clone();
        let response_sender_clone = response_sender.clone();
        tokio::spawn(async move {
            Self::execute_task(
                &id_clone,
                &path_clone,
                &project_clone,
                &root_clone,
                &tags_clone,
                &graph_clone,
                &kernels_clone,
                &patch_sender_clone,
                &write_sender_clone,
                &mut cancel_request_receiver,
                &mut execute_request_receiver,
                &response_sender_clone,
            )
            .await
        });

        let id_clone = id.to_string();
        let root_clone = root.clone();
        let last_write_clone = last_write.clone();
        let path_clone = path.to_path_buf();
        let format_clone = Some(format.extension.clone());
        tokio::spawn(async move {
            Self::write_task(
                &id_clone,
                &path_clone,
                format_clone.as_deref(),
                &root_clone,
                &last_write_clone,
                &mut write_request_receiver,
                &response_sender,
            )
            .await
        });

        let id_clone = id.to_string();
        let event_listeners_clone = event_listeners.clone();
        let request_senders = DocumentRequestSenders {
            patch: patch_request_sender.clone(),
            compile: compile_request_sender.clone(),
            execute: execute_request_sender.clone(),
            cancel: cancel_request_sender.clone(),
            write: write_request_sender.clone(),
        };
        tokio::spawn(async move {
            Self::listen_task(
                &id_clone,
                &mut event_receiver,
                &event_listeners_clone,
                &request_senders,
            )
            .await
        });

        (
            patch_request_sender,
            compile_request_sender,
            execute_request_sender,
            cancel_request_sender,
            write_request_sender,
            response_receiver,
        )
    }
}
