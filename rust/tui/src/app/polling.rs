use futures::FutureExt;

use stencila_server::preview::PreviewEvent;

use crate::{
    agent::ResponseSegment,
    config::WorkflowVerbosity,
    workflow::{PendingInterview, WorkflowEvent},
};

use super::{
    ActiveWorkflowState, App, AppMessage, ExchangeStatus, WorkflowProgressKind, WorkflowStatusState,
};

impl App {
    /// Poll all running commands for completion. Called on tick events.
    pub fn poll_running_commands(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);

        // Collect completed indices (iterate in reverse so removal doesn't shift later indices)
        let mut completed = Vec::new();
        for (i, (_msg_index, running)) in self.running_shell_commands.iter_mut().enumerate() {
            if let Some(result) = running.try_take_result() {
                completed.push((i, result));
            }
        }

        // Process completions in reverse order to safely remove by index
        for (i, result) in completed.into_iter().rev() {
            let (msg_index, _running) = self.running_shell_commands.remove(i);
            let status = if result.exit_code == 0 {
                ExchangeStatus::Succeeded
            } else {
                ExchangeStatus::Failed
            };
            let was_upgrade =
                self.upgrade_msg_index == Some(msg_index) && status == ExchangeStatus::Succeeded;
            if self.upgrade_msg_index == Some(msg_index) {
                self.upgrade_msg_index = None;
            }

            Self::update_exchange_at(
                &mut self.messages,
                msg_index,
                status,
                Some(result.output),
                Some(result.exit_code),
            );

            if was_upgrade {
                self.upgrade_available = None;
                self.messages.push(AppMessage::System {
                    content: "Restart to use the new version.".to_string(),
                });
            }
        }
    }

    /// Poll all running agent exchanges for progress. Called on tick events.
    ///
    /// Iterates ALL sessions (not just active) since background agents may
    /// still be streaming.
    pub fn poll_running_agent_exchanges(&mut self) {
        for session in &mut self.sessions {
            let mut completed = Vec::new();
            for (i, (msg_index, exchange)) in session.running_agent_exchanges.iter().enumerate() {
                // Update context usage from the latest event data
                let pct = exchange.context_usage_percent();
                if pct > 0 {
                    session.context_usage_percent = pct;
                }

                if let Some(result) = exchange.try_take_result() {
                    completed.push((i, *msg_index, result));
                } else {
                    // Streaming update: refresh both plain text and segments
                    let segments = exchange.current_segments();
                    if !segments.is_empty() {
                        let text = crate::agent::plain_text_from_segments(&segments);
                        Self::update_exchange_streaming(
                            &mut self.messages,
                            *msg_index,
                            text,
                            segments,
                        );
                    }
                }
            }

            // Process completions in reverse order to safely remove by index
            for (i, msg_index, result) in completed.into_iter().rev() {
                session.running_agent_exchanges.remove(i);
                let status = if result.error.is_some() {
                    ExchangeStatus::Failed
                } else {
                    ExchangeStatus::Succeeded
                };
                let (response, segments) = if let Some(err) = result.error {
                    // Preserve any streamed segments (tool calls, thinking,
                    // partial text) and append the error as a warning so the
                    // user sees what happened *without* losing context.
                    let mut segs = result.segments;
                    segs.push(ResponseSegment::Error(err.clone()));
                    let text = crate::agent::plain_text_from_segments(&segs);
                    let resp = if text.is_empty() {
                        err
                    } else {
                        format!("{text}\n\nError: {err}")
                    };
                    (Some(resp), Some(segs))
                } else if result.text.is_empty() {
                    (None, None)
                } else {
                    (Some(result.text), Some(result.segments))
                };
                Self::update_exchange_complete(
                    &mut self.messages,
                    msg_index,
                    status,
                    response,
                    segments,
                );
            }
        }
    }

    /// Poll the running workflow for events. Called on tick events.
    pub fn poll_workflow_events(&mut self) {
        // Drain events into a local vec to avoid borrow conflicts
        // (holding &mut self.active_workflow while also pushing to self.messages).
        let events: Vec<WorkflowEvent> = {
            let Some(workflow) = &mut self.active_workflow else {
                return;
            };
            let Some(handle) = &mut workflow.run_handle else {
                return;
            };
            let mut batch = Vec::new();
            while let Ok(event) = handle.event_rx.try_recv() {
                batch.push(event);
            }
            batch
        };

        if events.is_empty() {
            return;
        }

        for event in events {
            match event {
                WorkflowEvent::Pipeline(pe) => self.handle_pipeline_event(&pe),
                WorkflowEvent::InterviewQuestion {
                    question,
                    answer_tx,
                } => {
                    if let Some(workflow) = &mut self.active_workflow {
                        workflow.pending_interview = Some(PendingInterview {
                            question,
                            answer_tx,
                        });
                        // If the user detached to agent mode, auto-switch back
                        // so they can answer the interview prompt.
                        self.mode = super::AppMode::Workflow;
                    }
                }
                WorkflowEvent::Completed(result) => {
                    if let Some(workflow) = &mut self.active_workflow {
                        workflow.state = match &result {
                            Ok(outcome) if outcome.status.is_success() => {
                                ActiveWorkflowState::Succeeded
                            }
                            _ => ActiveWorkflowState::Failed,
                        };
                        if let Err(err) = &result {
                            let error_text = err.to_string();
                            match self.config.workflow_verbosity {
                                WorkflowVerbosity::Minimal => {
                                    if let Some(idx) = workflow.workflow_status_msg_index
                                        && let Some(AppMessage::WorkflowStatus {
                                            state,
                                            detail,
                                            ..
                                        }) = self.messages.get_mut(idx)
                                    {
                                        *state = WorkflowStatusState::Failed;
                                        *detail = Some(error_text);
                                    } else {
                                        self.messages.push(AppMessage::WorkflowStatus {
                                            state: WorkflowStatusState::Failed,
                                            label: format!("Workflow {}", workflow.info.name),
                                            detail: Some(error_text),
                                        });
                                    }
                                }
                                _ => {
                                    self.messages.push(AppMessage::WorkflowProgress {
                                        kind: WorkflowProgressKind::Failed,
                                        label: format!("Workflow {} failed", workflow.info.name),
                                        detail: Some(error_text),
                                    });
                                }
                            }
                        }
                        workflow.run_handle = None;
                    }
                }
            }
            self.scroll_pinned = true;
        }
    }

    /// Drain pending log messages from the tracing channel and display them
    /// as system messages. Called on tick events.
    pub fn poll_log_events(&mut self) {
        while let Ok(msg) = self.log_receiver.try_recv() {
            self.messages.push(AppMessage::System { content: msg });
        }
    }

    /// Poll the background upgrade check. Called on tick events.
    ///
    /// If the check has completed with a newer version, stores it in
    /// `upgrade_available` for display in the welcome banner.
    pub fn poll_upgrade_check(&mut self) {
        let Some(mut handle) = self.upgrade_handle.take() else {
            return;
        };

        match (&mut handle).now_or_never() {
            Some(Ok(Some(version))) => {
                self.upgrade_available = Some(version);
            }
            Some(Ok(None) | Err(_)) => {
                // Check completed with no upgrade or the task panicked — discard handle
            }
            None => {
                // Not ready yet — put the handle back for next tick
                self.upgrade_handle = Some(handle);
            }
        }
    }

    /// Poll the background site preview for events. Called on tick events.
    pub fn poll_site_preview(&mut self) {
        let Some(handle) = &mut self.site_preview else {
            return;
        };
        while let Ok(event) = handle.event_rx.try_recv() {
            match event {
                PreviewEvent::Rerendering { files } => {
                    let files_count = files.len();
                    let files_iter = files.into_iter();
                    let mut files_list = files_iter.take(3).collect::<Vec<_>>().join(", ");
                    let remainder = files_count.saturating_sub(3);
                    if remainder > 0 {
                        files_list.push_str(", and ");
                        files_list.push_str(&remainder.to_string());
                        files_list.push_str(" more.");
                    }
                    self.messages.push(AppMessage::System {
                        content: format!("Re-rendering {files_list}"),
                    });
                }
                PreviewEvent::ServerReady { url, token } => {
                    let url = format!("{url}/~login?sst={token}");
                    self.messages.push(AppMessage::SitePreviewReady { url });
                }
                PreviewEvent::Error(error) => {
                    self.messages.push(AppMessage::System {
                        content: format!("Site preview: {error}"),
                    });
                }
                PreviewEvent::RerenderComplete => {}
            }
        }
    }

    /// Update response text and segments during streaming without changing status.
    pub(super) fn update_exchange_streaming(
        messages: &mut [AppMessage],
        msg_index: usize,
        text: String,
        segments: Vec<ResponseSegment>,
    ) {
        if let Some(AppMessage::Exchange {
            response,
            response_segments,
            ..
        }) = messages.get_mut(msg_index)
        {
            *response = Some(text);
            *response_segments = Some(segments);
        }
    }

    /// Update an exchange on completion, setting status, response, segments, and exit code.
    fn update_exchange_complete(
        messages: &mut [AppMessage],
        msg_index: usize,
        new_status: ExchangeStatus,
        response: Option<String>,
        segments: Option<Vec<ResponseSegment>>,
    ) {
        if let Some(AppMessage::Exchange {
            status,
            response: resp,
            response_segments,
            exit_code,
            ..
        }) = messages.get_mut(msg_index)
        {
            *status = new_status;
            *resp = response;
            *response_segments = segments;
            *exit_code = None;
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::super::{App, AppMode};

    #[tokio::test]
    async fn interview_event_resumes_workflow_mode() {
        use crate::autocomplete::workflows::WorkflowDefinitionInfo;
        use crate::workflow::WorkflowEvent;
        use stencila_attractor::interviewer::{Question, QuestionType};
        use tokio::sync::{mpsc, oneshot};

        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            description: String::new(),
            goal: Some("goal".to_string()),
        });
        assert_eq!(app.mode, AppMode::Workflow);

        // Detach to agent mode
        app.exit_workflow_mode();
        assert_eq!(app.mode, AppMode::Agent);
        assert!(app.active_workflow.is_some());

        // Set up a channel and inject a run_handle so poll_workflow_events can drain it
        let (tx, rx) = mpsc::unbounded_channel();
        if let Some(wf) = &mut app.active_workflow {
            wf.run_handle = Some(crate::workflow::WorkflowRunHandle::new_for_test(rx));
        }

        // Send an interview question through the channel
        let (answer_tx, _answer_rx) = oneshot::channel();
        tx.send(WorkflowEvent::InterviewQuestion {
            question: Question {
                text: "Continue?".to_string(),
                question_type: QuestionType::Freeform,
                options: Vec::new(),
                default: None,
                timeout_seconds: None,
                stage: "test".to_string(),
            },
            answer_tx,
        })
        .unwrap();

        // Poll — should process the interview event and switch back to workflow mode
        app.poll_workflow_events();

        assert_eq!(app.mode, AppMode::Workflow);
        assert!(
            app.active_workflow
                .as_ref()
                .unwrap()
                .pending_interview
                .is_some()
        );
    }

    #[tokio::test]
    async fn scroll_stays_stable_during_streaming() {
        let mut app = App::new_for_test().await;
        app.total_message_lines = 30;
        app.visible_message_height = 10;

        // Scroll up to unpin
        app.scroll_up(5);
        assert!(!app.scroll_pinned);
        assert_eq!(app.scroll_offset, 15); // max_top(20) - 5

        // Simulate streaming: total lines grow but offset stays the same
        app.total_message_lines = 50;
        assert_eq!(app.scroll_offset, 15); // unchanged — view stays put

        // Esc snaps back to bottom
        app.scroll_to_bottom();
        assert!(app.scroll_pinned);
        assert_eq!(app.scroll_offset, 0);
    }
}
