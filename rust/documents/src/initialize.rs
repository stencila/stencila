use std::{path::Path, sync::Arc, time::Instant};

use common::tokio::{
    self,
    sync::{broadcast, mpsc, RwLock},
};
use formats::FormatSpec;
use graph::Graph;
use graph_triples::TagMap;
use kernels::KernelSpace;
use stencila_schema::Node;

use crate::{
    document::{Document, DocumentSubscribers},
    messages::{
        CancelRequest, CompileRequest, ExecuteRequest, PatchRequest, Response, WriteRequest,
    },
};

impl Document {
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub(crate) fn initialize(
        id: &str,
        path: &Path,
        project: &Path,
        format: &FormatSpec,
        root: &Arc<RwLock<Node>>,
        tags: &Arc<RwLock<TagMap>>,
        graph: &Arc<RwLock<Graph>>,
        kernels: &Arc<RwLock<KernelSpace>>,
        subscribers: &Arc<RwLock<DocumentSubscribers>>,
        last_write: &Arc<RwLock<Instant>>,
    ) -> (
        mpsc::UnboundedSender<PatchRequest>,
        mpsc::Sender<CompileRequest>,
        mpsc::Sender<ExecuteRequest>,
        mpsc::Sender<CancelRequest>,
        mpsc::UnboundedSender<WriteRequest>,
        broadcast::Receiver<Response>,
    ) {
        let (patch_request_sender, mut patch_request_receiver) =
            mpsc::unbounded_channel::<PatchRequest>();

        let (compile_request_sender, mut compile_request_receiver) =
            mpsc::channel::<CompileRequest>(100);

        let (execute_request_sender, mut execute_request_receiver) =
            mpsc::channel::<ExecuteRequest>(100);

        let (cancel_request_sender, mut cancel_request_receiver) =
            mpsc::channel::<CancelRequest>(100);

        let (write_request_sender, mut write_request_receiver) =
            mpsc::unbounded_channel::<WriteRequest>();

        let (response_sender, response_receiver) = broadcast::channel::<Response>(1);

        let id_clone = id.to_string();
        let root_clone = root.clone();
        let subscribers_clone = subscribers.clone();
        let compile_sender_clone = compile_request_sender.clone();
        let write_sender_clone = write_request_sender.clone();
        let response_sender_clone = response_sender.clone();
        tokio::spawn(async move {
            Self::patch_task(
                &id_clone,
                &root_clone,
                &subscribers_clone,
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

        let root_clone = root.clone();
        let last_write_clone = last_write.clone();
        let path_clone = path.to_path_buf();
        let format_clone = Some(format.extension.clone());
        tokio::spawn(async move {
            Self::write_task(
                &root_clone,
                &last_write_clone,
                &path_clone,
                format_clone.as_deref(),
                &mut write_request_receiver,
                &response_sender,
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
