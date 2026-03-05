use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use stencila_attractor::events::{EventEmitter, PipelineEvent};
use stencila_attractor::interviewer::{
    Answer, InterviewError, Interviewer, Question, parse_answer_text,
};
use stencila_attractor::types::Outcome;

use crate::autocomplete::workflows::WorkflowDefinitionInfo;

/// Events from the workflow runtime to the TUI.
pub enum WorkflowEvent {
    /// A pipeline event forwarded directly from the attractor engine.
    Pipeline(PipelineEvent),
    /// An interview question requiring user input.
    InterviewQuestion {
        question: Question,
        answer_tx: oneshot::Sender<String>,
    },
    /// The workflow run completed (success or failure).
    Completed(eyre::Result<Outcome>),
}

/// An interview question pending user response.
pub struct PendingInterview {
    pub question: Question,
    pub answer_tx: oneshot::Sender<String>,
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

/// An `Interviewer` that sends questions through the event channel
/// and waits for answers via oneshot channels.
///
/// This is the TUI-specific interviewer implementation, coupled to
/// [`WorkflowEvent`] for rendering questions in the terminal UI.
struct TuiInterviewer {
    event_tx: mpsc::UnboundedSender<WorkflowEvent>,
}

#[async_trait]
impl Interviewer for TuiInterviewer {
    async fn ask(&self, question: &Question) -> Result<Answer, InterviewError> {
        let (answer_tx, answer_rx) = oneshot::channel();
        self.event_tx
            .send(WorkflowEvent::InterviewQuestion {
                question: question.clone(),
                answer_tx,
            })
            .map_err(|_| InterviewError::ChannelClosed)?;
        match answer_rx.await {
            Ok(text) => Ok(parse_answer_text(&text, question)),
            Err(_) => Err(InterviewError::ChannelClosed),
        }
    }
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
        let interviewer: Arc<dyn Interviewer> = Arc::new(TuiInterviewer { event_tx });

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
    use stencila_attractor::interviewer::AnswerValue;

    fn freeform_question() -> Question {
        Question::freeform("What is your name?", "test-stage")
    }

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

    #[tokio::test]
    async fn tui_interviewer_sends_and_receives() -> Result<(), Box<dyn std::error::Error>> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let interviewer = TuiInterviewer { event_tx: tx };
        let question = freeform_question();

        let ask_handle = tokio::spawn({
            let q = question.clone();
            async move { interviewer.ask(&q).await }
        });

        let event = rx.recv().await.ok_or("expected interview event")?;
        if let WorkflowEvent::InterviewQuestion {
            question: q,
            answer_tx,
        } = event
        {
            assert_eq!(q.text, "What is your name?");
            answer_tx
                .send("Alice".to_string())
                .map_err(|_| "failed to send answer")?;
        } else {
            panic!("Expected InterviewQuestion event");
        }

        let answer = ask_handle.await??;
        assert_eq!(answer.value, AnswerValue::Text("Alice".to_string()));
        Ok(())
    }
}
