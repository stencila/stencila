use futures::FutureExt;

use stencila_server::preview::PreviewEvent;

use crate::{
    agent::ResponseSegment,
    config::WorkflowVerbosity,
    interview::{InterviewSource, InterviewState, InterviewStatus, PendingTuiInterview},
    workflow::WorkflowEvent,
};

use super::{
    ActiveWorkflowState, App, AppMessage, ExchangeStatus, WorkflowProgressKind,
    WorkflowStatusState, discover_agents, discover_workflows,
};

impl App {
    fn prompt_to_save_ephemeral_workflow(&mut self, workflow_name: &str) {
        self.messages.push(AppMessage::System {
            content: format!(
                "Workflow `{workflow_name}` is ephemeral. Use `/workflows save {workflow_name}` to keep it or `/workflows discard {workflow_name}` to remove it."
            ),
        });
    }

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
        let mut pending_delegations = Vec::new();

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

                // Check for delegation before processing as a normal completion
                if let Some(delegation) = result.delegation.clone() {
                    // Mark the exchange as succeeded, then queue the delegation
                    Self::update_exchange_complete(
                        &mut self.messages,
                        msg_index,
                        ExchangeStatus::Succeeded,
                        Some(result.text),
                        Some(result.segments),
                    );
                    pending_delegations.push(delegation);
                    continue;
                }

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
                } else if result.text.is_empty() && result.segments.is_empty() {
                    (None, None)
                } else {
                    // Preserve segments even when plain text is empty —
                    // non-text segments (tool calls, thinking) should remain
                    // visible after completion rather than being wiped.
                    let text = if result.text.is_empty() {
                        None
                    } else {
                        Some(result.text)
                    };
                    (text, Some(result.segments))
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

        // Process queued delegations after all sessions have been polled
        for delegation in pending_delegations {
            self.handle_delegation(delegation);
        }
    }

    /// Poll agent sessions for pending interview questions (from `ask_user` tool).
    ///
    /// First drains any previously buffered interviews from `pending_interviews`,
    /// then skips polling while an interview is already active. New interviews
    /// from agent channels are picked up on the next tick once the active one completes.
    pub fn poll_interviews(&mut self) {
        // First, try to start a previously buffered interview.
        // Buffered interviews are always workflow-sourced (agent interviews
        // are discovered per-session below and never buffered).
        if self.active_interview.is_none()
            && let Some((pending, source, agent_name)) = self.pending_interviews.pop_front()
        {
            self.start_interview(pending, &source, &agent_name, None);
        }

        if self.active_interview.is_some() {
            return;
        }

        // Check agent sessions for pending interviews
        for idx in 0..self.sessions.len() {
            let pending = {
                let session = &mut self.sessions[idx];
                let Some(agent) = &mut session.agent else {
                    continue;
                };
                match agent.interview_rx.try_recv() {
                    Ok(pending) => pending,
                    Err(_) => continue,
                }
            };
            let agent_name = self.sessions[idx].name.clone();
            self.start_interview(pending, &InterviewSource::Agent, &agent_name, Some(idx));
            return;
        }
    }

    /// Start an interview: create the `AppMessage::Interview`, set up
    /// `active_interview`, and pin scroll.
    ///
    /// `session_index` identifies the originating agent session (for
    /// `InterviewSource::Agent`) so the correct running exchange is used as
    /// the inline parent. `None` for workflow-sourced interviews.
    fn start_interview(
        &mut self,
        pending: PendingTuiInterview,
        source: &InterviewSource,
        agent_name: &str,
        session_index: Option<usize>,
    ) {
        // Find the parent message to embed the interview inline with.
        let parent_msg_index = self.find_interview_parent(source, session_index);

        let id = pending.interview.id.clone();
        self.messages.push(AppMessage::Interview {
            id: id.clone(),
            source: source.clone(),
            agent_name: agent_name.to_string(),
            status: InterviewStatus::Active,
            interview: pending.interview.clone(),
            answers: Vec::new(),
            parent_msg_index,
        });
        let msg_index = self.messages.len() - 1;

        if matches!(source, InterviewSource::Agent)
            && let Some(session_index) = session_index
            && let Some((_, running)) = self.sessions[session_index].running_agent_exchanges.last()
        {
            running.push_interview_segment(msg_index);
        }

        self.active_interview = Some(InterviewState::new(
            &pending.interview,
            msg_index,
            pending.result_tx,
        ));
        self.interview_cancel_confirm = false;
        self.scroll_pinned = true;

        // Populate the input area from the draft (which may be pre-populated
        // from a question default) so the user sees the default value.
        self.restore_interview_input_from_draft();

        // If the source is a workflow and user detached to agent mode,
        // switch back so they can answer the interview.
        if matches!(source, InterviewSource::Workflow) {
            self.mode = super::AppMode::Workflow;
        }
    }

    /// Find the parent message index for an interview so it can be rendered
    /// inline as a continuation of that message rather than as a standalone block.
    ///
    /// `session_index` is the originating agent session (only for agent-sourced
    /// interviews).
    fn find_interview_parent(
        &self,
        source: &InterviewSource,
        session_index: Option<usize>,
    ) -> Option<usize> {
        match source {
            InterviewSource::Workflow => {
                // Use the current stage's exchange (detailed mode) or stage
                // status message (simple mode), falling back to the workflow
                // status message (minimal mode).
                self.active_workflow.as_ref().and_then(|wf| {
                    wf.current_exchange_msg_index
                        .or(wf.stage_status_msg_index)
                        .or(wf.workflow_status_msg_index)
                })
            }
            InterviewSource::Agent => {
                // Find the most recent running exchange for the originating session.
                let session = &self.sessions[session_index.unwrap_or(self.active_session)];
                session.running_agent_exchanges.last().map(|(idx, _)| *idx)
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
                WorkflowEvent::Interview(pending) => {
                    let workflow_name = self
                        .active_workflow
                        .as_ref()
                        .map(|w| w.info.name.clone())
                        .unwrap_or_default();
                    if self.active_interview.is_none() {
                        self.start_interview(
                            pending,
                            &InterviewSource::Workflow,
                            &workflow_name,
                            None,
                        );
                    } else {
                        // Buffer the interview so it is started once the
                        // current one completes (via poll_interviews).
                        self.pending_interviews.push_back((
                            pending,
                            InterviewSource::Workflow,
                            workflow_name,
                        ));
                    }
                }
                WorkflowEvent::Completed(result) => {
                    let mut prompt_ephemeral = None;
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
                                        depth: 0,
                                    });
                                }
                            }
                        }
                        workflow.run_handle = None;

                        if workflow.is_ephemeral && !workflow.save_prompt_pending {
                            workflow.save_prompt_pending = true;
                            prompt_ephemeral = Some(workflow.info.name.clone());
                        }
                    }

                    if let Some(name) = prompt_ephemeral {
                        self.prompt_to_save_ephemeral_workflow(&name);
                    }

                    // Drop back to agent chat now that the workflow has finished
                    self.mode = super::AppMode::Agent;
                    self.input.clear();
                    self.input_scroll = 0;
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

    /// Handle a delegation request from a completed manager agent exchange.
    ///
    /// Depending on the delegation kind:
    /// - **agent**: creates or switches to the target agent session and auto-submits
    ///   the delegated task as the first message.
    /// - **workflow**: activates workflow mode and submits the delegated goal.
    fn handle_delegation(&mut self, delegation: crate::agent::DelegationRequest) {
        match delegation.kind {
            crate::agent::DelegationKind::Agent => {
                self.messages.push(AppMessage::System {
                    content: format!("Delegating to agent `{}`", delegation.name),
                });

                // Find or create the target agent session
                let name_lower = delegation.name.to_ascii_lowercase();
                let target_idx = self
                    .sessions
                    .iter()
                    .position(|s| s.name.to_ascii_lowercase() == name_lower);

                if let Some(idx) = target_idx {
                    self.switch_to_session(idx);
                } else {
                    // Create a new session from the discovered agent definition
                    let agents = discover_agents();
                    if let Some(def) = agents
                        .iter()
                        .find(|d| d.name.to_ascii_lowercase() == name_lower)
                    {
                        let info = crate::autocomplete::agents::AgentDefinitionInfo {
                            name: def.name.clone(),
                            description: def.description.clone(),
                            model: def.model.clone(),
                            provider: def.provider.clone(),
                            source: def.source().map(|s| s.to_string()).unwrap_or_default(),
                        };
                        self.create_session_from_definition(&info);
                    } else {
                        self.messages.push(AppMessage::System {
                            content: format!(
                                "Agent `{}` not found. Delegation aborted.",
                                delegation.name
                            ),
                        });
                        self.scroll_pinned = true;
                        self.scroll_offset = 0;
                        return;
                    }
                }

                // Auto-submit the delegated instruction
                if !delegation.instruction.is_empty() {
                    self.submit_agent_message(delegation.instruction);
                }
            }
            crate::agent::DelegationKind::Workflow => {
                self.messages.push(AppMessage::System {
                    content: format!("Delegating to workflow `{}`", delegation.name),
                });

                // Find the workflow definition, including newly created ephemeral workflows
                // that have been written to disk.
                let workflows = discover_workflows();
                if let Some(wf) = workflows.iter().find(|w| w.name == delegation.name) {
                    let info = crate::autocomplete::workflows::WorkflowDefinitionInfo {
                        name: wf.name.clone(),
                        description: wf.description.clone(),
                        goal: wf.goal.clone(),
                        goal_hint: wf.goal_hint.clone(),
                    };
                    self.activate_workflow(info);

                    // Auto-submit the delegated instruction (or the workflow's own goal)
                    let goal = if delegation.instruction.is_empty() {
                        wf.goal.clone().unwrap_or_default()
                    } else {
                        delegation.instruction
                    };
                    if !goal.is_empty() {
                        self.submit_workflow_goal(goal);
                    }
                } else {
                    self.messages.push(AppMessage::System {
                        content: format!(
                            "Workflow `{}` not found. Falling back to agent mode.",
                            delegation.name
                        ),
                    });
                }
            }
        }

        self.scroll_pinned = true;
        self.scroll_offset = 0;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::super::{App, AppMode};

    #[tokio::test]
    async fn interview_event_resumes_workflow_mode() {
        use crate::autocomplete::workflows::WorkflowDefinitionInfo;
        use crate::interview::PendingTuiInterview;
        use crate::workflow::WorkflowEvent;
        use stencila_attractor::interviewer::{Interview, Question};
        use tokio::sync::{mpsc, oneshot};

        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            goal: Some("goal".to_string()),
            ..Default::default()
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

        // Send an interview through the channel
        let (result_tx, _result_rx) = oneshot::channel();
        let question = Question::freeform("Continue?");
        tx.send(WorkflowEvent::Interview(PendingTuiInterview {
            interview: Interview::single(question, "test"),
            result_tx,
        }))
        .unwrap();

        // Poll — should process the interview event and switch back to workflow mode
        app.poll_workflow_events();

        assert_eq!(app.mode, AppMode::Workflow);
        assert!(app.active_interview.is_some());
    }

    #[tokio::test]
    async fn workflow_completed_returns_to_agent_mode() {
        use crate::app::AppMessage;
        use crate::autocomplete::workflows::WorkflowDefinitionInfo;
        use crate::workflow::WorkflowEvent;
        use std::fs;
        use stencila_attractor::types::Outcome;
        use tokio::sync::mpsc;

        let dir = tempfile::tempdir().expect("tempdir");
        let previous = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(dir.path()).expect("set cwd");
        fs::create_dir_all(dir.path().join(".stencila/workflows/test-wf")).expect("create wf dir");
        fs::write(
            dir.path().join(".stencila/workflows/test-wf/.gitignore"),
            "*\n",
        )
        .expect("write sentinel");

        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            goal: Some("goal".to_string()),
            ..Default::default()
        });
        assert_eq!(app.mode, AppMode::Workflow);

        // Set up a channel and inject a run_handle
        let (tx, rx) = mpsc::unbounded_channel();
        if let Some(wf) = &mut app.active_workflow {
            wf.run_handle = Some(crate::workflow::WorkflowRunHandle::new_for_test(rx));
        }

        // Send a completion event
        tx.send(WorkflowEvent::Completed(Ok(Outcome::success())))
            .unwrap();

        app.poll_workflow_events();

        assert_eq!(app.mode, AppMode::Agent);
        assert!(app.input.is_empty());
        assert!(app.messages.iter().any(|message| matches!(
            message,
            AppMessage::System { content } if content.contains("/workflows save test-wf")
        )));

        std::env::set_current_dir(previous).expect("restore cwd");
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
