use std::ops::ControlFlow;
use std::sync::{
    LazyLock,
    atomic::{AtomicI32, Ordering},
};

use async_lsp::{
    ClientSocket, Error, LanguageClient,
    lsp_types::{
        NumberOrString, ProgressParams, ProgressParamsValue, WorkDoneProgress,
        WorkDoneProgressBegin, WorkDoneProgressCancelParams, WorkDoneProgressCreateParams,
        WorkDoneProgressEnd, WorkDoneProgressReport,
    },
};
use eyre::Result;
use tokio::sync::mpsc;

use crate::ServerState;

static PROGRESS_TOKEN: LazyLock<AtomicI32> = LazyLock::new(AtomicI32::default);

/// Create and begin a progress notification
pub(super) async fn create_progress(
    mut client: ClientSocket,
    title: String,
    cancellable: bool,
) -> mpsc::UnboundedSender<(u32, Option<String>)> {
    // Create the token for the progress
    let token = NumberOrString::Number(PROGRESS_TOKEN.fetch_add(1, Ordering::Relaxed));

    // Request that the progress be created
    client
        .work_done_progress_create(WorkDoneProgressCreateParams {
            token: token.clone(),
        })
        .await
        .ok();

    // Begin the progress
    client
        .progress(ProgressParams {
            token: token.clone(),
            value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(WorkDoneProgressBegin {
                title,
                cancellable: Some(cancellable),
                ..Default::default()
            })),
        })
        .ok();

    // Create channel and async task to update progress
    let (sender, mut receiver) = mpsc::unbounded_channel::<(u32, Option<String>)>();
    tokio::spawn(async move {
        while let Some((percentage, message)) = receiver.recv().await {
            let work_done = if percentage >= 100 {
                WorkDoneProgress::End(WorkDoneProgressEnd {
                    ..Default::default()
                })
            } else {
                WorkDoneProgress::Report(WorkDoneProgressReport {
                    percentage: Some(percentage),
                    message: Some(message.unwrap_or_else(|| format!("{percentage}%"))),
                    ..Default::default()
                })
            };

            client
                .progress(ProgressParams {
                    token: token.clone(),
                    value: ProgressParamsValue::WorkDone(work_done),
                })
                .ok();
        }
    });

    sender
}

/// Handle a notification from the client to cancel a task previously associated
/// with `WorkDoneProgressBegin`
pub(crate) fn cancel_progress(
    _state: &mut ServerState,
    params: WorkDoneProgressCancelParams,
) -> ControlFlow<Result<(), Error>> {
    tracing::info!("cancel_progress: {:?}", params.token);

    // TODO: Cancel the task associated with the token

    ControlFlow::Continue(())
}
