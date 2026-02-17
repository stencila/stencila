use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use stencila_attractor::events::{EventEmitter, PipelineEvent};
use stencila_attractor::interviewer::{
    Answer, AnswerValue, Interviewer, Question, QuestionOption, QuestionType,
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
}

/// An `EventEmitter` that forwards events through a channel.
struct ChannelEventEmitter {
    tx: mpsc::UnboundedSender<WorkflowEvent>,
}

impl EventEmitter for ChannelEventEmitter {
    fn emit(&self, event: PipelineEvent) {
        let _ = self.tx.send(WorkflowEvent::Pipeline(event));
    }
}

/// An `Interviewer` that sends questions through the event channel
/// and waits for answers via oneshot channels.
struct ChannelInterviewer {
    event_tx: mpsc::UnboundedSender<WorkflowEvent>,
}

#[async_trait]
impl Interviewer for ChannelInterviewer {
    async fn ask(&self, question: &Question) -> Answer {
        let (answer_tx, answer_rx) = oneshot::channel();
        let _ = self.event_tx.send(WorkflowEvent::InterviewQuestion {
            question: question.clone(),
            answer_tx,
        });
        match answer_rx.await {
            Ok(text) => parse_answer_text(&text, question),
            Err(_) => Answer::new(AnswerValue::Skipped),
        }
    }
}

/// Parse a raw text answer into a typed `Answer` based on the question type.
fn parse_answer_text(text: &str, question: &Question) -> Answer {
    let trimmed = text.trim();
    match question.question_type {
        QuestionType::YesNo | QuestionType::Confirmation => {
            let lower = trimmed.to_ascii_lowercase();
            if matches!(lower.as_str(), "y" | "yes" | "true" | "1") {
                Answer::new(AnswerValue::Yes)
            } else if matches!(lower.as_str(), "n" | "no" | "false" | "0") {
                Answer::new(AnswerValue::No)
            } else {
                Answer::new(AnswerValue::Text(trimmed.to_string()))
            }
        }
        QuestionType::MultipleChoice => {
            if let Some(opt) = question
                .options
                .iter()
                .find(|o| o.key.eq_ignore_ascii_case(trimmed))
            {
                Answer::with_option(
                    AnswerValue::Selected(opt.key.clone()),
                    QuestionOption {
                        key: opt.key.clone(),
                        label: opt.label.clone(),
                    },
                )
            } else if let Some(opt) = question
                .options
                .iter()
                .find(|o| o.label.eq_ignore_ascii_case(trimmed))
            {
                Answer::with_option(
                    AnswerValue::Selected(opt.key.clone()),
                    QuestionOption {
                        key: opt.key.clone(),
                        label: opt.label.clone(),
                    },
                )
            } else {
                Answer::new(AnswerValue::Text(trimmed.to_string()))
            }
        }
        QuestionType::Freeform => Answer::new(AnswerValue::Text(trimmed.to_string())),
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
        let emitter: Arc<dyn EventEmitter> = Arc::new(ChannelEventEmitter {
            tx: event_tx.clone(),
        });
        let interviewer: Arc<dyn Interviewer> = Arc::new(ChannelInterviewer { event_tx });

        let cwd = std::env::current_dir().unwrap_or_default();
        let result = async {
            let mut workflow = stencila_workflows::get_by_name(&cwd, &name).await?;
            workflow.inner.goal = Some(goal);

            let logs_dir = cwd.join(".stencila").join("logs");
            let options = stencila_workflows::RunOptions {
                emitter,
                interviewer: Some(interviewer),
            };
            stencila_workflows::run_workflow_with_options(&workflow, &logs_dir, options).await
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
    use stencila_attractor::interviewer::QuestionOption;

    fn freeform_question() -> Question {
        Question::freeform("What is your name?", "test-stage")
    }

    fn yes_no_question() -> Question {
        Question::yes_no("Do you agree?", "test-stage")
    }

    fn multiple_choice_question() -> Question {
        Question::multiple_choice(
            "Pick one:",
            vec![
                QuestionOption {
                    key: "A".to_string(),
                    label: "Option Alpha".to_string(),
                },
                QuestionOption {
                    key: "B".to_string(),
                    label: "Option Beta".to_string(),
                },
            ],
            "test-stage",
        )
    }

    #[test]
    fn parse_freeform_answer() {
        let q = freeform_question();
        let answer = parse_answer_text("hello world", &q);
        assert_eq!(answer.value, AnswerValue::Text("hello world".to_string()));
    }

    #[test]
    fn parse_yes_no_yes() {
        let q = yes_no_question();
        assert_eq!(parse_answer_text("y", &q).value, AnswerValue::Yes);
        assert_eq!(parse_answer_text("YES", &q).value, AnswerValue::Yes);
        assert_eq!(parse_answer_text("true", &q).value, AnswerValue::Yes);
    }

    #[test]
    fn parse_yes_no_no() {
        let q = yes_no_question();
        assert_eq!(parse_answer_text("n", &q).value, AnswerValue::No);
        assert_eq!(parse_answer_text("NO", &q).value, AnswerValue::No);
        assert_eq!(parse_answer_text("false", &q).value, AnswerValue::No);
    }

    #[test]
    fn parse_multiple_choice_by_key() {
        let q = multiple_choice_question();
        let answer = parse_answer_text("A", &q);
        assert_eq!(answer.value, AnswerValue::Selected("A".to_string()));
        assert!(answer.selected_option.is_some());
    }

    #[test]
    fn parse_multiple_choice_by_label() {
        let q = multiple_choice_question();
        let answer = parse_answer_text("option beta", &q);
        assert_eq!(answer.value, AnswerValue::Selected("B".to_string()));
    }

    #[test]
    fn parse_multiple_choice_no_match() {
        let q = multiple_choice_question();
        let answer = parse_answer_text("unknown", &q);
        assert_eq!(answer.value, AnswerValue::Text("unknown".to_string()));
    }

    #[tokio::test]
    async fn channel_emitter_sends_events() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let emitter = ChannelEventEmitter { tx };
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
    async fn channel_interviewer_sends_and_receives() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let interviewer = ChannelInterviewer { event_tx: tx };
        let question = freeform_question();

        let ask_handle = tokio::spawn({
            let q = question.clone();
            async move { interviewer.ask(&q).await }
        });

        let event = rx.recv().await.expect("should receive interview event");
        if let WorkflowEvent::InterviewQuestion {
            question: q,
            answer_tx,
        } = event
        {
            assert_eq!(q.text, "What is your name?");
            answer_tx
                .send("Alice".to_string())
                .expect("should send answer");
        } else {
            panic!("Expected InterviewQuestion event");
        }

        let answer = ask_handle.await.expect("task should complete");
        assert_eq!(answer.value, AnswerValue::Text("Alice".to_string()));
    }
}
