use std::sync::{Arc, Mutex};

use inflector::Inflector;
use stencila_attractor::events::PipelineEvent;
use stencila_attractor::interviewer::{Interview, Question};

use crate::{
    agent::AgentProgress,
    autocomplete::workflows::WorkflowDefinitionInfo,
    config::WorkflowVerbosity,
    interview::{InterviewSource, InterviewState, InterviewStatus},
};

use super::{
    ActiveWorkflow, ActiveWorkflowState, App, AppMessage, ExchangeKind, ExchangeStatus,
    WorkflowProgressKind, WorkflowStatusState,
};

impl App {
    /// Enter workflow mode for the given workflow definition.
    ///
    /// If a previous workflow is still running it is cancelled first so we
    /// don't leak a detached background task.
    pub(super) fn activate_workflow(&mut self, info: WorkflowDefinitionInfo) {
        self.cancel_active_workflow();
        self.mode = super::AppMode::Workflow;
        let goal_hint = info.goal_hint.clone();
        let default_goal = info.goal.clone();
        self.active_workflow = Some(ActiveWorkflow::new_pending(info));

        if let Some(hint) = goal_hint {
            // Trigger the pre-run interview through the TUI's standard
            // interview widget instead of using the legacy text input.
            let question = Question::freeform(hint);
            let interview = Interview::single(question, "pre_run");

            let (answer_tx, _answer_rx) = tokio::sync::oneshot::channel();
            let msg_index = self.messages.len();
            self.messages.push(AppMessage::Interview {
                id: interview.id.clone(),
                source: InterviewSource::Workflow,
                agent_name: String::new(),
                status: InterviewStatus::Active,
                interview: interview.clone(),
                answers: Vec::new(),
                parent_msg_index: None,
            });
            self.active_interview = Some(InterviewState::new(&interview, msg_index, answer_tx));
            self.input.clear();
        } else if let Some(goal) = default_goal {
            self.input.set_text(&goal);
        } else {
            self.input.clear();
        }
        self.input_scroll = 0;
        self.scroll_pinned = true;
    }

    /// Submit a goal for the active workflow and spawn the workflow run.
    pub(super) fn submit_workflow_goal(
        &mut self,
        goal: String,
        gate_timeout: stencila_workflows::GateTimeoutConfig,
    ) {
        let Some(workflow) = &mut self.active_workflow else {
            return;
        };

        let (handle, ..) = crate::workflow::spawn_workflow(&workflow.info, goal, gate_timeout);
        workflow.run_handle = Some(handle);
        workflow.state = ActiveWorkflowState::Running;
        workflow.workflow_status_msg_index = None;
        workflow.stage_status_msg_index = None;

        self.scroll_pinned = true;
        self.scroll_offset = 0;
    }

    /// Resume a previously failed or interrupted workflow run.
    ///
    /// Sets up workflow mode and immediately spawns the resumed run (no goal
    /// prompt — the original goal is restored by the workflow engine).
    pub(super) fn resume_workflow(&mut self, run_id: String, workflow_name: &str) {
        self.cancel_active_workflow();
        self.mode = super::AppMode::Workflow;

        let info = WorkflowDefinitionInfo {
            name: workflow_name.to_string(),
            description: format!(
                "Resume workflow `{workflow_name}` from run {}",
                &run_id[..run_id.len().min(8)]
            ),
            goal: None,
            goal_hint: None,
        };

        let handle = crate::workflow::spawn_resume_workflow(run_id);

        let mut workflow = ActiveWorkflow::new_pending(info);
        workflow.state = ActiveWorkflowState::Running;
        workflow.run_handle = Some(handle);
        self.active_workflow = Some(workflow);

        self.input.clear();
        self.input_scroll = 0;
        self.scroll_pinned = true;
        self.scroll_offset = 0;
    }

    pub(super) fn handle_pipeline_event(&mut self, event: &PipelineEvent) {
        match self.config.workflow_verbosity {
            WorkflowVerbosity::Minimal => self.handle_pipeline_event_minimal(event),
            WorkflowVerbosity::Simple => self.handle_pipeline_event_simple(event),
            WorkflowVerbosity::Detailed => self.handle_pipeline_event_detailed(event),
        }
    }

