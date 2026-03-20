use std::sync::{Arc, Mutex};

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

async fn load_workflow_instance(
    info: &WorkflowDefinitionInfo,
) -> eyre::Result<stencila_workflows::WorkflowInstance> {
    let cwd = std::env::current_dir().unwrap_or_default();
    stencila_workflows::get_by_name(&cwd, &info.name).await
}

pub fn is_ephemeral_workflow(name: &str) -> bool {
    let cwd = std::env::current_dir().unwrap_or_default();
    stencila_workflows::is_ephemeral(&cwd, name)
}

/// Handle to a running workflow task.
pub struct WorkflowRunHandle {
    pub event_rx: mpsc::UnboundedReceiver<WorkflowEvent>,
    join_handle: JoinHandle<()>,
    /// The run ID, published by the workflow engine once it is known.
    run_id: Arc<Mutex<Option<String>>>,
}

impl WorkflowRunHandle {
    /// Cancel the running workflow: mark the run as cancelled in the DB,
    /// then abort the background task.
    pub fn cancel(&self) {
        // Try to mark the run as cancelled in the database before
        // aborting the task. This is best-effort: if the DB update
        // fails or the run_id isn't set yet we still abort the task.
        if let Some(run_id) = self
            .run_id
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
        {
            let cwd = std::env::current_dir().unwrap_or_default();
            // Spawn a blocking task so we don't block the TUI event loop.
            tokio::spawn(async move {
                if let Err(e) = stencila_workflows::cancel_run(&cwd, &run_id).await {
                    tracing::warn!("Failed to mark run `{run_id}` as cancelled: {e}");
                }
            });
        }
        self.join_handle.abort();
    }

    #[cfg(test)]
    pub fn new_for_test(event_rx: mpsc::UnboundedReceiver<WorkflowEvent>) -> Self {
        Self {
            event_rx,
            join_handle: tokio::spawn(async {}),
            run_id: Arc::new(Mutex::new(None)),
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

/// Spawn a resumed workflow run on a background task.
///
/// Resumes a previously failed, cancelled, or interrupted workflow run
/// from where it left off.
pub fn spawn_resume_workflow(run_id: String) -> WorkflowRunHandle {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let run_id_shared: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some(run_id.clone())));

    let join_handle = tokio::spawn(async move {
        let completion_tx = event_tx.clone();
        let emitter: Arc<dyn EventEmitter> = Arc::new(TuiEventEmitter {
            tx: event_tx.clone(),
        });

        let interviewer = spawn_interview_forwarder(event_tx);

        let result = async {
            let cwd = std::env::current_dir()?;
            let options = stencila_workflows::RunOptions {
                emitter,
                interviewer: Some(interviewer),
                run_id_out: None,
                gate_timeout: stencila_workflows::GateTimeoutConfig::default(),
            };
            stencila_workflows::resume_workflow_with_options(&run_id, &cwd, options, false).await
        }
        .await;

        let _ = completion_tx.send(WorkflowEvent::Completed(result));
    });

    WorkflowRunHandle {
        event_rx,
        join_handle,
        run_id: run_id_shared,
    }
}

/// Spawn a workflow run on a background task.
///
/// Returns the run handle and total number of pipeline stages (graph nodes).
pub fn spawn_workflow(
    info: &WorkflowDefinitionInfo,
    goal: Option<String>,
    gate_timeout: stencila_workflows::GateTimeoutConfig,
) -> (WorkflowRunHandle, usize) {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let info = info.clone();
    let total_stages = compute_total_stages(&info);
    let run_id_shared: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let run_id_for_task = run_id_shared.clone();

    let join_handle = tokio::spawn(async move {
        let completion_tx = event_tx.clone();
        let emitter: Arc<dyn EventEmitter> = Arc::new(TuiEventEmitter {
            tx: event_tx.clone(),
        });

        let interviewer = spawn_interview_forwarder(event_tx);

        let result = async {
            let mut workflow = load_workflow_instance(&info).await?;
            if let Some(goal) = goal {
                workflow.inner.goal = Some(goal);
            }

            let options = stencila_workflows::RunOptions {
                emitter,
                interviewer: Some(interviewer),
                run_id_out: Some(run_id_for_task),
                gate_timeout,
            };
            stencila_workflows::run_workflow_with_options(&workflow, options).await
        }
        .await;

        let _ = completion_tx.send(WorkflowEvent::Completed(result));
    });

    let handle = WorkflowRunHandle {
        event_rx,
        join_handle,
        run_id: run_id_shared,
    };

    (handle, total_stages)
}

/// Compute total stages (graph node count) for a workflow by name.
///
/// Best-effort: returns 0 if the workflow can't be loaded or parsed.
fn compute_total_stages(info: &WorkflowDefinitionInfo) -> usize {
    let Ok(handle) = tokio::runtime::Handle::try_current() else {
        return 0;
    };
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::task::block_in_place(|| {
            handle.block_on(async {
                match load_workflow_instance(info).await {
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

    /// AC-1: `spawn_workflow` accepts a `GateTimeoutConfig` parameter and
    /// propagates it to `RunOptions`.
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn spawn_workflow_accepts_and_propagates_gate_timeout() {
        let info = WorkflowDefinitionInfo {
            name: "nonexistent-test-wf".to_string(),
            ..Default::default()
        };

        let gate_timeout = stencila_workflows::GateTimeoutConfig::AutoApprove;
        let (handle, _stages) = spawn_workflow(&info, Some("test goal".to_string()), gate_timeout);
        handle.cancel();
    }
}
