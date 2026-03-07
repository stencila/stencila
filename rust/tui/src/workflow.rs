use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use stencila_attractor::events::{EventEmitter, PipelineEvent};
use stencila_attractor::interviewer::Interviewer;
use stencila_attractor::types::Outcome;

use crate::autocomplete::workflows::WorkflowDefinitionInfo;
use crate::interview::{PendingTuiInterview, TuiInterviewer};

/// Events from the workflow runtime to the TUI.
pub enum WorkflowEvent {
    /// A pipeline event forwarded directly from the attractor engine.
    Pipeline(PipelineEvent),
    /// An interview delivered through the shared `TuiInterviewer`.
    Interview(PendingTuiInterview),
    /// The workflow run completed (success or failure).
    Completed(eyre::Result<Outcome>),
}

/// Handle to a running workflow task.
pub struct WorkflowRunHandle {
    pub event_rx: mpsc::UnboundedReceiver<WorkflowEvent>,
    #[allow(dead_code)]
    join_handle: JoinHandle<()>,
}

impl WorkflowRunHandle {
    pub fn abort(&self) {
        self.join_handle.abort();
    }

    #[cfg(test)]
    pub fn new_for_test(event_rx: mpsc::UnboundedReceiver<WorkflowEvent>) -> Self {
        Self {
            event_rx,
            join_handle: tokio::spawn(async {}),
        }
    }
}

/// An `EventEmitter` that forwards pipeline events through a channel to the TUI.
struct TuiEventEmitter {
    tx: mpsc::UnboundedSender<WorkflowEvent>,
}

impl EventEmitter for TuiEventEmitter {
    fn emit(&self, event: PipelineEvent) {
        let _ = self.tx.send(WorkflowEvent::Pipeline(event));
    }
}

/// Bridge that forwards pending interviews from `TuiInterviewer` through the
/// workflow event channel so they can be picked up by `poll_workflow_events`.
fn spawn_interview_forwarder(
    event_tx: mpsc::UnboundedSender<WorkflowEvent>,
) -> Arc<dyn Interviewer> {
    let (itx, mut irx) = mpsc::unbounded_channel::<PendingTuiInterview>();
    let interviewer: Arc<dyn Interviewer> = Arc::new(TuiInterviewer::new(itx));
    tokio::spawn(async move {
        while let Some(pending) = irx.recv().await {
            if event_tx.send(WorkflowEvent::Interview(pending)).is_err() {
                break;
            }
        }
        tracing::trace!("interview forwarder task exited");
    });
    interviewer
}

/// Spawn a workflow run on a background task.
///
/// Returns the run handle and total number of pipeline stages (graph nodes).
pub fn spawn_workflow(info: &WorkflowDefinitionInfo, goal: String) -> (WorkflowRunHandle, usize) {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let name = info.name.clone();
    let total_stages = compute_total_stages(&name);

    let join_handle = tokio::spawn(async move {
        let completion_tx = event_tx.clone();
        let emitter: Arc<dyn EventEmitter> = Arc::new(TuiEventEmitter {
            tx: event_tx.clone(),
        });

        let interviewer = spawn_interview_forwarder(event_tx);

        let cwd = std::env::current_dir().unwrap_or_default();
        let result = async {
            let mut workflow = stencila_workflows::get_by_name(&cwd, &name).await?;
            workflow.inner.goal = Some(goal);

            let options = stencila_workflows::RunOptions {
                emitter,
                interviewer: Some(interviewer),
            };
            stencila_workflows::run_workflow_with_options(&workflow, options).await
        }
        .await;

        let _ = completion_tx.send(WorkflowEvent::Completed(result));
    });

    let handle = WorkflowRunHandle {
        event_rx,
        join_handle,
    };

    (handle, total_stages)
}

/// Compute total stages (graph node count) for a workflow by name.
///
/// Best-effort: returns 0 if the workflow can't be loaded or parsed.
fn compute_total_stages(name: &str) -> usize {
    let Ok(handle) = tokio::runtime::Handle::try_current() else {
        return 0;
    };
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::task::block_in_place(|| {
            handle.block_on(async {
                let cwd = std::env::current_dir().unwrap_or_default();
                match stencila_workflows::get_by_name(&cwd, name).await {
                    Ok(wf) => wf.graph().map(|g| g.nodes.len()).unwrap_or(0),
                    Err(_) => 0,
                }
            })
        })
    }))
    .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tui_emitter_sends_events() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let emitter = TuiEventEmitter { tx };
        emitter.emit(PipelineEvent::PipelineStarted {
            pipeline_name: "test".to_string(),
        });
        let event = rx.recv().await.expect("should receive pipeline event");
        assert!(matches!(
            event,
            WorkflowEvent::Pipeline(PipelineEvent::PipelineStarted { .. })
        ));
    }
}