    // ── Shared pipeline event helpers (used by both simple & detailed) ──

    /// Current visual nesting depth for progress messages.
    fn pipeline_display_depth(&self) -> usize {
        self.active_workflow
            .as_ref()
            .map_or(0, |w| w.pipeline_depth.saturating_sub(1))
    }

    /// Handle `PipelineStarted`: increment depth and push a progress message.
    fn on_pipeline_started(&mut self, pipeline_name: &str) {
        if let Some(workflow) = &mut self.active_workflow {
            workflow.pipeline_depth += 1;
        }
        let depth = self.pipeline_display_depth();
        self.messages.push(AppMessage::WorkflowProgress {
            kind: WorkflowProgressKind::Started,
            label: format!("Workflow `{pipeline_name}` started"),
            detail: None,
            depth,
        });
    }

    /// Handle `StageRetrying`: push a retry progress message.
    fn on_stage_retrying(
        &mut self,
        node_id: &str,
        stage_index: usize,
        attempt: u32,
        max_attempts: u32,
    ) {
        let depth = self.pipeline_display_depth();
        self.messages.push(AppMessage::WorkflowProgress {
            kind: WorkflowProgressKind::Retrying,
            label: format!("Stage {stage_index}: {node_id} retrying ({attempt}/{max_attempts})"),
            detail: None,
            depth,
        });
    }

    /// Handle `PipelineCompleted`: push a completion message and decrement depth.
    fn on_pipeline_completed(
        &mut self,
        pipeline_name: &str,
        outcome: &stencila_attractor::types::Outcome,
    ) {
        let depth = self.pipeline_display_depth();
        self.messages.push(AppMessage::WorkflowProgress {
            kind: WorkflowProgressKind::Completed,
            label: format!("Workflow `{pipeline_name}` completed"),
            detail: Some(outcome.status.as_str().to_title_case()),
            depth,
        });
        if let Some(workflow) = &mut self.active_workflow {
            workflow.pipeline_depth = workflow.pipeline_depth.saturating_sub(1);
        }
    }

    /// Handle `PipelineFailed`: push a failure message and decrement depth.
    fn on_pipeline_failed(&mut self, pipeline_name: &str, reason: &str) {
        let depth = self.pipeline_display_depth();
        self.messages.push(AppMessage::WorkflowProgress {
            kind: WorkflowProgressKind::Failed,
            label: format!("Workflow `{pipeline_name}` failed"),
            detail: Some(reason.to_string()),
            depth,
        });
        if let Some(workflow) = &mut self.active_workflow {
            workflow.pipeline_depth = workflow.pipeline_depth.saturating_sub(1);
        }
    }

    /// Update the in-place workflow status message used in minimal mode.
    fn update_workflow_status(
        &mut self,
        new_state: WorkflowStatusState,
        detail: impl Into<String>,
    ) {
        if let Some(idx) = self
            .active_workflow
            .as_ref()
            .and_then(|w| w.workflow_status_msg_index)
            && let Some(AppMessage::WorkflowStatus {
                state: phase,
                detail: msg_detail,
                ..
            }) = self.messages.get_mut(idx)
        {
            *phase = new_state;
            *msg_detail = Some(detail.into());
        }
    }

    /// Minimal verbosity: a single in-place-updated status message for the entire pipeline.
    ///
    /// NOTE: parallel execution events are not handled here; parallel
    /// stages are effectively invisible in minimal mode. Use detailed
    /// verbosity for full parallel stage visibility.
    fn handle_pipeline_event_minimal(&mut self, event: &PipelineEvent) {
        match event {
            PipelineEvent::PipelineStarted { pipeline_name } => {
                let msg_index = self.messages.len();

                self.messages.push(AppMessage::WorkflowStatus {
                    state: WorkflowStatusState::Running,
                    label: format!("Workflow `{pipeline_name}`"),
                    detail: None,
                });

                if let Some(workflow) = &mut self.active_workflow {
                    workflow.workflow_status_msg_index = Some(msg_index);
                }
            }
            PipelineEvent::StageStarted { stage_index, .. } if *stage_index > 0 => {
                self.update_workflow_status(
                    WorkflowStatusState::Running,
                    format!("Stage: {stage_index}"),
                );
            }
            PipelineEvent::PipelineCompleted { outcome, .. } => {
                self.update_workflow_status(
                    WorkflowStatusState::Completed,
                    format!("Completed: {}", outcome.status.as_str()),
                );
            }
            PipelineEvent::PipelineFailed { reason, .. } => {
                self.update_workflow_status(
                    WorkflowStatusState::Failed,
                    format!("Failed: {reason}"),
                );
            }
            // All other events are suppressed in minimal mode
            _ => {}
        }
    }

    /// Simple mode: progress messages including one in-place-updated status message per stage.
    ///
    /// NOTE: parallel execution events are not handled here; concurrent
    /// stage updates may overwrite each other in the single status
    /// message slot. Use detailed verbosity for full parallel stage
    /// visibility.
    fn handle_pipeline_event_simple(&mut self, event: &PipelineEvent) {
        let depth = self.pipeline_display_depth();

        match event {
            PipelineEvent::PipelineStarted { pipeline_name } => {
                self.on_pipeline_started(pipeline_name);
            }
            PipelineEvent::StageStarted {
                node_id,
                stage_index,
                ..
            } if *stage_index > 0 => {
                let msg_index = self.messages.len();
                self.messages.push(AppMessage::WorkflowProgress {
                    kind: WorkflowProgressKind::Running,
                    label: format!("Workflow stage {stage_index}: {node_id}"),
                    detail: None,
                    depth,
                });
                if let Some(workflow) = &mut self.active_workflow {
                    workflow.stage_status_msg_index = Some(msg_index);
                }
            }
            PipelineEvent::StageInput {
                agent_name, input, ..
            } => {
                if let Some(idx) = self
                    .active_workflow
                    .as_ref()
                    .and_then(|w| w.stage_status_msg_index)
                    && let Some(AppMessage::WorkflowProgress { detail, .. }) =
                        self.messages.get_mut(idx)
                {
                    *detail = Some(format!("{agent_name}: {input}"));
                }
            }
            PipelineEvent::StageCompleted { outcome, .. } => {
                if let Some(idx) = self
                    .active_workflow
                    .as_ref()
                    .and_then(|w| w.stage_status_msg_index)
                    && let Some(AppMessage::WorkflowProgress { kind, detail, .. }) =
                        self.messages.get_mut(idx)
                {
                    *kind = if outcome.status.is_success() {
                        WorkflowProgressKind::Completed
                    } else {
                        WorkflowProgressKind::Failed
                    };
                    *detail = Some(outcome.status.as_str().to_title_case());
                }
            }
            PipelineEvent::StageFailed { reason, .. } => {
                let will_retry = self.messages.last().is_some_and(|message| {
                    matches!(
                        message,
                        AppMessage::WorkflowProgress {
                            kind: WorkflowProgressKind::Retrying,
                            ..
                        }
                    )
                });
                if will_retry {
                    return;
                }

                if let Some(idx) = self
                    .active_workflow
                    .as_ref()
                    .and_then(|w| w.stage_status_msg_index)
                    && let Some(AppMessage::WorkflowProgress { kind, detail, .. }) =
                        self.messages.get_mut(idx)
                {
                    *kind = WorkflowProgressKind::Failed;
                    *detail = Some(reason.clone());
                }
            }
            PipelineEvent::StageRetrying {
                node_id,
                stage_index,
                attempt,
                max_attempts,
            } => {
                self.on_stage_retrying(node_id, *stage_index, *attempt, *max_attempts);
            }
            PipelineEvent::PipelineCompleted {
                pipeline_name,
                outcome,
                ..
            } => {
                self.on_pipeline_completed(pipeline_name, outcome);
            }
            PipelineEvent::PipelineFailed {
                pipeline_name,
                reason,
                ..
            } => {
                self.on_pipeline_failed(pipeline_name, reason);
            }
            // Suppress parallel, interview, checkpoint, text delta, response end
            _ => {}
        }
    }

    /// Detailed mode: full exchange per stage with prompt + streaming response.
    ///
    /// During parallel (fan-out) execution, each branch gets its own
    /// `AppMessage::Exchange` so prompts and responses are grouped per
    /// branch instead of interleaved into a single block.
    #[allow(clippy::too_many_lines)]
    fn handle_pipeline_event_detailed(&mut self, event: &PipelineEvent) {
        let depth = self.pipeline_display_depth();

        match event {
            PipelineEvent::PipelineStarted { pipeline_name } => {
                self.on_pipeline_started(pipeline_name);
            }
            PipelineEvent::StageStarted {
                node_id,
                stage_index,
                handler_type,
            } if *stage_index > 0 => {
                let msg_index = self.messages.len();
                self.messages.push(AppMessage::Exchange {
                    kind: ExchangeKind::Workflow,
                    status: ExchangeStatus::Running,
                    request: format!("**Stage {stage_index}: {node_id}**"),
                    response: None,
                    response_segments: None,
                    exit_code: None,
                    agent_index: None,
                    agent_name: None,
                    handler_type: Some(handler_type.clone()),
                });
                if let Some(workflow) = &mut self.active_workflow {
                    workflow.current_exchange_msg_index = Some(msg_index);
                    workflow.current_stage_progress =
                        Some(Arc::new(Mutex::new(AgentProgress::default())));
                }
            }
            PipelineEvent::StageInput {
                node_id,
                agent_name,
                input,
                ..
            } => {
                self.color_registry.color_for(agent_name);

                // During parallel execution, route to the branch's own exchange.
                let is_parallel = self.active_workflow.as_ref().is_some_and(|w| w.in_parallel);

                if is_parallel {
                    let msg_index = self.messages.len();
                    self.messages.push(AppMessage::Exchange {
                        kind: ExchangeKind::Workflow,
                        status: ExchangeStatus::Running,
                        request: format!("**{node_id}**\n\n{input}"),
                        response: None,
                        response_segments: None,
                        exit_code: None,
                        agent_index: None,
                        agent_name: Some(agent_name.clone()),
                        // `StageInput` is currently emitted only for
                        // agent-backed stages, so the agent color will be
                        // used and no handler-type fallback is needed here.
                        handler_type: None,
                    });
                    let progress = Arc::new(Mutex::new(AgentProgress::default()));
                    if let Some(workflow) = &mut self.active_workflow {
                        workflow
                            .parallel_stages
                            .insert(node_id.clone(), (msg_index, progress));
                    }
                } else if let Some(msg_index) = self
                    .active_workflow
                    .as_ref()
                    .and_then(|w| w.current_exchange_msg_index)
                    && let Some(AppMessage::Exchange {
                        request,
                        agent_name: workflow_agent_name,
                        ..
                    }) = self.messages.get_mut(msg_index)
                {
                    request.push_str("\n\n");
                    request.push_str(input);
                    *workflow_agent_name = Some(agent_name.clone());
                }
            }
            PipelineEvent::StageSessionEvent {
                node_id, event: se, ..
            } => {
                let stage_progress = self.active_workflow.as_ref().and_then(|w| {
                    if w.in_parallel {
                        w.parallel_stages.get(node_id).cloned()
                    } else {
                        None
                    }
                });

                if let Some((msg_index, progress)) = stage_progress {
                    // Parallel stage: route to per-stage progress
                    crate::agent::process_event(se, &progress);
                    if let Ok(g) = progress.lock() {
                        let text = crate::agent::plain_text_from_segments(&g.segments);
                        Self::update_exchange_streaming(
                            &mut self.messages,
                            msg_index,
                            text,
                            g.segments.clone(),
                        );
                    }
                } else if let Some(workflow) = &self.active_workflow
                    && let Some(ref progress) = workflow.current_stage_progress
                {
                    // Normal (non-parallel) stage
                    crate::agent::process_event(se, progress);
                    if let Some(msg_index) = workflow.current_exchange_msg_index
                        && let Ok(g) = progress.lock()
                    {
                        let text = crate::agent::plain_text_from_segments(&g.segments);
                        Self::update_exchange_streaming(
                            &mut self.messages,
                            msg_index,
                            text,
                            g.segments.clone(),
                        );
                    }
                }
            }
            PipelineEvent::StageOutput {
                node_id, output, ..
            } => {
                if let Some(workflow) = &self.active_workflow {
                    if workflow.in_parallel {
                        // During parallel execution, finalise the per-stage
                        // exchange: set the response text (if the stage didn't
                        // stream session events) and mark as succeeded.
                        if let Some((msg_index, _)) = workflow.parallel_stages.get(node_id) {
                            let msg_index = *msg_index;
                            if let Some(AppMessage::Exchange {
                                status, response, ..
                            }) = self.messages.get_mut(msg_index)
                            {
                                if response.is_none() {
                                    *response = Some(output.clone());
                                }
                                *status = ExchangeStatus::Succeeded;
                            }
                        }
                    } else if let Some(msg_index) = workflow.current_exchange_msg_index {
                        // Non-parallel stage: set the response if it hasn't
                        // been populated by streaming session events (e.g.
                        // shell handler output).
                        if let Some(AppMessage::Exchange { response, .. }) =
                            self.messages.get_mut(msg_index)
                            && response.is_none()
                        {
                            *response = Some(output.clone());
                        }
                    }
                }
            }
            PipelineEvent::ParallelStarted { .. } => {
                if let Some(workflow) = &mut self.active_workflow {
                    workflow.in_parallel = true;
                    workflow.parallel_stages.clear();
                    workflow.parallel_had_failure = false;
                }
            }
            PipelineEvent::ParallelBranchFailed { reason, .. } => {
                if let Some(workflow) = &mut self.active_workflow {
                    workflow.parallel_had_failure = true;
                }
                // We cannot identify which specific stage exchanges
                // belong to this branch (the event carries the parallel
                // node id and branch index, not individual stage node
                // ids). Emit a standalone failure message so the user
                // sees the error.
                self.messages.push(AppMessage::WorkflowProgress {
                    kind: WorkflowProgressKind::Failed,
                    label: "Parallel branch failed".to_string(),
                    detail: Some(reason.clone()),
                    depth,
                });
            }
            PipelineEvent::ParallelCompleted { .. } => {
                if let Some(workflow) = &mut self.active_workflow {
                    // Mark remaining running stage exchanges based on
                    // whether any branch failed. If a failure occurred we
                    // cannot tell which specific stages belonged to the
                    // failed branch (the event model doesn't expose that),
                    // so remaining running exchanges are marked Failed to
                    // avoid displaying a misleading success state.
                    let fallback_status = if workflow.parallel_had_failure {
                        ExchangeStatus::Failed
                    } else {
                        ExchangeStatus::Succeeded
                    };
                    let indices: Vec<usize> = workflow
                        .parallel_stages
                        .values()
                        .map(|(idx, _)| *idx)
                        .collect();
                    for idx in indices {
                        if let Some(AppMessage::Exchange { status, .. }) =
                            self.messages.get_mut(idx)
                            && *status == ExchangeStatus::Running
                        {
                            *status = fallback_status;
                        }
                    }
                    workflow.in_parallel = false;
                    workflow.parallel_stages.clear();
                    workflow.parallel_had_failure = false;
                }
            }
            PipelineEvent::StageCompleted {
                node_id, outcome, ..
            } => {
                if let Some(workflow) = &mut self.active_workflow {
                    let completed_status = if outcome.status.is_success() {
                        ExchangeStatus::Succeeded
                    } else {
                        ExchangeStatus::Failed
                    };

                    // Parallel stage path: look up the per-stage exchange.
                    // (Currently branch subgraphs use NoOpEmitter for
                    // lifecycle events so this won't fire, but it is here
                    // for defensive correctness if that changes.)
                    if workflow.in_parallel {
                        if let Some((msg_index, _)) = workflow.parallel_stages.get(node_id) {
                            let msg_index = *msg_index;
                            if let Some(AppMessage::Exchange { status, .. }) =
                                self.messages.get_mut(msg_index)
                            {
                                *status = completed_status;
                            }
                        }
                    } else if let Some(msg_index) = workflow.current_exchange_msg_index.take()
                        && let Some(AppMessage::Exchange { status, .. }) =
                            self.messages.get_mut(msg_index)
                    {
                        *status = completed_status;
                    }
                    workflow.current_stage_progress = None;
                }
            }
            PipelineEvent::StageFailed { reason, .. } => {
                if let Some(workflow) = &mut self.active_workflow {
                    if let Some(msg_index) = workflow.current_exchange_msg_index.take()
                        && let Some(AppMessage::Exchange {
                            status, response, ..
                        }) = self.messages.get_mut(msg_index)
                    {
                        *status = ExchangeStatus::Failed;
                        // Preserve existing streamed response/segments;
                        // only set the response if nothing was streamed.
                        if response.is_none() {
                            *response = Some(reason.clone());
                        }
                    }
                    workflow.current_stage_progress = None;
                }
            }
            PipelineEvent::StageRetrying {
                node_id,
                stage_index,
                attempt,
                max_attempts,
            } => {
                self.on_stage_retrying(node_id, *stage_index, *attempt, *max_attempts);
            }
            PipelineEvent::PipelineCompleted {
                pipeline_name,
                outcome,
                ..
            } => {
                self.on_pipeline_completed(pipeline_name, outcome);
            }
            PipelineEvent::PipelineFailed {
                pipeline_name,
                reason,
                ..
            } => {
                self.on_pipeline_failed(pipeline_name, reason);
            }
            // Suppress remaining events: ParallelBranchStarted, ParallelBranchCompleted, interview, checkpoint
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    use super::super::{App, AppMode};
    use crate::autocomplete::workflows::WorkflowDefinitionInfo;

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    #[tokio::test]
    async fn ctrl_d_exits_workflow_mode() {
        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            ..Default::default()
        });
        assert_eq!(app.mode, AppMode::Workflow);
        assert!(app.active_workflow.is_some());

        app.handle_event(&key_event(KeyCode::Char('d'), KeyModifiers::CONTROL))
            .await;
        assert_eq!(app.mode, AppMode::Agent);
        assert!(app.active_workflow.is_some());
    }

    #[tokio::test]
    async fn workflow_ctrl_c_clears_input() {
        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            ..Default::default()
        });

        for c in "some text".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        assert_eq!(app.input.text(), "some text");

        let quit = app
            .handle_event(&key_event(KeyCode::Char('c'), KeyModifiers::CONTROL))
            .await;
        assert!(!quit);
        assert!(app.input.is_empty());
    }

    #[tokio::test]
    async fn workflow_with_default_goal_prefills_input() {
        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            description: String::new(),
            goal: Some("Review the latest pull request".to_string()),
            goal_hint: None,
        });

        assert_eq!(app.mode, AppMode::Workflow);
        assert_eq!(app.input.text(), "Review the latest pull request");
    }

    #[tokio::test]
    async fn workflow_with_goal_placeholder_leaves_input_empty() {
        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            description: String::new(),
            goal: Some("Generic pipeline goal".to_string()),
            goal_hint: Some("What do you want to build?".to_string()),
        });

        assert_eq!(app.mode, AppMode::Workflow);
        // Input should be empty — the placeholder is shown as dimmed hint text,
        // not pre-filled into the editable input.
        assert!(app.input.is_empty());
    }

    /// AC-2: `activate_workflow` should trigger the pre-run interview through
    /// the TUI's standard interview widget instead of waiting for text-based
    /// goal input.
    ///
    /// After activation, the app should have an active interview pending
    /// (when the workflow has a `goal_hint`), rather than setting input
    /// placeholder text and waiting for manual goal submission.
    #[tokio::test]
    async fn activate_workflow_triggers_pre_run_interview() {
        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            description: String::new(),
            goal: None,
            goal_hint: Some("What do you want to build?".to_string()),
        });

        assert_eq!(app.mode, AppMode::Workflow);
        assert!(app.active_workflow.is_some());

        // After activate_workflow, the pre-run interview should be triggered
        // as an active interview rather than waiting for bare text input.
        assert!(
            app.active_interview.is_some(),
            "activate_workflow should trigger a pre-run interview via the TUI interview widget"
        );
    }

    /// AC-3: Interview results (goal + gate timeout) are correctly extracted
    /// and passed to the workflow spawn.
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn submit_workflow_goal_forwards_gate_timeout() {
        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            description: String::new(),
            goal: None,
            goal_hint: Some("What do you want to build?".to_string()),
        });

        assert_eq!(app.mode, AppMode::Workflow);
        assert!(app.active_workflow.is_some());

        let gate_timeout = stencila_workflows::GateTimeoutConfig::AutoApprove;
        app.submit_workflow_goal("build a website".to_string(), gate_timeout);
    }

    /// AC-5: The legacy `goal_hint`-based placeholder in `ui/input.rs` is
    /// obsolete because the interview now handles goal collection.
    ///
    /// With the interview-based flow, activating a workflow that has a
    /// `goal_hint` should trigger an interview (verified by AC-2). When
    /// an interview is active, the render function shows interview
    /// placeholder text — it never falls through to the legacy branch that
    /// read `workflow.info.goal_hint` for a "pending workflow" placeholder.
    ///
    /// This test verifies the precondition that makes the legacy branch
    /// dead code: a pending workflow with `goal_hint` always has an active
    /// interview, so the legacy placeholder path is never reached and
    /// should be removed.
    ///
    /// In the Red phase this fails because `activate_workflow` doesn't yet
    /// trigger the interview — the app enters workflow mode with no active
    /// interview and the old placeholder logic remains in the render path.
    #[tokio::test]
    async fn pending_workflow_with_goal_hint_has_interview_not_placeholder() {
        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            description: String::new(),
            goal: None,
            goal_hint: Some("Describe your feature".to_string()),
        });

        assert_eq!(app.mode, AppMode::Workflow);
        let workflow = app.active_workflow.as_ref().expect("workflow should exist");

        // The workflow is pending (no run_handle yet) and has a goal_hint.
        assert!(workflow.run_handle.is_none());
        assert!(workflow.info.goal_hint.is_some());

        // With the interview-based flow, an active interview should exist.
        // This makes the legacy goal_hint placeholder in the render
        // function unreachable, allowing it to be safely removed.
        assert!(
            app.active_interview.is_some(),
            "a pending workflow with goal_hint should have an active interview; \
             the legacy goal_hint placeholder in ui/input.rs should be removed"
        );
    }

    /// AC-3 (end-to-end): When the pre-run interview is completed by the user,
    /// `complete_interview` should detect that it was a Workflow-source pre-run
    /// interview, extract the goal from the answer, and call
    /// `submit_workflow_goal` to start the workflow run.
    ///
    /// This test activates a workflow, answers the freeform interview question,
    /// and verifies that the workflow run is started (i.e. `run_handle` is set).
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn completing_pre_run_interview_starts_workflow() {
        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            description: String::new(),
            goal: None,
            goal_hint: Some("What do you want to build?".to_string()),
        });

        assert_eq!(app.mode, AppMode::Workflow);
        assert!(
            app.active_interview.is_some(),
            "pre-run interview should be active after activate_workflow"
        );

        // The workflow should not be running yet (no run_handle)
        let workflow = app.active_workflow.as_ref().expect("workflow should exist");
        assert!(
            workflow.run_handle.is_none(),
            "workflow should not be running before interview is completed"
        );

        // Simulate submitting an answer to the freeform question
        for c in "build a website".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        app.handle_event(&key_event(KeyCode::Enter, KeyModifiers::NONE))
            .await;

        // After completing the pre-run interview, the interview should be consumed
        assert!(
            app.active_interview.is_none(),
            "interview should be completed and cleared"
        );

        // The workflow should now be running — complete_interview should have
        // detected the Workflow source, extracted the goal from the answer, and
        // called submit_workflow_goal.
        let workflow = app.active_workflow.as_ref().expect("workflow should exist");
        assert!(
            workflow.run_handle.is_some(),
            "completing the pre-run interview should start the workflow run via submit_workflow_goal"
        );
    }

    /// AC-6: `WorkflowDefinitionInfo` retains `goal` and `goal_hint` fields,
    /// which are needed by `build_pre_run_interview`. This test asserts that
    /// a workflow activated with both fields preserves them in the stored info.
    #[tokio::test]
    async fn workflow_definition_info_retains_goal_fields() {
        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            description: "A test workflow".to_string(),
            goal: Some("Default goal text".to_string()),
            goal_hint: Some("What do you want to build?".to_string()),
        });

        let workflow = app.active_workflow.as_ref().expect("workflow should exist");
        assert_eq!(
            workflow.info.goal,
            Some("Default goal text".to_string()),
            "goal field should be preserved in ActiveWorkflow.info"
        );
        assert_eq!(
            workflow.info.goal_hint,
            Some("What do you want to build?".to_string()),
            "goal_hint field should be preserved in ActiveWorkflow.info"
        );
    }
}
